<script lang="ts">
	// Renders the shared tooltip next to its anchor, flipped below / kept inside the window when needed.
	import { tooltip } from '../game/tooltip.svelte';

	let el = $state<HTMLElement | null>(null);
	let pos = $state({ left: 0, top: 0, below: false });

	$effect(() => {
		const t = tooltip.tip;
		if (!t || !el) return;
		const w = el.offsetWidth, h = el.offsetHeight;
		const below = t.y - h - 10 < 4;
		const top = below ? t.y + t.h + 10 : t.y - h - 10;
		const left = Math.max(6, Math.min(window.innerWidth - w - 6, t.x + t.w / 2 - w / 2));
		pos = { left, top, below };
	});
</script>

<!-- clicking / dragging / scrolling always closes it -->
<svelte:window onpointerdown={() => tooltip.hide()} onwheel={() => tooltip.hide()} onkeydown={() => tooltip.hide()} />

{#if tooltip.tip}
	{@const t = tooltip.tip}
	<div class="tip" class:below={pos.below} bind:this={el} style="left:{pos.left}px; top:{pos.top}px; --c:{t.color}; --ax:{t.x + t.w / 2 - pos.left}px" role="tooltip">
		<div class="head"><span class="material-symbols-rounded">{t.icon}</span>{t.title}</div>
		<div class="body">{t.text}</div>
	</div>
{/if}

<style>
	.tip {
		position: fixed;
		z-index: 200;
		max-width: 260px;
		pointer-events: none;
		background: #1b2a34;
		color: #fff;
		border: 2px solid var(--c);
		border-radius: 12px;
		box-shadow: 0 8px 22px rgb(0 0 0 / 0.45);
		font-family: 'Fredoka', system-ui, sans-serif;
		animation: tip-in 0.12s ease-out;
	}
	.tip::after {
		content: '';
		position: absolute;
		left: clamp(10px, var(--ax), calc(100% - 10px));
		bottom: -8px;
		width: 12px;
		height: 12px;
		background: #1b2a34;
		border-right: 2px solid var(--c);
		border-bottom: 2px solid var(--c);
		transform: translateX(-50%) rotate(45deg);
	}
	.tip.below::after {
		bottom: auto;
		top: -8px;
		transform: translateX(-50%) rotate(225deg);
	}
	@keyframes tip-in {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
	}
	.head {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		background: var(--c);
		border-radius: 9px 9px 0 0;
		font-weight: 700;
		font-size: 0.85rem;
		color: #fff;
		text-shadow: 0 1px 2px rgb(0 0 0 / 0.45);
	}
	.head .material-symbols-rounded {
		font-size: 18px;
	}
	.body {
		padding: 7px 10px 9px;
		font-size: 0.85rem;
		line-height: 1.3;
	}
</style>
