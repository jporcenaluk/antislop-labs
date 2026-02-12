<script lang="ts">
	import { onMount } from 'svelte';
	import Timer from '$lib/components/Timer.svelte';
	import History from '$lib/components/History.svelte';
	import { initTimer, setupListeners } from '$lib/stores/timer';

	let currentView = $state<'timer' | 'history'>('timer');

	onMount(() => {
		initTimer();
		const cleanup = setupListeners();
		return () => {
			cleanup.then((fn) => fn());
		};
	});
</script>

<div class="app">
	<header class="app-header">
		<h1 class="app-title">PomodoroAI</h1>
		<nav class="nav">
			<button
				class="nav-btn"
				class:active={currentView === 'timer'}
				onclick={() => (currentView = 'timer')}
			>
				Timer
			</button>
			<button
				class="nav-btn"
				class:active={currentView === 'history'}
				onclick={() => (currentView = 'history')}
			>
				History
			</button>
		</nav>
	</header>

	<main class="app-main">
		{#if currentView === 'timer'}
			<Timer />
		{:else}
			<History />
		{/if}
	</main>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
		background: var(--color-bg);
		color: var(--color-text);
	}

	.app-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1rem;
		border-bottom: 1px solid var(--color-border);
	}

	.app-title {
		font-size: 1.1rem;
		font-weight: 700;
		margin: 0;
	}

	.nav {
		display: flex;
		gap: 0.25rem;
	}

	.nav-btn {
		padding: 0.35rem 0.75rem;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-secondary);
		cursor: pointer;
		font-size: 0.85rem;
		font-weight: 500;
		transition: all 0.15s;
	}

	.nav-btn:hover {
		background: var(--color-bg-secondary);
	}

	.nav-btn.active {
		background: var(--color-bg-secondary);
		color: var(--color-text);
		font-weight: 600;
	}

	.app-main {
		flex: 1;
		display: flex;
		justify-content: center;
		padding-top: 2rem;
	}

</style>
