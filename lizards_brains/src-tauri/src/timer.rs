use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Origin {
    Human,
    Agent,
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::Human => write!(f, "human"),
            Origin::Agent => write!(f, "agent"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Running,
    Completed,
    Stopped,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Running => write!(f, "running"),
            SessionStatus::Completed => write!(f, "completed"),
            SessionStatus::Stopped => write!(f, "stopped"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub label: String,
    pub duration_secs: u64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub origin: Origin,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TimerEvent {
    Started {
        session: Session,
    },
    Tick {
        remaining_secs: u64,
        session: Session,
    },
    Completed {
        session: Session,
    },
    Stopped {
        session: Session,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStatus {
    pub session: Option<Session>,
    pub remaining_secs: u64,
    pub is_running: bool,
}

#[derive(Debug, Error)]
pub enum TimerError {
    #[error("A timer is already running")]
    AlreadyRunning,
    #[error("No timer is running")]
    NotRunning,
    #[error("Invalid label: {0}")]
    InvalidLabel(String),
    #[error("Invalid duration: must be between 1 and 1440 minutes")]
    InvalidDuration,
}

impl Serialize for TimerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct TimerInner {
    session: Option<Session>,
    remaining_secs: u64,
    cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Clone)]
pub struct TimerEngine {
    inner: Arc<Mutex<TimerInner>>,
    event_tx: broadcast::Sender<TimerEvent>,
}

impl Default for TimerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerEngine {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(64);
        TimerEngine {
            inner: Arc::new(Mutex::new(TimerInner {
                session: None,
                remaining_secs: 0,
                cancel_tx: None,
            })),
            event_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TimerEvent> {
        self.event_tx.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<TimerEvent> {
        self.event_tx.clone()
    }

    fn validate_label(label: &str) -> Result<String, TimerError> {
        let trimmed = label.trim().to_string();
        if trimmed.is_empty() {
            return Err(TimerError::InvalidLabel("label cannot be empty".into()));
        }
        if trimmed.len() > 64 {
            return Err(TimerError::InvalidLabel(
                "label must be 64 characters or fewer".into(),
            ));
        }
        if trimmed.chars().any(|c| c.is_control()) {
            return Err(TimerError::InvalidLabel(
                "label cannot contain control characters".into(),
            ));
        }
        Ok(trimmed)
    }

    fn validate_duration(minutes: u32) -> Result<u64, TimerError> {
        if !(1..=1440).contains(&minutes) {
            return Err(TimerError::InvalidDuration);
        }
        Ok(minutes as u64 * 60)
    }

    pub async fn start(
        &self,
        duration_minutes: u32,
        label: &str,
        origin: Origin,
    ) -> Result<Session, TimerError> {
        let label = Self::validate_label(label)?;
        let duration_secs = Self::validate_duration(duration_minutes)?;

        let mut inner = self.inner.lock().await;
        if inner.session.is_some() {
            return Err(TimerError::AlreadyRunning);
        }

        let session = Session {
            id: Uuid::new_v4().to_string(),
            label,
            duration_secs,
            started_at: Utc::now().to_rfc3339(),
            ended_at: None,
            origin,
            status: SessionStatus::Running,
        };

        inner.session = Some(session.clone());
        inner.remaining_secs = duration_secs;

        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
        inner.cancel_tx = Some(cancel_tx);

        let _ = self.event_tx.send(TimerEvent::Started {
            session: session.clone(),
        });

        // Spawn the tick loop
        let engine = self.clone();
        tokio::spawn(async move {
            engine.tick_loop(cancel_rx).await;
        });

        Ok(session)
    }

    async fn tick_loop(&self, mut cancel_rx: tokio::sync::oneshot::Receiver<()>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        interval.tick().await; // consume the immediate first tick

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let mut inner = self.inner.lock().await;
                    if inner.remaining_secs == 0 {
                        break;
                    }
                    inner.remaining_secs -= 1;
                    let remaining = inner.remaining_secs;
                    let session = inner.session.clone();
                    drop(inner);

                    if let Some(session) = session {
                        if remaining == 0 {
                            // Timer completed
                            let mut inner = self.inner.lock().await;
                            if let Some(ref mut s) = inner.session {
                                s.status = SessionStatus::Completed;
                                s.ended_at = Some(Utc::now().to_rfc3339());
                                let _ = self.event_tx.send(TimerEvent::Completed {
                                    session: s.clone(),
                                });
                            }
                            inner.session = None;
                            inner.cancel_tx = None;
                            break;
                        } else {
                            let _ = self.event_tx.send(TimerEvent::Tick {
                                remaining_secs: remaining,
                                session,
                            });
                        }
                    }
                }
                _ = &mut cancel_rx => {
                    break;
                }
            }
        }
    }

    pub async fn stop(&self) -> Result<Session, TimerError> {
        let mut inner = self.inner.lock().await;
        match inner.session.take() {
            Some(mut session) => {
                session.status = SessionStatus::Stopped;
                session.ended_at = Some(Utc::now().to_rfc3339());

                if let Some(cancel_tx) = inner.cancel_tx.take() {
                    let _ = cancel_tx.send(());
                }

                let _ = self.event_tx.send(TimerEvent::Stopped {
                    session: session.clone(),
                });

                inner.remaining_secs = 0;
                Ok(session)
            }
            None => Err(TimerError::NotRunning),
        }
    }

    pub async fn get_status(&self) -> TimerStatus {
        let inner = self.inner.lock().await;
        TimerStatus {
            session: inner.session.clone(),
            remaining_secs: inner.remaining_secs,
            is_running: inner.session.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{self, Duration};

    #[tokio::test]
    async fn test_start_timer() {
        let engine = TimerEngine::new();
        let session = engine
            .start(25, "Work session", Origin::Human)
            .await
            .unwrap();
        assert_eq!(session.label, "Work session");
        assert_eq!(session.duration_secs, 25 * 60);
        assert_eq!(session.status, SessionStatus::Running);
        assert_eq!(session.origin, Origin::Human);
        assert!(session.ended_at.is_none());
    }

    #[tokio::test]
    async fn test_stop_timer() {
        let engine = TimerEngine::new();
        engine.start(25, "Work", Origin::Human).await.unwrap();
        let session = engine.stop().await.unwrap();
        assert_eq!(session.status, SessionStatus::Stopped);
        assert!(session.ended_at.is_some());
    }

    #[tokio::test]
    async fn test_stop_when_not_running() {
        let engine = TimerEngine::new();
        let result = engine.stop().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimerError::NotRunning));
    }

    #[tokio::test]
    async fn test_double_start() {
        let engine = TimerEngine::new();
        engine.start(25, "First", Origin::Human).await.unwrap();
        let result = engine.start(25, "Second", Origin::Human).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimerError::AlreadyRunning));
    }

    #[tokio::test]
    async fn test_label_validation_empty() {
        let engine = TimerEngine::new();
        let result = engine.start(25, "", Origin::Human).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimerError::InvalidLabel(_)));
    }

    #[tokio::test]
    async fn test_label_validation_whitespace_only() {
        let engine = TimerEngine::new();
        let result = engine.start(25, "   ", Origin::Human).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_label_validation_too_long() {
        let engine = TimerEngine::new();
        let long_label = "a".repeat(65);
        let result = engine.start(25, &long_label, Origin::Human).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimerError::InvalidLabel(_)));
    }

    #[tokio::test]
    async fn test_label_validation_control_chars() {
        let engine = TimerEngine::new();
        let result = engine.start(25, "test\x00label", Origin::Human).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_label_trimmed() {
        let engine = TimerEngine::new();
        let session = engine.start(25, "  Work  ", Origin::Human).await.unwrap();
        assert_eq!(session.label, "Work");
    }

    #[tokio::test]
    async fn test_duration_validation_zero() {
        let engine = TimerEngine::new();
        let result = engine.start(0, "Work", Origin::Human).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimerError::InvalidDuration));
    }

    #[tokio::test]
    async fn test_duration_validation_too_large() {
        let engine = TimerEngine::new();
        let result = engine.start(1441, "Work", Origin::Human).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimerError::InvalidDuration));
    }

    #[tokio::test]
    async fn test_get_status_idle() {
        let engine = TimerEngine::new();
        let status = engine.get_status().await;
        assert!(!status.is_running);
        assert!(status.session.is_none());
        assert_eq!(status.remaining_secs, 0);
    }

    #[tokio::test]
    async fn test_get_status_running() {
        let engine = TimerEngine::new();
        engine.start(25, "Work", Origin::Human).await.unwrap();
        let status = engine.get_status().await;
        assert!(status.is_running);
        assert!(status.session.is_some());
        assert_eq!(status.remaining_secs, 25 * 60);
    }

    #[tokio::test]
    async fn test_events_on_start() {
        let engine = TimerEngine::new();
        let mut rx = engine.subscribe();
        engine.start(25, "Work", Origin::Human).await.unwrap();
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, TimerEvent::Started { .. }));
    }

    #[tokio::test]
    async fn test_events_on_stop() {
        let engine = TimerEngine::new();
        let mut rx = engine.subscribe();
        engine.start(25, "Work", Origin::Human).await.unwrap();
        let _ = rx.recv().await.unwrap(); // Started
        engine.stop().await.unwrap();
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, TimerEvent::Stopped { .. }));
    }

    #[tokio::test]
    async fn test_tick_events() {
        time::pause();
        let engine = TimerEngine::new();
        let mut rx = engine.subscribe();
        engine.start(1, "Quick", Origin::Human).await.unwrap();
        let _ = rx.recv().await.unwrap(); // Started

        time::advance(Duration::from_secs(1)).await;
        tokio::task::yield_now().await;

        // Should get a tick or completion
        let event = rx.recv().await.unwrap();
        match event {
            TimerEvent::Tick { remaining_secs, .. } => {
                assert!(remaining_secs < 60);
            }
            TimerEvent::Completed { .. } => {
                // 1-minute timer might complete quickly in test
            }
            _ => panic!("Expected Tick or Completed event"),
        }
    }

    #[tokio::test]
    async fn test_restart_after_stop() {
        let engine = TimerEngine::new();
        engine.start(25, "First", Origin::Human).await.unwrap();
        engine.stop().await.unwrap();
        let session = engine.start(15, "Second", Origin::Agent).await.unwrap();
        assert_eq!(session.label, "Second");
        assert_eq!(session.origin, Origin::Agent);
    }

    #[tokio::test]
    async fn test_agent_origin() {
        let engine = TimerEngine::new();
        let session = engine.start(25, "AI Task", Origin::Agent).await.unwrap();
        assert_eq!(session.origin, Origin::Agent);
    }

    #[tokio::test]
    async fn test_valid_uuid() {
        let engine = TimerEngine::new();
        let session = engine.start(25, "Work", Origin::Human).await.unwrap();
        assert!(Uuid::parse_str(&session.id).is_ok());
    }
}
