<script lang="ts">
	// The PC in your room: an HTML "desktop" drawn exactly over the 3D monitor while the camera
	// looks at it in first person. Sized in em, so it scales with the monitor on screen.
	import Card from './Card.svelte';
	import { CARD_MAP, POOL } from '../game/cards';
	import { game } from '../game/state.svelte';
	import { GOLDEN_STUDS } from '../three/town';
	import { sfx } from '../game/sfx';
	import { STICKERS } from '../game/stickers';
	import { album } from '../game/photos.svelte';

	let { rect, onClose }: { rect: { left: number; top: number; width: number; height: number }; onClose: () => void } = $props();

	type App = 'home' | 'msg' | 'stats' | 'beasts' | 'stickers' | 'photos';
	let big = $state<string | null>(null);
	let app = $state<App>('home');
	const s = $derived(game.save);
	const best = $derived([...s.collection].sort((a, b) => CARD_MAP[b].cost - CARD_MAP[a].cost)[0] ?? null);
	const messages = $derived([
		{ from: 'Dex', text: 'SNAP!' },
		{ from: 'Dex', text: 'snap.' },
		{ from: 'Dex', text: 'SNAP!!!' },
		...(s.rivalWins ? [{ from: 'Dex', text: 'ok fine. you won. REMATCH??' }] : []),
		{ from: 'Mom', text: "Don't forget to eat something! ♥" },
		{ from: 'Lumi', text: 'Tip: win 2 of 3 locations, not all of them :)' }
	]);
	const facts = [
		'Glimmerlings fold light into tiny bricks.',
		'Brick Beasts love music. Especially drums.',
		'A shiny pack sparkles in the grass. Rarely!',
		'Ongoing cards keep working while they stay.',
		'Golden studs hide in every house.'
	];
	const fact = facts[Math.floor(Math.random() * facts.length)];
	function open(a: App) {
		app = a;
		sfx.blip();
	}
	const APPS: { id: App; icon: string; label: string; color: string }[] = [
		{ id: 'msg', icon: 'chat', label: 'Messages', color: '#3d9bff' },
		{ id: 'stats', icon: 'trophy', label: 'Battle log', color: '#f2b705' },
		{ id: 'beasts', icon: 'pets', label: 'Beasts', color: '#4fc06a' },
		{ id: 'stickers', icon: 'auto_awesome', label: 'Stickers', color: '#e96ed1' },
		{ id: 'photos', icon: 'photo_library', label: 'Photos', color: '#ff8a1f' }
	];
</script>

