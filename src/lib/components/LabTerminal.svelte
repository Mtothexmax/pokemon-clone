<script lang="ts">
	// Prof. Quill's lab computer: a read-only "hacker" research terminal drawn over the 3D screen while you look
	// at it in first person. It types by itself (you can't type), next to animated science panels.
	import { onMount } from 'svelte';
	import { CARD_MAP, POOL } from '../game/cards';
	import { game } from '../game/state.svelte';
	import { sfx } from '../game/sfx';

	let { rect, onClose }: { rect: { left: number; top: number; width: number; height: number }; onClose: () => void } = $props();

	const MAX_LINES = 18;
	let lines = $state<{ text: string; cls: string }[]>([]);
	let typing = $state('');
	let typingCls = $state('');

	const hex = (n: number) => Array.from({ length: n }, () => Math.floor(Math.random() * 256).toString(16).padStart(2, '0')).join(' ');
	const pick = <T,>(a: readonly T[]) => a[Math.floor(Math.random() * a.length)];
	const bar = (p: number) => '[' + '#'.repeat(Math.round(p * 20)).padEnd(20, '.') + `] ${Math.round(p * 100)}%`;
	const genome = () => Array.from({ length: 24 }, () => pick(['B', 'R', 'I', 'C', 'K'])).join('');

	/** one "session": a list of [text, css class, is a typed command?] */
	function script(): [string, string, boolean][] {
		const s = game.save;
		const specimen = pick(POOL);
		const d = CARD_MAP[specimen];
		const out: [string, string, boolean][] = [
			[`./analyze --specimen ${specimen}`, 'cmd', true],
			[`loading sample ${specimen.toUpperCase()}.brk ...`, 'dim', false],
			[`  cost ${d.cost} | power ${d.power} | ${d.kind}`, '', false],
			[`  genome  ${genome()}`, 'gene', false],
			[`  ${bar(0.3 + Math.random() * 0.7)} stud resonance`, 'ok', false],
			[`cat /var/log/route1.log | tail -3`, 'cmd', true],
			[`  [${(Math.random() * 99).toFixed(2)}] wild pack detected in tall grass`, 'warn', false],
			[`  [${(Math.random() * 99).toFixed(2)}] glimmer flux ${(Math.random() * 3).toFixed(3)} kBr`, '', false],
			[`hexdump -n 16 /dev/brickcore`, 'cmd', true],
			[`  0000  ${hex(8)}`, 'dim', false],
			[`  0008  ${hex(8)}`, 'dim', false],
			[`./collection --status`, 'cmd', true],
			[`  ${bar(s.collection.length / POOL.length)} ${s.collection.length}/${POOL.length} beasts`, 'ok', false],
			[`  battles won: ${s.wins}  ·  rival defeated: ${s.rivalWins}x`, '', false],
			[pick(['ping dex-house.local', 'sudo make coffee', 'ssh quill@moon-base', 'git log --oneline -1']), 'cmd', true],
			[pick(['  64 bytes from dex-house: "SNAP!" time=1ms', '  [sudo] password for quill: ******  ACCESS DENIED (no coffee)', '  connection refused: the moon is made of bricks', '  a1b2c3d fix: beasts escaping through the vents again']), 'warn', false]
		];
		return out;
	}

	onMount(() => {
		let queue = script();
		let stopped = false;
		let timer: ReturnType<typeof setTimeout>;
		const push = (text: string, cls: string) => {
			lines = [...lines, { text, cls }].slice(-MAX_LINES);
		};
		const next = () => {
			if (stopped) return;
			if (!queue.length) queue = script();
			const [text, cls, isCmd] = queue.shift()!;
			if (!isCmd) {
				push(text, cls);
				timer = setTimeout(next, 120 + Math.random() * 260);
				return;
			}
			// commands are "typed" letter by letter after the prompt
			typingCls = cls;
			let i = 0;
			const typeChar = () => {
				if (stopped) return;
				typing = text.slice(0, ++i);
				if (i % 3 === 0) sfx.blip();
				if (i < text.length) timer = setTimeout(typeChar, 35 + Math.random() * 45);
				else
					timer = setTimeout(() => {
						push(`quill@lab:~$ ${text}`, 'cmd');
						typing = '';
						timer = setTimeout(next, 250);
					}, 280);
			};
			timer = setTimeout(typeChar, 300);
		};
		push('QuillOS 3.7 · Brick Beast Research Terminal', 'title');
		push('read-only session · logged by Prof. Quill', 'dim');
		timer = setTimeout(next, 600);
		return () => {
			stopped = true;
			clearTimeout(timer);
		};
	});

	// animated science panels
	let t = $state(0);
	onMount(() => {
		let raf = 0;
		const loop = (now: number) => {
			t = now / 1000;
			raf = requestAnimationFrame(loop);
		};
		raf = requestAnimationFrame(loop);
		return () => cancelAnimationFrame(raf);
	});
	/** double helix of studs, turning */
	const helix = $derived(
		Array.from({ length: 14 }, (_, i) => {
			const y = 8 + i * 6.4;
			const a = t * 1.6 + i * 0.55;
			return { y, x1: 50 + Math.sin(a) * 30, x2: 50 - Math.sin(a) * 30, z: Math.cos(a) };
		})
	);
	const wave = $derived(
		Array.from({ length: 60 }, (_, i) => {
			const x = (i / 59) * 100;
			const y = 20 + Math.sin(i * 0.35 + t * 4) * 9 * Math.sin(t * 0.7 + i * 0.05) + Math.sin(i * 1.7 + t * 9) * 2;
			return `${x.toFixed(1)},${y.toFixed(1)}`;
		}).join(' ')
	);
