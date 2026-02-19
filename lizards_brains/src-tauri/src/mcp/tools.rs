use crate::state::StateManager;
use crate::timer::{Origin, TimerEngine};
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{schemars, tool, ServerHandler};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct PomodoroMcpService {
    pub engine: TimerEngine,
    pub state: Arc<StateManager>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StartTimerParams {
    /// Duration in minutes (1-1440)
    pub duration_minutes: u32,
    /// Label for this focus session (1-64 chars)
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetHistoryParams {
    /// Optional start date filter (ISO 8601)
    pub start_date: Option<String>,
    /// Optional end date filter (ISO 8601)
    pub end_date: Option<String>,
}

#[tool(tool_box)]
impl PomodoroMcpService {
    #[tool(
        description = "Start a new Pomodoro focus timer. Only one timer can be active at a time."
    )]
    async fn start_timer(&self, #[tool(aggr)] params: StartTimerParams) -> String {
        match self
            .engine
            .start(params.duration_minutes, &params.label, Origin::Agent)
            .await
        {
            Ok(session) => serde_json::to_string_pretty(&session).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Stop the currently running timer.")]
    async fn stop_timer(&self) -> String {
        match self.engine.stop().await {
            Ok(session) => serde_json::to_string_pretty(&session).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get the current timer status including session info and remaining time.")]
    async fn get_status(&self) -> String {
        let status = self.engine.get_status().await;
        serde_json::to_string_pretty(&status).unwrap()
    }

    #[tool(description = "Get session history with optional date range filtering.")]
    async fn get_history(&self, #[tool(aggr)] params: GetHistoryParams) -> String {
        match self
            .state
            .get_history(params.start_date.as_deref(), params.end_date.as_deref())
        {
            Ok(sessions) => serde_json::to_string_pretty(&sessions).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for PomodoroMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("PomodoroAI - A focus timer that can be controlled by AI agents. Start timers, check status, and view session history.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
