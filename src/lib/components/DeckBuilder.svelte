<script lang="ts">
	// Marvel-Snap-style decks: a list of decks; an opened deck shows its 12 cards in one row on top
	// and the whole collection below (sized so everything fits without scrolling).
	// Add: tap / double-click / drag into the deck row. Remove: right-click / double-click / drag out.
	import { tick } from 'svelte';
	import Card from './Card.svelte';
	import FluentIcon from './FluentIcon.svelte';
	import { CARD_MAP, DECK_SIZE, POOL } from '../game/cards';
	import { game } from '../game/state.svelte';
	import { sfx } from '../game/sfx';
	import { STAGES, VOXEL_LEVEL, canBecomeVoxel, maxLevel } from '../game/levels';

	import ModelViewer from './ModelViewer.svelte';
	import Card3D from './Card3D.svelte';
	import { hasVoxelModel } from '../three/voxel';
	import { CARDS } from '../game/cards';

	let { onClose, onVoxelmon, debug = false }: { onClose: () => void; /** a card just became a real Voxelmon */ onVoxelmon?: (id: string) => void; /** debug mode: a 3D button shows any card as a voxel model */ debug?: boolean } = $props();
	/** debug: card shown in the 2D / 3D model viewer */
	let viewer = $state<string | null>(null);
	let open = $state<number | null>(null);
	let big = $state<string | null>(null);
	const exp = $derived(game.save.exp ?? 0);
	const lv = (id: string) => game.save.cardLevels?.[id] ?? 0;
	let upFx = $state(0);
	const cur = $derived(big ? lv(big) : 0);
	const max = $derived(big ? maxLevel(big) : 0);
	const nextStage = $derived(STAGES[cur + 1]);
	function upgrade(id: string) {
		const n = game.upgrade(id);
		if (n === null) return;
		upFx++;
		if (n >= VOXEL_LEVEL) {
			sfx.fanfare();
			big = null;
			onVoxelmon?.(id);
		} else sfx.reward(n >= 3);
	}

	let confirmDelete = $state(false);
	let shake = $state(0);
	let flashId = $state<string | null>(null);

	const byCost = (a: string, b: string) => CARD_MAP[a].cost - CARD_MAP[b].cost || CARD_MAP[a].name.localeCompare(CARD_MAP[b].name);
	const decks = $derived(game.save.decks);
	const deck = $derived(open !== null ? (decks[open] ?? null) : null);
	const inDeck = $derived(new Set(deck?.cards ?? []));
	const deckCards = $derived(deck ? [...deck.cards].sort(byCost) : []);
	const gallery = $derived([...game.save.collection].sort(byCost));
	const curve = $derived([1, 2, 3, 4, 5, 6].map((c) => deckCards.filter((id) => Math.min(6, CARD_MAP[id].cost) === c).length));

	// ---------- sizes: everything fits on screen ----------
	let galW = $state(0);
	let galH = $state(0);
	let rowW = $state(0);
	let rowH = $state(0);
	const GAP = 10;
	const galCardW = $derived.by(() => {
		const n = gallery.length;
		if (!n || !galW || !galH) return 100;
		let best = 0;
		for (let c = 1; c <= n; c++) {
			const rows = Math.ceil(n / c);
			const w = Math.min((galW - 4 - GAP * (c - 1)) / c, (galH - 4 - GAP * (rows - 1)) / rows / 1.42);
			if (w > best) best = w;
		}
		return Math.max(62, Math.floor(best - 2));
	});
	// row size includes its 8px padding + 2px border on each side
	// one row of 12; on narrow phones two rows of 6 so the cards stay readable
	const rowCols = $derived((rowW - 24 - 8 * (DECK_SIZE - 1)) / DECK_SIZE >= 44 ? DECK_SIZE : DECK_SIZE / 2);
	const rowCardW = $derived.by(() => {
		const rows = DECK_SIZE / rowCols;
		const w = Math.min((rowW - 24 - 8 * (rowCols - 1)) / rowCols, (rowH - 24 - 8 * (rows - 1)) / rows / 1.42);
		return Math.max(30, Math.floor(w));
	});

	// ---------- add / remove ----------
	function add(id: string, from?: { cx: number; cy: number; w: number }) {
		if (open === null || !deck) return;
		if (inDeck.has(id)) {
			flashId = id;
			setTimeout(() => (flashId = null), 500);
			return;
		}
		if (deck.cards.length >= DECK_SIZE) {
			shake++;
			sfx.blip();
			return;
		}
		game.saveDeck(open, [...deck.cards, id]);
		sfx.place();
		if (from) land(`[data-deckcard="${id}"]`, from);
	}
	function remove(id: string, from?: { cx: number; cy: number; w: number }) {
		if (open === null || !deck) return;
		game.saveDeck(open, deck.cards.filter((c) => c !== id));
		sfx.blip();
		if (from) land(`[data-galcard="${id}"]`, from);
	}
	/** the card glides from where it was dropped into its new place */
	async function land(sel: string, from: { cx: number; cy: number; w: number }) {
		await tick();
		const el = document.querySelector(sel) as HTMLElement | null;
		if (!el) return;
		const to = el.getBoundingClientRect();
		if (!to.width) return;
		el.animate(
			[
				{ transform: `translate(${from.cx - (to.left + to.width / 2)}px, ${from.cy - (to.top + to.height / 2)}px) scale(${from.w / to.width})`, zIndex: 30 },
				{ transform: 'translate(0, 0) scale(1.08)', offset: 0.8, zIndex: 30 },
				{ transform: 'none', zIndex: 30 }
			],
			{ duration: 300, easing: 'cubic-bezier(.2,.8,.3,1)' }
		);
	}

	// one rule for mouse and touch, in the gallery and in the deck:
	//   single click / tap  -> details (big view)
	//   double click / double tap -> gallery: add to the deck · deck: remove from the deck
	// (own timing instead of the dblclick event, because touch browsers don't send dblclick reliably)
	const DOUBLE_MS = 300;
	let lastTap: { key: string; at: number; timer: ReturnType<typeof setTimeout> } | null = null;
	function tap(id: string, where: 'gallery' | 'deck') {
		if (suppressClick) return;
		const key = `${where}:${id}`;
		const now = performance.now();
		if (lastTap && lastTap.key === key && now - lastTap.at < DOUBLE_MS) {
			clearTimeout(lastTap.timer);
			lastTap = null;
			if (where === 'gallery') add(id);
			else remove(id);
			return;
		}
		if (lastTap) clearTimeout(lastTap.timer);
		lastTap = {
			key,
			at: now,
			timer: setTimeout(() => {
				lastTap = null;
				big = id;
			}, DOUBLE_MS)
		};
	}

	// ---------- drag: the real card follows the pointer ----------
	// only id / moved / overDeck are reactive; the position goes straight into the element's transform
	// once per frame, and the deck row's rectangle is cached at drag start (no layout work while moving)
	type Drag = { id: string; from: 'deck' | 'gallery'; moved: boolean; overDeck: boolean };
	let drag = $state<Drag | null>(null);
	const motion = { x0: 0, y0: 0, dx: 0, dy: 0, tilt: 0, lastX: 0, raf: 0 };
	let dragEl: HTMLElement | null = null;
	let rowRect: DOMRect | null = null;
	let suppressClick = false;
	function down(e: PointerEvent, id: string, from: 'deck' | 'gallery') {
		if (e.button !== 0 || open === null) return;
		dragEl = e.currentTarget as HTMLElement;
		Object.assign(motion, { x0: e.clientX, y0: e.clientY, dx: 0, dy: 0, tilt: 0, lastX: e.clientX });
		drag = { id, from, moved: false, overDeck: from === 'deck' };
		window.addEventListener('pointermove', move, { passive: true });
		window.addEventListener('pointerup', up);
		window.addEventListener('pointercancel', cancel);
	}
	function overDeckRow(x: number, y: number) {
		const r = rowRect;
		return !!r && x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
	}
	function paint() {
		motion.raf = 0;
		if (dragEl && drag?.moved) dragEl.style.transform = `translate3d(${motion.dx}px, ${motion.dy}px, 0) rotate(${motion.tilt}deg) scale(1.06)`;
	}
	function move(e: PointerEvent) {
		if (!drag) return;
		motion.dx = e.clientX - motion.x0;
		motion.dy = e.clientY - motion.y0;
		const vx = e.clientX - motion.lastX;
		motion.lastX = e.clientX;
		motion.tilt = Math.max(-14, Math.min(14, motion.tilt * 0.75 + vx * 0.6));
		if (!drag.moved && Math.hypot(motion.dx, motion.dy) > 8) {
			drag.moved = true;
			rowRect = document.querySelector('.deckrow')?.getBoundingClientRect() ?? null;
		}
		if (!drag.moved) return;
		const over = overDeckRow(e.clientX, e.clientY);
		if (over !== drag.overDeck) drag.overDeck = over;
		if (!motion.raf) motion.raf = requestAnimationFrame(paint);
	}
	function stop() {
		window.removeEventListener('pointermove', move);
		window.removeEventListener('pointerup', up);
		window.removeEventListener('pointercancel', cancel);
		if (motion.raf) cancelAnimationFrame(motion.raf);
		motion.raf = 0;
	}
	function cancel() {
		stop();
		if (dragEl) dragEl.style.transform = '';
		drag = null;
	}
	function up(e: PointerEvent) {
		stop();
		const d = drag;
		if (!d || !d.moved) {
			drag = null;
			return;
		}
		paint();
		suppressClick = true;
		setTimeout(() => (suppressClick = false), 0);
		const box = dragEl?.getBoundingClientRect();
		const at = box && dragEl ? { cx: box.left + box.width / 2, cy: box.top + box.height / 2, w: dragEl.offsetWidth } : undefined;
		if (dragEl) dragEl.style.transform = '';
		drag = null;
		const over = overDeckRow(e.clientX, e.clientY);
		if (d.from === 'gallery' && over) add(d.id, at);
		else if (d.from === 'deck' && !over) remove(d.id, at);
		else if (at) land(d.from === 'deck' ? `[data-deckcard="${d.id}"]` : `[data-galcard="${d.id}"]`, at); // back to where it was
	}
	const isDragging = (id: string, from: 'deck' | 'gallery') => !!drag?.moved && drag.id === id && drag.from === from;

	// ---------- deck list ----------
	function openDeck(i: number) {
		open = i;
		confirmDelete = false;
	}
	function createDeck() {
		openDeck(game.newDeck());
	}
	/** a few cards to show on a deck tile: the most expensive ones */
	const preview = (cards: string[]) => [...cards].sort((a, b) => CARD_MAP[b].cost - CARD_MAP[a].cost).slice(0, 3);
	function onKey(e: KeyboardEvent) {
		// already handled (e.g. Esc closing the 3D viewer on top)
		if (e.key !== 'Escape' || e.defaultPrevented || viewer) return;
		if (big) big = null;
		else if (open !== null) open = null;
		else onClose();
	}
