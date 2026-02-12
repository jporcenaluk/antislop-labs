<script lang="ts">
	import { onMount } from 'svelte';
	import SessionList from './SessionList.svelte';
	import { getHistory } from '$lib/api';
	import type { Session } from '$lib/types';

	let sessions = $state<Session[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function loadHistory() {
		loading = true;
		error = null;
		try {
			sessions = await getHistory();
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadHistory();
	});
</script>

<div class="history-container">
	{#if loading}
		<p class="loading">Loading history...</p>
	{:else if error}
		<p class="error">{error}</p>
		<button class="btn-retry" onclick={loadHistory}>Retry</button>
	{:else}
		<SessionList {sessions} />
	{/if}
</div>

<style>
	.history-container {
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.loading {
		color: var(--color-text-secondary);
		font-size: 0.9rem;
	}

	.error {
		color: var(--color-danger);
		font-size: 0.9rem;
	}

	.btn-retry {
		padding: 0.4rem 0.8rem;
		border: none;
		border-radius: 6px;
		background: var(--color-bg-secondary);
		color: var(--color-text);
		cursor: pointer;
		font-size: 0.85rem;
	}
</style>
