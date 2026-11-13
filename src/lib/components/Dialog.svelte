<script lang="ts" module>
	export interface Choice {
		label: string;
		icon?: string;
		action: () => void;
	}
	export interface DialogData {
		speaker: string;
		lines: string[];
		choices?: Choice[];
		onEnd?: () => void;
	}
</script>

<script lang="ts">
	import { playVoice, stopVoice } from '../game/voice';
	let { data, onClose }: { data: DialogData; onClose: () => void } = $props();

	let idx = $state(0);
	let shown = $state('');
	let timer: ReturnType<typeof setInterval> | undefined;

	const line = $derived(data.lines[idx] ?? '');
	const last = $derived(idx >= data.lines.length - 1);
	const typing = $derived(shown.length < line.length);

	$effect(() => {
		const full = line;
		playVoice(data.speaker, full);
		shown = '';
		clearInterval(timer);
		let i = 0;
		timer = setInterval(() => {
			i += 2;
			shown = full.slice(0, i);
			if (i >= full.length) clearInterval(timer);
		}, 16);
		return () => clearInterval(timer);
	});

	function next() {
		if (typing) {
			clearInterval(timer);
			shown = line;
			return;
		}
		if (!last) {
			idx++;
			return;
		}
		if (data.choices?.length) return;
		const end = data.onEnd;
		stopVoice();
		onClose();
		end?.();
	}

	function onKey(e: KeyboardEvent) {
		const k = e.key.toLowerCase();
		if (k === ' ' || k === 'e' || k === 'enter') {
			e.preventDefault();
			if (e.repeat) return;
			next();
		}
		if (k === 'escape' && !data.choices?.length && !data.onEnd) onClose();
	}
</script>

<svelte:window onkeydown={onKey} />

<div class="fixed inset-x-0 bottom-0 z-30 flex justify-center p-4 pointer-events-none">
	<div class="box pointer-events-auto" role="dialog" aria-live="polite">
		<div class="speaker">{data.speaker}</div>
		<button class="text-left w-full min-h-[3.5em] text-lg leading-snug" onclick={next}>
			{shown}
		</button>
		{#if last && !typing && data.choices?.length}
			<div class="flex flex-wrap gap-2 mt-3">
				{#each data.choices as c}
					<button
						class="choice"
						onclick={() => {
							onClose();
							c.action();
						}}
					>
						{#if c.icon}<span class="material-symbols-rounded">{c.icon}</span>{/if}
						{c.label}
					</button>
				{/each}
			</div>
		{:else}
			<div class="more">
				<span class="material-symbols-rounded">{last ? 'check' : 'expand_more'}</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.box {
		position: relative;
		width: min(760px, 100%);
		background: #fff;
		color: #1b2a34;
		border-radius: 16px;
		border: 4px solid #1b2a34;
		box-shadow:
			inset 0 -6px 0 rgb(0 0 0 / 0.08),
			0 10px 30px rgb(0 0 0 / 0.35);
		padding: 22px 22px 16px;
		font-family: 'Fredoka', system-ui, sans-serif;
		animation: up 0.2s ease-out;
	}
	@keyframes up {
		from {
			transform: translateY(20px);
			opacity: 0;
		}
	}
	.speaker {
		position: absolute;
		top: -18px;
		left: 18px;
		background: #c91a09;
		color: #fff;
		font-weight: 700;
		padding: 2px 14px;
		border-radius: 10px;
		border: 3px solid #1b2a34;
	}
	.more {
		position: absolute;
		right: 14px;
		bottom: 8px;
		color: #c91a09;
		animation: bob 0.8s infinite alternate;
	}
	@keyframes bob {
		to {
			transform: translateY(3px);
		}
	}
	.choice {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-radius: 10px;
		font-weight: 700;
		background: #f2cd37;
		border: 3px solid #1b2a34;
		box-shadow: inset 0 -3px 0 rgb(0 0 0 / 0.2);
	}
	.choice:hover {
		filter: brightness(1.08);
	}
</style>
