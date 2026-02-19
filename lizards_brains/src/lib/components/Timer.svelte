<script lang="ts">
	import { timerState, formattedTime, progressPercent, handleStart, handleStop } from '$lib/stores/timer';

	let label = $state('');
	let durationMinutes = $state(25);
	let customDuration = $state('');

	const presets = [5, 15, 25, 45];

	function selectPreset(minutes: number) {
		durationMinutes = minutes;
		customDuration = '';
	}

	function onCustomDurationChange() {
		const val = parseInt(customDuration);
		if (!isNaN(val) && val >= 1 && val <= 1440) {
			durationMinutes = val;
		}
	}

	async function onStart() {
		if (!label.trim()) return;
		await handleStart(durationMinutes, label.trim());
	}

	async function onStop() {
		await handleStop();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
			if (!$timerState.isRunning) {
				onStart();
			}
		} else if (event.key === 'Escape') {
			if ($timerState.isRunning) {
				onStop();
			}
		}
	}

	// SVG circle parameters
	const radius = 90;
	const circumference = 2 * Math.PI * radius;
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="timer-container">
	{#if $timerState.isRunning && $timerState.session}
		<!-- Active Timer State -->
		<div class="timer-active">
			<div class="progress-ring-container">
				<svg viewBox="0 0 200 200" class="progress-ring">
					<circle
						cx="100"
						cy="100"
						r={radius}
						fill="none"
						stroke="var(--color-bg-secondary)"
						stroke-width="8"
					/>
					<circle
						cx="100"
						cy="100"
						r={radius}
						fill="none"
						stroke="var(--color-accent)"
						stroke-width="8"
						stroke-linecap="round"
						stroke-dasharray={circumference}
						stroke-dashoffset={circumference - (circumference * $progressPercent) / 100}
						transform="rotate(-90 100 100)"
						class="progress-circle"
					/>
				</svg>
				<div class="time-display">
					<span class="time-text">{$formattedTime}</span>
				</div>
			</div>

			<div class="session-info">
				<h2 class="session-label">{$timerState.session.label}</h2>
				<span class="origin-badge" class:agent={$timerState.session.origin === 'Agent'}>
					{$timerState.session.origin === 'Agent' ? 'AI' : 'You'}
				</span>
			</div>

			<button class="btn btn-stop" onclick={onStop} disabled={$timerState.loading}>
				{$timerState.loading ? 'Stopping...' : 'Stop'}
			</button>
			<p class="hint">Press Escape to stop</p>
		</div>
	{:else}
		<!-- Idle State -->
		<div class="timer-idle">
			<div class="progress-ring-container idle">
				<svg viewBox="0 0 200 200" class="progress-ring">
					<circle
						cx="100"
						cy="100"
						r={radius}
						fill="none"
						stroke="var(--color-bg-secondary)"
						stroke-width="8"
					/>
				</svg>
				<div class="time-display">
					<span class="time-text idle-time">{String(durationMinutes).padStart(2, '0')}:00</span>
				</div>
			</div>

			<div class="form-group">
				<input
					type="text"
					bind:value={label}
					placeholder="What are you working on?"
					maxlength={64}
					class="input-label"
				/>
			</div>

			<div class="duration-presets">
				{#each presets as preset}
					<button
						class="btn btn-preset"
						class:active={durationMinutes === preset && customDuration === ''}
						onclick={() => selectPreset(preset)}
					>
						{preset}m
					</button>
				{/each}
				<input
					type="number"
					bind:value={customDuration}
					oninput={onCustomDurationChange}
					placeholder="Custom"
					min="1"
					max="1440"
					class="input-custom-duration"
				/>
			</div>

			<button
				class="btn btn-start"
				onclick={onStart}
				disabled={$timerState.loading || !label.trim()}
			>
				{$timerState.loading ? 'Starting...' : 'Start Focus'}
			</button>
			<p class="hint">Ctrl+Enter to start</p>
		</div>
	{/if}

	{#if $timerState.error}
		<div class="error-message">{$timerState.error}</div>
	{/if}
</div>

<style>
	.timer-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 1rem;
		width: 100%;
		max-width: 380px;
		margin: 0 auto;
	}

	.timer-active,
	.timer-idle {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 100%;
		gap: 1rem;
	}

	.progress-ring-container {
		position: relative;
		width: 220px;
		height: 220px;
	}

	.progress-ring {
		width: 100%;
		height: 100%;
	}

	.progress-circle {
		transition: stroke-dashoffset 0.5s ease;
	}

	.time-display {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
	}

	.time-text {
		font-size: 2.5rem;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
		color: var(--color-text);
	}

	.idle-time {
		opacity: 0.5;
	}

	.session-info {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.session-label {
		font-size: 1.1rem;
		font-weight: 600;
		margin: 0;
		color: var(--color-text);
	}

	.origin-badge {
		font-size: 0.7rem;
		font-weight: 600;
		padding: 0.15rem 0.4rem;
		border-radius: 4px;
		background: var(--color-bg-secondary);
		color: var(--color-text-secondary);
		text-transform: uppercase;
	}

	.origin-badge.agent {
		background: var(--color-accent);
		color: white;
	}

	.form-group {
		width: 100%;
	}

	.input-label {
		width: 100%;
		padding: 0.6rem 0.8rem;
		border: 2px solid var(--color-border);
		border-radius: 8px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 0.95rem;
		outline: none;
		transition: border-color 0.2s;
		box-sizing: border-box;
	}

	.input-label:focus {
		border-color: var(--color-accent);
	}

	.duration-presets {
		display: flex;
		gap: 0.4rem;
		align-items: center;
		flex-wrap: wrap;
		justify-content: center;
	}

	.btn {
		border: none;
		border-radius: 8px;
		cursor: pointer;
		font-weight: 600;
		transition: all 0.15s;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-preset {
		padding: 0.4rem 0.8rem;
		background: var(--color-bg-secondary);
		color: var(--color-text);
		font-size: 0.85rem;
	}

	.btn-preset:hover {
		background: var(--color-border);
	}

	.btn-preset.active {
		background: var(--color-accent);
		color: white;
	}

	.input-custom-duration {
		width: 70px;
		padding: 0.4rem 0.5rem;
		border: 2px solid var(--color-border);
		border-radius: 8px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 0.85rem;
		outline: none;
		text-align: center;
	}

	.input-custom-duration:focus {
		border-color: var(--color-accent);
	}

	.btn-start {
		width: 100%;
		padding: 0.75rem;
		background: var(--color-accent);
		color: white;
		font-size: 1rem;
	}

	.btn-start:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.btn-stop {
		width: 100%;
		padding: 0.75rem;
		background: var(--color-danger);
		color: white;
		font-size: 1rem;
	}

	.btn-stop:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.hint {
		font-size: 0.75rem;
		color: var(--color-text-secondary);
		margin: 0;
	}

	.error-message {
		margin-top: 0.5rem;
		padding: 0.5rem 0.75rem;
		background: var(--color-danger-bg);
		color: var(--color-danger);
		border-radius: 6px;
		font-size: 0.85rem;
		width: 100%;
		text-align: center;
	}
</style>
