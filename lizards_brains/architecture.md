# PomodoroAI Architecture

## Overview

PomodoroAI is built as a Tauri 2.0 desktop application with a Rust backend and web-based frontend. This architecture provides a small, fast, cross-platform application with native system integration.

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Frontend | SvelteKit + TypeScript | UI rendering, state visualization |
| Backend | Rust (Tauri) | Timer logic, state management, MCP server |
| Storage | SQLite | Session history persistence |
| IPC | Tauri Commands + Events | Frontend ↔ Backend communication |
| Agent Integration | MCP over stdio | AI agent control and notifications |

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Desktop App (Tauri)                      │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Frontend (Webview)                      │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │  │
│  │  │ Timer View  │  │  History    │  │  Settings       │   │  │
│  │  │             │  │  View       │  │                 │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘   │  │
│  │                         │                                  │  │
│  │              Tauri Commands / Events                       │  │
│  └─────────────────────────│─────────────────────────────────┘  │
│                            │                                     │
│  ┌─────────────────────────│─────────────────────────────────┐  │
│  │                    Backend (Rust)                          │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │                  Timer Engine                        │  │  │
│  │  │  - Single active timer enforcement                   │  │  │
│  │  │  - Tick events (1s resolution)                       │  │  │
│  │  │  - Completion detection                              │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                                                            │  │
│  │  ┌──────────────────┐  ┌──────────────────────────────┐  │  │
│  │  │  State Manager   │  │  MCP Server (stdio)          │  │  │
│  │  │  - In-memory     │  │  - startTimer                │  │  │
│  │  │    active timer  │  │  - stopTimer                 │  │  │
│  │  │  - SQLite for    │  │  - getStatus                 │  │  │
│  │  │    history       │  │  - getHistory                │  │  │
│  │  └──────────────────┘  │  - notifications/message     │  │  │
│  │                        └──────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                 System Integration                         │  │
│  │  - System tray icon with status                           │  │
│  │  - Native notifications on completion                     │  │
│  │  - Sound playback                                         │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
           │
           │ stdio (JSON-RPC)
           ▼
    ┌─────────────┐
    │  AI Agent   │
    │  (Claude,   │
    │   etc.)     │
    └─────────────┘
```

## Core Components

### 1. Timer Engine (Rust)

Central component managing timer lifecycle:

```rust
pub struct TimerEngine {
    active_session: Option<Session>,
    tx: broadcast::Sender<TimerEvent>,
}

pub struct Session {
    id: Uuid,
    label: String,           // max 64 chars, UTF-8
    duration_secs: u32,
    started_at: DateTime<Utc>,
    origin: Origin,          // Human | Agent
}

pub enum TimerEvent {
    Started(Session),
    Tick { remaining_secs: u32 },
    Completed(Session),
    Stopped(Session),
}
```

- Enforces single active timer at any time
- Emits events consumed by UI and MCP server
- Runs on Tokio async runtime

### 2. State Manager

Handles persistence and state queries:

- **Active timer**: In-memory (lost on crash - acceptable per goals)
- **History**: SQLite database in app data directory

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    duration_secs INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    origin TEXT NOT NULL,      -- 'human' | 'agent'
    status TEXT NOT NULL       -- 'running' | 'completed' | 'stopped'
);
```

### 3. MCP Server

Embedded MCP server using stdio transport:

**Tools:**
| Tool | Parameters | Description |
|------|------------|-------------|
| `startTimer` | `duration_minutes: number`, `label: string` | Start a new timer |
| `stopTimer` | none | Stop the active timer |
| `getStatus` | none | Get current timer state |
| `getHistory` | `start_date?: string`, `end_date?: string` | Query past sessions |

**Notifications:**
- `notifications/message` - Emitted when timer completes or stops

The MCP server runs as a child process spawned by Tauri, communicating via stdio.

### 4. Frontend (SvelteKit)

Lightweight UI with three views:

- **Timer View**: Shows active timer, remaining time, controls
- **History View**: Lists past sessions with filtering
- **Settings**: Sound preferences, theme

Communication via Tauri:
- `invoke()` for commands (start, stop, query)
- `listen()` for events (tick, completion)

## Data Flow

### Starting a Timer (Human)

```
UI Button Click
    │
    ▼
invoke('start_timer', { duration, label })
    │
    ▼
Rust: TimerEngine.start(duration, label, Origin::Human)
    │
    ├──► Insert into SQLite (status: running)
    ├──► Emit TimerEvent::Started
    │        │
    │        ├──► UI receives event, updates display
    │        └──► MCP broadcasts to connected agents
    │
    └──► Start tick loop (1s interval)
             │
             └──► Emit TimerEvent::Tick every second
```

### Starting a Timer (Agent via MCP)

```
Agent calls MCP tool 'startTimer'
    │
    ▼
MCP Server receives JSON-RPC request
    │
    ▼
Rust: TimerEngine.start(duration, label, Origin::Agent)
    │
    └──► Same flow as human-initiated
```

### Timer Completion

```
Tick loop detects remaining_secs == 0
    │
    ▼
TimerEngine.complete()
    │
    ├──► Update SQLite (status: completed, ended_at: now)
    ├──► Emit TimerEvent::Completed
    │        │
    │        ├──► UI: Show completion, play sound
    │        ├──► System: Native notification
    │        └──► MCP: Send notifications/message
    │
    └──► Clear active_session
```

## Directory Structure

```
pomodoro-ai/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # Tauri entry point
│   │   ├── timer.rs          # Timer engine
│   │   ├── state.rs          # State manager + SQLite
│   │   ├── mcp/
│   │   │   ├── mod.rs        # MCP server setup
│   │   │   ├── tools.rs      # Tool implementations
│   │   │   └── transport.rs  # stdio transport
│   │   └── commands.rs       # Tauri commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── routes/
│   │   ├── +page.svelte      # Timer view
│   │   └── history/
│   │       └── +page.svelte  # History view
│   ├── lib/
│   │   ├── stores/
│   │   │   └── timer.ts      # Reactive timer state
│   │   └── components/
│   │       ├── Timer.svelte
│   │       └── SessionList.svelte
│   └── app.html
├── package.json
└── svelte.config.js
```

## Key Design Decisions

### Why Tauri 2.0?
- **Small binary**: ~10MB vs ~150MB for Electron
- **Native performance**: Rust backend, no V8 overhead for logic
- **Cross-platform**: Windows, macOS, Linux from one codebase
- **System tray**: Native support for background operation
- **Security**: Sandboxed webview, explicit command allowlist

### Why SQLite for History?
- Embedded, no external dependencies
- ACID transactions for reliability
- Efficient range queries for date filtering
- Single file, easy backup

### Why Embedded MCP Server?
- Agents connect directly to running app
- No separate process to manage
- Shared state with UI (single source of truth)
- stdio transport for simplicity

### Why SvelteKit?
- Small bundle size (important for Tauri)
- Reactive state fits timer UI well
- Simple component model
- Fast compile times during development

## Reliability Guarantees

| Scenario | Behavior |
|----------|----------|
| App crash during timer | Timer lost, no history entry |
| App quit during timer | Timer stopped, history updated |
| System sleep | Timer pauses, resumes on wake |
| Multiple app launches | Single instance enforced |

## Open Questions

1. **MCP transport**: stdio vs HTTP for agent connection?
2. **Multi-monitor**: Which screen for notifications?
3. **Portable mode**: Store data alongside binary option?
