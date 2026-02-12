mod commands;
pub mod mcp;
pub mod state;
pub mod timer;

use state::StateManager;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;
use timer::{SessionStatus, TimerEngine, TimerEvent};

pub fn run_mcp_shim() {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        if let Err(e) = mcp::transport::run_mcp_shim().await {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    });
}

pub fn run_gui() {
    let engine = TimerEngine::new();
    let mut rx = engine.subscribe();
    let socket_engine = engine.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            commands::start_timer,
            commands::stop_timer,
            commands::get_status,
            commands::get_history,
        ])
        .setup(move |app| {
            // System tray
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .tooltip("PomodoroAI")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Hide window on close instead of quitting
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
            });

            // Initialize SQLite database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            let db_path = app_data_dir.join("pomodoro.db");
            let state_manager = StateManager::new(db_path.to_str().expect("invalid db path"))
                .expect("failed to initialize database");

            // Clean up stale running sessions from previous crash
            if let Ok(count) = state_manager.cleanup_stale_running() {
                if count > 0 {
                    eprintln!("Cleaned up {} stale running sessions", count);
                }
            }

            let state_manager = Arc::new(state_manager);
            app.manage(Arc::clone(&state_manager));

            let db = Arc::clone(&state_manager);
            let handle = app.handle().clone();

            // Unix socket listener for MCP clients
            let socket_state = Arc::clone(&state_manager);
            tauri::async_runtime::spawn(async move {
                if let Err(e) =
                    mcp::transport::start_socket_listener(socket_engine, socket_state).await
                {
                    eprintln!("MCP socket listener error: {}", e);
                }
            });

            // Event forwarding: broadcast channel -> Tauri events + DB persistence
            tauri::async_runtime::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            let (event_name, payload) = match &event {
                                TimerEvent::Started { session } => {
                                    if let Err(e) = db.save_session(session) {
                                        eprintln!("Failed to save session: {}", e);
                                    }
                                    ("timer:started", serde_json::to_string(session).unwrap())
                                }
                                TimerEvent::Tick {
                                    remaining_secs,
                                    session,
                                } => {
                                    // Update tray tooltip with remaining time
                                    let mins = remaining_secs / 60;
                                    let secs = remaining_secs % 60;
                                    if let Some(tray) = handle.tray_by_id("main") {
                                        let _ = tray.set_tooltip(Some(&format!(
                                            "{} - {:02}:{:02}",
                                            session.label, mins, secs
                                        )));
                                    }
                                    (
                                        "timer:tick",
                                        serde_json::json!({
                                            "remaining_secs": remaining_secs,
                                            "session": session
                                        })
                                        .to_string(),
                                    )
                                }
                                TimerEvent::Completed { session } => {
                                    if let Some(ended) = &session.ended_at {
                                        if let Err(e) = db.update_session(
                                            &session.id,
                                            &SessionStatus::Completed,
                                            ended,
                                        ) {
                                            eprintln!("Failed to update session: {}", e);
                                        }
                                    }
                                    // Send system notification
                                    if let Ok(true) =
                                        handle.notification().permission_state().map(|p| {
                                            p == tauri_plugin_notification::PermissionState::Granted
                                        })
                                    {
                                        let _ = handle
                                            .notification()
                                            .builder()
                                            .title("Timer Complete!")
                                            .body(format!("\"{}\" session finished", session.label))
                                            .show();
                                    }
                                    // Reset tray tooltip
                                    if let Some(tray) = handle.tray_by_id("main") {
                                        let _ = tray.set_tooltip(Some("PomodoroAI"));
                                    }
                                    ("timer:completed", serde_json::to_string(session).unwrap())
                                }
                                TimerEvent::Stopped { session } => {
                                    if let Some(ended) = &session.ended_at {
                                        if let Err(e) = db.update_session(
                                            &session.id,
                                            &SessionStatus::Stopped,
                                            ended,
                                        ) {
                                            eprintln!("Failed to update session: {}", e);
                                        }
                                    }
                                    // Reset tray tooltip
                                    if let Some(tray) = handle.tray_by_id("main") {
                                        let _ = tray.set_tooltip(Some("PomodoroAI"));
                                    }
                                    ("timer:stopped", serde_json::to_string(session).unwrap())
                                }
                            };
                            let _ = handle.emit(event_name, payload);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            eprintln!("Event forwarder lagged by {} events", n);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

