<script lang="ts">
	// Lumi's piano in Dex & Lumi's house: drawn exactly over the 3D piano while you sit in front of it.
	// Play with the mouse / touch or the keyboard (A W S E D F T G Y H U J K), or follow a song from the song book.
	import { sfx } from '../game/sfx';

	let { rect, onClose, onSong }: { rect: { left: number; top: number; width: number; height: number }; onClose: () => void; onSong?: (name: string) => void } = $props();

	// one octave C4..C5: 8 white keys, 5 black keys
	const NOTES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B', 'C'];
	const KEYS = ['a', 'w', 's', 'e', 'd', 'f', 't', 'g', 'y', 'h', 'u', 'j', 'k'];
	const isBlack = (i: number) => NOTES[i].includes('#');
	const WHITE = NOTES.map((_, i) => i).filter((i) => !isBlack(i));
	const BLACK = NOTES.map((_, i) => i).filter(isBlack);
	const freq = (i: number) => 261.63 * Math.pow(2, i / 12);

	const SONGS: { name: string; notes: number[] }[] = [
		{ name: 'Twinkle Twinkle', notes: [0, 0, 7, 7, 9, 9, 7, 5, 5, 4, 4, 2, 2, 0] },
		{ name: 'Ode to Joy', notes: [4, 4, 5, 7, 7, 5, 4, 2, 0, 0, 2, 4, 4, 2, 2] },
		{ name: 'Brickhollow Tune', notes: [0, 4, 7, 4, 5, 9, 7, 4, 7, 12, 7, 5, 2, 0] }
	];
	let song = $state<number | null>(null);
	let step = $state(0);
	let bravo = $state(0);
	const want = $derived(song === null ? null : SONGS[song].notes[step]);

	let down = $state(new Set<number>());
	function press(i: number) {
		sfx.piano(freq(i));
		down = new Set(down).add(i);
		setTimeout(() => {
			const d = new Set(down);
			d.delete(i);
			down = d;
		}, 180);
		if (song !== null && i === want) {
			step++;
			if (step >= SONGS[song].notes.length) {
				onSong?.(SONGS[song].name);
				bravo++;
				song = null;
				step = 0;
			}
		}
	}
	function pickSong(i: number) {
		song = song === i ? null : i;
		step = 0;
	}

	function onKey(e: KeyboardEvent) {
		const k = e.key.toLowerCase();
		if (k === 'escape') {
			e.preventDefault();
			onClose();
			return;
		}
		const i = KEYS.indexOf(k);
		if (i >= 0 && !e.repeat) {
			e.preventDefault();
			press(i);
		}
	}
</script>

<svelte:window onkeydown={onKey} />

