<script>
	import { onMount } from 'svelte';
	import { startGame } from '#lib/main.js';
	import { setKey, releaseAll } from '#lib/core.js';

	let canvas;
	let place = $state('PALLET TOWN');
	let debugging = $state(false);
	let touch = $state(false);

	onMount(() => {
		// Show the on-screen pad on touch devices. `?touch=1` / `?touch=0`
		// forces it on/off (handy for testing and desktop).
		const q = new URLSearchParams(location.search).get('touch');
		const coarse = window.matchMedia('(pointer: coarse)').matches;
		const ua = /Android|iPhone|iPad|iPod|Mobile|Silk|Kindle|Windows Phone/i.test(navigator.userAgent);
		touch = q != null ? q === '1' : coarse || (ua && navigator.maxTouchPoints > 0);

		const { dispose } = startGame(canvas, {
			onMap: (map) => {
				place = map.name;
			},
			onDebug: (on) => {
				debugging = on;
			}
		});
		return () => {
			releaseAll();
			dispose();
		};
	});

	// Pointer handler set for one pad button: press on down, release on up.
	// Pointer capture keeps the release reliable even if the finger slides off.
	function press(name) {
		return {
			onpointerdown: (e) => {
				e.preventDefault();
				e.currentTarget.setPointerCapture?.(e.pointerId);
				setKey(name, true);
			},
			onpointerup: (e) => {
				e.preventDefault();
				setKey(name, false);
			},
			onpointercancel: () => setKey(name, false),
			onlostpointercapture: () => setKey(name, false),
			oncontextmenu: (e) => e.preventDefault()
		};
	}
</script>

<svelte:head>
	<title>Pallet Quest — tiny 2D Pokémon</title>
</svelte:head>

<main class="fixed inset-0 grid touch-none place-items-center overflow-hidden overscroll-none bg-[#0b1220]">
	<canvas id="game" bind:this={canvas} class="rounded-md"></canvas>

	<!-- Location badge -->
	<div class="pointer-events-none absolute left-3 top-3 flex items-center gap-2 rounded-full bg-black/55 px-3 py-1.5 text-white backdrop-blur-sm">
		<span class="material-symbols-outlined text-[18px] text-amber-300">location_on</span>
		<span class="text-xs font-bold tracking-widest">{place}</span>
	</div>

	<!-- Controls hint (desktop only; the touch pad is self-labelling) -->
	{#if !touch}
		<div class="pointer-events-none absolute bottom-3 left-0 right-0 flex items-center justify-center">
			<div class="flex items-center gap-3 rounded-full bg-black/55 px-4 py-1.5 text-[11px] text-white/75 backdrop-blur-sm">
				<span class="flex items-center gap-1">
					<span class="material-symbols-outlined text-[15px] text-amber-300">keyboard</span>
					WASD move
				</span>
				<span class="flex items-center gap-1">
					<span class="material-symbols-outlined text-[15px] text-amber-300">chat_bubble</span>
					Z talk
				</span>
				<span class="flex items-center gap-1">
					<span class="material-symbols-outlined text-[15px] text-amber-300">directions_run</span>
					Shift run
				</span>
				<span class="flex items-center gap-1" class:text-emerald-300={debugging}>
					<span class="material-symbols-outlined text-[15px] text-amber-300">bug_report</span>
					^ inspect
				</span>
			</div>
		</div>
	{/if}

	{#if debugging}
		<div class="pointer-events-none absolute right-3 top-3 rounded-full bg-emerald-500/85 px-3 py-1.5 text-xs font-bold tracking-widest text-white">
			DEBUG — CLICK COPIES ID
		</div>
	{/if}

	{#if touch}
		<div class="touch-pad" aria-hidden="true">
			<div class="dpad">
				<button class="key up" aria-label="Up" {...press('up')}>
					<span class="material-symbols-outlined">arrow_upward</span>
				</button>
				<button class="key left" aria-label="Left" {...press('left')}>
					<span class="material-symbols-outlined">arrow_back</span>
				</button>
				<button class="key right" aria-label="Right" {...press('right')}>
					<span class="material-symbols-outlined">arrow_forward</span>
				</button>
				<button class="key down" aria-label="Down" {...press('down')}>
					<span class="material-symbols-outlined">arrow_downward</span>
				</button>
			</div>
			<div class="ab">
				<button class="key round b" aria-label="B — cancel" {...press('b')}>B</button>
				<button class="key round a" aria-label="A — confirm / jump" {...press('a')}>A</button>
			</div>
		</div>
	{/if}
</main>

<style>
	canvas {
		image-rendering: pixelated;
	}
	.material-symbols-outlined {
		font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 24;
	}

	/* On-screen pad. Everything is sized from --u, which is viewport-relative
	   (vmin), so the buttons keep the same thumb-reach proportion on a huge
	   4K display as on a tiny 320×240 phone. */
	.touch-pad {
		position: fixed;
		inset: 0;
		z-index: 40;
		pointer-events: none;
		--u: max(34px, 9vmin);
		touch-action: none;
	}
	.touch-pad .key {
		pointer-events: auto;
		display: grid;
		place-items: center;
		margin: 0;
		padding: 0;
		border: 0;
		color: #fff;
		background: rgba(255, 255, 255, 0.14);
		box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.28), 0 0.4vmin 1.2vmin rgba(0, 0, 0, 0.45);
		-webkit-tap-highlight-color: transparent;
		-webkit-touch-callout: none;
		touch-action: none;
		user-select: none;
		transition: transform 60ms ease, background 60ms ease;
	}
	.touch-pad .key:active {
		background: rgba(253, 224, 71, 0.55);
		transform: scale(0.94);
	}
	.touch-pad .key .material-symbols-outlined {
		font-size: calc(var(--u) * 0.6);
		line-height: 1;
	}

	.dpad {
		position: absolute;
		left: max(10px, 3vmin);
		bottom: max(10px, 3vmin);
		width: calc(var(--u) * 3);
		height: calc(var(--u) * 3);
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		grid-template-rows: repeat(3, 1fr);
		gap: calc(var(--u) * 0.07);
	}
	.dpad .key { border-radius: 26%; }
	.dpad .up { grid-area: 1 / 2 / 2 / 3; }
	.dpad .left { grid-area: 2 / 1 / 3 / 2; }
	.dpad .right { grid-area: 2 / 3 / 3 / 4; }
	.dpad .down { grid-area: 3 / 2 / 4 / 3; }

	.ab {
		position: absolute;
		right: max(14px, 4vmin);
		bottom: max(14px, 4vmin);
		display: flex;
		align-items: flex-end;
		gap: calc(var(--u) * 0.45);
	}
	.ab .round {
		width: calc(var(--u) * 1.75);
		height: calc(var(--u) * 1.75);
		border-radius: 50%;
		font-size: calc(var(--u) * 0.7);
		font-weight: 800;
		letter-spacing: 0.02em;
	}
	.ab .b { background: rgba(56, 189, 248, 0.3); }
	.ab .a { background: rgba(244, 63, 94, 0.34); margin-bottom: calc(var(--u) * 0.5); }
</style>
