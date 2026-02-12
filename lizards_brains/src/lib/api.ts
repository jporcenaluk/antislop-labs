import { invoke } from '@tauri-apps/api/core';
import type { Session, TimerStatus } from './types';

export async function startTimer(durationMinutes: number, label: string): Promise<Session> {
	const json = await invoke<string>('start_timer', {
		durationMinutes,
		label
	});
	return JSON.parse(json);
}

export async function stopTimer(): Promise<Session> {
	const json = await invoke<string>('stop_timer');
	return JSON.parse(json);
}

export async function getStatus(): Promise<TimerStatus> {
	return invoke<TimerStatus>('get_status');
}

export async function getHistory(
	startDate?: string,
	endDate?: string
): Promise<Session[]> {
	return invoke<Session[]>('get_history', { startDate, endDate });
}
