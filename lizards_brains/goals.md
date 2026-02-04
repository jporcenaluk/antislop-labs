PomodoroAI – Goals
Purpose
PomodoroAI is a local-first timeboxing tool designed for human developers and AI agents working together. Its purpose is to bring clarity, discipline, and reliable handoffs into AI-assisted development workflows using simple, explicit timeblocks.
The tool is intentionally minimal: it optimizes for control, visibility, and signaling, not productivity theater.

Core Goals
1. Local-first by default
   Runs entirely on a local machine
   No required cloud services
   State and history are stored locally
   Single active timer at a time (true to Pomodoro philosophy)
   Completed sessions are stored in queryable history
2. Human + AI symmetry
   Humans and AI agents can create, start, and stop timeblocks
   Both are treated as first-class actors in the system
   Every session records who initiated it (human or agent)
3. Simple timeblocks
   A session is defined by:
   Duration (N minutes)
   Label (single freeform string, max 64 characters)
   Label constraints:
     - Any UTF-8 characters except control characters
     - Case-insensitive for search/filtering
     - Optimized for UI readability, not long descriptions
   No mandatory task hierarchies or project structures
4. Clear signaling on completion
   When a timer ends or is stopped:
   Humans are notified via UI and sound
   AI agents are notified via MCP `notifications/message` when connected
   Agents that cannot receive notifications can poll via `getStatus` as fallback
   Delivery is best-effort; if the agent disconnects, the notification is lost
   Completion is an explicit event, not something inferred
5. Explicit state and lifecycle
   Timer state is always clear and queryable
   One source of truth for running sessions
   Well-defined session lifecycle (created → running → stopped/completed)
   History stores per session: label, duration, start time, end time, origin, completion status
   Completion status distinguishes: completed (ran full duration) vs stopped (ended early)
   History retained indefinitely; user manages deletion manually
6. UI as primary interface
   Visual UI for humans is a first-class concern
   UI clearly shows:
   Active session
   Remaining time
   Label and origin (human / agent)
7. MCP-native integration
   PomodoroAI exposes an MCP interface with:
   Tools: `startTimer`, `stopTimer`, `getStatus`, `getHistory`
   `getHistory` supports optional date range filter
   Notifications: `notifications/message` emitted on timer completion
   AI agents can control timers and react programmatically to completion signals
8. Reliability over cleverness
   Timer completion events must be reliable for connected clients
   Agent notifications are best-effort; disconnected agents will not receive them
   Agents requiring guaranteed delivery should poll `getStatus`
   Restarting the app should not corrupt active state

Design Principles
Small surface area: fewer concepts, stronger guarantees
Explicit over implicit: state changes are events
Interrupt-friendly: safe to stop, resume, or restart
Composable: works alongside existing tools and agent workflows

Open Questions (to refine later)
Whether breaks are modeled explicitly or left to the user

