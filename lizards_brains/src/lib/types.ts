export type Origin = 'Human' | 'Agent';

export type SessionStatus = 'Running' | 'Completed' | 'Stopped';

export interface Session {
	id: string;
	label: string;
	duration_secs: number;
	started_at: string;
	ended_at: string | null;
	origin: Origin;
	status: SessionStatus;
}

export interface TimerStatus {
	session: Session | null;
	remaining_secs: number;
	is_running: boolean;
}

export interface TickPayload {
	remaining_secs: number;
	session: Session;
}
