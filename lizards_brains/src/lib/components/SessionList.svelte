<script lang="ts">
	import type { Session } from '$lib/types';

	let { sessions }: { sessions: Session[] } = $props();

	function formatDuration(secs: number): string {
		const mins = Math.floor(secs / 60);
		const s = secs % 60;
		if (mins >= 60) {
			const hrs = Math.floor(mins / 60);
			const m = mins % 60;
			return `${hrs}h ${m}m`;
		}
		return s > 0 ? `${mins}m ${s}s` : `${mins}m`;
	}

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return d.toLocaleDateString(undefined, {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function totalMinutes(sessions: Session[]): number {
		return Math.round(sessions.reduce((acc, s) => acc + s.duration_secs, 0) / 60);
	}

	function completedCount(sessions: Session[]): number {
		return sessions.filter((s) => s.status === 'Completed').length;
	}
</script>

<div class="session-list">
	{#if sessions.length > 0}
		<div class="stats">
			<div class="stat">
				<span class="stat-value">{sessions.length}</span>
				<span class="stat-label">Sessions</span>
			</div>
			<div class="stat">
				<span class="stat-value">{completedCount(sessions)}</span>
				<span class="stat-label">Completed</span>
			</div>
			<div class="stat">
				<span class="stat-value">{totalMinutes(sessions)}</span>
				<span class="stat-label">Total min</span>
			</div>
		</div>

		<div class="list">
			{#each sessions as session (session.id)}
				<div class="session-card">
					<div class="session-header">
						<span class="session-label">{session.label}</span>
						<div class="badges">
							<span
								class="status-badge"
								class:completed={session.status === 'Completed'}
								class:stopped={session.status === 'Stopped'}
							>
								{session.status}
							</span>
							{#if session.origin === 'Agent'}
								<span class="origin-badge">AI</span>
							{/if}
						</div>
					</div>
					<div class="session-meta">
						<span>{formatDuration(session.duration_secs)}</span>
						<span>{formatDate(session.started_at)}</span>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="empty">
			<p>No sessions yet. Start a timer to begin tracking!</p>
		</div>
	{/if}
</div>

<style>
	.session-list {
		width: 100%;
		max-width: 380px;
		padding: 0 1rem;
	}

	.stats {
		display: flex;
		justify-content: space-around;
		padding: 0.75rem;
		background: var(--color-bg-secondary);
		border-radius: 10px;
		margin-bottom: 1rem;
	}

	.stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.15rem;
	}

	.stat-value {
		font-size: 1.3rem;
		font-weight: 700;
		color: var(--color-text);
	}

	.stat-label {
		font-size: 0.7rem;
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		max-height: 400px;
		overflow-y: auto;
		padding-right: 0.25rem;
	}

	.session-card {
		padding: 0.65rem 0.8rem;
		background: var(--color-bg-secondary);
		border-radius: 8px;
		transition: background 0.15s;
	}

	.session-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.3rem;
	}

	.session-label {
		font-weight: 600;
		font-size: 0.9rem;
		color: var(--color-text);
	}

	.badges {
		display: flex;
		gap: 0.3rem;
	}

	.status-badge {
		font-size: 0.65rem;
		font-weight: 600;
		padding: 0.1rem 0.35rem;
		border-radius: 4px;
		background: var(--color-border);
		color: var(--color-text-secondary);
		text-transform: uppercase;
	}

	.status-badge.completed {
		background: rgba(76, 175, 80, 0.2);
		color: var(--color-success);
	}

	.status-badge.stopped {
		background: var(--color-danger-bg);
		color: var(--color-danger);
	}

	.origin-badge {
		font-size: 0.65rem;
		font-weight: 600;
		padding: 0.1rem 0.35rem;
		border-radius: 4px;
		background: var(--color-accent);
		color: white;
		text-transform: uppercase;
	}

	.session-meta {
		display: flex;
		justify-content: space-between;
		font-size: 0.75rem;
		color: var(--color-text-secondary);
	}

	.empty {
		text-align: center;
		padding: 2rem;
		color: var(--color-text-secondary);
		font-size: 0.9rem;
	}
</style>
