PomodoroAI – Goals
Purpose
PomodoroAI is a local-first timeboxing tool designed for human developers and AI agents working together. Its purpose is to bring clarity, discipline, and reliable handoffs into AI-assisted development workflows using simple, explicit timeblocks.
The tool is intentionally minimal: it optimizes for control, visibility, and signaling, not productivity theater.

Core Goals
1. Local-first by default
   Runs entirely on a local machine
   No required cloud services
   State and history are stored locally
2. Human + AI symmetry
   Humans and AI agents can create, start, and stop timeblocks
   Both are treated as first-class actors in the system
   Every session records who initiated it (human or agent)
3. Simple timeblocks
   A session is defined by:
   Duration (N minutes)
   Labels (short phrases, lightweight context)
   Labels are optimized for UI readability, not long descriptions
   No mandatory task hierarchies or project structures
4. Clear signaling on completion
   When a timer ends or is stopped:
   Humans are notified via UI and sound
   AI agents are notified via an MCP backward channel
   Completion is an explicit event, not something inferred
5. Explicit state and lifecycle
   Timer state is always clear and queryable
   One source of truth for running sessions
   Well-defined session lifecycle (created → running → stopped/completed)
6. UI as primary interface
   Visual UI for humans is a first-class concern
   UI clearly shows:
   Active session
   Remaining time
   Labels and origin (human / agent)
7. MCP-native integration
   PomodoroAI exposes an MCP interface
   AI agents can:
   Control timers
   Subscribe to timer events
   React programmatically to completion signals
8. Reliability over cleverness
   Timer completion events must be reliable
   Agent notifications should not be silently lost when possible
   Restarting the app should not corrupt active state

Design Principles
Small surface area: fewer concepts, stronger guarantees
Explicit over implicit: state changes are events
Interrupt-friendly: safe to stop, resume, or restart
Composable: works alongside existing tools and agent workflows

Open Questions (to refine later)
Single active timer vs multiple concurrent timers
Strict vs free-form label conventions
Event buffering guarantees for MCP subscribers
Whether breaks are modeled explicitly or left to the user

