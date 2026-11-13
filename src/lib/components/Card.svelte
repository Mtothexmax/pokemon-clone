<script lang="ts">
	import { CARD_MAP, KIND_INFO, inkOn } from '../game/cards';
	import { cardArt, cardArtBackdrop, cardArtCreature } from '../game/art';
	import { tooltip } from '../game/tooltip.svelte';

	let {
		id,
		size = 'md',
		power = null,
		faceDown = false,
		selected = false,
		staged = false,
		dim = false,
		pulse = 0,
		width = null,
		ready = false,
		fxN = 0,
		fxId = 0,
		fxDelay = 0,
		aura = false,
		hideCost = false,
		level = 0,
		backdrop = false
	}: {
		id: string;
		size?: 'sm' | 'md' | 'lg';
		power?: number | null;
		faceDown?: boolean;
		selected?: boolean;
		staged?: boolean;
		dim?: boolean;
		pulse?: number;
		width?: string | null;
		/** highlights an Activate card that can be used now */
		ready?: boolean;
		/** last power change and a counter that triggers the effect */
		fxN?: number;
		fxId?: number;
		/** ms to wait before showing the effect (projectile travel time) */
		fxDelay?: number;
		/** Ongoing card in play: radiate continuously */
		aura?: boolean;
		/** cards lying on the table don't need their energy cost */
		hideCost?: boolean;
		/** upgrade stage (levels.ts): 1 bronze, 2 silver + glitter, 3 gold + holo, 4 Voxelmon */
		level?: number;
		/** empty picture window: the 3D Voxelmon is drawn on top of it (Card3D) */
		backdrop?: boolean;
	} = $props();
	// glitter specks for silver and up: fixed spots per card, so they don't jump around
	const specks = $derived(
		level >= 2
			? Array.from({ length: level >= 3 ? 9 : 6 }, (_, i) => {
					const h = (i * 2654435761 + id.length * 97) >>> 0;
					return { x: (h % 90) + 5, y: ((h >> 8) % 85) + 5, d: ((h >> 16) % 30) / 10, s: 0.5 + ((h >> 4) % 6) / 10 };
				})
			: []
	);
	const wStyle = $derived(width ? `--w:${width};` : '');

	const def = $derived(CARD_MAP[id]);
	/** the beast alone (drawn above the holo / glitter), or null: then the picture is one image */
	const creature = $derived(backdrop ? null : cardArtCreature(id, level >= 4));

	/** long ability texts: shrink the font step by step (down to 50 %) until the text fits into the card */
	function fitText(node: HTMLElement, _text: string) {
		const fit = () => {
			const inner = node.parentElement;
			if (!inner) return;
			node.style.fontSize = '';
			const base = parseFloat(getComputedStyle(node).fontSize);
			for (let k = 1; k >= 0.5; k -= 0.05) {
				node.style.fontSize = `${base * k}px`;
				if (node.offsetTop + node.scrollHeight <= inner.clientHeight - 2) return;
			}
		};
		const ro = new ResizeObserver(fit);
		if (node.parentElement) ro.observe(node.parentElement);
		fit();
		return { update: fit, destroy: () => ro.disconnect() };
	}
	// hide the shared tooltip if this card disappears while it is hovered
	let hovering = false;
	$effect(() => () => {
		if (hovering) tooltip.hide();
	});
	const tc = $derived({ bg: KIND_INFO[def.kind].color, dark: KIND_INFO[def.kind].dark, icon: KIND_INFO[def.kind].icon, label: KIND_INFO[def.kind].label });
	const shown = $derived(power ?? def.power);
	const diff = $derived(shown - def.power);
	// the ability kind is already shown as a label above the text, so drop the "On Reveal:" etc. prefix
	const bodyText = $derived(def.text.replace(/^(On Reveal|Ongoing|Activate|End of Turn|Game Start):\s*/, ''));

	// power-change effect: replay the animation whenever fxId changes
	let anim = $state<'' | 'buff' | 'hit'>('');
	let shownFx = $state(0);
	let lastFx = 0;
	$effect(() => {
		const id = fxId;
		if (!id || id === lastFx) return;
		lastFx = id;
		const n = fxN;
		const t0 = setTimeout(() => {
			anim = '';
			shownFx = id;
			requestAnimationFrame(() => (anim = n > 0 ? 'buff' : 'hit'));
		}, fxDelay);
		const t = setTimeout(() => (anim = ''), fxDelay + 900);
		return () => {
			clearTimeout(t);
			clearTimeout(t0);
		};
	});
