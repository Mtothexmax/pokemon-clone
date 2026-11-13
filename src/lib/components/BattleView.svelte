<script lang="ts">
	import Card from './Card.svelte';
	import FluentIcon from './FluentIcon.svelte';
	import { CARD_MAP, ZONE_MAP, inkOn } from '../game/cards';
	import type { Battle, CardInst, Result } from '../game/battle.svelte';
	import { sfx } from '../game/sfx';
	import { playVoice } from '../game/voice';
	import { game } from '../game/state.svelte';
	import { tick } from 'svelte';

	let { battle, onDone }: { battle: Battle; onDone: (r: Result, stakes: number) => void } = $props();

	let selected = $state<number | null>(null);
	/** big preview: a card (from hand or board) or a location */
	let inspect = $state<{ card: CardInst; where: 'hand' | 'board' } | { zone: number } | null>(null);
	let logOpen = $state(false);
	// measured slot height -> card width -> zone column width (2 cards side by side) on wide screens
	let slotH = $state(0);
	// hand: cards overlap when there isn't enough width (e.g. 4:3 screens)
	let handW = $state(0);
	let cardW = $state(0);
	const handGap = $derived.by(() => {
		const n = battle.hand.length;
		if (n < 2 || !cardW || !handW) return 6;
		return Math.min(6, (handW - n * cardW) / (n - 1));
	});
	const colW = $derived(Math.max(170, Math.round(((slotH - 14) / 2) * 0.714 * 2 + 5 + 14)));

	const selCard = $derived(battle.hand.find((c) => c.uid === selected) ?? null);
	// sound for power changes
	$effect(() => {
		battle.onFx = (n, target, src) => attackFx(n, target, src);
		battle.onDestroy = (target, src) => destroyFx(target, src);
	});

	// ---------- opponent taunts ----------
	const LINES: Record<string, { start: string[]; turn: string[]; snap: string[]; win: string[]; lose: string[] }> = {
		rival: {
			start: ['Grandma gave YOU a deck? This will be quick.', 'Ready to lose?'],
			turn: ['Is that all you got?', 'My Beasts are way cooler.', 'Watch this!', 'Hmm… lucky draw.'],
			snap: ['SNAP! Double or nothing, slowpoke!', 'SNAP! I like my odds.'],
			win: ['Told you my deck was better!'],
			lose: ['No way… my deck was perfect!']
		},
		practice: {
			start: ['Let’s have a fun practice round!'],
			turn: ['Nice move!', 'Ooh, careful with that zone.', 'Remember: win 2 of 3!'],
			snap: ['Snap! Let’s make it exciting!'],
			win: ['Good try! You’re getting better.'],
			lose: ['Wow, you learn fast!']
		},
		wild: {
			start: ['*rustle rustle*', 'Grrrr!'],
			turn: ['*snarl*', '*brick noises*', 'Rrraah!'],
			snap: ['*SNAP!* (it bares its teeth)'],
			win: ['*victory howl*'],
			lose: ['*runs back into the grass*']
		}
	};
	let taunt = $state<string | null>(null);
	let tauntTimer: ReturnType<typeof setTimeout> | undefined;
	const pickLine = (arr: string[]) => arr[Math.floor(Math.random() * arr.length)];
	function say(line: string, ms = 2600) {
		taunt = line;
		const speaker = battle.opts.kind === 'rival' ? 'Dex' : battle.opts.kind === 'practice' ? 'Lumi' : '';
		if (speaker) playVoice(speaker, line);
		clearTimeout(tauntTimer);
		tauntTimer = setTimeout(() => (taunt = null), ms);
	}
	const lines = $derived(LINES[battle.opts.kind] ?? LINES.wild);
	$effect(() => {
		say(pickLine(lines.start), 3000);
	});
	let lastTurn = 1;
	$effect(() => {
		const t = battle.turn;
		if (t > lastTurn && t < battle.maxTurns && Math.random() < 0.45) say(pickLine(lines.turn));
		lastTurn = t;
	});
	$effect(() => {
		if (battle.aiSnapped) say(pickLine(lines.snap), 3200);
	});

	// ---------- victory: LEGO brick confetti ----------
	let celebrated = false;
	$effect(() => {
		const r = battle.result;
		if (!r || celebrated) return;
		celebrated = true;
		if (r === 'win') {
			sfx.fanfare();
			confetti();
			say(pickLine(lines.lose), 4000);
		} else if (r === 'lose') say(pickLine(lines.win), 4000);
	});
	function confetti() {
		const colors = ['#c91a09', '#f2cd37', '#0055bf', '#237841', '#fe8a18', '#f4f4f4', '#ff8ac0'];
		for (let i = 0; i < 70; i++) {
			const b = document.createElement('div');
			b.className = 'confetti-brick';
			const w = Math.random() < 0.5 ? 2 : 1;
			b.style.setProperty('--c', colors[i % colors.length]);
			b.style.width = `${w * 14}px`;
			b.style.left = `${Math.random() * 100}vw`;
			document.body.appendChild(b);
			const drift = (Math.random() - 0.5) * 240;
			const rot = (Math.random() - 0.5) * 900;
			const dur = 1800 + Math.random() * 1600;
			b.animate(
				[
					{ transform: `translate(0, -40px) rotate(0deg)` },
					{ transform: `translate(${drift}px, 110vh) rotate(${rot}deg)` }
				],
				{ duration: dur, delay: Math.random() * 700, easing: 'cubic-bezier(.3,.1,.6,1)', fill: 'both' }
			);
			setTimeout(() => b.remove(), dur + 800);
		}
	}

	// ---------- attack animation: the source lunges at the target, a projectile flies, then the impact ----------
	const TRAVEL = 320;
	const lastLunge = new Map<number, number>();
	function centerOf(uid: number) {
		const el = document.querySelector(`[data-uid="${uid}"]`) as HTMLElement | null;
		if (!el) return null;
		const r = el.getBoundingClientRect();
		return { el, x: r.left + r.width / 2, y: r.top + r.height / 2 };
	}
	/** the destroyed card is frozen as a copy, gets hit by a projectile and bursts into shards */
	function destroyFx(target: CardInst, src: CardInst) {
		const t = centerOf(target.uid);
		if (!t) return;
		const r = t.el.getBoundingClientRect();
		const ghost = t.el.cloneNode(true) as HTMLElement;
		Object.assign(ghost.style, { position: 'fixed', left: `${r.left}px`, top: `${r.top}px`, width: `${r.width}px`, height: `${r.height}px`, margin: '0', zIndex: '78', pointerEvents: 'none' });
		document.body.appendChild(ghost);
		// projectile + lunge from the attacker, using the same animation as damage
		attackFx(-1, target, src, true);
		setTimeout(() => {
			ghost.remove();
			sfx.shatter();
			const cols = 3, rows = 4;
			const cx = r.left + r.width / 2, cy = r.top + r.height / 2;
			for (let i = 0; i < cols; i++)
				for (let j = 0; j < rows; j++) {
					// jagged clip polygon for this cell
					const j1 = () => (Math.random() - 0.5) * 8;
					const x0 = (i / cols) * 100, x1 = ((i + 1) / cols) * 100, y0 = (j / rows) * 100, y1 = ((j + 1) / rows) * 100;
					const poly = `polygon(${x0 + j1()}% ${y0 + j1()}%, ${x1 + j1()}% ${y0 + j1()}%, ${x1 + j1()}% ${y1 + j1()}%, ${x0 + j1()}% ${y1 + j1()}%)`;
					const shard = t.el.cloneNode(true) as HTMLElement;
					Object.assign(shard.style, { position: 'fixed', left: `${r.left}px`, top: `${r.top}px`, width: `${r.width}px`, height: `${r.height}px`, margin: '0', zIndex: '78', pointerEvents: 'none', clipPath: poly });
					document.body.appendChild(shard);
					const px = r.left + ((i + 0.5) / cols) * r.width, py = r.top + ((j + 0.5) / rows) * r.height;
					const dx = (px - cx) * (2.5 + Math.random() * 2) + (Math.random() - 0.5) * 40;
					const dy = (py - cy) * (2.5 + Math.random() * 2) - 40 - Math.random() * 40;
					const rot = (Math.random() - 0.5) * 540;
					shard.animate(
						[
							{ transform: 'translate(0,0) rotate(0deg)', opacity: 1, filter: 'brightness(1.6)' },
							{ transform: `translate(${dx * 0.6}px, ${dy * 0.6}px) rotate(${rot * 0.6}deg)`, opacity: 1, offset: 0.45 },
							{ transform: `translate(${dx}px, ${dy + 140}px) rotate(${rot}deg)`, opacity: 0, filter: 'brightness(1)' }
						],
						{ duration: 850 + Math.random() * 250, easing: 'cubic-bezier(.2,.7,.4,1)', fill: 'forwards' }
					);
					setTimeout(() => shard.remove(), 1150);
				}
			// flash ring
			const ring = document.createElement('div');
			ring.className = 'impact hit';
			ring.style.left = `${cx}px`;
			ring.style.top = `${cy}px`;
			document.body.appendChild(ring);
			ring.animate([{ transform: 'translate(-50%,-50%) scale(0.4)', opacity: 1 }, { transform: 'translate(-50%,-50%) scale(2.6)', opacity: 0 }], { duration: 500, easing: 'ease-out', fill: 'forwards' });
			setTimeout(() => ring.remove(), 520);
		}, TRAVEL + 120);
	}

	function attackFx(n: number, target: CardInst, src: CardInst, silent = false) {
		const impact = () => (silent ? undefined : n > 0 ? sfx.buff() : sfx.hit());
		const signature = () => sfx.attack(src.id, CARD_MAP[src.id].cost, n > 0);
		if (target.uid === src.uid) {
			signature();
			impact(); // self-boost: just the glow on the card
			return;
		}
		const a = centerOf(src.uid);
		const b = centerOf(target.uid);
		if (!a || !b) {
			signature();
			setTimeout(impact, TRAVEL);
			return;
		}
		const dx = b.x - a.x, dy = b.y - a.y;
		// lunge once per source, even when it hits several cards
		const now = performance.now();
		if (now - (lastLunge.get(src.uid) ?? 0) > 250) {
			lastLunge.set(src.uid, now);
			signature(); // every card has its own attack sound
			const len = Math.hypot(dx, dy) || 1;
			const lx = (dx / len) * 22, ly = (dy / len) * 22;
			a.el.animate(
				[
					{ transform: 'translate(0,0) scale(1)' },
					{ transform: `translate(${-lx * 0.3}px, ${-ly * 0.3}px) scale(0.95)`, offset: 0.25 },
					{ transform: `translate(${lx}px, ${ly}px) scale(1.15)`, offset: 0.55 },
					{ transform: 'translate(0,0) scale(1)' }
				],
				{ duration: 420, easing: 'ease-out' }
			);
		}
		// projectile
		const p = document.createElement('div');
		p.className = `projectile ${n > 0 ? 'buff' : 'hit'}`;
		p.style.left = `${a.x}px`;
		p.style.top = `${a.y}px`;
		document.body.appendChild(p);
		const arc = n > 0 ? -40 : -18;
		p.animate(
			[
				{ transform: 'translate(-50%,-50%) scale(0.4)', opacity: 0.4 },
				{ transform: `translate(calc(-50% + ${dx * 0.5}px), calc(-50% + ${dy * 0.5 + arc}px)) scale(1.2)`, opacity: 1, offset: 0.5 },
				{ transform: `translate(calc(-50% + ${dx}px), calc(-50% + ${dy}px)) scale(0.9)`, opacity: 1 }
			],
			{ duration: TRAVEL, delay: 120, easing: 'ease-in', fill: 'forwards' }
		);
		// impact on a timer (not onfinish) so it always happens, even if animations are throttled
		setTimeout(() => {
			p.remove();
			// impact burst
			const burst = document.createElement('div');
			burst.className = `impact ${n > 0 ? 'buff' : 'hit'}`;
			burst.style.left = `${b.x}px`;
			burst.style.top = `${b.y}px`;
			document.body.appendChild(burst);
			burst.animate([{ transform: 'translate(-50%,-50%) scale(0.3)', opacity: 1 }, { transform: 'translate(-50%,-50%) scale(1.8)', opacity: 0 }], { duration: 380, easing: 'ease-out', fill: 'forwards' });
			setTimeout(() => burst.remove(), 400);
			impact();
		}, TRAVEL + 120);
	}
	/** your own cards show their upgrade stage (the opponent's are plain) */
	const myLv = (id: string) => game.save.cardLevels?.[id] ?? 0;
	const inspCard = $derived(inspect && 'card' in inspect ? inspect : null);
	const inspZone = $derived(inspect && 'zone' in inspect ? inspect.zone : null);

	function clickHand(c: CardInst) {
		// first tap: select it and show it big
		selected = battle.phase === 'plan' ? c.uid : null;
		inspect = { card: c, where: 'hand' };
		sfx.blip();
	}
	function clickZone(i: number) {
		if (selCard && battle.canPlay(selCard, i)) {
			battle.stage(selCard, i);
			sfx.place();
			selected = null;
			return;
		}
		inspect = { zone: i };
	}
	function playTo(i: number) {
		if (selCard && battle.stage(selCard, i)) {
			selected = null;
			sfx.place();
		}
		inspect = null;
	}
	function boardCard(c: CardInst, zone: number) {
		if (c.owner === 'p' && selCard && battle.canPlay(selCard, zone)) {
			clickZone(zone);
			return;
		}
		if (c.revealed || c.owner === 'p') inspect = { card: c, where: 'board' };
	}
	// ---------- drawing: every new hand card visibly flies out of the deck pile ----------
	let deckEl = $state<HTMLElement | null>(null);
	const seenHand = new Set<number>();
	$effect(() => {
		const fresh = battle.hand.filter((c) => !seenHand.has(c.uid)).map((c) => c.uid);
		if (!fresh.length) return;
		for (const uid of fresh) seenHand.add(uid);
		tick().then(() => {
			const from = deckEl?.getBoundingClientRect();
			fresh.forEach((uid, k) => {
				const el = document.querySelector(`.hand-card[data-uid="${uid}"]`) as HTMLElement | null;
				if (!el || !from) return;
				const to = el.getBoundingClientRect();
				const dx = from.left + from.width / 2 - (to.left + to.width / 2);
				const dy = from.top + from.height / 2 - (to.top + to.height / 2);
				const s0 = from.width / to.width;
				const delay = 120 + k * 170;
				el.animate(
					[
						{ transform: `translate(${dx}px, ${dy}px) scale(${s0}) rotate(-12deg)`, opacity: 0.6, zIndex: 40 },
						{ transform: 'translate(0, -34px) scale(1.12) rotate(2deg)', opacity: 1, offset: 0.65, zIndex: 40, filter: 'drop-shadow(0 0 14px #ffe066)' },
						{ transform: 'translate(0, 0) scale(1) rotate(0deg)', opacity: 1, zIndex: 40, filter: 'drop-shadow(0 0 0 transparent)' }
					],
					{ duration: 520, delay, easing: 'cubic-bezier(.2,.8,.3,1)', fill: 'backwards' }
				);
				setTimeout(() => sfx.blip(), delay);
			});
		});
	});

	// ---------- drag & drop: the real card follows the pointer and then flies into its slot ----------
	// works for hand cards (play them) and for cards staged this turn (move them or take them back)
	// Performance: only the rarely changing bits (which card, moved, which zone is hovered) are reactive.
	// The position is written straight to the element's transform once per animation frame, and drop
	// targets are found with rectangles cached at drag start (no layout work while moving).
	type Drag = { c: CardInst; from: 'hand' | 'board'; moved: boolean; over: number | 'hand' | null };
	let drag = $state<Drag | null>(null);
	const motion = { x0: 0, y0: 0, dx: 0, dy: 0, tilt: 0, lastX: 0, raf: 0 };
	let dragEl: HTMLElement | null = null;
	let targets: { zone: number | 'hand'; r: DOMRect }[] = [];
	let suppressClick = false;
	function dragStart(e: PointerEvent, c: CardInst, from: 'hand' | 'board') {
		if (e.button !== 0 || battle.phase !== 'plan') return;
		if (from === 'board' && !c.staged) return;
		if (from === 'hand' && !playable(c)) return;
		dragEl = e.currentTarget as HTMLElement;
		Object.assign(motion, { x0: e.clientX, y0: e.clientY, dx: 0, dy: 0, tilt: 0, lastX: e.clientX });
		drag = { c, from, moved: false, over: null };
		window.addEventListener('pointermove', dragMove, { passive: true });
		window.addEventListener('pointerup', dragEnd);
		window.addEventListener('pointercancel', dragCancel);
	}
	function cacheTargets() {
		targets = [...document.querySelectorAll<HTMLElement>('[data-zone]')].map((el) => ({ zone: Number(el.dataset.zone), r: el.getBoundingClientRect() }));
		const bottom = document.querySelector('.bottom');
		if (bottom) targets.push({ zone: 'hand', r: bottom.getBoundingClientRect() });
	}
	function dropTarget(x: number, y: number): number | 'hand' | null {
		for (const t of targets) if (x >= t.r.left && x <= t.r.right && y >= t.r.top && y <= t.r.bottom) return t.zone;
		return null;
	}
	function lift() {
		return drag?.over === null || drag?.over === 'hand' ? 1.08 : 0.92;
	}
	function paint() {
		motion.raf = 0;
		if (dragEl && drag?.moved) dragEl.style.transform = `translate3d(${motion.dx}px, ${motion.dy}px, 0) rotate(${motion.tilt}deg) scale(${lift()})`;
	}
	function dragMove(e: PointerEvent) {
		if (!drag) return;
		motion.dx = e.clientX - motion.x0;
		motion.dy = e.clientY - motion.y0;
		// tilt with the horizontal speed, like a real card in the hand
		const vx = e.clientX - motion.lastX;
		motion.lastX = e.clientX;
		motion.tilt = Math.max(-14, Math.min(14, motion.tilt * 0.75 + vx * 0.6));
		if (!drag.moved && Math.hypot(motion.dx, motion.dy) > 8) {
			drag.moved = true;
			if (drag.from === 'hand') selected = drag.c.uid; // light up the zones it can go to
			cacheTargets();
			sfx.blip();
		}
		if (!drag.moved) return;
		const over = dropTarget(e.clientX, e.clientY);
		if (over !== drag.over) drag.over = over;
		if (!motion.raf) motion.raf = requestAnimationFrame(paint);
	}
	function stopListening() {
		window.removeEventListener('pointermove', dragMove);
		window.removeEventListener('pointerup', dragEnd);
		window.removeEventListener('pointercancel', dragCancel);
		if (motion.raf) cancelAnimationFrame(motion.raf);
		motion.raf = 0;
	}
	function clearDragStyle() {
		if (dragEl) dragEl.style.transform = '';
	}
	function dragCancel() {
		stopListening();
		clearDragStyle();
		drag = null;
	}
	/** FLIP: the card element (possibly a new one in another container) glides from where it was dropped into its slot */
	async function land(uid: number, from: { cx: number; cy: number; w: number }, placed: boolean, tilt = 0) {
		await tick();
		const el = document.querySelector(`[data-uid="${uid}"]`) as HTMLElement | null;
		if (!el) return;
		// stop the hand's CSS transform transition first, otherwise it would be measured as the target
		for (const a of el.getAnimations()) if (a instanceof CSSTransition) a.cancel();
		const to = el.getBoundingClientRect();
		if (!to.width) return;
		const sx = from.w / to.width;
		const dx = from.cx - (to.left + to.width / 2);
		const dy = from.cy - (to.top + to.height / 2);
		el.animate(
			placed
				? [
						{ transform: `translate(${dx}px, ${dy}px) scale(${sx}) rotate(${tilt}deg)`, zIndex: 75 },
						{ transform: 'translate(0, 0) scale(1.08)', offset: 0.8, zIndex: 75 },
						{ transform: 'translate(0, 0) scale(1)', zIndex: 75 }
					]
				: [
						{ transform: `translate(${dx}px, ${dy}px) scale(${sx})`, zIndex: 75 },
						{ transform: 'translate(0, 0) scale(1)', zIndex: 75 }
					],
			{ duration: placed ? 300 : 240, easing: 'cubic-bezier(.2,.8,.3,1)' }
		);
	}
	function dragEnd(e: PointerEvent) {
		stopListening();
		const d = drag;
		if (!d) return;
		if (!d.moved) {
			drag = null;
			return; // a tap: handled by onclick
		}
		paint();
		suppressClick = true;
		setTimeout(() => (suppressClick = false), 0);
		// where the card is now: centre of its (tilted) box, size without the tilt
		const box = dragEl?.getBoundingClientRect();
		const rect = box && dragEl ? { cx: box.left + box.width / 2, cy: box.top + box.height / 2, w: dragEl.offsetWidth * lift() } : null;
		const target = dropTarget(e.clientX, e.clientY);
		clearDragStyle();
		let placed = false;
		if (d.from === 'hand') {
			if (typeof target === 'number' && battle.stage(d.c, target)) placed = true;
			selected = null;
		} else {
			const orig = d.c.zone;
			if (typeof target === 'number' && target !== orig) {
				battle.unstage(d.c);
				if (battle.stage(d.c, target)) placed = true;
				else if (orig !== undefined) battle.stage(d.c, orig); // not allowed there: back where it was
			} else if (target === 'hand') {
				battle.unstage(d.c);
				placed = true;
			}
		}
		if (placed) sfx.place();
		drag = null;
		if (rect) land(d.c.uid, rect, placed, motion.tilt);
	}

	function closeInspect() {
		inspect = null;
	}
	function endTurn() {
		selected = null;
		battle.endTurn();
	}
	function undoAll() {
		for (const c of [...battle.staged]) battle.unstage(c);
	}
	function onKey(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (inspect) closeInspect();
			else if (logOpen) logOpen = false;
			else selected = null;
		}
		if (e.key === 'Enter' && battle.phase === 'plan' && !inspect) endTurn();
	}
	const playable = (c: CardInst) => CARD_MAP[c.id].cost <= battle.energyLeft && battle.phase === 'plan';
	/** nothing left to do this turn: no card can go anywhere and no Activate is ready */
	const noMoves = $derived(
		battle.phase === 'plan' &&
			!battle.hand.some((c) => [0, 1, 2].some((i) => battle.canPlay(c, i))) &&
			!battle.allCards('p').some((c) => battle.canActivate(c))
	);
	const zoneOk = (i: number) => (selCard ? battle.canPlay(selCard, i) : false);

	// Openmon battle backgrounds: wild = forest, rival = cave, practice = lab
	const bgName = $derived(battle.opts.kind === 'wild' ? 'forest' : battle.opts.kind === 'rival' ? 'cave' : 'lab');

	/** css vars for a colored tile: background, readable text and shadow */
	const tileVars = (color: string, img?: string) => {
		if (img) {
			// picture background: white text on a dark fade, the location color as a frame
			return `--zc:${color};--zbg:linear-gradient(to top, rgb(0 0 0 / 0.72), rgb(0 0 0 / 0.15) 75%), url(/zones/${img}.jpg) center / cover, ${color};--zt:#fff;--zs:0 1px 3px rgb(0 0 0 / 0.9);--zb:${color}`;
		}
		const k = inkOn(color);
		return `--zc:${color};--zt:${k.ink};--zs:${k.shadow}`;
	};

	const resultText: Record<Result, string> = { win: 'VICTORY!', lose: 'DEFEAT', draw: 'DRAW', retreat: 'RETREATED', fold: 'FOLDED' };
	const studDelta = $derived(
		battle.result === 'win' ? battle.stakes : battle.result === 'lose' ? -battle.stakes : battle.result === 'retreat' ? -Math.max(1, Math.floor(battle.stakes / 2)) : 0
	);
