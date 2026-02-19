# PomodoroAI Implementation Plan

## Phase 1: Project Scaffolding

**Goal**: Working Tauri 2.0 app with basic UI shell

### Tasks
- [ ] Initialize Tauri 2.0 project with SvelteKit frontend
- [ ] Configure Tauri permissions (notifications, system tray)
- [ ] Set up development environment (hot reload, Rust analyzer)
- [ ] Create basic layout: header, main content area, tray icon
- [ ] Verify cross-platform build (Linux, macOS, Windows)

### Deliverable
Empty app launches, shows placeholder UI, appears in system tray.

---

## Phase 2: Testing & QA Infrastructure

**Goal**: Linting, formatting, testing frameworks, and CI pipeline — enforced from day one

### Rust (Backend)
- [ ] Configure `clippy` with strict lint rules (`cargo clippy -- -D warnings`)
- [ ] Configure `rustfmt` with project `.rustfmt.toml`
- [ ] Set up `cargo test` harness with module-based test structure
- [ ] Add integration test directory (`src-tauri/tests/`)

### Frontend (SvelteKit + TypeScript)
- [ ] Configure ESLint with `eslint-plugin-svelte` and TypeScript rules
- [ ] Configure Prettier with Svelte plugin
- [ ] Set up `svelte-check` for type checking
- [ ] Add Vitest for unit/component tests
- [ ] Add Playwright for end-to-end tests (Tauri webview)

### CI Pipeline (GitHub Actions)
- [ ] Create CI workflow triggered on push/PR to `main`
  - [ ] `cargo fmt --check` — fail on unformatted Rust
  - [ ] `cargo clippy -- -D warnings` — fail on lint warnings
  - [ ] `cargo test` — run all Rust tests
  - [ ] `npm run lint` — ESLint + Prettier check
  - [ ] `npm run check` — `svelte-check` type validation
  - [ ] `npm run test` — Vitest unit tests
- [ ] Add build step to verify compilation on Linux, macOS, Windows

### Pre-commit Hooks
- [ ] Set up `pre-commit` or `husky` + `lint-staged`
  - [ ] Run `rustfmt` on staged `.rs` files
  - [ ] Run `eslint --fix` + `prettier --write` on staged `.ts`/`.svelte` files

### Deliverable
All linters, formatters, and test runners configured. CI pipeline blocks merges on failures. Pre-commit hooks catch issues locally before push.

---

## Phase 3: Timer Engine (Rust)

**Goal**: Core timer logic with in-memory state

### Tasks
- [ ] Define `Session` and `TimerEvent` types
- [ ] Implement `TimerEngine` struct
  - [ ] `start(duration, label, origin)` - creates session, spawns tick loop
  - [ ] `stop()` - ends session early
  - [ ] `get_status()` - returns current state
- [ ] Enforce single active timer constraint
- [ ] Emit events via Tokio broadcast channel
- [ ] Add unit tests for timer lifecycle

### Deliverable
Timer engine works in isolation (tested via Rust tests).

---

## Phase 4: Tauri Commands + Events

**Goal**: Connect frontend to timer engine

### Tasks
- [ ] Expose Tauri commands:
  - [ ] `start_timer(duration_minutes, label)`
  - [ ] `stop_timer()`
  - [ ] `get_status()`
- [ ] Emit Tauri events from timer engine:
  - [ ] `timer:started`
  - [ ] `timer:tick`
  - [ ] `timer:completed`
  - [ ] `timer:stopped`
- [ ] Wire up event listeners in frontend

### Deliverable
UI can start/stop timers, receives real-time updates.

---

## Phase 5: Timer UI

**Goal**: Functional timer display and controls

### Tasks
- [ ] Create `Timer.svelte` component
  - [ ] Circular progress indicator
  - [ ] Remaining time display (MM:SS)
  - [ ] Label display
  - [ ] Origin indicator (human/agent badge)
- [ ] Create start form: duration input, label input
- [ ] Add stop button (visible when timer active)
- [ ] Reactive state store synced with Tauri events

### Deliverable
Users can start timers, watch countdown, stop early.

---

## Phase 6: Persistence (SQLite)

**Goal**: Session history survives app restarts