<div class="piano" style="left:{rect.left}px; top:{rect.top}px; width:{rect.width}px; height:{rect.height}px; font-size:{rect.height / 16}px">
	<div class="book">
		<span class="material-symbols-rounded">library_music</span>
		{#each SONGS as s, i}
			<button class:on={song === i} onclick={() => pickSong(i)}>{s.name}</button>
		{/each}
		<span class="grow"></span>
		<button class="off" onclick={onClose} title="Stop playing (Esc)"><span class="material-symbols-rounded">close</span></button>
	</div>
	{#if song !== null}
		<div class="progress"><div style="width:{(step / SONGS[song].notes.length) * 100}%"></div></div>
	{/if}
	<div class="keys">
		{#each WHITE as i, w}
			<button class="white" class:down={down.has(i)} class:want={want === i} style="left:{(w / WHITE.length) * 100}%; width:{100 / WHITE.length}%" onpointerdown={() => press(i)}>
				<small>{KEYS[i].toUpperCase()}</small>
			</button>
		{/each}
		{#each BLACK as i}
			{@const w = WHITE.indexOf(i - 1) + 1}
			<button class="black" class:down={down.has(i)} class:want={want === i} style="left:{(w / WHITE.length) * 100 - 100 / WHITE.length / 3}%; width:{(100 / WHITE.length) * 0.66}%" onpointerdown={() => press(i)}>
				<small>{KEYS[i].toUpperCase()}</small>
			</button>
		{/each}
	</div>
	{#key bravo}
		{#if bravo > 0}<div class="bravo">Bravo! <span class="material-symbols-rounded">music_note</span></div>{/if}
	{/key}
</div>

<style>
	.piano {
		position: fixed;
		z-index: 35;
		display: flex;
		flex-direction: column;
		gap: 0.3em;
		padding: 0.4em;
		border-radius: 0.3em;
		background: linear-gradient(#2a3a46, #1b2a34);
		color: #fff;
		font-family: 'Fredoka', system-ui, sans-serif;
		animation: in 0.3s ease-out;
		user-select: none;
		touch-action: none;
	}
	@keyframes in {
		from {
			opacity: 0;
		}
	}
	.book {
		display: flex;
		align-items: center;
		gap: 0.3em;
		font-size: 0.85em;
	}
	.book button {
		padding: 0.15em 0.6em;
		border-radius: 999px;
		background: rgb(255 255 255 / 0.14);
		font-weight: 600;
	}
	.book button.on {
		background: #ffd23d;
		color: #1b2a34;
	}
	.book .off {
		display: grid;
		place-items: center;
		padding: 0.15em;
		background: #c91a09;
	}
	.grow {
		flex: 1;
	}
	.progress {
		height: 0.35em;
		border-radius: 999px;
		background: rgb(255 255 255 / 0.15);
		overflow: hidden;
	}
	.progress div {
		height: 100%;
		background: #ffd23d;
		transition: width 0.15s;
	}
	.keys {
		position: relative;
		flex: 1;
	}
	.keys button {
		position: absolute;
		top: 0;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		padding-bottom: 0.4em;
		transition: transform 0.05s, background 0.1s;
	}
	.white {
		bottom: 0;
		background: #f7f4ec;
		color: #8a8f94;
		border: 0.08em solid #1b2a34;
		border-radius: 0 0 0.35em 0.35em;
		box-shadow: inset 0 -0.4em 0 #d9d4c6;
	}
	.black {
		z-index: 1;
		height: 60%;
		background: #1b2a34;
		color: #8a9aa6;
		border-radius: 0 0 0.25em 0.25em;
		box-shadow: inset 0 -0.3em 0 #000;
		font-size: 0.8em;
	}
	.white.want {
		background: #fff2b0;
		box-shadow: inset 0 -0.4em 0 #ffd23d;
		animation: hint 0.8s ease-in-out infinite;
	}
	.black.want {
		background: #f2a705;
		animation: hint 0.8s ease-in-out infinite;
	}
	@keyframes hint {
		50% {
			filter: brightness(1.15);
		}
	}
	.white.down {
		background: #ffd23d;
		transform: translateY(0.12em);
		box-shadow: inset 0 -0.15em 0 #d9a800;
	}
	.black.down {
		background: #3d6fd1;
		transform: translateY(0.1em);
	}
	small {
		font-size: 0.75em;
		font-weight: 700;
	}
	.bravo {
		position: absolute;
		left: 50%;
		top: 40%;
		translate: -50% -50%;
		display: flex;
		align-items: center;
		gap: 0.3em;
		padding: 0.3em 1em;
		border-radius: 999px;
		background: #ffd23d;
		color: #1b2a34;
		font-size: 1.6em;
		font-weight: 900;
		pointer-events: none;
		animation: bravo 1.8s ease-out forwards;
	}
	@keyframes bravo {
		0% {
			opacity: 0;
			scale: 0.5;
		}
		15% {
			opacity: 1;
			scale: 1.1;
		}
		25% {
			scale: 1;
		}
		80% {
			opacity: 1;
		}
		100% {
			opacity: 0;
		}
	}
</style>