</script>

<svelte:window onkeydown={onKey} />

<div class="battle fixed inset-0 z-40 flex flex-col text-white select-none" class:shiny={battle.opts.shiny} style="--bgimg:url(/backgrounds/{bgName}.png)">
	<!-- top bar -->
	<div class="top">
		<div class="avatar"><span class="material-symbols-rounded">{battle.opts.kind === 'wild' ? 'pets' : 'person'}</span></div>
		<div class="min-w-0 leading-tight">
			<div class="font-bold truncate opp">{battle.opts.opponent}</div>
			{#if taunt}
				{#key taunt}<div class="taunt">{taunt}</div>{/key}
			{/if}
			<div class="counts text-[11px] opacity-80 flex gap-2 whitespace-nowrap">
				<span><span class="material-symbols-rounded text-sm">back_hand</span>{battle.aiHand.length}</span>
				<span><span class="material-symbols-rounded text-sm">stacks</span>{battle.aiDeck.length}</span>
			</div>
		</div>

		<div class="mx-auto flex items-center gap-2 shrink-0">
			<div class="stake" class:hot={battle.stakes > 1} title="Stakes">
				<span class="material-symbols-rounded">deployed_code</span>
				<span class="text-lg font-extrabold">{battle.stakes}</span>
			</div>
			<div class="text-sm whitespace-nowrap">T <b class="text-lg">{battle.turn}</b>/{battle.maxTurns}</div>
		</div>

		<button class="btn icon" class:on={logOpen} onclick={() => (logOpen = !logOpen)} title="Battle log">
			<span class="material-symbols-rounded">receipt_long</span>
		</button>
		<button class="btn snap" disabled={battle.pSnapped || battle.phase === 'over'} onclick={() => battle.snap()} title="Snap: double the stakes">
			<FluentIcon name="flash" size={20} /><span class="lbl">SNAP</span>
		</button>
		<button class="btn ghost" disabled={battle.phase === 'over'} onclick={() => battle.retreat()} title="Retreat: lose only half the stakes">
			<FluentIcon name="flag" size={20} /><span class="lbl">Retreat</span>
		</button>
	</div>

	<!-- board: three zones use the full width and height -->
	<div class="board" style="--colw:{colW}px">
		{#each battle.zones as z, i}
			{@const zd = ZONE_MAP[z.id]}
			{@const win = battle.zoneWinner(i)}
			<div class="zone-col">
				<div class="slots enemy" bind:clientHeight={slotH}>
					<div class="grid2">
						{#each z.ai as c (c.uid)}
							<button class="slot-card" data-uid={c.uid} onclick={() => boardCard(c, i)}>
								{#key !c.revealed}<div class="flip"><Card id={c.id} width="var(--cw)" hideCost faceDown={!c.revealed} power={battle.power(c)} pulse={c.pulse} fxN={c.fxN} fxId={c.fxId} fxDelay={c.fxFrom && c.fxFrom !== c.uid ? 440 : 0} aura={c.revealed && CARD_MAP[c.id].kind === 'ongoing'} /></div>{/key}
							</button>
						{/each}
					</div>
				</div>

				<button
					class="zone-tile"
					data-zone={i}
					class:ok={zoneOk(i) || (!!drag?.moved && drag.from === 'board' && drag.c.zone !== i && battle.zoneOpen(i, 'p'))}
					class:hover={!!drag?.moved && drag.over === i}
					style={tileVars(z.revealed ? zd.color : '#555555', z.revealed ? z.id : undefined)}
					onclick={() => clickZone(i)}
				>
					<div class="pw pw-top" class:lead={win === 'ai'}>{battle.zonePower(i, 'ai')}</div>
					{#if z.revealed}
						<div class="zname">{zd.name}</div>
						<div class="ztext">{zd.text}</div>
					{:else}
						<span class="material-symbols-rounded zicon">help</span>
						<div class="zname">Unrevealed</div>
						<div class="ztext">Reveals on turn {i + 1}</div>
					{/if}
					<div class="pw pw-bot" class:lead={win === 'p'}>{battle.zonePower(i, 'p')}</div>
				</button>

				<div class="slots mine" role="list" data-zone={i} class:hover={!!drag?.moved && drag.over === i} onclick={() => clickZone(i)}>
					<div class="grid2">
						{#each z.p as c (c.uid)}
							<button
								class="slot-card"
								data-uid={c.uid}
								class:dragging={drag?.moved && drag.c.uid === c.uid}
								onpointerdown={(e) => dragStart(e, c, 'board')}
								onclick={(e) => {
									e.stopPropagation();
									if (suppressClick) return;
									boardCard(c, i);
								}}
							>
								<!-- after End Turn your own new cards lie face down until they are revealed (tap = details) -->
								{#key !c.revealed && battle.phase !== 'plan'}<div class="flip"><Card id={c.id} width="var(--cw)" hideCost faceDown={!c.revealed && battle.phase !== 'plan'} staged={(c.staged || battle.queued(c)) && battle.phase === 'plan'} power={battle.power(c)} pulse={c.pulse} fxN={c.fxN} fxId={c.fxId} fxDelay={c.fxFrom && c.fxFrom !== c.uid ? 440 : 0} aura={c.revealed && CARD_MAP[c.id].kind === 'ongoing'} ready={battle.canActivate(c)} level={myLv(c.id)} /></div>{/key}
							</button>
						{/each}
					</div>
				</div>
			</div>
		{/each}
	</div>

	<!-- hand + controls -->
	<div class="bottom">
		<div class="energy" title="Energy left / energy this turn (= turn {battle.turn} of {battle.maxTurns})">
			<!-- big: what you can still spend; small: this turn's energy, which is also the turn number -->
			<div class="num">{battle.energyLeft}<small>/{battle.energy}</small></div>
			<div class="lbl2">energy</div>
			<div class="deckpile" bind:this={deckEl} title="Your deck: {battle.deck.length} cards left">
				<span>{battle.deck.length}</span>
			</div>
		</div>

		<div class="hand" bind:clientWidth={handW} style="--gap:{handGap}px">
			{#each battle.hand as c (c.uid)}
				<button
					class="hand-card"
					bind:clientWidth={cardW}
					class:sel={selected === c.uid}
					data-uid={c.uid}
					class:dragging={drag?.moved && drag.c.uid === c.uid}
					onpointerdown={(e) => dragStart(e, c, 'hand')}
					onclick={() => {
						if (!suppressClick) clickHand(c);
					}}
				>
					<Card id={c.id} width="var(--hw)" selected={selected === c.uid} dim={!playable(c)} level={myLv(c.id)} />
				</button>
			{/each}
		</div>

		<div class="controls">
			<button class="btn ghost" disabled={!battle.staged.length || battle.phase !== 'plan'} onclick={undoAll}>
				<FluentIcon name="undo" size={20} /><span class="lbl">Undo</span>
			</button>
			<button class="btn end" class:nudge={noMoves} disabled={battle.phase !== 'plan'} onclick={endTurn}>
				{#if battle.phase === 'reveal'}
					…
				{:else}
					End Turn
				{/if}
			</button>
		</div>
	</div>

	{#if drag?.moved && drag.from === 'board'}
		<div class="drag-hint">Drop on your hand to take it back · or on another location to move it</div>
	{/if}

	{#if noMoves && !inspect}
		<button class="suggest" onclick={endTurn}>
			<span class="material-symbols-rounded">done_all</span>
			<span>No more moves — <b>End Turn</b></span>
		</button>
	{/if}

	{#if battle.phase === 'plan' && battle.turn === 1 && !battle.staged.length && !selCard && !noMoves}
		<div class="hint">Tap a card to see it, then pick a zone to play it.</div>
	{/if}

	{#if logOpen}
		<div class="log-drawer">
			<div class="flex items-center mb-2">
				<b>Battle log</b>
				<button class="ml-auto" onclick={() => (logOpen = false)}><FluentIcon name="dismiss" size={18} /></button>
			</div>
			<div class="overflow-auto text-xs space-y-1 flex-1">
				{#each battle.log as l}
					<div class="opacity-90">• {l}</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if inspCard}
		{@const c = inspCard.card}
		<div class="inspect" role="presentation" onclick={closeInspect}>
			<div class="insp-inner" role="presentation" onclick={(e) => e.stopPropagation()}>
				<Card id={c.id} size="lg" width="min(72vw, 42dvh, 290px)" power={inspCard.where === 'board' ? battle.power(c) : null} level={c.owner === 'p' ? myLv(c.id) : 0} />
				<div class="insp-actions">
					{#if inspCard.where === 'hand' && battle.phase === 'plan' && battle.hand.some((h) => h.uid === c.uid)}
						{#if CARD_MAP[c.id].cost > battle.energyLeft}
							<div class="insp-note">Not enough energy ({CARD_MAP[c.id].cost} needed)</div>
						{:else}
							<div class="insp-note">Play to:</div>
							<div class="zone-btns">
								{#each battle.zones as z, i}
									<button class="zbtn" style={tileVars(z.revealed ? ZONE_MAP[z.id].color : '#555555')} disabled={!battle.canPlay(c, i)} onclick={() => playTo(i)}>
										{z.revealed ? ZONE_MAP[z.id].name : `Zone ${i + 1}`}
									</button>
								{/each}
							</div>
						{/if}
					{:else if inspCard.where === 'board' && c.owner === 'p' && c.staged}
						<button class="btn ghost" onclick={() => { battle.unstage(c); closeInspect(); }}><FluentIcon name="undo" size={20} /> Take back</button>
					{:else if inspCard.where === 'board' && c.owner === 'p' && battle.canActivate(c)}
						<button class="btn activate" onclick={() => { battle.activate(c); closeInspect(); }}>
							<span class="material-symbols-rounded">touch_app</span> Activate
						</button>
					{:else if inspCard.where === 'board' && c.owner === 'p' && battle.queued(c) && battle.phase === 'plan'}
						<div class="insp-note">Activates when the turn is revealed, in the order you played</div>
						<button class="btn ghost" onclick={() => { battle.cancelActivate(c); closeInspect(); }}><FluentIcon name="undo" size={20} /> Cancel activate</button>
					{:else if CARD_MAP[c.id].kind === 'activate' && c.activated}
						<div class="insp-note">Already activated</div>
					{/if}
					<button class="btn ghost" onclick={closeInspect}><FluentIcon name="dismiss" size={20} /> Close</button>
				</div>
			</div>
		</div>
	{/if}

	{#if inspZone !== null}
		{@const z = battle.zones[inspZone]}
		{@const zd = ZONE_MAP[z.id]}
		<div class="inspect" role="presentation" onclick={closeInspect}>
			<div class="zone-big" style={tileVars(z.revealed ? zd.color : '#555555', z.revealed ? z.id : undefined)}>
				{#if !z.revealed}<span class="material-symbols-rounded big-icon">help</span>{/if}
				<div class="text-3xl font-black">{z.revealed ? zd.name : 'Unrevealed'}</div>
				<div class="text-lg mt-1 opacity-95">{z.revealed ? zd.text : `This location reveals on turn ${inspZone + 1}.`}</div>
				<div class="flex gap-6 justify-center mt-5 text-center">
					<div><div class="text-xs opacity-80">{battle.opts.opponent}</div><div class="text-3xl font-black">{battle.zonePower(inspZone, 'ai')}</div></div>
					<div><div class="text-xs opacity-80">You</div><div class="text-3xl font-black">{battle.zonePower(inspZone, 'p')}</div></div>
				</div>
				<div class="text-sm opacity-80 mt-4">Cards here: {z.p.length}/{battle.capacity(inspZone)} yours · {z.ai.length}/{battle.capacity(inspZone)} theirs</div>
			</div>
			<span class="text-sm opacity-80 mt-3">Tap anywhere to close</span>
		</div>
	{/if}

	{#if battle.phase === 'over' && battle.result}
		<div class="absolute inset-0 bg-black/60 grid place-items-center z-50 p-3">
			<div class="result-box">
				<div class="text-4xl sm:text-5xl font-black tracking-wide {battle.result === 'win' ? 'text-yellow-300' : 'text-white'}">
					{resultText[battle.result]}
				</div>
				<div class="flex gap-4 sm:gap-6 justify-center my-4">
					{#each battle.zones as z, i}
						{@const w = battle.zoneWinner(i)}
						<div class="text-center">
							<div class="text-xs sm:text-sm opacity-80">{ZONE_MAP[z.id].name}</div>
							<div class="text-xl sm:text-2xl font-bold">{battle.zonePower(i, 'p')} : {battle.zonePower(i, 'ai')}</div>
							<span class="material-symbols-rounded {w === 'p' ? 'text-green-400' : w === 'ai' ? 'text-red-400' : 'opacity-60'}">
								{w === 'p' ? 'check_circle' : w === 'ai' ? 'cancel' : 'remove'}
							</span>
						</div>
					{/each}
				</div>
				<div class="text-xl mb-4 flex items-center justify-center gap-2">
					<span class="material-symbols-rounded text-yellow-300">toll</span>
					{studDelta >= 0 ? '+' : ''}{studDelta} Studs
				</div>
				<button class="btn end text-lg px-8" onclick={() => onDone(battle.result!, battle.stakes)}>Continue</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.battle {
		height: 100dvh;
		background:
			linear-gradient(rgb(8 16 12 / 0.35), rgb(8 16 12 / 0.55)),
			var(--bgimg) center / cover no-repeat,
			#1a4a28;
		image-rendering: pixelated;
		font-family: 'Fredoka', system-ui, sans-serif;
		animation: enter 0.35s ease-out;
		padding: env(safe-area-inset-top) env(safe-area-inset-right) env(safe-area-inset-bottom) env(safe-area-inset-left);
		--hw: clamp(58px, min(15vw, 17dvh), 128px);
	}
	@keyframes enter {
		from {
			opacity: 0;
			transform: scale(1.03);
		}
	}
	.top {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		background: rgb(0 0 0 / 0.3);
	}
	.opp {
		font-size: clamp(0.85rem, 3.5vw, 1.1rem);
		max-width: 34vw;
	}
	/* shiny battle: golden shimmer over the whole board */
	.battle.shiny::after {
		content: '';
		position: absolute;
		inset: 0;
		pointer-events: none;
		z-index: 0;
		background:
			radial-gradient(circle at 20% 30%, rgb(255 230 120 / 0.35), transparent 30%),
			radial-gradient(circle at 80% 60%, rgb(255 210 80 / 0.3), transparent 35%),
			linear-gradient(115deg, transparent 30%, rgb(255 240 170 / 0.25) 45%, transparent 60%);
		background-size: 100% 100%, 100% 100%, 300% 100%;
		animation: shimmer 4s linear infinite;
		mix-blend-mode: screen;
	}
	@keyframes shimmer {
		from {
			background-position: 0 0, 0 0, 150% 0;
		}
		to {
			background-position: 0 0, 0 0, -150% 0;
		}
	}
	.taunt {
		position: absolute;
		top: 48px;
		left: 14px;
		z-index: 60;
		max-width: min(70vw, 340px);
		padding: 8px 14px;
		border-radius: 14px;
		background: #fff;
		color: #1b2a34;
		font-weight: 700;
		font-size: 0.95rem;
		border: 3px solid #1b2a34;
		box-shadow: 0 6px 16px rgb(0 0 0 / 0.35);
		white-space: normal;
		animation: tauntIn 0.25s ease-out;
	}
	.taunt::before {
		content: '';
		position: absolute;
		top: -12px;
		left: 18px;
		border: 9px solid transparent;
		border-bottom-color: #1b2a34;
		border-top: 0;
	}
	@keyframes tauntIn {
		from {
			transform: translateY(-6px) scale(0.9);
			opacity: 0;
		}
	}
	:global(.confetti-brick) {
		position: fixed;
		top: 0;
		z-index: 90;
		height: 10px;
		border-radius: 2px;
		pointer-events: none;
		background: var(--c);
		box-shadow:
			inset 0 -3px 0 rgb(0 0 0 / 0.2),
			0 2px 4px rgb(0 0 0 / 0.25);
	}
	:global(.confetti-brick)::before {
		content: '';
		position: absolute;
		top: -4px;
		left: 2px;
		right: 2px;
		height: 5px;
		background: radial-gradient(circle, var(--c) 0 3px, transparent 3.5px) 0 0 / 14px 5px repeat-x;
		filter: brightness(1.1);
	}
	.avatar {
		width: 36px;
		height: 36px;
		flex-shrink: 0;
		border-radius: 10px;
		display: grid;
		place-items: center;
		background: #7b2fbe;
		border: 2px solid rgb(255 255 255 / 0.6);
	}
	.stake {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 1px 10px;
		border-radius: 999px;
		background: rgb(0 0 0 / 0.35);
		border: 2px solid rgb(255 255 255 / 0.4);
	}
	.stake.hot {
		background: linear-gradient(#ff9a3d, #d9361b);
		box-shadow: 0 0 14px #ff9a3d;
	}
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		padding: 7px 11px;
		border-radius: 10px;
		font-weight: 700;
		border: 2px solid rgb(0 0 0 / 0.3);
		box-shadow: inset 0 -3px 0 rgb(0 0 0 / 0.25);
		transition: transform 0.1s, filter 0.1s;
		white-space: nowrap;
	}
	.btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}
	.btn:disabled {
		opacity: 0.4;
	}
	.btn.icon {
		padding: 6px 8px;
		background: rgb(255 255 255 / 0.12);
	}
	.btn.icon.on {
		background: rgb(255 255 255 / 0.35);
	}
	.btn.snap {
		background: linear-gradient(#ffd23d, #f0a020);
		color: #3a2a00;
	}
	.btn.ghost {
		background: rgb(255 255 255 / 0.15);
	}
	.btn.end {
		background: linear-gradient(#3d9bff, #1655b0);
		font-size: 1.05rem;
		padding: 12px 16px;
	}

	/* ---------- board ---------- */
	.board {
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: clamp(4px, 1.2vw, 14px);
		padding: 8px clamp(4px, 1.2vw, 14px);
	}
	.zone-col {
		min-width: 0;
		min-height: 0;
		display: grid;
		grid-template-rows: minmax(0, 1fr) auto minmax(0, 1fr);
		gap: 16px;
	}
	.slots {
		container-type: size;
		min-height: 0;
		border-radius: 10px;
		display: flex;
		justify-content: center;
		padding: 4px;
	}
	.slots.enemy {
		align-items: flex-end;
		background: rgb(0 0 0 / 0.14);
	}
	.slots.mine {
		align-items: flex-start;
		background: rgb(255 255 255 / 0.07);
	}
	.grid2 {
		/* card width that fits a 2x2 grid in the available space */
		--cw: min(calc((100cqw - 14px) / 2), calc((100cqh - 14px) / 2 * 0.714));
		display: grid;
		grid-template-columns: repeat(2, var(--cw));
		gap: 5px;
	}
	.zone-tile {
		position: relative;
		width: 100%;
		min-height: clamp(78px, 15dvh, 130px);
		border-radius: 14px;
		background: var(
			--zbg,
			radial-gradient(circle, rgb(255 255 255 / 0.25) 0 18%, transparent 20%) 0 0 / 18px 18px,
			var(--zc)
		);
		border: 3px solid var(--zb, rgb(0 0 0 / 0.35));
		justify-content: flex-end !important;
		box-shadow:
			inset 0 -6px 0 rgb(0 0 0 / 0.2),
			0 4px 10px rgb(0 0 0 / 0.3);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-align: center;
		padding: 16px 4px;
		color: var(--zt, #fff);
		text-shadow: var(--zs);
		transition: box-shadow 0.15s, transform 0.15s;
	}
	.zicon {
		font-size: clamp(20px, 4vw, 32px);
	}
	.zname {
		font-weight: 800;
		line-height: 1.1;
		font-size: clamp(0.72rem, 2.6vw, 1.05rem);
	}
	.ztext {
		font-size: clamp(9px, 2.2vw, 12px);
		line-height: 1.15;
		opacity: 0.95;
	}
	.zone-tile.ok {
		box-shadow:
			0 0 0 3px #fff,
			0 0 22px 6px #ffd23d;
		transform: scale(1.03);
	}
	.pw {
		position: absolute;
		left: 50%;
		transform: translateX(-50%);
		min-width: 34px;
		padding: 0 7px;
		border-radius: 999px;
		background: #1b2a34;
		color: #fff;
		text-shadow: none;
		border: 2px solid rgb(255 255 255 / 0.5);
		font-weight: 800;
		font-size: 1rem;
	}
	.pw-top {
		top: -13px;
	}
	.pw-bot {
		bottom: -13px;
	}
	.pw.lead.pw-top {
		background: #d9361b;
		box-shadow: 0 0 12px #ff5a3d;
	}
	.pw.lead.pw-bot {
		background: #f0a020;
		box-shadow: 0 0 12px #ffd23d;
		color: #2a1a00;
	}

	/* ---------- bottom ---------- */
	.bottom {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto;
		grid-template-areas: 'energy hand controls';
		align-items: stretch;
		gap: 10px;
		padding: 6px 10px 10px;
		/* height of the hand row = card height + lift room */
		--rowh: calc(var(--hw) * 1.4 + 18px);
		background: rgb(0 0 0 / 0.2);
	}
	.energy {
		grid-area: energy;
		/* fills the full height of the hand row */
		height: var(--rowh);
		width: var(--rowh);
		align-self: end;
		border-radius: 50%;
		display: grid;
		place-content: center;
		text-align: center;
		background: radial-gradient(circle at 35% 30%, #7ac0ff, #1655b0);
		border: 4px solid rgb(255 255 255 / 0.7);
		box-shadow: 0 0 20px rgb(61 155 255 / 0.6);
	}
	/* your deck: a small stack of card backs on the energy orb; drawn cards fly out of it */
	.energy {
		position: relative;
	}
	.deckpile {
		position: absolute;
		right: -14%;
		top: -10%;
		width: calc(var(--rowh) * 0.34);
		min-width: 26px;
		aspect-ratio: 5 / 7;
		border-radius: 6px;
		border: 2px solid #fff;
		background: repeating-linear-gradient(45deg, #7b2fbe 0 6px, #5a1f94 6px 12px);
		box-shadow: 2px 2px 0 #d8d8e8, 4px 4px 0 #9a9ab0, 0 4px 10px rgb(0 0 0 / 0.5);
		display: grid;
		place-items: center;
		pointer-events: auto;
	}
	.deckpile span {
		font-weight: 900;
		font-size: calc(var(--rowh) * 0.13);
		min-width: 1.6em;
		text-align: center;
		background: rgb(0 0 0 / 0.55);
		border-radius: 999px;
		line-height: 1.4;
	}
	.energy .num {
		font-size: calc(var(--rowh) * 0.42);
		font-weight: 900;
		line-height: 1;
	}
	.energy .num small {
		font-size: 0.4em;
		font-weight: 700;
		opacity: 0.75;
		margin-left: 0.05em;
	}
	.lbl2 {
		font-size: calc(var(--rowh) * 0.1);
		text-transform: uppercase;
		letter-spacing: 0.08em;
		opacity: 0.85;
	}
	.hand {
		grid-area: hand;
		display: flex;
		justify-content: center;
		align-items: flex-end;
		min-width: 0;
		padding-top: 18px;
	}
	.hand-card {
		position: relative;
		transition: transform 0.15s, margin 0.15s;
		margin: 0;
	}
	.hand-card + .hand-card {
		margin-left: var(--gap, 6px);
	}
	.hand-card:hover,
	.hand-card.sel {
		z-index: 2;
	}
	.hand-card.sel {
		z-index: 2;
	}
	.info {
		position: absolute;
		top: -14px;
		right: -6px;
		background: #fff;
		color: #1655b0;
		border-radius: 50%;
		font-size: 24px;
		box-shadow: 0 2px 6px rgb(0 0 0 / 0.4);
		z-index: 3;
	}
	.controls {
		grid-area: controls;
		display: flex;
		flex-direction: column;
		gap: 6px;
		/* the button group fills the full height of the hand row */
		height: var(--rowh);
		align-self: end;
		min-width: 150px;
	}
	.controls .btn {
		flex: 1;
	}
	.controls .btn.end {
		flex: 2;
		font-size: 1.25rem;
	}
	/* wide screens (16:9 etc.): each location is only as wide as two cards, so they sit closer together */
	@media (min-aspect-ratio: 16/10) {
		.board {
			grid-template-columns: repeat(3, var(--colw));
			justify-content: center;
			gap: 18px;
		}
	}
	.hint {
		position: absolute;
		left: 50%;
		bottom: calc(var(--hw) * 1.4 + 40px);
		transform: translateX(-50%);
		background: rgb(0 0 0 / 0.65);
		padding: 5px 12px;
		border-radius: 999px;
		font-size: 0.85rem;
		pointer-events: none;
		white-space: nowrap;
	}
	.log-drawer {
		position: absolute;
		top: 52px;
		right: 8px;
		width: min(320px, 86vw);
		max-height: 55dvh;
		display: flex;
		flex-direction: column;
		background: rgb(16 28 36 / 0.95);
		border: 2px solid rgb(255 255 255 / 0.25);
		border-radius: 14px;
		padding: 10px 12px;
		z-index: 45;
	}
	.inspect {
		position: absolute;
		inset: 0;
		z-index: 55;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		background: rgb(0 0 0 / 0.65);
		backdrop-filter: blur(3px);
		animation: enter 0.15s ease-out;
	}
	.slot-card,
	.hand-card {
		touch-action: none;
	}
	/* a card turning over (face down after End Turn, face up when revealed) */
	.flip {
		animation: flip 0.38s ease-out;
		transform-origin: 50% 50%;
	}
	@keyframes flip {
		from {
			transform: perspective(500px) rotateY(90deg) scale(1.08);
		}
		to {
			transform: perspective(500px) rotateY(0) scale(1);
		}
	}
	/* the real card is lifted and follows the pointer (no ghost copy) */
	.slot-card.dragging,
	.hand-card.dragging {
		position: relative;
		z-index: 75;
		pointer-events: none;
		transition: none;
		will-change: transform;
		filter: drop-shadow(0 14px 22px rgb(0 0 0 / 0.55));
		cursor: grabbing;
	}
	.zone-tile.hover,
	.slots.mine.hover {
		box-shadow: 0 0 0 3px #fff, 0 0 24px 6px rgb(255 255 255 / 0.55);
		border-radius: 12px;
	}
	.drag-hint {
		position: absolute;
		left: 50%;
		bottom: calc(var(--hw) * 1.4 + 46px);
		transform: translateX(-50%);
		z-index: 31;
		background: rgb(0 0 0 / 0.7);
		padding: 6px 14px;
		border-radius: 999px;
		font-size: 0.85rem;
		white-space: nowrap;
		pointer-events: none;
	}
	.suggest {
		position: absolute;
		left: 50%;
		bottom: calc(var(--hw) * 0.55);
		transform: translateX(-50%);
		z-index: 30;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 12px 22px;
		border-radius: 999px;
		font-size: 1.1rem;
		white-space: nowrap;
		background: linear-gradient(#3d9bff, #1655b0);
		border: 3px solid #fff;
		box-shadow: 0 0 0 4px rgb(61 155 255 / 0.35), 0 8px 24px rgb(0 0 0 / 0.4);
		animation: suggest 1s ease-in-out infinite alternate;
	}
	@keyframes suggest {
		to {
			box-shadow: 0 0 0 10px rgb(61 155 255 / 0.15), 0 8px 24px rgb(0 0 0 / 0.4);
			transform: translateX(-50%) translateY(-3px);
		}
	}
	.btn.end.nudge {
		box-shadow: 0 0 0 3px #fff, 0 0 22px 6px #3d9bff;
		animation: suggest-btn 0.9s ease-in-out infinite alternate;
	}
	@keyframes suggest-btn {
		to {
			filter: brightness(1.25);
		}
	}
	.insp-inner {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
	}
	.insp-actions {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		width: min(92vw, 420px);
	}
	.insp-note {
		font-size: 0.9rem;
		opacity: 0.9;
	}
	.zone-btns {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 6px;
		width: 100%;
	}
	.zbtn {
		padding: 10px 6px;
		border-radius: 12px;
		font-weight: 800;
		font-size: 0.85rem;
		line-height: 1.1;
		background: var(--zc);
		border: 3px solid rgb(0 0 0 / 0.35);
		box-shadow: inset 0 -4px 0 rgb(0 0 0 / 0.2);
		color: var(--zt, #fff);
		text-shadow: var(--zs);
	}
	.zbtn:disabled {
		opacity: 0.35;
	}
	.btn.activate {
		background: linear-gradient(#5fd07a, #1e7a35);
		font-size: 1.05rem;
		padding: 10px 20px;
	}
	.zone-big {
		width: min(88vw, 380px);
		padding: 28px 20px;
		border-radius: 22px;
		text-align: center;
		min-height: 260px;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		background: var(
			--zbg,
			radial-gradient(circle, rgb(255 255 255 / 0.25) 0 18%, transparent 20%) 0 0 / 22px 22px,
			var(--zc)
		);
		border: 4px solid var(--zb, rgb(0 0 0 / 0.35));
		box-shadow:
			inset 0 -8px 0 rgb(0 0 0 / 0.2),
			0 12px 30px rgb(0 0 0 / 0.4);
		color: var(--zt, #fff);
		text-shadow: var(--zs);
		animation: enter 0.15s ease-out;
	}
	.big-icon {
		font-size: 64px;
	}
	.result-box {
		background: linear-gradient(#2a3a5a, #1b2a34);
		border: 4px solid rgb(255 255 255 / 0.3);
		border-radius: 20px;
		padding: 24px clamp(16px, 5vw, 40px);
		text-align: center;
	}

	/* ---------- phones ---------- */
	@media (max-width: 640px) {
		.lbl,
		.counts {
			display: none;
		}
		.top {
			gap: 6px;
			padding: 6px;
		}
		.board {
			gap: 4px;
			padding: 6px 4px;
		}
		.bottom {
			grid-template-columns: auto 1fr;
			grid-template-areas:
				'hand hand'
				'energy controls';
			align-items: center;
			padding-top: 2px;
		}
		.energy {
			width: 54px;
			height: 54px;
		}
		.energy .num {
			font-size: 1.4rem;
		}
		.controls {
			flex-direction: row;
			justify-content: flex-end;
			height: 54px;
			min-width: 0;
		}
		.controls .btn {
			flex: 0 0 auto;
		}
		.controls .btn.end {
			flex: 1;
			max-width: 220px;
			font-size: 1.05rem;
		}
		.lbl2 {
			font-size: 8px;
		}
		.battle {
			--hw: clamp(52px, 19vw, 96px);
		}
		.hand-card {
			margin: 0;
		}
		.hand-card.sel {
			transform: translateY(-6px);
		}
		.zone-col {
			gap: 14px;
		}
	}
	@media (max-height: 520px) and (orientation: landscape) {
		.battle {
			--hw: clamp(48px, 10vw, 80px);
		}
		.zone-tile {
			min-height: 64px;
			padding: 12px 4px;
		}
		.zone-col {
			gap: 12px;
		}
	}
	:global(.projectile) {
		position: fixed;
		z-index: 80;
		width: 34px;
		height: 34px;
		border-radius: 50%;
		pointer-events: none;
	}
	:global(.projectile.hit) {
		background: radial-gradient(circle at 40% 40%, #fff6c8 0 18%, #ffb020 35%, #ff3a1a 65%, rgb(255 40 0 / 0) 72%);
		box-shadow: 0 0 18px 8px rgb(255 80 20 / 0.7);
	}
	:global(.projectile.buff) {
		background: radial-gradient(circle at 40% 40%, #ffffff 0 18%, #b6ff8f 40%, #2fd02a 62%, rgb(40 200 40 / 0) 72%);
		box-shadow: 0 0 18px 8px rgb(90 255 60 / 0.7);
	}
	:global(.impact) {
		position: fixed;
		z-index: 79;
		width: 90px;
		height: 90px;
		border-radius: 50%;
		pointer-events: none;
	}
	:global(.impact.hit) {
		background: radial-gradient(circle, rgb(255 230 120 / 0.95) 0 20%, rgb(255 70 30 / 0.7) 45%, rgb(255 0 0 / 0) 70%);
	}
	:global(.impact.buff) {
		background: radial-gradient(circle, rgb(255 255 255 / 0.95) 0 18%, rgb(120 255 90 / 0.7) 45%, rgb(0 255 0 / 0) 70%);
	}
</style>