### Tasks
- [ ] Add SQLite dependency (`rusqlite` or `sqlx`)
- [ ] Create database schema migration
- [ ] Implement `StateManager`:
  - [ ] `save_session(session)`
  - [ ] `update_session(id, status, ended_at)`
  - [ ] `get_history(start_date, end_date)`
- [ ] Hook into timer engine: save on start, update on complete/stop
- [ ] Store database in platform-appropriate app data directory

### Deliverable
Sessions persist, queryable after restart.

---

## Phase 7: History UI

**Goal**: View and filter past sessions

### Tasks
- [ ] Create `SessionList.svelte` component
- [ ] Add `/history` route
- [ ] Expose `get_history` Tauri command
- [ ] Implement date range picker for filtering
- [ ] Display: label, duration, date, origin, status
- [ ] Show summary stats (total time today/week)

### Deliverable
Users can browse and filter their session history.

---

## Phase 8: System Notifications

**Goal**: Alert users when timer completes

### Tasks
- [ ] Configure Tauri notification plugin
- [ ] Trigger native notification on `TimerEvent::Completed`
- [ ] Add completion sound (bundled audio file)
- [ ] Implement sound playback via Tauri or web audio
- [ ] Add settings: notification toggle, sound toggle, volume

### Deliverable
Timer completion triggers visible + audible alert.

---

## Phase 9: System Tray

**Goal**: Background operation with tray status

### Tasks
- [ ] Configure Tauri system tray
- [ ] Show timer status in tray tooltip
- [ ] Tray menu: Show/Hide window, Start quick timer, Quit
- [ ] Update tray icon based on state (idle/running)
- [ ] Minimize to tray instead of closing

### Deliverable
App runs in background, accessible from tray.

---

## Phase 10: MCP Server

**Goal**: AI agents can control timers

### Tasks
- [ ] Add MCP SDK dependency (or implement minimal JSON-RPC)
- [ ] Implement stdio transport handler
- [ ] Register tools:
  - [ ] `startTimer` → calls timer engine
  - [ ] `stopTimer` → calls timer engine
  - [ ] `getStatus` → returns current state
  - [ ] `getHistory` → queries SQLite
- [ ] Subscribe to timer events, emit `notifications/message`
- [ ] Document MCP interface for agent integration
- [ ] Test with Claude Code or similar MCP client

### Deliverable
AI agents can start/stop timers and receive completion notifications.

---

## Phase 11: Polish & Release

**Goal**: Production-ready application

### Tasks
- [ ] Error handling and user-friendly messages
- [ ] Loading states and edge cases
- [ ] Keyboard shortcuts (start/stop)
- [ ] App icon and branding
- [ ] Auto-updater configuration
- [ ] Build installers (DMG, MSI, AppImage)
- [ ] Write user documentation
- [ ] License and README

### Deliverable
Distributable application ready for users.

---

## Timeline Estimate

| Phase | Complexity |
|-------|------------|
| 1. Scaffolding | Low |
| 2. Testing & QA Infrastructure | Medium |
| 3. Timer Engine | Medium |
| 4. Tauri Commands | Low |
| 5. Timer UI | Medium |
| 6. Persistence | Medium |
| 7. History UI | Low |
| 8. Notifications | Low |
| 9. System Tray | Low |
| 10. MCP Server | High |
| 11. Polish | Medium |

## Dependencies Between Phases

```
Phase 1 (Scaffolding)
    │
    ▼
Phase 2 (Testing & QA) ◄── sets up linters, CI, test frameworks
    │
    ▼
Phase 3 (Timer Engine) ──────────────────┐
    │                                     │
    ▼                                     ▼
Phase 4 (Tauri Commands)            Phase 6 (SQLite)
    │                                     │
    ▼                                     ▼
Phase 5 (Timer UI)                  Phase 7 (History UI)
    │
    ├──► Phase 8 (Notifications)
    │
    ├──► Phase 9 (System Tray)
    │
    └──► Phase 10 (MCP Server) ◄── Phase 6 (SQLite)
              │
              ▼
         Phase 11 (Polish)
```

All phases after Phase 2 benefit from the CI pipeline — every PR is automatically checked for lint, format, and test failures.

## Next Steps

1. Set up development environment (Rust, Node.js, Tauri CLI)
2. Run `npm create tauri-app@latest` with SvelteKit template
3. Begin Phase 1 tasks
