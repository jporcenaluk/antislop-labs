import { writable, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { getStatus, startTimer, stopTimer } from '$lib/api';
import { playCompletionSound } from '$lib/sounds';
import type { Session, TickPayload } from '$lib/types';

interface TimerState {
	session: Session | null;
	remainingSecs: number;
	isRunning: boolean;
	error: string | null;
	loading: boolean;
}

const initialState: TimerState = {
	session: null,
	remainingSecs: 0,
	isRunning: false,
	error: null,
	loading: false
};

export const timerState = writable<TimerState>(initialState);

export const formattedTime = derived(timerState, ($state) => {
	const mins = Math.floor($state.remainingSecs / 60);
	const secs = $state.remainingSecs % 60;
	return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
});

export const progressPercent = derived(timerState, ($state) => {
	if (!$state.session || $state.session.duration_secs === 0) return 0;
	return (($state.session.duration_secs - $state.remainingSecs) / $state.session.duration_secs) * 100;
});

export async function initTimer() {
	try {
		const status = await getStatus();
		timerState.set({
			session: status.session,
			remainingSecs: status.remaining_secs,
			isRunning: status.is_running,
			error: null,
			loading: false
		});
	} catch (e) {
		timerState.update((s) => ({ ...s, error: String(e) }));
	}
}

export async function handleStart(durationMinutes: number, label: string) {
	timerState.update((s) => ({ ...s, loading: true, error: null }));
	try {
		await startTimer(durationMinutes, label);
	} catch (e) {
		timerState.update((s) => ({ ...s, error: String(e), loading: false }));
	}
}

export async function handleStop() {
	timerState.update((s) => ({ ...s, loading: true, error: null }));
	try {
		await stopTimer();
	} catch (e) {
		timerState.update((s) => ({ ...s, error: String(e), loading: false }));
	}
}

export async function setupListeners() {
	const unlisteners: (() => void)[] = [];

	unlisteners.push(
		await listen<string>('timer:started', (event) => {
			const session: Session = JSON.parse(event.payload);
			timerState.set({
				session,
				remainingSecs: session.duration_secs,
				isRunning: true,
				error: null,
				loading: false
			});
		})
	);

	unlisteners.push(
		await listen<string>('timer:tick', (event) => {
			const payload: TickPayload = JSON.parse(event.payload);
			timerState.update((s) => ({
				...s,
				remainingSecs: payload.remaining_secs,
				session: payload.session
			}));
		})
	);

	unlisteners.push(
		await listen<string>('timer:completed', (event) => {
			const session: Session = JSON.parse(event.payload);
			timerState.set({
				session: null,
				remainingSecs: 0,
				isRunning: false,
				error: null,
				loading: false
			});
			// Play completion sound
			playCompletionSound();
		})
	);

	unlisteners.push(
		await listen<string>('timer:stopped', (event) => {
			const session: Session = JSON.parse(event.payload);
			timerState.set({
				session: null,
				remainingSecs: 0,
				isRunning: false,
				error: null,
				loading: false
			});
		})
	);

	return () => unlisteners.forEach((fn) => fn());
}

// Utility for formatting - also used in tests
export function formatTime(totalSecs: number): string {
	const mins = Math.floor(totalSecs / 60);
	const secs = totalSecs % 60;
	return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}