</script>

<div class="term" style="left:{rect.left}px; top:{rect.top}px; width:{rect.width}px; height:{rect.height}px; font-size:{rect.height / 34}px">
	<div class="bar">
		<span class="dot r"></span><span class="dot y"></span><span class="dot g"></span>
		<b>quill@lab — research</b>
		<span class="grow"></span>
		<button onclick={onClose} title="Log off (Esc)"><span class="material-symbols-rounded">power_settings_new</span></button>
	</div>
	<div class="body">
		<div class="console">
			{#each lines as l}
				<div class="line {l.cls}">{l.text}</div>
			{/each}
			<div class="line cmd">quill@lab:~$ {typing}<span class="cursor">█</span></div>
		</div>
		<div class="side">
			<div class="panel">
				<div class="ptitle">GENOME HELIX</div>
				<svg viewBox="0 0 100 100" preserveAspectRatio="xMidYMid meet">
					{#each helix as h}
						<line x1={h.x1} y1={h.y} x2={h.x2} y2={h.y} stroke="#1f6f4a" stroke-width="1.2" />
						<circle cx={h.x1} cy={h.y} r={2.4 + h.z * 0.9} fill={h.z > 0 ? '#5dff9d' : '#2a9d60'} />
						<circle cx={h.x2} cy={h.y} r={2.4 - h.z * 0.9} fill={h.z < 0 ? '#5dff9d' : '#2a9d60'} />
					{/each}
				</svg>
			</div>
			<div class="panel">
				<div class="ptitle">STUD RESONANCE <span class="blink">● LIVE</span></div>
				<svg viewBox="0 0 100 40" preserveAspectRatio="none">
					{#each [10, 20, 30] as gy}<line x1="0" y1={gy} x2="100" y2={gy} stroke="#14402c" stroke-width="0.5" />{/each}
					<polyline points={wave} fill="none" stroke="#5dff9d" stroke-width="1.2" />
				</svg>
			</div>
		</div>
	</div>
</div>

<style>
	.term {
		position: fixed;
		z-index: 35;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		background: #03130c;
		color: #5dff9d;
		font-family: 'Cascadia Mono', 'Consolas', 'Courier New', monospace;
		border-radius: 0.3em;
		animation: boot 0.45s ease-out;
		box-shadow: inset 0 0 3em rgb(0 0 0 / 0.8);
	}
	/* CRT feel: scanlines + slight glow */
	.term::after {
		content: '';
		position: absolute;
		inset: 0;
		pointer-events: none;
		background: repeating-linear-gradient(0deg, rgb(0 0 0 / 0.25) 0 1px, transparent 1px 3px);
		mix-blend-mode: multiply;
	}
	@keyframes boot {
		0% {
			clip-path: inset(49% 0 49% 0);
			filter: brightness(4);
		}
		60% {
			clip-path: inset(0 0 0 0);
		}
	}
	.bar {
		display: flex;
		align-items: center;
		gap: 0.4em;
		padding: 0.2em 0.6em;
		background: #0b2a1b;
		color: #9dffc6;
		font-size: 0.9em;
	}
	.dot {
		width: 0.6em;
		height: 0.6em;
		border-radius: 50%;
	}
	.dot.r {
		background: #ff5f56;
	}
	.dot.y {
		background: #ffbd2e;
	}
	.dot.g {
		background: #27c93f;
	}
	.grow {
		flex: 1;
	}
	.bar button {
		display: grid;
		place-items: center;
		background: #c91a09;
		color: #fff;
		border-radius: 0.3em;
	}
	.bar .material-symbols-rounded {
		font-size: 1.2em;
	}
	.body {
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: 1fr 34%;
		gap: 0.5em;
		padding: 0.4em 0.6em;
	}
	.console {
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		overflow: hidden;
		text-shadow: 0 0 0.35em rgb(93 255 157 / 0.6);
	}
	.line {
		white-space: pre;
		overflow: hidden;
		text-overflow: ellipsis;
		line-height: 1.3;
	}
	.line.cmd {
		color: #c8ffe0;
	}
	.line.dim {
		color: #2f9e64;
	}
	.line.warn {
		color: #ffd23d;
	}
	.line.ok {
		color: #5dff9d;
	}
	.line.gene {
		color: #7fd8ff;
		letter-spacing: 0.08em;
	}
	.line.title {
		color: #fff;
		font-weight: 700;
	}
	.cursor {
		animation: blink 1s steps(1) infinite;
	}
	@keyframes blink {
		50% {
			opacity: 0;
		}
	}
	.side {
		display: flex;
		flex-direction: column;
		gap: 0.5em;
		min-height: 0;
	}
	.panel {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		border: 1px solid #1f6f4a;
		border-radius: 0.3em;
		padding: 0.2em 0.3em;
		background: rgb(10 50 30 / 0.4);
	}
	.panel svg {
		flex: 1;
		min-height: 0;
		width: 100%;
	}
	.ptitle {
		font-size: 0.7em;
		color: #9dffc6;
		letter-spacing: 0.08em;
	}
	.blink {
		color: #ff5f56;
		animation: blink 1.2s steps(1) infinite;
	}
</style>
