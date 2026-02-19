use crate::state::StateManager;
use crate::timer::{Origin, Session, TimerEngine, TimerStatus};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn start_timer(
    engine: State<'_, TimerEngine>,
    duration_minutes: u32,
    label: String,
) -> Result<String, String> {
    let session = engine
        .start(duration_minutes, &label, Origin::Human)
        .await
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&session).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_timer(engine: State<'_, TimerEngine>) -> Result<String, String> {
    let session = engine.stop().await.map_err(|e| e.to_string())?;
    serde_json::to_string(&session).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_status(engine: State<'_, TimerEngine>) -> Result<TimerStatus, String> {
    Ok(engine.get_status().await)
}

#[tauri::command]
pub fn get_history(
    state: State<'_, Arc<StateManager>>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<Session>, String> {
    state
        .get_history(start_date.as_deref(), end_date.as_deref())
        .map_err(|e| e.to_string())
}