</script>

<svelte:window onkeydown={onKey} />

{#if viewer}
	<ModelViewer id={viewer} ids={CARDS.map((c) => c.id)} onClose={() => (viewer = null)} />
{/if}

{#if big}
	<div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm grid place-items-center p-4" role="presentation" onclick={() => (big = null)}>
		<div class="flex flex-col items-center gap-3" role="presentation" onclick={(e) => e.stopPropagation()}>
			{#key upFx}
				<div class:upfx={upFx > 0}>
					<!-- a real Voxelmon: the 3D beast stands in the card, drag to turn it -->
					{#if lv(big) >= VOXEL_LEVEL && hasVoxelModel(big)}
						<Card3D id={big} width="min(72vw, 44dvh, 270px)" level={lv(big)} />
					{:else}
						<Card id={big} size="lg" width="min(72vw, 44dvh, 270px)" level={lv(big)} />
					{/if}
				</div>
			{/key}
			<!-- upgrade: EXP from battles buys the next stage -->
			<div class="stages">
				{#each STAGES.slice(0, max + 1) as st, i}
					<div class="stage" class:done={i <= cur} class:next={i === cur + 1} title="{st.name}: {st.blurb}">
						<span class="material-symbols-rounded">{st.icon}</span><small>{st.name}</small>
					</div>
				{/each}
			</div>
			{#if cur < max}
				<div class="upnote">Next: <b>{nextStage.name}</b> · {nextStage.blurb}</div>
				<button class="btn up" disabled={exp < nextStage.cost} onclick={() => upgrade(big!)}>
					<span class="material-symbols-rounded">{nextStage.icon}</span>
					{exp < nextStage.cost ? `Needs ${nextStage.cost} EXP (you have ${exp})` : `Upgrade · ${nextStage.cost} EXP`}
				</button>
			{:else if cur >= VOXEL_LEVEL}
				<div class="upnote">A real Voxelmon! {game.save.voxelmon === big ? 'It walks with you.' : ''}</div>
				{#if game.save.voxelmon !== big}
					<button class="btn up" onclick={() => { game.save.voxelmon = big!; game.persist(); sfx.place(); }}><span class="material-symbols-rounded">pets</span> Take it along</button>
				{/if}
			{:else}
				<div class="upnote">Fully upgraded!{canBecomeVoxel(big) ? '' : ' Only 6-energy cards can become a 3D Voxelmon.'}</div>
			{/if}
			<div class="flex gap-2">
				{#if deck}
					<button
						class="btn primary"
						disabled={!inDeck.has(big) && deck.cards.length >= DECK_SIZE}
						onclick={() => {
							if (inDeck.has(big!)) remove(big!);
							else add(big!);
							big = null;
						}}
					>
						<span class="material-symbols-rounded">{inDeck.has(big) ? 'remove_circle' : 'add_circle'}</span>
						{inDeck.has(big) ? 'Remove from deck' : deck.cards.length >= DECK_SIZE ? 'Deck is full' : 'Add to deck'}
					</button>
				{/if}
				<button class="btn" title="Hear its attack" onclick={() => sfx.attack(big!, CARD_MAP[big!].cost)}><span class="material-symbols-rounded">volume_up</span></button>
				{#if debug}
					<button class="btn" title="Debug: see this beast as a 3D voxel model" onclick={() => (viewer = big)}><span class="material-symbols-rounded">view_in_ar</span> 3D</button>
				{/if}
				<button class="btn" onclick={() => (big = null)}><FluentIcon name="dismiss" /> Close</button>
			</div>
		</div>
	</div>
{/if}

<div class="fixed inset-0 z-40 bg-[#1b2a34]/95 backdrop-blur-sm text-white flex flex-col" style="font-family: 'Fredoka', system-ui">
	{#if open === null || !deck}
		<!-- ================= deck list ================= -->
		<div class="flex items-center gap-3 px-5 py-3 bg-black/30">
			<FluentIcon name="deck" size={26} />
			<h2 class="text-2xl font-bold">My Decks</h2>
			<div class="text-sm opacity-80 ml-2">Collection: {game.save.collection.length}/{POOL.length} cards</div>
			<div class="expchip" title="EXP from battles: open a deck and tap a card to upgrade it"><span class="material-symbols-rounded">auto_awesome</span>{exp} EXP</div>
			<div class="ml-auto flex gap-2">
				<button class="btn" onclick={onClose}><FluentIcon name="dismiss" /> Close</button>
			</div>
		</div>
		<div class="flex-1 min-h-0 overflow-y-auto p-5">
			<div class="decklist">
				{#each decks as d, i}
					<button class="decktile" class:active={i === game.save.active} onclick={() => openDeck(i)}>
						<div class="fan">
							{#each preview(d.cards) as id, k}
								<div class="fan-card" style="--k:{k - 1}"><Card {id} width="62px" level={lv(id)} /></div>
							{:else}
								<span class="material-symbols-rounded text-5xl opacity-40">style</span>
							{/each}
						</div>
						<div class="font-bold text-lg truncate w-full">{d.name}</div>
						<div class="text-sm" class:incomplete={d.cards.length !== DECK_SIZE}>{d.cards.length}/{DECK_SIZE} cards</div>
						{#if i === game.save.active}
							<span class="badge"><span class="material-symbols-rounded">check_circle</span> In use</span>
						{:else if d.cards.length === DECK_SIZE}
							<span
								class="badge use"
								role="button"
								tabindex="0"
								onclick={(e) => {
									e.stopPropagation();
									game.useDeck(i);
									sfx.place();
								}}
								onkeydown={(e) => e.key === 'Enter' && game.useDeck(i)}>Use</span
							>
						{/if}
					</button>
				{/each}
				<button class="decktile newdeck" onclick={createDeck}>
					<span class="material-symbols-rounded text-5xl">add_circle</span>
					<div class="font-bold">New deck</div>
				</button>
			</div>
		</div>
	{:else}
		<!-- ================= deck editor ================= -->
		<div class="flex items-center gap-3 px-4 py-2 bg-black/30 flex-wrap">
			<button class="btn" onclick={() => (open = null)} title="Back to all decks"><span class="material-symbols-rounded">arrow_back</span></button>
			<div class="expchip" title="Tap a card to upgrade it with EXP"><span class="material-symbols-rounded">auto_awesome</span>{exp} EXP</div>
			<input
				class="name"
				value={deck.name}
				maxlength="24"
				onchange={(e) => game.renameDeck(open!, (e.currentTarget as HTMLInputElement).value)}
				aria-label="Deck name"
			/>
			<div class="count" class:incomplete={deck.cards.length !== DECK_SIZE}>{deck.cards.length}/{DECK_SIZE}</div>
			<div class="curve" title="Energy curve">
				{#each curve as n, i}
					<div class="bar"><div style="height:{Math.min(100, n * 25)}%"></div><span>{i + 1}{i === 5 ? '+' : ''}</span></div>
				{/each}
			</div>
			<div class="ml-auto flex gap-2 items-center">
				{#if open === game.save.active}
					<span class="chip"><span class="material-symbols-rounded">check_circle</span> In use</span>
				{:else}
					<button class="btn primary" disabled={deck.cards.length !== DECK_SIZE} onclick={() => game.useDeck(open!)}>Use this deck</button>
				{/if}
				{#if decks.length > 1}
					<button
						class="btn"
						class:danger={confirmDelete}
						onclick={() => {
							if (!confirmDelete) return (confirmDelete = true);
							game.deleteDeck(open!);
							open = null;
						}}
						title="Delete deck"
					>
						<span class="material-symbols-rounded">delete</span>{confirmDelete ? 'Really delete?' : ''}
					</button>
				{/if}
				<button class="btn" onclick={onClose}><FluentIcon name="dismiss" /></button>
			</div>
		</div>

		<!-- the deck: one card high -->
		<div class="deckrow" data-drop="deck" class:over={!!drag?.moved && drag.from === 'gallery' && drag.overDeck} bind:clientWidth={rowW} bind:clientHeight={rowH}>
			{#key shake}
				<div class="slots" class:shake={shake > 0} style="--cw:{rowCardW}px; width:{rowCols * rowCardW + (rowCols - 1) * 8}px">
					{#each deckCards as id (id)}
						<button
							class="dcard"
							data-deckcard={id}
							class:flash={flashId === id}
							class:dragging={isDragging(id, 'deck')}
							onpointerdown={(e) => down(e, id, 'deck')}
							onclick={() => tap(id, 'deck')}
							oncontextmenu={(e) => {
								e.preventDefault();
								remove(id);
							}}
							title="Click: details · double-click / right-click / drag out: remove"
						>
							<Card {id} width="var(--cw)" level={lv(id)} />
						</button>
					{/each}
					{#each Array(Math.max(0, DECK_SIZE - deckCards.length)) as _}
						<div class="empty"></div>
					{/each}
				</div>
			{/key}
		</div>
		<div class="hint">
			{#if deck.cards.length < DECK_SIZE}Double-tap or drag a card into the deck · tap for details · {DECK_SIZE - deck.cards.length} missing{:else}Deck complete! Double-tap or drag a card out to swap it · tap for details{/if}
			{#if open === game.save.active && deck.cards.length !== DECK_SIZE}<span class="warn"> · battles use your last complete version</span>{/if}
		</div>

		<!-- the collection: fills the rest, sized so all cards fit -->
		<div class="gallery" bind:clientWidth={galW} bind:clientHeight={galH}>
			<div class="ggrid" style="--gw:{galCardW}px">
				{#each gallery as id (id)}
					<button
						class="gcard"
						data-galcard={id}
						class:indeck={inDeck.has(id)}
						class:dragging={isDragging(id, 'gallery')}
						onpointerdown={(e) => !inDeck.has(id) && down(e, id, 'gallery')}
						onclick={() => tap(id, 'gallery')}
						oncontextmenu={(e) => {
							e.preventDefault();
							big = id;
						}}
						title="Click: details · double-click / drag into the deck: add"
					>
						<Card {id} width="var(--gw)" dim={inDeck.has(id)} level={lv(id)} />
						{#if inDeck.has(id)}<span class="material-symbols-rounded check">check_circle</span>{/if}
					</button>
				{/each}
			</div>
		</div>
	{/if}
</div>

<style>
	.expchip {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.15rem 0.7rem;
		border-radius: 999px;
		background: linear-gradient(135deg, #7a5aff, #3dbbff);
		font-weight: 700;
		font-size: 0.9rem;
	}
	.stages {
		display: flex;
		gap: 0.3rem;
	}
	.stage {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 4rem;
		padding: 0.2rem 0;
		border-radius: 0.6rem;
		background: rgb(255 255 255 / 0.08);
		color: rgb(255 255 255 / 0.45);
		font-size: 0.75rem;
	}
	.stage.done {
		background: linear-gradient(135deg, #ffd23d, #f29b05);
		color: #1b2a34;
	}
	.stage.next {
		outline: 0.12rem dashed #ffd23d;
		color: #fff;
	}
	.upnote {
		max-width: 22rem;
		text-align: center;
		font-size: 0.9rem;
		color: #fff;
	}
	.btn.up {
		background: linear-gradient(135deg, #7a5aff, #3dbbff);
		color: #fff;
	}
	.btn.up:disabled {
		opacity: 0.55;
	}
	.upfx {
		animation: upfx 0.7s ease-out;
	}
	@keyframes upfx {
		0% {
			scale: 1;
			filter: brightness(1);
		}
		30% {
			scale: 1.08;
			filter: brightness(2.2);
		}
		100% {
			scale: 1;
			filter: brightness(1);
		}
	}
	.btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border-radius: 10px;
		font-weight: 700;
		background: rgb(255 255 255 / 0.15);
		border: 2px solid rgb(0 0 0 / 0.3);
	}
	.btn.primary {
		background: linear-gradient(#3d9bff, #1655b0);
	}
	.btn.danger {
		background: #c91a09;
	}
	.btn:disabled {
		opacity: 0.4;
	}
	/* ----- deck list ----- */
	.decklist {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
		gap: 16px;
	}
	.decktile {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 14px 12px;
		border-radius: 16px;
		background: rgb(255 255 255 / 0.08);
		border: 3px solid rgb(255 255 255 / 0.12);
		transition: transform 0.15s, border-color 0.15s;
		min-height: 210px;
		justify-content: flex-end;
	}
	.decktile:hover {
		transform: translateY(-3px);
		border-color: rgb(255 255 255 / 0.4);
	}
	.decktile.active {
		border-color: #5ccb4c;
		box-shadow: 0 0 18px rgb(92 203 76 / 0.35);
	}
	.newdeck {
		justify-content: center;
		border-style: dashed;
		opacity: 0.8;
	}
	.fan {
		position: relative;
		height: 100px;
		width: 100%;
		display: flex;
		justify-content: center;
		align-items: center;
	}
	.fan-card {
		position: absolute;
		transform: translateX(calc(var(--k) * 38px)) rotate(calc(var(--k) * 9deg));
	}
	.incomplete {
		color: #ffb070;
	}
	.badge {
		position: absolute;
		top: 8px;
		right: 8px;
		display: inline-flex;
		align-items: center;
		gap: 3px;
		font-size: 0.75rem;
		font-weight: 700;
		padding: 2px 8px;
		border-radius: 999px;
		background: #5ccb4c;
	}
	.badge .material-symbols-rounded {
		font-size: 16px;
	}
	.badge.use {
		background: #1655b0;
		cursor: pointer;
	}
	/* ----- editor ----- */
	.name {
		background: rgb(0 0 0 / 0.3);
		border: 2px solid rgb(255 255 255 / 0.2);
		border-radius: 10px;
		padding: 6px 10px;
		font-size: 1.2rem;
		font-weight: 700;
		width: 14em;
		max-width: 40vw;
		color: #fff;
	}
	.count {
		font-weight: 900;
		font-size: 1.3rem;
	}
	.curve {
		display: flex;
		gap: 3px;
		height: 36px;
		align-items: flex-end;
	}
	.curve .bar {
		width: 14px;
		height: 100%;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		align-items: center;
		font-size: 0.6rem;
	}
	.curve .bar div {
		width: 100%;
		background: #3d9bff;
		border-radius: 3px 3px 0 0;
		min-height: 2px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 6px 12px;
		border-radius: 999px;
		background: #5ccb4c;
		font-weight: 700;
	}
	.deckrow {
		height: clamp(84px, 24dvh, 220px);
		margin: 10px 12px 0;
		padding: 8px;
		border-radius: 14px;
		background: rgb(0 0 0 / 0.28);
		border: 2px dashed rgb(255 255 255 / 0.15);
		transition: border-color 0.15s, background 0.15s;
		display: flex;
		align-items: center;
		justify-content: center;
		box-sizing: border-box;
	}
	.deckrow.over {
		border-color: #ffe066;
		background: rgb(255 224 102 / 0.12);
	}
	.slots {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		align-items: center;
	}
	.slots.shake {
		animation: shake 0.35s;
	}
	@keyframes shake {
		25% {
			transform: translateX(-8px);
		}
		75% {
			transform: translateX(8px);
		}
	}
	.dcard,
	.gcard {
		display: block;
		line-height: 0;
		position: relative;
		touch-action: none;
		transition: filter 0.15s;
	}
	.dcard.dragging,
	.gcard.dragging {
		z-index: 60;
		pointer-events: none;
		transition: none;
		will-change: transform;
		filter: drop-shadow(0 14px 20px rgb(0 0 0 / 0.55));
	}
	.dcard.flash {
		animation: flash 0.5s;
	}
	@keyframes flash {
		50% {
			transform: scale(1.12);
			filter: drop-shadow(0 0 12px #ffe066);
		}
	}
	.empty {
		width: var(--cw);
		aspect-ratio: 5 / 7;
		border-radius: 8px;
		border: 2px dashed rgb(255 255 255 / 0.2);
		background: rgb(255 255 255 / 0.03);
	}
	.hint {
		text-align: center;
		font-size: 0.85rem;
		opacity: 0.8;
		padding: 4px 12px;
	}
	.warn {
		color: #ffb070;
	}
	.gallery {
		flex: 1;
		min-height: 0;
		margin: 0 12px 12px;
		overflow-y: auto;
		overflow-x: hidden;
	}
	.ggrid {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		justify-content: center;
		align-content: flex-start;
	}
	.check {
		position: absolute;
		bottom: 6px;
		right: 6px;
		color: #5ccb4c;
		background: #fff;
		border-radius: 50%;
		font-size: 20px;
	}
</style>