</script>

{#if faceDown}
	<!-- a real playing-card back: border, lattice pattern, gold frame, point-symmetric medallion -->
	<div class="card back {size}" style={wStyle}>
		<div class="inner backface">
			<div class="frame"></div>
			<div class="medal">
				<span class="brick"><i></i><i></i></span>
				<b>BRICK<br />BEASTS</b>
			</div>
		</div>
	</div>
{:else}
<div class="fxwrap {anim}" class:aura style={wStyle}>
	{#if aura}<span class="glow"></span><span class="ring r1"></span><span class="ring r2"></span><span class="ring r3"></span>{/if}
	{#key pulse}
		<div
			class="card {size}"
			class:selected
			class:staged
			class:dim
			class:pop={pulse > 0}
			class:ready
			class:lv1={level === 1}
			class:lv2={level === 2}
			class:lv3={level >= 3}
			class:lv4={level >= 4}
			style="{wStyle}--t:{tc.bg};--td:{tc.dark};--kt:{inkOn(tc.bg).ink}"
		>
			<div class="inner">
				<!-- the picture in two layers: background (gets the holo / glitter) and the beast above the shine -->
				<img src={backdrop || creature ? cardArtBackdrop(id) : cardArt(id, level >= 4)} alt={def.name} draggable="false" />
				{#if creature}<img class="creature" src={creature} alt="" draggable="false" />{/if}
				{#if !hideCost}<div class="cost">{def.cost}</div>{/if}
				<div class="power" class:up={diff > 0} class:down={diff < 0}>{shown}</div>
				<div class="name">{def.name}</div>
				{#if level >= 3}<div class="holo"></div>{/if}
				{#each specks as p}<span class="speck" style="left:{p.x}%; top:{p.y}%; animation-delay:{p.d}s; --s:{p.s}"></span>{/each}
				{#if size === 'lg'}
					<div class="kind"><span class="material-symbols-rounded">{tc.icon}</span>{tc.label}</div>
					<div class="text" use:fitText={bodyText}>{bodyText}</div>
				{:else if def.kind !== 'none'}
					<div
						class="badge"
						role="img"
						aria-label="{tc.label}: {bodyText}"
						onmouseenter={(e) => {
							hovering = true;
							tooltip.show(e.currentTarget, { title: tc.label, text: bodyText, icon: tc.icon, color: tc.dark });
						}}
						onmouseleave={() => {
							hovering = false;
							tooltip.hide();
						}}
					>
						<span class="material-symbols-rounded">{tc.icon}</span>
					</div>
				{:else}
					<!-- no ability: keep the space, so the name sits at the same height on every card -->
					<div class="badge placeholder" aria-hidden="true"></div>
				{/if}
			</div>
		</div>
	{/key}
	{#key shownFx}
		{#if shownFx > 0}
			<div class="fx-flash {fxN > 0 ? 'buff' : 'hit'}"></div>
			<div class="fx-num {fxN > 0 ? 'buff' : 'hit'}">{fxN > 0 ? '+' : ''}{fxN}</div>
		{/if}
	{/key}
</div>
{/if}

<style>
	.card {
		--w: 120px;
		position: relative;
		width: var(--w);
		aspect-ratio: 5 / 7;
		font-size: calc(var(--w) / 10);
		border-radius: 0.6em;
		/* white outer edge like a printed card, then a frame in the ability colour */
		background:
			linear-gradient(150deg, color-mix(in srgb, var(--t) 70%, white) 0%, var(--t) 38%, var(--td) 100%) padding-box,
			#fff border-box;
		border: 0.16em solid transparent;
		padding: 0.3em;
		box-shadow:
			0 0 0 0.06em rgb(0 0 0 / 0.35),
			0 0.25em 0.6em rgb(0 0 0 / 0.35);
		transition:
			transform 0.15s,
			box-shadow 0.15s,
			opacity 0.15s;
		user-select: none;
		flex-shrink: 0;
		container-type: inline-size;
	}
	@container (max-width: 96px) {
		.text {
			display: none;
		}
		.name {
			font-size: 1.25em;
		}
	}
	.sm {
		--w: 58px;
	}
	.md {
		--w: 104px;
	}
	.lg {
		--w: 230px;
	}
	/* a soft foil sheen across the whole card */
	.card:not(.back)::after {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: inherit;
		pointer-events: none;
		background: linear-gradient(125deg, rgb(255 255 255 / 0.35) 0%, transparent 28%, transparent 62%, rgb(255 255 255 / 0.18) 78%, transparent 92%);
	}
	/* ---------- upgrade stages: metal frames instead of the white printed edge ---------- */
	.card.lv1 {
		border-width: 0.22em;
		background:
			linear-gradient(150deg, color-mix(in srgb, var(--t) 70%, white) 0%, var(--t) 38%, var(--td) 100%) padding-box,
			linear-gradient(135deg, #f3c58f, #a8642a 30%, #ffdcae 52%, #8a4e1c 75%, #e0a56b) border-box;
	}
	.card.lv2 {
		border-width: 0.24em;
		background:
			linear-gradient(150deg, color-mix(in srgb, var(--t) 70%, white) 0%, var(--t) 38%, var(--td) 100%) padding-box,
			linear-gradient(135deg, #ffffff, #9aa6b2 28%, #f4f8fb 50%, #7d8995 74%, #e8eef3) border-box;
	}
	.card.lv3 {
		border-width: 0.26em;
		background:
			linear-gradient(150deg, color-mix(in srgb, var(--t) 70%, white) 0%, var(--t) 38%, var(--td) 100%) padding-box,
			linear-gradient(135deg, #fff3a8, #d9a300 26%, #fffbe0 48%, #b07d00 72%, #ffe066) border-box;
		box-shadow:
			0 0 0 0.06em rgb(0 0 0 / 0.35),
			0 0 0.7em rgb(255 210 61 / 0.55),
			0 0.25em 0.6em rgb(0 0 0 / 0.35);
	}
	.card.lv4 {
		animation: voxglow 2.4s ease-in-out infinite;
	}
	@keyframes voxglow {
		50% {
			box-shadow:
				0 0 0 0.06em rgb(0 0 0 / 0.35),
				0 0 1.3em rgb(120 220 255 / 0.85),
				0 0.25em 0.6em rgb(0 0 0 / 0.35);
		}
	}
	/* gold: a rainbow holo foil sweeping over the whole card */
	.holo {
		position: absolute;
		inset: 0;
		pointer-events: none;
		mix-blend-mode: color-dodge;
		opacity: 0.38;
		background: linear-gradient(115deg, transparent 20%, #ff6ad5 32%, #ffd23d 40%, #6affb0 48%, #3dbbff 56%, #b06aff 64%, transparent 76%);
		background-size: 260% 100%;
		animation: holo 3.6s linear infinite;
	}
	@keyframes holo {
		from {
			background-position: 130% 0;
		}
		to {
			background-position: -130% 0;
		}
	}
	/* silver and up: twinkling glitter */
	.speck {
		position: absolute;
		width: calc(0.7em * var(--s));
		height: calc(0.7em * var(--s));
		pointer-events: none;
		background: radial-gradient(circle, #fff 0 18%, transparent 20%), conic-gradient(from 45deg, transparent 0 20%, #fff 22% 28%, transparent 30% 45%, #fff 47% 53%, transparent 55% 70%, #fff 72% 78%, transparent 80%);
		mask: radial-gradient(circle, #000 30%, transparent 72%);
		opacity: 0;
		animation: twinkle 2.2s ease-in-out infinite;
	}
	@keyframes twinkle {
		0%,
		100% {
			opacity: 0;
			scale: 0.3;
		}
		50% {
			opacity: 1;
			scale: 1;
		}
	}
	.inner {
		position: relative;
		height: 100%;
		border-radius: 0.38em;
		overflow: hidden;
		background: linear-gradient(#fffdf7, color-mix(in srgb, var(--t) 18%, #fffaf0));
		box-shadow:
			0 0 0 0.1em rgb(0 0 0 / 0.3),
			inset 0 0 0 0.06em rgb(255 255 255 / 0.8);
	}
	/* the art sits in an inset window */
	img {
		image-rendering: pixelated;
		width: 100%;
		aspect-ratio: 6 / 5;
		object-fit: cover;
		display: block;
		border-radius: 0.3em 0.3em 0.15em 0.15em;
		box-shadow: 0 0.12em 0 rgb(0 0 0 / 0.18);
	}
	.cost,
	.power {
		position: absolute;
		top: 0.15em;
		width: 1.9em;
		height: 1.9em;
		display: grid;
		place-items: center;
		font-weight: 800;
		font-size: 1.15em;
		color: white;
		text-shadow: 0 0.08em 0 rgb(0 0 0 / 0.5);
		/* glossy badge with a white ring */
		border: 0.11em solid #fff;
		box-shadow:
			0 0 0 0.06em rgb(0 0 0 / 0.45),
			0 0.12em 0.25em rgb(0 0 0 / 0.4),
			inset 0 0.25em 0.2em rgb(255 255 255 / 0.35);
	}
	.cost {
		left: 0.15em;
		background: radial-gradient(circle at 35% 30%, #3f86e0, #0a3478 75%);
		border-radius: 50%;
	}
	.power {
		right: 0.15em;
		background: radial-gradient(circle at 35% 30%, #ffc08a, #d9531b 75%);
		border-radius: 0.4em;
	}
	/* power: normal = orange, raised = green, lowered = red */
	.power.up {
		background: radial-gradient(circle at 35% 30%, #b6ff8f, #1f9a12);
		box-shadow: 0 0 0.5em #5cff3a;
	}
	.power.down {
		background: radial-gradient(circle at 35% 30%, #ff9a9a, #c01010);
		box-shadow: 0 0 0.5em #ff3a3a;
	}

	/* ---------- Ongoing aura: rings radiate from the card while it is in play ---------- */
	.ring {
		position: absolute;
		inset: 0;
		border-radius: 0.8em;
		border: 0.18em solid #6fb6ff;
		box-shadow:
			0 0 0.6em #3d9bff,
			inset 0 0 0.6em #3d9bff;
		pointer-events: none;
		opacity: 0;
		animation: radiate 2.4s ease-out infinite;
	}
	.ring.r2 {
		animation-delay: 0.8s;
	}
	.ring.r3 {
		animation-delay: 1.6s;
	}
	@keyframes radiate {
		0% {
			transform: scale(1);
			opacity: 0.9;
		}
		100% {
			transform: scale(1.45);
			opacity: 0;
		}
	}
	/* steady glow behind the card (separate element, so the card's own reveal animation stays intact) */
	.glow {
		position: absolute;
		inset: -0.15em;
		border-radius: 0.75em;
		pointer-events: none;
		box-shadow: 0 0 0.6em 0.2em rgb(61 155 255 / 0.7);
		animation: auraglow 2.4s ease-in-out infinite;
	}
	@keyframes auraglow {
		50% {
			box-shadow: 0 0 1.2em 0.45em rgb(61 155 255 / 0.95);
		}
	}
	.fxwrap.aura .badge .material-symbols-rounded {
		animation: spin 3s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* ---------- power change effects ---------- */
	.fxwrap {
		position: relative;
		display: inline-block;
		width: var(--w);
		font-size: calc(var(--w) / 10);
	}
	.fxwrap.hit {
		animation: shake 0.5s ease-out;
	}
	.fxwrap.buff {
		animation: bounce 0.55s ease-out;
	}
	@keyframes shake {
		0%,
		100% {
			transform: translateX(0);
		}
		15% {
			transform: translateX(-0.5em) rotate(-4deg);
		}
		35% {
			transform: translateX(0.45em) rotate(3deg);
		}
		55% {
			transform: translateX(-0.3em) rotate(-2deg);
		}
		75% {
			transform: translateX(0.15em);
		}
	}
	@keyframes bounce {
		0% {
			transform: scale(1);
		}
		35% {
			transform: scale(1.18) translateY(-0.6em);
		}
		70% {
			transform: scale(0.97);
		}
		100% {
			transform: scale(1);
		}
	}
	.fx-flash {
		position: absolute;
		inset: 0;
		border-radius: 0.6em;
		pointer-events: none;
		animation: flash 0.7s ease-out forwards;
	}
	.fx-flash.buff {
		background: radial-gradient(circle, rgb(140 255 110 / 0.85), rgb(40 200 40 / 0.2) 70%);
		box-shadow: 0 0 1.5em 0.4em #5cff3a;
	}
	.fx-flash.hit {
		background: radial-gradient(circle, rgb(255 90 90 / 0.85), rgb(200 20 20 / 0.25) 70%);
		box-shadow: 0 0 1.5em 0.4em #ff3a3a;
	}
	@keyframes flash {
		from {
			opacity: 1;
		}
		to {
			opacity: 0;
		}
	}
	.fx-num {
		position: absolute;
		left: 50%;
		top: 30%;
		z-index: 5;
		font-size: 3.4em;
		font-weight: 900;
		line-height: 1;
		pointer-events: none;
		-webkit-text-stroke: 0.06em #1b2a34;
		paint-order: stroke fill;
		animation: floatup 1s ease-out forwards;
	}
	.fx-num.buff {
		color: #6dff4a;
		text-shadow: 0 0 0.3em #2a9a1a;
	}
	.fx-num.hit {
		color: #ff4a4a;
		text-shadow: 0 0 0.3em #b01818;
	}
	@keyframes floatup {
		0% {
			transform: translate(-50%, 0.4em) scale(0.4);
			opacity: 0;
		}
		20% {
			transform: translate(-50%, -0.2em) scale(1.25);
			opacity: 1;
		}
		70% {
			opacity: 1;
		}
		100% {
			transform: translate(-50%, -1.6em) scale(1);
			opacity: 0;
		}
	}
	/* name on a little banner that overlaps the bottom of the art window */
	/* stacking: background picture < holo / glitter < the beast < cost, power and labels */
	.holo,
	.speck {
		z-index: 1;
	}
	.creature {
		position: absolute;
		z-index: 2;
		left: 0;
		top: 0;
		pointer-events: none;
		box-shadow: none;
	}
	.cost,
	.power,
	.name,
	.kind,
	.text,
	.badge {
		z-index: 3;
	}
	.kind,
	.text,
	.badge {
		position: relative;
	}
	.name {
		position: relative;
		margin: -0.55em 0.35em 0;
		text-align: center;
		font-weight: 800;
		font-size: 1.05em;
		line-height: 1.45em;
		color: #1b2a34;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		padding: 0 0.3em;
		background: linear-gradient(#ffffff, #f1ebdc);
		border-radius: 0.35em;
		box-shadow:
			0 0 0 0.07em color-mix(in srgb, var(--td) 70%, black),
			0 0.1em 0.2em rgb(0 0 0 / 0.3);
	}
	.lg .name {
		margin-bottom: 0.35em;
	}
	/* small cards (table, hand, deck list): the art fills the space, name and ability symbol sit at the bottom.
	   The big detail card (lg) keeps the art on top and the text below. */
	.card:not(.lg) .inner {
		display: flex;
		flex-direction: column;
	}
	.card:not(.lg) img {
		flex: 1;
		min-height: 0;
		aspect-ratio: auto;
		object-position: 50% 55%;
	}
	.card:not(.lg) .name {
		flex-shrink: 0;
		margin: -0.6em 0.3em 0.3em;
	}
	.card:not(.lg) .badge {
		flex-shrink: 0;
		margin: -0.1em auto 0.3em;
	}
	.card:not(.lg) .name:has(+ .badge) {
		margin-bottom: 0.15em;
	}
	.sm .name {
		font-size: 1.25em;
	}
	.text {
		font-size: 0.78em;
		line-height: 1.2;
		padding: 0 0.5em;
		color: #333;
		text-align: center;
	}
	.lg .text {
		font-size: 0.85em;
	}
	.kind {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.25em;
		margin: 0 auto 0.3em;
		width: fit-content;
		padding: 0.05em 0.6em;
		border-radius: 999px;
		background: var(--t);
		color: var(--kt);
		font-weight: 700;
		font-size: 0.75em;
		
	}
	.kind .material-symbols-rounded {
		font-size: 1.2em;
	}
	/* ability symbol sits in the text area below the name, never on top of the monster */
	.badge {
		margin: 0.05em auto 0;
		width: 1.7em;
		height: 1.7em;
		border-radius: 50%;
		display: grid;
		place-items: center;
		background: var(--t);
		color: var(--kt);
		border: 0.12em solid rgb(255 255 255 / 0.85);
		box-shadow: 0 0.1em 0.3em rgb(0 0 0 / 0.35);
	}
	.badge .material-symbols-rounded {
		font-size: 1.15em;
	}
	.badge.placeholder {
		visibility: hidden;
	}
	.ready {
		box-shadow:
			0 0 0 2px #fff,
			0 0 14px 4px #4fc06a;
		animation: readyglow 1.2s ease-in-out infinite alternate;
	}
	@keyframes readyglow {
		to {
			box-shadow:
				0 0 0 2px #fff,
				0 0 22px 8px #4fc06a;
		}
	}
	.selected {
		transform: translateY(-14px) scale(1.06);
		box-shadow:
			0 0 0 3px #fff,
			0 0 18px 6px #ffd23d;
	}
	.staged {
		opacity: 0.75;
		outline: 2px dashed #fff;
		outline-offset: 2px;
	}
	.dim {
		opacity: 0.45;
		filter: grayscale(0.6);
	}
	.pop {
		animation: pop 0.45s ease-out;
	}
	@keyframes pop {
		0% {
			transform: scale(1.25) rotateY(80deg);
		}
		60% {
			transform: scale(1.1) rotateY(0);
		}
		100% {
			transform: scale(1);
		}
	}
	/* ---------- card back: like a real playing card ---------- */
	.back {
		--td: #8a1006;
		background: #fbf6ea; /* cream outer border */
		border: none;
		padding: 0.32em;
		box-shadow:
			0 0 0 0.06em rgb(0 0 0 / 0.35),
			0 0.25em 0.6em rgb(0 0 0 / 0.35);
	}
	.backface {
		background:
			radial-gradient(ellipse at 50% 50%, rgb(255 255 255 / 0.18), transparent 62%),
			url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='20' height='20'%3E%3Cpath d='M10 0L20 10L10 20L0 10Z' fill='none' stroke='%23e8604c' stroke-width='1.3'/%3E%3Ccircle cx='10' cy='10' r='1.8' fill='%23f2cd37'/%3E%3Ccircle cx='0' cy='0' r='1.2' fill='%23f2cd37'/%3E%3Ccircle cx='20' cy='0' r='1.2' fill='%23f2cd37'/%3E%3Ccircle cx='0' cy='20' r='1.2' fill='%23f2cd37'/%3E%3Ccircle cx='20' cy='20' r='1.2' fill='%23f2cd37'/%3E%3C/svg%3E")
				center / 1.3em 1.3em,
			#a3140a;
		box-shadow: inset 0 0 0 0.06em rgb(0 0 0 / 0.35);
		display: grid;
		place-items: center;
	}
	/* gold double frame line */
	.frame {
		position: absolute;
		inset: 0.3em;
		border-radius: 0.28em;
		border: 0.07em solid #f2cd37;
		box-shadow:
			inset 0 0 0 0.13em #a3140a,
			inset 0 0 0 0.19em #f2cd37;
	}
	/* centre medallion (same upside down) */
	.medal {
		position: relative;
		width: 62%;
		aspect-ratio: 3 / 4;
		border-radius: 50%;
		display: grid;
		place-items: center;
		align-content: center;
		gap: 0.25em;
		background: radial-gradient(circle at 50% 40%, #fffaf0, #f2e3c0);
		border: 0.12em solid #f2cd37;
		box-shadow:
			0 0 0 0.16em #a3140a,
			0 0 0 0.24em #f2cd37,
			0 0.2em 0.4em rgb(0 0 0 / 0.4);
	}
	.medal b {
		font-size: 0.62em;
		font-weight: 900;
		line-height: 1.05;
		text-align: center;
		color: #a3140a;
		letter-spacing: 0.04em;
	}
	/* a tiny red 2x1 LEGO brick as emblem */
	.brick {
		position: relative;
		width: 2.2em;
		height: 1.1em;
		margin-top: 0.3em;
		background: linear-gradient(#e63a26, #b3170a);
		border-radius: 0.15em;
		box-shadow: inset 0 -0.15em 0 rgb(0 0 0 / 0.25);
		display: flex;
		justify-content: space-around;
	}
	.brick i {
		width: 0.65em;
		height: 0.3em;
		margin-top: -0.28em;
		background: linear-gradient(#f05a46, #c9230f);
		border-radius: 0.15em 0.15em 0 0;
	}
	@container (max-width: 60px) {
		.medal b {
			display: none;
		}
	}
</style>
