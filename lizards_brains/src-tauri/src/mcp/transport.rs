use crate::mcp::tools::PomodoroMcpService;
use crate::state::StateManager;
use crate::timer::TimerEngine;
use rmcp::ServiceExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::{UnixListener, UnixStream};

/// Returns the canonical socket path used by both the GUI listener and CLI shim.
pub fn socket_path() -> PathBuf {
    let data_dir = dirs::data_dir().expect("failed to resolve data directory");
    data_dir.join("com.pomodoroai.app").join("pomodoro.sock")
}

/// Starts a Unix socket listener on the canonical path.
pub async fn start_socket_listener(
    engine: TimerEngine,
    state: Arc<StateManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    start_socket_listener_on(&socket_path(), engine, state).await
}

/// Starts a Unix socket listener on a given path.
/// Accepts MCP client connections and serves each one with a fresh PomodoroMcpService.
pub async fn start_socket_listener_on(
    path: &Path,
    engine: TimerEngine,
    state: Arc<StateManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove stale socket file from a previous crash
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    let listener = UnixListener::bind(path)?;
    eprintln!("MCP socket listener started on {:?}", path);

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let service = PomodoroMcpService {
                    engine: engine.clone(),
                    state: Arc::clone(&state),
                };
                tokio::spawn(async move {
                    match service.serve(stream).await {
                        Ok(running) => {
                            let _ = running.waiting().await;
                        }
                        Err(e) => {
                            eprintln!("MCP client session error: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept MCP connection: {}", e);
            }
        }
    }
}

/// CLI shim: connects to the GUI's Unix socket and proxies stdin/stdout ↔ socket.
/// This makes the shim appear as a normal stdio MCP server to Claude Code.
pub async fn run_mcp_shim() -> Result<(), Box<dyn std::error::Error>> {
    let path = socket_path();

    let stream = UnixStream::connect(&path).await.map_err(|e| {
        format!(
            "Failed to connect to PomodoroAI socket at {:?}: {}. Is the GUI running?",
            path, e
        )
    })?;

    let (mut sock_read, mut sock_write) = tokio::io::split(stream);
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    // Bidirectional pipe: stdin → socket, socket → stdout
    tokio::select! {
        result = tokio::io::copy(&mut stdin, &mut sock_write) => {
            result?;
        }
        result = tokio::io::copy(&mut sock_read, &mut stdout) => {
            result?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::TimerEngine;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn temp_socket_path() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pomodoro-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("test.sock")
    }

    #[test]
    fn test_socket_path_structure() {
        let path = socket_path();
        assert!(path.ends_with("com.pomodoroai.app/pomodoro.sock"));
    }

    #[tokio::test]
    async fn test_socket_listener_accepts_connections() {
        let path = temp_socket_path();
        let engine = TimerEngine::new();
        let state = Arc::new(StateManager::in_memory().unwrap());

        let listener_path = path.clone();
        let _listener = tokio::spawn(async move {
            start_socket_listener_on(&listener_path, engine, state)
                .await
                .unwrap();
        });

        // Give the listener time to bind
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Client connects successfully
        let stream = UnixStream::connect(&path).await;
        assert!(stream.is_ok(), "Should connect to socket listener");

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_socket_listener_serves_mcp() {
        let path = temp_socket_path();
        let engine = TimerEngine::new();
        let state = Arc::new(StateManager::in_memory().unwrap());

        let listener_path = path.clone();
        let _listener = tokio::spawn(async move {
            start_socket_listener_on(&listener_path, engine, state)
                .await
                .unwrap();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut stream = UnixStream::connect(&path).await.unwrap();

        // Send MCP initialize request (JSON-RPC over newline-delimited transport)
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0.1.0" }
            }
        });
        let mut msg = serde_json::to_vec(&init_request).unwrap();
        msg.push(b'\n');
        stream.write_all(&msg).await.unwrap();

        // Read response
        let mut buf = vec![0u8; 4096];
        let n = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf))
            .await
            .expect("response timed out")
            .expect("read failed");

        let response: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        // Server should return capabilities with tools enabled
        assert!(response["result"]["capabilities"]["tools"].is_object());

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_socket_listener_multiple_clients() {
        let path = temp_socket_path();
        let engine = TimerEngine::new();
        let state = Arc::new(StateManager::in_memory().unwrap());

        let listener_path = path.clone();
        let _listener = tokio::spawn(async move {
            start_socket_listener_on(&listener_path, engine, state)
                .await
                .unwrap();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Connect two clients simultaneously
        let stream1 = UnixStream::connect(&path).await;
        let stream2 = UnixStream::connect(&path).await;
        assert!(stream1.is_ok(), "First client should connect");
        assert!(stream2.is_ok(), "Second client should connect");

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_socket_listener_removes_stale_socket() {
        let path = temp_socket_path();

        // Create a stale socket file
        std::fs::write(&path, "stale").unwrap();
        assert!(path.exists());

        let engine = TimerEngine::new();
        let state = Arc::new(StateManager::in_memory().unwrap());

        let listener_path = path.clone();
        let _listener = tokio::spawn(async move {
            start_socket_listener_on(&listener_path, engine, state)
                .await
                .unwrap();
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should still be able to connect (stale file was removed)
        let stream = UnixStream::connect(&path).await;
        assert!(stream.is_ok(), "Should connect after stale socket removal");

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn test_shim_fails_when_no_socket() {
        // Connect to a non-existent socket
        let bad_path = std::env::temp_dir().join("nonexistent-pomodoro.sock");
        let result = UnixStream::connect(&bad_path).await;
        assert!(result.is_err(), "Should fail when socket doesn't exist");
    }
}
