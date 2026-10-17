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

	// Pointer handler set for one round button (A/B): press on down, release on up.
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

	// Analog stick: the knob follows the thumb and its offset picks a direction
	// (grid movement is 4-way, so the dominant axis wins). Dead zone in the
	// middle; the knob is clamped to a travel radius.
	let knob = $state({ x: 0, y: 0 });
	let stickId = null;

	function moveStick(e) {
		const el = e.currentTarget;
		const r = el.getBoundingClientRect();
		const cx = r.left + r.width / 2;
		const cy = r.top + r.height / 2;
		const R = r.width / 2;
		const dx = e.clientX - cx;
		const dy = e.clientY - cy;
		const dist = Math.hypot(dx, dy);
		const travel = R * 0.55;
		const k = dist > travel ? travel / dist : 1;
		knob = { x: dx * k, y: dy * k };

		const dead = R * 0.26;
		let up = false, down = false, left = false, right = false;
		if (dist >= dead) {
			if (Math.abs(dx) >= Math.abs(dy)) {
				if (dx > 0) right = true; else left = true;
			} else {
				if (dy > 0) down = true; else up = true;
			}
		}
		setKey('up', up);
		setKey('down', down);
		setKey('left', left);
		setKey('right', right);
	}

	function grabStick(e) {
		e.preventDefault();
		stickId = e.pointerId;
		e.currentTarget.setPointerCapture?.(e.pointerId);
		moveStick(e);
	}

	function releaseStick() {
		stickId = null;
		knob = { x: 0, y: 0 };
		setKey('up', false);
		setKey('down', false);
		setKey('left', false);
		setKey('right', false);
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
			<div
				class="stick"
				onpointerdown={grabStick}
				onpointermove={(e) => {
					if (e.pointerId === stickId) moveStick(e);
				}}
				onpointerup={releaseStick}
				onpointercancel={releaseStick}
				onlostpointercapture={releaseStick}
				oncontextmenu={(e) => e.preventDefault()}
			>
				<div class="stick-knob" style="transform: translate({knob.x}px, {knob.y}px)"></div>
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
	/* Analog stick (bottom-left). Sized in vmin so the thumb reach stays
	   consistent from a small phone to a 4K display. */
	.stick {
		position: absolute;
		left: max(12px, 4vmin);
		bottom: max(12px, 4vmin);
		width: calc(var(--u) * 3.4);
		height: calc(var(--u) * 3.4);
		border-radius: 50%;
		pointer-events: auto;
		display: grid;
		place-items: center;
		touch-action: none;
		-webkit-tap-highlight-color: transparent;
		background: radial-gradient(circle at 50% 42%, rgba(255, 255, 255, 0.18), rgba(255, 255, 255, 0.06) 72%);
		box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.25), 0 0.4vmin 1.2vmin rgba(0, 0, 0, 0.45);
	}
	.stick-knob {
		width: calc(var(--u) * 1.5);
		height: calc(var(--u) * 1.5);
		border-radius: 50%;
		background: rgba(255, 255, 255, 0.3);
		box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.55), 0 0.3vmin 0.9vmin rgba(0, 0, 0, 0.45);
		will-change: transform;
	}

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
