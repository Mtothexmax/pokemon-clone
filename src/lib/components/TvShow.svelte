<script lang="ts">
	// The TV in the living room: drawn as HTML + animated SVG exactly over the 3D screen while you
	// look at it in first person. Three channels, static noise when switching.
	import { POOL } from '../game/cards';
	import { game } from '../game/state.svelte';
	import { sfx } from '../game/sfx';
	import { playVoice, stopVoice } from '../game/voice';
	import { TV_SCRIPTS, TV_NAMES, type TvLine } from '../game/tvScripts';

	let { rect, onClose, tod = 0.2, rainy = false }: { rect: { left: number; top: number; width: number; height: number }; onClose: () => void; tod?: number; rainy?: boolean } = $props();

	const CHANNELS = ['BBN News', 'Weather', 'Cartoons', 'Wild Bricks'] as const;
	let ch = $state(0);
	let noise = $state(0);
	function zap(d: number) {
		ch = (ch + d + CHANNELS.length) % CHANNELS.length;
		noise++;
		sfx.blip();
	}

	const s = $derived(game.save);
	const news = $derived([
		'Glimmerlings spotted folding light into tiny bricks!',
		`Local trainer owns ${s.collection.length} of ${POOL.length} Brick Beast cards`,
		s.rivalWins ? 'BREAKING: Dex loses again, demands a rematch' : 'Dex claims his deck is "unbeatable"',
		`${s.golden.length} golden studs found so far. Where are the rest?`,
		'Mom reminds everyone: jackets on Route 1!'
	]);
	const ticker = $derived(news.join('   ★   '));
	const night = $derived(tod > 0.55 || tod < 0.05);
	const temp = $derived(Math.round(14 + 8 * Math.sin(tod * Math.PI * 2)));

	// ---- the presenters speak (voiced lines, one after another, with subtitles) ----
	const script = $derived<TvLine[]>(
		ch === 0 ? TV_SCRIPTS.news : ch === 1 ? (rainy ? TV_SCRIPTS.weatherRain : night ? TV_SCRIPTS.weatherNight : TV_SCRIPTS.weatherDay) : ch === 3 ? TV_SCRIPTS.nature : []
	);
	let lineIdx = $state(0);
	const sub = $derived(script[lineIdx % Math.max(1, script.length)]);
	$effect(() => {
		const lines = script;
		lineIdx = 0;
		if (!lines.length) {
			stopVoice();
			return;
		}
		let timer: ReturnType<typeof setTimeout> | undefined;
		let alive = true;
		const say = (i: number) => {
			if (!alive) return;
			lineIdx = i;
			const l = lines[i % lines.length];
			// after the last line the show starts over (after a short break)
			const next = () => alive && (timer = setTimeout(() => say(i + 1), (i + 1) % lines.length ? 700 : 2600));
			// without audio (voices off, tab unfocused): just read along
			if (!playVoice(l.speaker, l.text, next)) timer = setTimeout(() => say(i + 1), 1200 + l.text.length * 55);
		};
		// wait for the channel-switch static to clear
		timer = setTimeout(() => say(0), 500);
		return () => {
			alive = false;
			clearTimeout(timer);
			stopVoice();
		};
	});
</script>