<div class="pc" style="left:{rect.left}px; top:{rect.top}px; width:{rect.width}px; height:{rect.height}px; font-size:{rect.height / 17}px">
	<div class="bar">
		<span class="material-symbols-rounded">deployed_code</span>
		<b>BrickOS</b>
		{#if app !== 'home'}<button class="back" onclick={() => open('home')}><span class="material-symbols-rounded">arrow_back</span></button>{/if}
		<span class="grow"></span>
		<span class="studs"><span class="material-symbols-rounded">toll</span>{s.studs}</span>
		<button class="close" onclick={onClose} title="Log off (Esc)"><span class="material-symbols-rounded">power_settings_new</span></button>
	</div>

	{#if app === 'home'}
		<div class="desk">
			{#if best}<div class="wall"><Card id={best} width="{Math.round(rect.height * 0.5 / 1.4)}px" /></div>{/if}
			<div class="icons">
				{#each APPS as a}
					<button class="icon" onclick={() => open(a.id)}>
						<span class="material-symbols-rounded" style="background:{a.color}">{a.icon}</span>
						<span>{a.label}</span>
						{#if a.id === 'msg'}<i class="badge">{messages.length}</i>{/if}
						{#if a.id === 'stickers'}<i class="badge">{STICKERS.filter((k) => k.done(s)).length}/{STICKERS.length}</i>{/if}
					</button>
				{/each}
			</div>
		</div>
	{:else if app === 'msg'}
		<div class="list">
			{#each messages as m, i}
				<div class="msg" style="animation-delay:{i * 70}ms"><b>{m.from}</b> {m.text}</div>
			{/each}
		</div>
	{:else if app === 'stats'}
		<div class="stats">
			<div><span class="material-symbols-rounded">military_tech</span> Wins <b>{s.wins}</b></div>
			<div><span class="material-symbols-rounded">heart_broken</span> Losses <b>{s.losses}</b></div>
			<div><span class="material-symbols-rounded">swords</span> Beat Dex <b>{s.rivalWins}×</b></div>
			<div><span class="material-symbols-rounded">stars</span> Golden studs <b>{s.golden.length}/{GOLDEN_STUDS.length}</b></div>
			<div><span class="material-symbols-rounded">flutter_dash</span> Butterflies caught <b>{s.butterflies ?? 0}</b></div>
		</div>
	{:else if app === 'photos'}
		{#if big}
			<button class="bigphoto" onclick={() => (big = null)} title="Back to the album"><img src={big} alt="Photo" /></button>
		{:else if album.list.length}
			<div class="photos">
				{#each album.list as ph, i}
					<button class="photo" style="--r:{((i * 37) % 9) - 4}deg; animation-delay:{i * 50}ms" onclick={() => (big = ph.src)}>
						<img src={ph.src} alt="Photo from {ph.where}" />
						<small>{ph.where} · {new Date(ph.t).toLocaleDateString()}</small>
					</button>
				{/each}
			</div>
		{:else}
			<div class="nophotos"><span class="material-symbols-rounded">photo_camera</span>No photos yet. Press <b>P</b> outside to take one!</div>
		{/if}
	{:else if app === 'stickers'}
		<div class="stickers">
			{#each STICKERS as k, i}
				{@const got = k.done(s)}
				<div class="sticker" class:got style="--c:{k.color}; animation-delay:{i * 40}ms" title={k.hint}>
					<span class="material-symbols-rounded">{got ? k.icon : 'lock'}</span>
					<b>{got ? k.name : '???'}</b>
					<small>{k.hint}</small>
				</div>
			{/each}
		</div>
	{:else}
		<div class="beasts">
			<div>Collection <b>{s.collection.length}/{POOL.length}</b></div>
			<div class="prog"><div style="width:{(s.collection.length / POOL.length) * 100}%"></div></div>
			<div class="fact"><span class="material-symbols-rounded">lightbulb</span> {fact}</div>
		</div>
	{/if}
</div>

<style>
	.photos {
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 0.35em;
		padding: 0.4em 0.6em;
		overflow: hidden;
		align-content: start;
	}
	.photo {
		padding: 0.15em 0.15em 0.05em;
		background: #fffdf6;
		border-radius: 0.1em;
		box-shadow: 0 0.08em 0.2em rgb(0 0 0 / 0.4);
		rotate: var(--r);
		animation: pop 0.3s ease-out both;
		color: #5a4a3a;
		min-width: 0;
	}
	.photo:hover {
		rotate: 0deg;
		scale: 1.05;
	}
	.photo img {
		display: block;
		width: 100%;
		aspect-ratio: 16 / 9;
		object-fit: cover;
	}
	.photo small {
		display: block;
		font-size: 0.32em;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.bigphoto {
		flex: 1;
		min-height: 0;
		padding: 0.4em;
		display: grid;
		place-items: center;
	}
	.bigphoto img {
		max-width: 100%;
		max-height: 100%;
		border: 0.2em solid #fffdf6;
		box-shadow: 0 0.1em 0.3em rgb(0 0 0 / 0.4);
	}
	.nophotos {
		flex: 1;
		display: grid;
		place-content: center;
		justify-items: center;
		gap: 0.2em;
		font-size: 0.7em;
		opacity: 0.85;
	}
	.nophotos .material-symbols-rounded {
		font-size: 2.5em;
	}
	.stickers {
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		gap: 0.3em;
		padding: 0.4em 0.5em;
		overflow: hidden;
	}
	.sticker {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-align: center;
		min-height: 0;
		padding: 0.2em;
		border-radius: 0.5em;
		background: rgb(0 0 0 / 0.22);
		border: 0.08em dashed rgb(255 255 255 / 0.35);
		color: rgb(255 255 255 / 0.55);
		line-height: 1.05;
		animation: pop 0.3s ease-out both;
	}
	.sticker .material-symbols-rounded {
		font-size: 1.3em;
	}
	.sticker b {
		font-size: 0.5em;
	}
	.sticker small {
		font-size: 0.38em;
		opacity: 0.85;
	}
	/* an earned sticker: glossy, coloured, slightly tilted like it was stuck on by hand */
	.sticker.got {
		color: #fff;
		border: 0.1em solid #fff;
		background: radial-gradient(circle at 30% 25%, rgb(255 255 255 / 0.45), transparent 45%), var(--c);
		box-shadow: 0 0.1em 0.25em rgb(0 0 0 / 0.35);
		transform: rotate(-3deg);
		text-shadow: 0 0.05em 0 rgb(0 0 0 / 0.35);
	}
	.sticker.got:nth-child(2n) {
		transform: rotate(2.5deg);
	}
	@keyframes pop {
		from {
			opacity: 0;
			scale: 0.6;
		}
	}
	.pc {
		position: fixed;
		z-index: 35;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		color: #fff;
		font-family: 'Fredoka', system-ui, sans-serif;
		background:
			radial-gradient(circle at 75% 30%, rgb(255 255 255 / 0.18), transparent 40%),
			repeating-linear-gradient(0deg, rgb(0 0 0 / 0.06) 0 2px, transparent 2px 4px),
			linear-gradient(135deg, #1d6fd8, #0d3f8a);
		animation: boot 0.35s ease-out;
		border-radius: 0.2em;
	}
	@keyframes boot {
		from {
			opacity: 0;
			filter: brightness(3);
		}
	}
	.bar {
		display: flex;
		align-items: center;
		gap: 0.3em;
		padding: 0.25em 0.5em;
		background: rgb(0 0 0 / 0.35);
		font-size: 0.85em;
	}
	.bar .material-symbols-rounded {
		font-size: 1.2em;
	}
	.grow {
		flex: 1;
	}
	.studs {
		display: inline-flex;
		align-items: center;
		gap: 0.15em;
		color: #ffd23d;
	}
	.close,
	.back {
		display: grid;
		place-items: center;
		border-radius: 0.3em;
		padding: 0 0.2em;
	}
	.close {
		background: #c91a09;
	}
	.desk {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: space-around;
		padding: 0.5em;
	}
	.wall {
		transform: rotate(-4deg);
		filter: drop-shadow(0 0.2em 0.3em rgb(0 0 0 / 0.5));
	}
	.icons {
		display: flex;
		gap: 0.8em;
	}
	.icon {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.2em;
		font-size: 0.75em;
		transition: transform 0.12s;
	}
	.icon:hover {
		transform: translateY(-0.15em) scale(1.06);
	}
	.icon .material-symbols-rounded {
		font-size: 2.2em;
		padding: 0.15em;
		border-radius: 0.3em;
		box-shadow: 0 0.1em 0 rgb(0 0 0 / 0.3);
	}
	.badge {
		position: absolute;
		top: -0.3em;
		right: -0.3em;
		background: #c91a09;
		border-radius: 999px;
		padding: 0 0.35em;
		font-style: normal;
		font-weight: 700;
	}
	.list {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.3em;
		padding: 0.5em 0.8em;
		font-size: 0.8em;
		overflow: hidden;
	}
	.msg {
		align-self: flex-start;
		background: #fff;
		color: #1b2a34;
		border-radius: 0.6em 0.6em 0.6em 0.1em;
		padding: 0.15em 0.6em;
		animation: pop 0.25s ease-out both;
	}
	.msg b {
		color: #1655b0;
		margin-right: 0.3em;
	}
	@keyframes pop {
		from {
			opacity: 0;
			transform: translateY(0.4em) scale(0.9);
		}
	}
	.stats,
	.beasts {
		flex: 1;
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 0.35em;
		padding: 0.5em 1em;
	}
	.stats div {
		display: flex;
		align-items: center;
		gap: 0.4em;
	}
	.stats b {
		margin-left: auto;
		color: #ffd23d;
	}
	.prog {
		height: 0.7em;
		background: rgb(0 0 0 / 0.35);
		border-radius: 999px;
		overflow: hidden;
	}
	.prog div {
		height: 100%;
		background: linear-gradient(90deg, #5ccb4c, #ffd23d);
	}
	.fact {
		font-size: 0.75em;
		opacity: 0.9;
		display: flex;
		gap: 0.3em;
		align-items: center;
	}
</style>