<div class="tv" style="left:{rect.left}px; top:{rect.top}px; width:{rect.width}px; height:{rect.height}px; font-size:{rect.height / 14}px">
	<div class="screen">
		{#if ch === 0}
			<!-- ===== BBN: Brick Beast News ===== -->
			<svg viewBox="0 0 320 176" preserveAspectRatio="xMidYMid slice" class="art">
				<defs>
					<linearGradient id="studio" x1="0" y1="0" x2="0" y2="1">
						<stop offset="0" stop-color="#1e5aa8" />
						<stop offset="1" stop-color="#0b2550" />
					</linearGradient>
				</defs>
				<rect width="320" height="176" fill="url(#studio)" />
				{#each [0, 1, 2, 3, 4, 5] as i}
					<circle cx={30 + i * 52} cy="30" r="14" fill="#ffffff" opacity="0.06" />
				{/each}
				<!-- big screen behind the anchor -->
				<rect x="176" y="22" width="120" height="70" rx="6" fill="#0d1b2a" stroke="#ffd23d" stroke-width="2" />
				<g transform="translate(236 57)">
					<polygon points="0,-22 6,-6 22,-6 9,4 14,20 0,10 -14,20 -9,4 -22,-6 -6,-6" fill="#ffd23d">
						<animateTransform attributeName="transform" type="rotate" from="0" to="360" dur="6s" repeatCount="indefinite" />
					</polygon>
				</g>
				<!-- anchor: a friendly brick beast -->
				<g transform="translate(92 66)">
					<g>
						<animateTransform attributeName="transform" type="translate" values="0 0; 0 -2; 0 0" dur="1.6s" repeatCount="indefinite" />
						<rect x="-30" y="0" width="60" height="58" rx="10" fill="#4fc06a" stroke="#1e7a35" stroke-width="3" />
						{#each [-18, 0, 18] as x}<rect x={x - 6} y="-6" width="12" height="8" rx="3" fill="#4fc06a" stroke="#1e7a35" stroke-width="2" />{/each}
						<circle cx="-12" cy="22" r="7" fill="#fff" />
						<circle cx="12" cy="22" r="7" fill="#fff" />
						<circle cx="-11" cy="23" r="3.5" fill="#1b2a34">
							<animate attributeName="r" values="3.5;3.5;0.4;3.5" keyTimes="0;0.9;0.95;1" dur="3.2s" repeatCount="indefinite" />
						</circle>
						<circle cx="13" cy="23" r="3.5" fill="#1b2a34">
							<animate attributeName="r" values="3.5;3.5;0.4;3.5" keyTimes="0;0.9;0.95;1" dur="3.2s" repeatCount="indefinite" />
						</circle>
						<!-- talking mouth -->
						<ellipse cx="0" cy="40" rx="8" ry="3" fill="#1b2a34">
							<animate attributeName="ry" values="2;6;2;4;2" dur="0.7s" repeatCount="indefinite" />
						</ellipse>
						<!-- tie -->
						<polygon points="-4,50 4,50 6,58 -6,58" fill="#c91a09" />
					</g>
				</g>
				<!-- desk -->
				<rect x="20" y="118" width="150" height="40" rx="4" fill="#c91a09" />
				<rect x="20" y="118" width="150" height="8" fill="#ff5a4a" />
				<text x="95" y="146" text-anchor="middle" font-size="16" font-weight="900" fill="#fff" font-family="Fredoka, sans-serif">BBN</text>
			</svg>
			<div class="lower">
				<div class="live"><span class="dot"></span>LIVE</div>
				<div class="headline">{news[0]}</div>
			</div>
			<div class="ticker"><span>{ticker}   ★   {ticker}</span></div>
		{:else if ch === 1}
			<!-- ===== Weather ===== -->
			<svg viewBox="0 0 320 176" preserveAspectRatio="xMidYMid slice" class="art">
				<rect width="320" height="176" fill={night ? '#16213e' : '#7cc6ff'} />
				{#if night}
					{#each Array(18) as _, i}
						<circle cx={(i * 53) % 320} cy={(i * 37) % 110} r="1.4" fill="#fff">
							<animate attributeName="opacity" values="0.2;1;0.2" dur="{2 + (i % 3)}s" repeatCount="indefinite" />
						</circle>
					{/each}
					<circle cx="230" cy="58" r="26" fill="#fff6c8" />
					<circle cx="242" cy="50" r="24" fill="#16213e" />
				{:else}
					<g transform="translate(230 58)">
						<g>
							<animateTransform attributeName="transform" type="rotate" from="0" to="360" dur="14s" repeatCount="indefinite" />
							{#each Array(12) as _, i}
								<rect x="-3" y="-46" width="6" height="14" rx="3" fill="#ffd23d" transform="rotate({i * 30})" />
							{/each}
						</g>
						<circle r="26" fill="#ffd23d" stroke="#f2a705" stroke-width="3" />
					</g>
				{/if}
				{#each [[40, 50, 18], [120, 80, 24], [280, 110, 16]] as [x, y, d], i}
					<g opacity="0.95">
						<animateTransform attributeName="transform" type="translate" values="-20 0; 20 0; -20 0" dur="{d}s" repeatCount="indefinite" />
						<g transform="translate({x} {y})">
							<ellipse cx="0" cy="0" rx="26" ry="12" fill="#fff" />
							<ellipse cx="-12" cy="-6" rx="14" ry="11" fill="#fff" />
							<ellipse cx="10" cy="-9" rx="16" ry="13" fill="#fff" />
						</g>
					</g>
				{/each}
				<!-- brick hills -->
				<path d="M0 150 Q 60 118 120 146 T 240 140 T 320 136 V176 H0 Z" fill="#3e9a32" />
				<path d="M0 162 Q 80 140 160 160 T 320 154 V176 H0 Z" fill="#4fc06a" />
			</svg>
			<div class="weather">
				<div class="temp">{temp}°</div>
				<div>
					<div class="wtitle">Brickhollow</div>
					<div class="wsub">{night ? 'Clear night · fireflies likely' : temp > 18 ? 'Sunny · perfect for battles' : 'Mild · a few clouds'}</div>
				</div>
			</div>
		{:else if ch === 3}
			<!-- ===== Wild Bricks: a nature documentary ===== -->
			<svg viewBox="0 0 320 176" preserveAspectRatio="xMidYMid slice" class="art">
				<defs>
					<linearGradient id="dusk" x1="0" y1="0" x2="0" y2="1">
						<stop offset="0" stop-color="#ffcf8a" />
						<stop offset="1" stop-color="#ffefc9" />
					</linearGradient>
					<radialGradient id="lens" cx="0.5" cy="0.5" r="0.6">
						<stop offset="0.7" stop-color="#000" stop-opacity="0" />
						<stop offset="1" stop-color="#000" stop-opacity="0.75" />
					</radialGradient>
				</defs>
				<rect width="320" height="176" fill="url(#dusk)" />
				<circle cx="250" cy="60" r="22" fill="#fff3c4" opacity="0.9" />
				<!-- slow camera pan over the grass -->
				<g>
					<animateTransform attributeName="transform" type="translate" values="0 0; -30 0; 0 0" dur="24s" repeatCount="indefinite" />
					<path d="M-20 130 Q 80 100 180 126 T 380 118 V176 H-20 Z" fill="#6bb04a" />
					<!-- the little beast, wobbling along -->
					<g transform="translate(150 118)">
						<g>
							<animateTransform attributeName="transform" type="translate" values="-30 0; -26 -3; -22 0; -18 -3; -14 0; -18 -3; -22 0; -26 -3; -30 0" dur="9s" repeatCount="indefinite" />
							<rect x="-14" y="-18" width="28" height="18" rx="4" fill="#3d9bff" stroke="#1e5aa8" stroke-width="2" />
							{#each [-7, 7] as x}<rect x={x - 4} y="-23" width="8" height="6" rx="2" fill="#3d9bff" stroke="#1e5aa8" stroke-width="1.5" />{/each}
							<circle cx="-5" cy="-10" r="3" fill="#fff" /><circle cx="6" cy="-10" r="3" fill="#fff" />
							<circle cx="-4" cy="-10" r="1.5" fill="#1b2a34">
								<animate attributeName="cx" values="-4;-3;-4;-5;-4" dur="5s" repeatCount="indefinite" />
							</circle>
							<circle cx="7" cy="-10" r="1.5" fill="#1b2a34">
								<animate attributeName="cx" values="7;8;7;6;7" dur="5s" repeatCount="indefinite" />
							</circle>
						</g>
					</g>
					<!-- the lost 2x4 brick -->
					<g transform="translate(196 116)">
						<rect x="-12" y="-6" width="24" height="7" rx="1.5" fill="#c91a09" />
						{#each [-9, -3, 3, 9] as x}<rect x={x - 2} y="-9" width="4" height="3" rx="1" fill="#c91a09" />{/each}
					</g>
					<!-- tall grass in front -->
					{#each Array(26) as _, i}
						<path d="M{i * 14 - 10} 176 q {3 + (i % 3)} -{30 + ((i * 7) % 22)} {(i % 2 ? 6 : -4)} -{44 + ((i * 11) % 26)}" stroke={i % 2 ? '#3e8a2a' : '#4f9e33'} stroke-width="4" fill="none" stroke-linecap="round">
							<animateTransform attributeName="transform" type="rotate" values="0 {i * 14} 176; {i % 2 ? 2 : -2} {i * 14} 176; 0 {i * 14} 176" dur="{3 + (i % 4)}s" repeatCount="indefinite" />
						</path>
					{/each}
				</g>
				<rect width="320" height="176" fill="url(#lens)" />
				<text x="16" y="24" font-size="12" font-weight="700" fill="#fff" opacity="0.85" font-family="Fredoka, sans-serif">WILD BRICKS</text>
			</svg>
		{:else}
			<!-- ===== Cartoons: "Brick & Stud" ===== -->
			<svg viewBox="0 0 320 176" preserveAspectRatio="xMidYMid slice" class="art">
				<rect width="320" height="176" fill="#ffe7a8" />
				<g>
					<animateTransform attributeName="transform" type="translate" from="0 0" to="-160 0" dur="4s" repeatCount="indefinite" />
					{#each Array(6) as _, i}
						<path d="M{i * 80 - 10} 150 q 40 -50 80 0" fill="#f2a705" opacity="0.5" />
					{/each}
				</g>
				<rect y="150" width="320" height="26" fill="#c96a2a" />
				<g>
					<animateTransform attributeName="transform" type="translate" from="0 0" to="-40 0" dur="0.8s" repeatCount="indefinite" />
					{#each Array(10) as _, i}<rect x={i * 40} y="150" width="20" height="5" fill="#a85520" />{/each}
				</g>
				<!-- Brick, bouncing -->
				<g transform="translate(130 0)">
					<g>
						<animateTransform attributeName="transform" type="translate" values="0 120; 0 64; 0 120" keyTimes="0;0.5;1" keySplines="0.2 0.8 0.4 1; 0.6 0 0.8 0.2" calcMode="spline" dur="0.8s" repeatCount="indefinite" />
						<rect x="-24" y="-30" width="48" height="30" rx="5" fill="#c91a09" stroke="#7a0f05" stroke-width="3" />
						{#each [-12, 12] as x}<rect x={x - 7} y="-37" width="14" height="8" rx="3" fill="#c91a09" stroke="#7a0f05" stroke-width="2" />{/each}
						<circle cx="-8" cy="-17" r="4" fill="#fff" /><circle cx="10" cy="-17" r="4" fill="#fff" />
						<circle cx="-7" cy="-17" r="2" fill="#1b2a34" /><circle cx="11" cy="-17" r="2" fill="#1b2a34" />
						<path d="M-6 -8 q 7 6 14 0" stroke="#1b2a34" stroke-width="2.5" fill="none" stroke-linecap="round" />
					</g>
				</g>
				<!-- Stud, rolling after him -->
				<g transform="translate(60 138)">
					<g>
						<animateTransform attributeName="transform" type="rotate" from="0" to="360" dur="1.2s" repeatCount="indefinite" />
						<circle r="12" fill="#ffd23d" stroke="#a87a00" stroke-width="3" />
						<rect x="-6" y="-2" width="12" height="4" rx="2" fill="#a87a00" />
					</g>
				</g>
				<text x="300" y="26" text-anchor="end" font-size="14" font-weight="900" fill="#c91a09" font-family="Fredoka, sans-serif">BRICK &amp; STUD</text>
			</svg>
		{/if}

		<!-- static when switching channels -->
		{#key noise}
			{#if noise > 0}
				<svg class="static" viewBox="0 0 100 60" preserveAspectRatio="none">
					<filter id="tvnoise"><feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" seed="1"><animate attributeName="seed" values="1;9;3;7;2" dur="0.3s" repeatCount="indefinite" /></feTurbulence><feColorMatrix type="saturate" values="0" /></filter>
					<rect width="100" height="60" filter="url(#tvnoise)" />
				</svg>
			{/if}
		{/key}
		{#if sub}
			{#key sub}
				<div class="sub" class:top={ch === 0}><b>{TV_NAMES[sub.speaker]}:</b> {sub.text}</div>
			{/key}
		{/if}
		<div class="scan"></div>
	</div>

	<div class="osd">
		<button onclick={() => zap(-1)} title="Previous channel"><span class="material-symbols-rounded">chevron_left</span></button>
		<span class="chname">CH {ch + 1} · {CHANNELS[ch]}</span>
		<button onclick={() => zap(1)} title="Next channel"><span class="material-symbols-rounded">chevron_right</span></button>
		<span class="grow"></span>
		<button class="off" onclick={onClose} title="Turn off (Esc)"><span class="material-symbols-rounded">power_settings_new</span></button>
	</div>
</div>

<style>
	.tv {
		position: fixed;
		z-index: 35;
		overflow: hidden;
		border-radius: 0.25em;
		background: #000;
		color: #fff;
		font-family: 'Fredoka', system-ui, sans-serif;
		animation: on 0.4s ease-out;
	}
	@keyframes on {
		0% {
			clip-path: inset(49% 0 49% 0);
			filter: brightness(4);
		}
		60% {
			clip-path: inset(0 0 0 0);
		}
	}
	.screen {
		position: absolute;
		inset: 0;
	}
	.art {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}
	.scan {
		position: absolute;
		inset: 0;
		pointer-events: none;
		background: repeating-linear-gradient(0deg, rgb(0 0 0 / 0.12) 0 1px, transparent 1px 3px);
		box-shadow: inset 0 0 2em rgb(0 0 0 / 0.6);
	}
	.static {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
		animation: fadeout 0.45s ease-in forwards;
	}
	@keyframes fadeout {
		to {
			opacity: 0;
		}
	}
	.lower {
		position: absolute;
		left: 4%;
		right: 4%;
		bottom: 2.3em;
		display: flex;
		align-items: stretch;
		font-size: 0.8em;
		box-shadow: 0 0.2em 0.5em rgb(0 0 0 / 0.4);
	}
	.live {
		display: flex;
		align-items: center;
		gap: 0.3em;
		background: #c91a09;
		padding: 0.2em 0.6em;
		font-weight: 900;
	}
	.dot {
		width: 0.5em;
		height: 0.5em;
		border-radius: 50%;
		background: #fff;
		animation: blink 1s infinite;
	}
	@keyframes blink {
		50% {
			opacity: 0.2;
		}
	}
	.headline {
		flex: 1;
		background: #fff;
		color: #1b2a34;
		padding: 0.2em 0.6em;
		font-weight: 700;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.ticker {
		position: absolute;
		left: 0;
		right: 0;
		bottom: 1.15em;
		background: #ffd23d;
		color: #1b2a34;
		font-size: 0.7em;
		font-weight: 700;
		white-space: nowrap;
		overflow: hidden;
		padding: 0.1em 0;
	}
	.ticker span {
		display: inline-block;
		padding-left: 100%;
		animation: scroll 28s linear infinite;
	}
	@keyframes scroll {
		to {
			transform: translateX(-100%);
		}
	}
	.weather {
		position: absolute;
		left: 6%;
		top: 10%;
		display: flex;
		align-items: center;
		gap: 0.5em;
		background: rgb(0 0 0 / 0.35);
		padding: 0.3em 0.8em;
		border-radius: 0.5em;
		text-shadow: 0 0.08em 0.2em rgb(0 0 0 / 0.5);
	}
	.temp {
		font-size: 2.4em;
		font-weight: 900;
		line-height: 1;
	}
	.wtitle {
		font-weight: 900;
	}
	.wsub {
		font-size: 0.75em;
		opacity: 0.9;
	}
	/* subtitles of what is being said */
	.sub {
		position: absolute;
		left: 6%;
		right: 6%;
		bottom: 1.9em;
		padding: 0.25em 0.6em;
		border-radius: 0.3em;
		background: rgb(0 0 0 / 0.62);
		font-size: 0.72em;
		line-height: 1.25;
		text-align: center;
		animation: subin 0.25s ease-out;
	}
	.sub.top {
		top: 4%;
		bottom: auto;
	}
	.sub b {
		color: #ffd23d;
	}
	@keyframes subin {
		from {
			opacity: 0;
			translate: 0 -0.3em;
		}
	}
	.osd {
		position: absolute;
		left: 0;
		right: 0;
		bottom: 0;
		display: flex;
		align-items: center;
		gap: 0.3em;
		padding: 0.1em 0.4em;
		background: rgb(0 0 0 / 0.55);
		font-size: 0.7em;
	}
	.osd button {
		display: grid;
		place-items: center;
		border-radius: 0.3em;
		background: rgb(255 255 255 / 0.15);
	}
	.osd .off {
		background: #c91a09;
	}
	.chname {
		font-weight: 700;
	}
	.grow {
		flex: 1;
	}
</style>
