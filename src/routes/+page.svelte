<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { Town, NPCS } from '../lib/three/town';
	import type { VoxEvent, VoxelPlay } from '../lib/three/voxelmon';
	import { preloadFigure } from '../lib/three/minifig';
	import { GOLDEN_STUDS, READABLES, SIGNS } from '../lib/three/town';
	import { sfx, setSfxFocused } from '../lib/game/sfx';
	import { preloadSprites } from '../lib/game/art';
	import { loadVoices, setVoicesEnabled, voicesEnabled, stopVoice, setVoiceFocused } from '../lib/game/voice';
	import { Battle, type Result } from '../lib/game/battle.svelte';
	import { game } from '../lib/game/state.svelte';
	import { STICKERS, earnedStickers } from '../lib/game/stickers';
	import { album } from '../lib/game/photos.svelte';
	import { CARD_MAP, POOL, PRACTICE_DECK, RIVAL_DECK, RIVAL_DECK_2, STARTER_COLLECTION, CARDS } from '../lib/game/cards';
	import BattleView from '../lib/components/BattleView.svelte';
	import Dialog, { type DialogData } from '../lib/components/Dialog.svelte';
	import PcDesktop from '../lib/components/PcDesktop.svelte';
	import TvShow from '../lib/components/TvShow.svelte';
	import PianoKeys from '../lib/components/PianoKeys.svelte';
	import VoxelReveal from '../lib/components/VoxelReveal.svelte';
	import { battleExp, WILD_EXP } from '../lib/game/levels';
	import LabTerminal from '../lib/components/LabTerminal.svelte';
	import Tooltip from '../lib/components/Tooltip.svelte';
	import DeckBuilder from '../lib/components/DeckBuilder.svelte';
	import Card from '../lib/components/Card.svelte';
	import FluentIcon from '../lib/components/FluentIcon.svelte';

	let host: HTMLDivElement;
	let town: Town | null = null;
	let started = $state(false);
	let nearby = $state<string | null>(null);
	let dialog = $state<DialogData | null>(null);
	let dialogKey = $state(0);
	let battle = $state<Battle | null>(null);
	let deckOpen = $state(false);
	let helpOpen = $state(false);
	let rewards = $state<{ title: string; ids: string[]; then?: () => void } | null>(null);
	let flash = $state(false);
	let encounter = $state(false);
	/** a card just became a Voxelmon: the big reveal */
	let voxReveal = $state<string | null>(null);
	let vw = $state(1024);
	let vh = $state(768);
	/** card width in the pack-opening screen so that all cards fit without scrolling */
	const packCardW = $derived.by(() => {
		const n = rewards?.ids.length ?? 0;
		if (!n) return 120;
		const W = Math.min(vw - 32 - 56 - 16, 1000 - 56 - 16);
		const H = vh * 0.6 - 16;
		const gap = 12;
		let best = 0;
		for (let c = 1; c <= n; c++) {
			const rows = Math.ceil(n / c);
			const w = Math.min((W - gap * (c - 1)) / c, (H - gap * (rows - 1)) / rows / 1.42);
			if (w > best) best = w;
		}
		return Math.max(48, Math.floor(Math.min(best - 1, n <= 4 ? 230 : 170)));
	});
	/** sign currently read in close-up (camera zoomed in, no menu) */
	let reading = $state<string | null>(null);
	/** 3D screens you can look at in first person; an HTML overlay is drawn exactly on them */
	const SCREENS = ['pc:bedroom', 'tv:home', 'tv:dex', 'piano:dex', 'pc:lab1', 'pc:lab2'];
	/** where the looked-at screen is on the page (the HTML desktop / TV show is drawn there) */
	let pcRect = $state<{ left: number; top: number; width: number; height: number } | null>(null);
	$effect(() => {
		const id = reading;
		if (!id || !SCREENS.includes(id)) {
			pcRect = null;
			return;
		}
		let raf = 0;
		const follow = () => {
			const r = town?.projectedRect(id) ?? null;
			if (!r !== !pcRect || (r && pcRect && (Math.abs(r.left - pcRect.left) > 0.5 || Math.abs(r.width - pcRect.width) > 0.5 || Math.abs(r.top - pcRect.top) > 0.5))) pcRect = r;
			raf = requestAnimationFrame(follow);
		};
		follow();
		return () => cancelAnimationFrame(raf);
	});
	let voicesOn = $state(voicesEnabled());
	// time of day shown in the HUD (polled from the 3D scene)
	let tod = $state(0.15);
	const todInfo = $derived(
		tod < 0.04 || tod > 0.96 ? { icon: 'wb_twilight', label: 'Dawn' }
		: tod < 0.21 ? { icon: 'light_mode', label: 'Morning' }
		: tod < 0.42 ? { icon: 'sunny', label: 'Day' }
		: tod < 0.54 ? { icon: 'wb_twilight', label: 'Evening' }
		: { icon: 'bedtime', label: 'Night' }
	);
	// card pack opening: which reward cards are flipped face-up
	let revealed = $state(new Set<number>());
	/** packs being torn open right now (foil animation running) */
	let tearing = $state(new Set<number>());
	/** tear the foil pack open: top strip rips off, the halves fly apart, then the card pops out with a jingle */
	function reveal(i: number, jingle = true) {
		if (revealed.has(i) || tearing.has(i) || !rewards) return;
		const d = CARD_MAP[rewards.ids[i]];
		const rare = d.cost >= 5 || !!d.special;
		tearing = new Set([...tearing, i]);
		sfx.crinkle();
		setTimeout(() => {
			revealed = new Set([...revealed, i]);
			if (jingle) sfx.reward(rare);
		}, 420);
		setTimeout(() => {
			const t = new Set(tearing);
			t.delete(i);
			tearing = t;
		}, 1100);
	}
	function revealAll() {
		if (!rewards) return;
		// one melody at the end instead of many overlapping ones
		const ids = rewards.ids;
		const anyRare = ids.some((id, i) => !revealed.has(i) && (CARD_MAP[id].cost >= 5 || !!CARD_MAP[id].special));
		const todo = ids.map((_, i) => i).filter((i) => !revealed.has(i) && !tearing.has(i));
		todo.forEach((i, k) => setTimeout(() => reveal(i, false), k * 260));
		if (todo.length) setTimeout(() => sfx.reward(anyRare), (todo.length - 1) * 260 + 420);
	}
	// ---------------- dev / debug mode (on by default, remembered; F9 toggles) ----------------
	const readFlag = (k: string, def: boolean) => {
		try {
			const v = localStorage.getItem(k);
			return v === null ? def : v === 'on';
		} catch {
			return def;
		}
	};
	const saveFlag = (k: string, on: boolean) => {
		try {
			localStorage.setItem(k, on ? 'on' : 'off');
		} catch {
			/* ignore */
		}
	};
	const POSE_KEY = 'brickhollow-pose';
	let debugOn = $state(readFlag('brickhollow-debug', true));
	let dbgPick = $state(readFlag('brickhollow-debug-pick', true));
	let dbgWire = $state(readFlag('brickhollow-debug-wire', false));
	let dbgOpen = $state(true);
	let dbgStats = $state({ fps: 0, calls: 0, tris: 0, geometries: 0 });
	let dbgReport = $state<{ total: number; rows: { name: string; tris: number; each: number; n: number }[] } | null>(null);
	function toggleDebug() {
		debugOn = !debugOn;
		saveFlag('brickhollow-debug', debugOn);
		applyDebug();
		toastIcon = 'bug_report';
		showToast(debugOn ? 'Debug mode ON' : 'Debug mode OFF');
	}
	function applyDebug() {
		if (!town) return;
		town.setDebug(debugOn);
		town.pickObjects = dbgPick;
		town.setWireframe(debugOn && dbgWire);
	}
	$effect(() => {
		if (!debugOn) return;
		const id = setInterval(() => {
			if (!town) return;
			dbgStats = town.debugStats();
			if (dbgWire) town.setWireframe(true); // also catches meshes that were loaded later
		}, 500);
		return () => clearInterval(id);
	});
	function onGlobalKey(e: KeyboardEvent) {
		if (e.key === 'F9' || e.key === '`') {
			e.preventDefault();
			toggleDebug();
		}
		if ((e.key === 'p' || e.key === 'P') && !e.repeat && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLTextAreaElement)) takePhoto();
	}
	/** a song from the song book was played to the end on Lumi's piano */
	function playedSong(name: string) {
		game.save.pianoSongs = (game.save.pianoSongs ?? 0) + 1;
		game.persist();
		sfx.reward(false);
		toastIcon = 'piano';
		showToast(`You played "${name}"!`);
	}
	/** activities listed in the help screen: [icon, text] */
	const FUN: [string, string][] = [
		['photo_camera', '<b>P</b>: take a photo (album on your PC)'],
		['celebration', '<b>Q</b>: dance! Villagers nearby join in'],
		['piano', "Play <b>Lumi's piano</b> at Dex's house"],
		['tv', 'Watch <b>TV</b>: news, weather and a nature show'],
		['flutter_dash', 'Catch <b>butterflies</b> by walking into them'],
		['nightlight', 'At night, catch glowing <b>fireflies</b>'],
		['sports_soccer', 'Kick the <b>ball</b> into the park goal'],
		['directions_run', 'Sprint into <b>Pip</b> to play tag'],
		['visibility_off', 'Ask Pip for <b>hide and seek</b>'],
		['bakery_dining', 'Feed the <b>ducks</b> at the pond (E)'],
		['chair', 'Sit on the <b>park bench</b> and watch time fly'],
		['water_drop', 'Splash through <b>puddles</b> after rain'],
		['redeem', 'Once a day a <b>treasure chest</b> sparkles somewhere'],
		['stars', 'Find all <b>golden studs</b>']
	];
	// ---------------- photos (P / camera button) ----------------
	let photoFlash = $state(0);
	let polaroid = $state<string | null>(null);
	let polaroidTimer: ReturnType<typeof setTimeout> | null = null;
	const canSnap = () => !!town && started && !battle && !dialog && !deckOpen && !helpOpen && !rewards && !encounter && !seekCount;
	function takePhoto() {
		if (!canSnap()) return;
		const src = town!.snapPhoto();
		if (!src) return;
		sfx.shutter();
		photoFlash++;
		album.add({ src, t: Date.now(), where: areaLabel });
		const s = game.save;
		s.photos = (s.photos ?? 0) + 1;
		game.persist();
		polaroid = src;
		if (polaroidTimer) clearTimeout(polaroidTimer);
		polaroidTimer = setTimeout(() => (polaroid = null), 2600);
	}
	let toastIcon = $state('grass');
	let toast = $state<string | null>(null);
	let musicOn = $state(true);
	// ---------------- music (crossfaded) ----------------
	// 'home' in the village, 'battle-intro' (12 s) when a battle starts, then 'fight', 'victory' after a win
	type Track = 'home' | 'fight' | 'battle-intro' | 'victory' | 'defeat';
	const LOOPS: Record<Track, boolean> = { home: true, fight: true, 'battle-intro': false, victory: false, defeat: false };
	const tracks: Partial<Record<Track, HTMLAudioElement>> = {};
	let current: Track = 'home';
	const VOL = 0.15;
	let introTimer: ReturnType<typeof setTimeout> | null = null;
	let tabFocused = true;
	function track(name: Track) {
		let a = tracks[name];
		if (!a) {
			a = new Audio(`/audio/${name}.mp3`);
			a.loop = LOOPS[name];
			a.muted = !tabFocused;
			a.volume = 0;
			tracks[name] = a;
		}
		return a;
	}
	function fadeAudio(a: HTMLAudioElement, to: number, ms = 600) {
		const from = a.volume;
		const t0 = performance.now();
		const step = () => {
			const k = Math.min(1, (performance.now() - t0) / ms);
			a.volume = from + (to - from) * k;
			if (k < 1) requestAnimationFrame(step);
			else if (to === 0) a.pause();
		};
		requestAnimationFrame(step);
	}
	function playTrack(name: Track) {
		const prevName = current;
		current = name;
		if (introTimer && name !== 'battle-intro') {
			clearTimeout(introTimer);
			introTimer = null;
		}
		if (!musicOn) return;
		const prev = tracks[prevName];
		if (prev && prevName !== name) fadeAudio(prev, 0);
		const a = track(name);
		if (name !== 'home') a.currentTime = 0;
		a.play().catch(() => {});
		fadeAudio(a, VOL);
	}
	let isTouch = $state(false);
	let ready = $state(false);
	let areaLabel = $state('Your Room');
	let fade = $state(0);
	let joy = $state<{ id: number; cx: number; cy: number; dx: number; dy: number } | null>(null);
	/** the menu behind the top-right button (things you rarely need) */
	let menuOpen = $state(false);
	/** touch controls: 'swipe' (default: swipe once to walk, tap to stop) or 'dpad' (the stick) */
	let controls = $state<'swipe' | 'dpad'>('swipe');
	try {
		if (localStorage.getItem('brickhollow-controls') === 'dpad') controls = 'dpad';
	} catch {
		/* storage unavailable */
	}
	function setControls(c: 'swipe' | 'dpad') {
		controls = c;
		town?.stopAutoWalk();
		town?.setVirtual(0, 0);
		try {
			localStorage.setItem('brickhollow-controls', c);
		} catch {
			/* ignore */
		}
	}
	/** swipe: where the finger went down (a short tap stops, a swipe walks that way) */
	let swipe: { id: number; x: number; y: number; t: number } | null = null;
	let swipeMark = $state<{ x: number; y: number; dx: number; dy: number; k: number } | null>(null);
	function swipeDown(e: PointerEvent) {
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		swipe = { id: e.pointerId, x: e.clientX, y: e.clientY, t: performance.now() };
	}
	function swipeMove(e: PointerEvent) {
		if (!swipe || e.pointerId !== swipe.id) return;
		const dx = e.clientX - swipe.x, dy = e.clientY - swipe.y;
		// already walking while the finger is still down, as soon as it is clearly a swipe
		if (Math.hypot(dx, dy) > 28) {
			town?.autoWalk(dx, dy);
			swipeMark = { x: swipe.x, y: swipe.y, dx, dy, k: (swipeMark?.k ?? 0) + 1 };
		}
	}
	function swipeUp(e: PointerEvent) {
		if (!swipe || e.pointerId !== swipe.id) return;
		const dx = e.clientX - swipe.x, dy = e.clientY - swipe.y;
		if (Math.hypot(dx, dy) > 28) town?.autoWalk(dx, dy);
		else town?.stopAutoWalk(); // a tap stops
		swipe = null;
	}

	const npcName = (id: string) => (id === 'gate:prof' ? 'Prof. Quill' : id === 'chest:treasure' ? 'the treasure chest' : (NPCS.find((n) => n.id === id)?.name ?? READABLES.find((r) => r.id === id)?.label ?? id));
	const busy = $derived(!!voxReveal || !!reading || !!dialog || !!battle || deckOpen || helpOpen || !!rewards || flash || encounter || !started);

	/** current goal shown in the HUD */
	const quest = $derived.by(() => {
		const s = game.save;
		if (!s.hasDeck) return { icon: 'science', text: 'Visit Prof. Quill in her lab (south-east)' };
		if (!s.rivalWins) return { icon: 'swords', text: 'Beat Dex — he waits at his big house (east)' };
		if (!s.voxelmon) return { icon: 'view_in_ar', text: `Win card battles for EXP and upgrade a 6-energy card into a Voxelmon · ${s.exp ?? 0} EXP` };
		if (!s.gateOpen) return { icon: 'pets', text: 'Show your Voxelmon to Prof. Quill at the village exit (north)' };
		if (s.collection.length < POOL.length) return { icon: 'grass', text: `Battle wild Voxelmon on Route 1 · ${s.collection.length}/${POOL.length} cards` };
		return { icon: 'military_tech', text: 'Collection complete! You are a Brick Beast master.' };
	});

	$effect(() => {
		const b = busy;
		if (town) {
			town.paused = b;
			if (b) town.clearKeys();
		}
	});

	function say(speaker: string, lines: string[], extra: Partial<DialogData> = {}) {
		dialogKey++;
		dialog = { speaker, lines, ...extra };
	}
	// ---------------- sticker book: announce newly earned stickers ----------------
	$effect(() => {
		if (!game.loaded) return;
		const s = game.save;
		const earned = earnedStickers(s);
		const known = untrack(() => s.stickers);
		if (!known) {
			// first time with the sticker book: older saves get their stickers silently
			untrack(() => {
				s.stickers = earned;
				game.persist();
			});
			return;
		}
		const fresh = earned.filter((id) => !known.includes(id));
		if (!fresh.length) return;
		untrack(() => {
			s.stickers = [...known, ...fresh];
			game.persist();
			const k = STICKERS.find((x) => x.id === fresh[0])!;
			// after the event's own message
			setTimeout(() => {
				sfx.fanfare();
				toastIcon = k.icon;
				showToast(`New sticker: ${k.name}! (sticker book on your PC)`);
			}, 1800);
		});
	});
	function showToast(t: string) {
		toast = t;
		setTimeout(() => toast === t && (toast = null), 2600);
	}

	// ---------------- NPC talk ----------------
	const READ_TEXT: Record<string, () => [string, string[]]> = {
		'mail:home': () => ['Mailbox', game.save.wins ? ['A letter: "Congrats on your first wins! — the Brickhollow Gazette"'] : ['It is empty. Maybe someone will write once you win a battle?']],
		'pc:bedroom': () => ['Your PC', ['The wallpaper is a very cool Brick Beast. Obviously.', '3 new messages from Dex: "SNAP!"  "snap."  "SNAP!!!"', game.save.wins ? `Your battle log says ${game.save.wins} win${game.save.wins === 1 ? '' : 's'}. Not bad!` : 'Your battle log is still empty. Time to change that!']],
		'tv:home': () => ['TV', ['📺 "…and today’s Brick Beast report: Glimmerlings were spotted folding light into tiny bricks!"', 'You turn the TV off. Mom would want you outside anyway.']]
	};

	/** sleep in your bed: skips to the next morning (or naps until the evening) */
	function sleep() {
		if (!town) return;
		const night = town.timeOfDay > 0.55 || town.timeOfDay < 0.05;
		town.timeOfDay = night ? 0.1 : 0.56;
		town.vox.heal(); // your Voxelmon rests too
		tod = town.timeOfDay;
		fade++;
		sfx.blip();
		toastIcon = night ? 'wb_sunny' : 'bedtime';
		showToast(night ? 'Zzz… Good morning, Brickhollow!' : 'Zzz… A cosy nap until the evening.');
	}
	/** verb + icon for the interaction prompt */
	function promptFor(id: string): [string, string] {
		if (id.startsWith('bed:')) return ['Sleep in', 'bedtime'];
		if (id.startsWith('bench:')) return ['Sit on', 'chair'];
		if (id.startsWith('chest:')) return ['Open', 'redeem'];
		if (id === 'pond:ducks') return ['Feed', 'bakery_dining'];
		if (id.startsWith('pc:')) return ['Use', 'computer'];
		if (id.startsWith('tv:')) return ['Watch', 'tv'];
		if (id.startsWith('piano:')) return ['Play', 'piano'];
		if (READ_TEXT[id] || SIGNS[id]) return ['Read', 'visibility'];
		return ['Talk to', 'chat'];
	}

	function talk(id: string) {
		if (id === 'bed:home') return sleep();
		if (id === 'gate:prof') return gateTalk();
		if (id === 'chest:treasure') {
			const n = town?.openChest() ?? 0;
			if (n) {
				const s = game.save;
				s.chests = (s.chests ?? 0) + 1;
				s.studs += n;
				game.persist();
				sfx.reward(true);
				toastIcon = 'redeem';
				showToast(`Treasure! You found ${n} studs.`);
			}
			return;
		}
		if (id === 'pond:ducks') {
			if (town?.feedDucks()) {
				const s = game.save;
				s.duckFeeds = (s.duckFeeds ?? 0) + 1;
				game.persist();
				if (s.duckFeeds === 1 || s.duckFeeds % 10 === 0) {
					toastIcon = 'bakery_dining';
					showToast(s.duckFeeds === 1 ? 'Quack! The ducks love bread.' : `You've fed the ducks ${s.duckFeeds} times!`);
				}
			}
			return;
		}
		if (id.startsWith('bench:')) {
			if (town?.sit(id)) sfx.blip();
			return;
		}
		if (SCREENS.includes(id) && town?.readSign(id)) {
			sfx.blip();
			return;
		}
		if (SIGNS[id]) {
			if (town?.readSign(id)) sfx.blip();
			return;
		}
		if (READ_TEXT[id]) {
			const [title, lines] = READ_TEXT[id]();
			sfx.blip();
			say(title, lines);
			return;
		}
		if (id === 'mom') momTalked = true;
		if (id === 'kid') kidTalked = true;
		updateMarkers();
		const s = game.save;
		const owned = s.collection.length;
		switch (id) {
			case 'mom':
				if (!s.hasDeck)
					say('Mom', [
						"Morning, sweetie! Professor Quill was looking for you earlier.",
						'Her lab is the big tan building with the solar panels — just across the road to the south-east.'
					]);
				else
					say('Mom', [
						s.wins ? `${s.wins} win${s.wins === 1 ? '' : 's'} already? I'm so proud of you!` : 'Off on an adventure? Be careful in that tall grass!',
						'Remember: you can rebuild your deck any time with the Deck button up top.',
						'And snap only when you feel lucky. Your father lost a whole bucket of studs that way!'
					]);
				break;
			case 'prof':
				if (!s.hasDeck) {
					say('Prof. Hazel Quill', [
						"Ah, there you are! Welcome to Quill Lab.",
						'All around Brickhollow live BRICK BEASTS — little creatures snapped together from living bricks!',
						'Trainers here battle with decks of 12 Beast cards, fighting over 3 zones for 6 turns.',
						`Here are ${STARTER_COLLECTION.length} cards to start your collection. I've already built you a deck of 12!`,
						'Wild Beasts hide in the tall grass on Route 1, north of the village. Beat them to collect new cards!'
					], {
						onEnd: () => {
							game.giveStarter();
							rewards = { title: 'You received your starter collection!', ids: STARTER_COLLECTION, then: () =>
								say('Prof. Hazel Quill', ["Lumi, Dex's sister, knows the rules by heart — visit their big house in the east for a practice match.", 'And watch out for my grandson Dex. He has been itching for a battle all week!']) };
						}
					});
				} else
					say('Prof. Hazel Quill', [
						`You have ${owned} of ${POOL.length} Brick Beast cards. ${owned >= POOL.length ? 'A complete collection — incredible!' : 'Keep exploring Route 1!'}`,
						'Did you know? Glimmerlings fold light into tiny bricks. Fascinating creatures.'
					]);
				break;
			case 'rival':
				if (!s.hasDeck) {
					say('Dex', ["Hey, slowpoke! You don't even have a deck yet.", "Go see Grandma Quill at the lab. Then I'll crush you!"]);
				} else {
					say('Dex', s.rivalWins ? ['Back for more? This time I upgraded my deck!', 'Ready?'] : ['So Grandma gave you a deck, huh? Mine is WAY cooler.', "Let's battle — right here, right now!"], {
						choices: [
							{ label: 'Battle!', icon: 'swords', action: () => startBattle('rival') },
							{ label: 'Not now', icon: 'close', action: () => say('Dex', ['Scared? Ha! Come back when you are ready.']) }
						]
					});
				}
				break;
			case 'sister':
				say('Lumi', [
					"Hi! I'm Lumi, Dex's sister. Want a quick lesson?",
					'Each turn you get energy equal to the turn number — 1 on turn 1, up to 6 on turn 6.',
					'Play cards into the 3 zones. Zones reveal one by one on turns 1 to 3 and each has its own effect.',
					'Both players plan in secret, then cards reveal. Whoever is ahead reveals first.',
					'Cards come in ability types: On Reveal fires when the card flips, Ongoing works all the time.',
					'Activate cards can be used once — tap them on the board. End of Turn cards trigger after every turn, and Game Start cards do their thing before the first draw.',
					'Tap any card to see it big, and tap a location to read what it does!',
					'After turn 6, win 2 of the 3 zones to win! Hit SNAP to double the stakes if you feel confident.'
				], {
					choices: s.hasDeck
						? [
								{ label: 'Practice battle', icon: 'school', action: () => startBattle('practice') },
								{ label: 'Thanks!', icon: 'thumb_up', action: () => {} }
							]
						: [{ label: 'Got it!', icon: 'thumb_up', action: () => {} }]
				});
				break;
			case 'knight':
				say('Sir Bricksworth', [
					'Welcome to Stonecrest, traveller! I guard the old castle.',
					'It was built from the very first bricks of the valley. Some say Brick Beasts still live in its towers.',
					'The gate is closed for now. Come back when you are a famous trainer!'
				]);
				break;
			case 'baker':
				say('Marla', ['Oh, a visitor from Brickhollow! You came all the way through the forest?', 'Have a look around. Our fountain is the prettiest in the whole valley.']);
				break;
			case 'kid':
				say('Pip', [
					'Psst! I saw something wiggle in the tall grass up north!',
					`Also… there are ${GOLDEN_STUDS.length} golden studs hidden around Brickhollow. Some are even inside houses! Find them all and something legendary happens.`,
					`My big brother says there are ${POOL.length} different Brick Beasts. I only know the fishy ones.`
				], {
					choices: [
						{ label: 'Play hide and seek', icon: 'visibility_off', action: startHideSeek },
						{ label: 'Bye, Pip!', icon: 'waving_hand', action: () => {} }
					]
				});
				break;
		}
	}

	// ---------------- Prof. Quill at the village exit ----------------
	function gateTalk() {
		const s = game.save;
		if (!s.hasDeck)
			say('Prof. Hazel Quill', [
				"Wait! Don't go out there!",
				"Wild Brick Beasts live in the tall grass. You can't go without any Beast cards!",
				"Come to my lab first. I'll give you your very first cards."
			]);
		else if (!game.voxelmons().length)
			say('Prof. Hazel Quill', [
				"Hold on! Cards alone won't protect you out there.",
				'The wild Beasts on Route 1 are real Voxelmon, made of living bricks!',
				'Win card battles to earn EXP, and upgrade your cards in the Deck menu.',
				'Once a six energy card reaches its final stage, it becomes a real Voxelmon. Then you may pass!'
			]);
		else
			say('Prof. Hazel Quill', ['Oh my! A real Voxelmon! Then you are ready.', 'Press 1 and 2 in battle to use its attacks. Good luck out there!'], { onEnd: openGate });
	}
	function openGate() {
		game.save.gateOpen = true;
		game.persist();
		sfx.fanfare();
		toastIcon = 'grass';
		showToast('The way to Route 1 is open!');
	}

	// ---------------- Voxelmon battles in the tall grass ----------------
	let voxHud = $state<ReturnType<VoxelPlay['hud']>>(null);
	function pollVox() {
		voxHud = town?.vox.hud() ?? null;
		if (town?.vox.busy) requestAnimationFrame(pollVox);
	}
	function onVox(e: VoxEvent) {
		const s = game.save;
		if (e.type === 'start') {
			town?.exclaim(700);
			if (e.shiny) {
				sfx.fanfare();
				toastIcon = 'auto_awesome';
				showToast(`A SHINY wild ${CARD_MAP[e.wild].name} appeared!`);
			} else {
				sfx.alert();
				toastIcon = 'grass';
				showToast(`A wild ${CARD_MAP[e.wild].name} jumped out of the grass!`);
			}
			playTrack('fight');
			pollVox();
		} else if (e.type === 'skill') sfx.attack(e.id, CARD_MAP[e.id].cost);
		else if (e.type === 'hit') sfx.hit();
		else {
			playTrack('home');
			setTimeout(() => (voxHud = town?.vox.hud() ?? null), 1700);
			if (e.type === 'win') {
				s.voxWins = (s.voxWins ?? 0) + 1;
				s.exp = (s.exp ?? 0) + WILD_EXP;
				s.studs += e.shiny ? 10 : 3;
				// beating it wins you its card (a shiny one gives a rare card)
				const id = e.shiny ? game.rewardRareCard() : !s.collection.includes(e.wild) ? (game.giveCard(e.wild), e.wild) : null;
				game.persist();
				sfx.reward(e.shiny);
				toastIcon = 'auto_awesome';
				showToast(`You won! +${WILD_EXP} EXP · +${e.shiny ? 10 : 3} studs`);
				if (id) setTimeout(() => (rewards = { title: `${CARD_MAP[id].name}'s card joined your collection!`, ids: [id] }), 1400);
			} else if (e.type === 'lose') {
				sfx.alert();
				toastIcon = 'bedtime';
				showToast(`${CARD_MAP[s.voxelmon!].name} is worn out… Let it rest, or sleep in your bed.`);
			} else {
				toastIcon = 'directions_run';
				showToast('Got away safely!');
			}
		}
	}

	// your Voxelmon walks with you (changes when you pick another one in the deck builder)
	$effect(() => {
		const id = game.save.voxelmon ?? null;
		if (!started) return;
		town?.vox.setCompanion(id);
		voxHud = town?.vox.hud() ?? null;
	});

	// ---------------- hide and seek with Pip ----------------
	let seekCount = $state(0);
	let seekEnd = $state(0);
	let seekNow = $state(0);
	function startHideSeek() {
		// eyes closed: the screen goes dark and counts down while Pip runs off
		town?.clearKeys();
		seekCount = 3;
		sfx.blip();
		const tick = () => {
			seekCount--;
			if (seekCount === 2 && !town?.startHideSeek()) {
				seekCount = 0;
				toastIcon = 'sentiment_dissatisfied';
				showToast('Pip found no hiding spot. Try again outside!');
				return;
			}
			if (seekCount > 0) {
				sfx.blip();
				setTimeout(tick, 900);
			} else {
				sfx.alert();
				seekEnd = performance.now() + 60000;
				toastIcon = 'visibility';
				showToast('Ready or not! Find Pip within 60 seconds.');
				const clock = () => {
					seekNow = performance.now();
					if (seekEnd && seekNow < seekEnd) requestAnimationFrame(clock);
				};
				clock();
			}
		};
		setTimeout(tick, 900);
	}

	// ---------------- battles ----------------
	function randomWildDeck() {
		const pool = [...POOL].sort(() => Math.random() - 0.5);
		// keep a reasonable curve
		const deck: string[] = [];
		for (const id of pool) {
			const c = CARD_MAP[id].cost;
			if (deck.filter((d) => CARD_MAP[d].cost === c).length >= 3) continue;
			deck.push(id);
			if (deck.length === 12) break;
		}
		return deck;
	}

	function startBattle(kind: 'wild' | 'rival' | 'practice', shiny = false) {
		const s = game.save;
		const aiDeck = kind === 'rival' ? (s.rivalWins ? RIVAL_DECK_2 : RIVAL_DECK) : kind === 'practice' ? PRACTICE_DECK : randomWildDeck();
		const lead = kind === 'wild' ? CARD_MAP[aiDeck[Math.floor(Math.random() * aiDeck.length)]].name : '';
		const opponent = kind === 'rival' ? (s.rivalWins ? 'Dex ★ Rematch' : 'Dex') : kind === 'practice' ? 'Lumi' : `${shiny ? '✨ Shiny' : 'Wild'} ${lead} pack`;
		flash = true;
		// short fanfare as transition, then the battle music takes over (the intro fades out at 10.5–12 s)
		playTrack('battle-intro');
		introTimer = setTimeout(() => {
			introTimer = null;
			if (current === 'battle-intro') playTrack('fight');
		}, 10800);
		setTimeout(() => {
			battle = new Battle({ kind, opponent, playerDeck: s.deck, aiDeck, shiny });
			if (import.meta.env.DEV) Object.assign(window, { __battle: battle });
			flash = false;
		}, 750);
	}

	function battleDone(r: Result, stakes: number) {
		const b = battle!;
		battle = null;
		playTrack('home');
		const s = game.save;
		let delta = 0;
		if (r === 'win') {
			delta = stakes;
			s.wins++;
		} else if (r === 'lose') {
			delta = -stakes;
			s.losses++;
		} else if (r === 'retreat') delta = -Math.max(1, Math.floor(stakes / 2));
		s.studs = Math.max(0, s.studs + delta);
		// every card battle teaches you something: EXP to upgrade your cards
		const exp = b.opts.kind === 'wild' ? 0 : battleExp(r, b.opts.kind);
		if (exp) s.exp = (s.exp ?? 0) + exp;

		const ids: string[] = [];
		if (r === 'win') {
			if (b.opts.shiny) {
				const id = game.rewardRareCard();
				if (id) ids.push(id);
				s.studs += 10;
			} else {
				const n = b.opts.kind === 'rival' && s.rivalWins === 0 ? 2 : 1;
				for (let i = 0; i < n; i++) {
					const id = game.rewardCard();
					if (id) ids.push(id);
				}
			}
			if (b.opts.kind === 'rival') s.rivalWins++;
		}
		game.persist();

		const after = () => {
			if (b.opts.kind === 'rival')
				say('Dex', r === 'win' ? ['WHAT?! No way... my deck was perfect!', "Fine. You win this time. I'm going to train on Route 1!"] : ["Ha! Told you my deck was cooler.", 'Come back when you have some better cards!']);
			else if (b.opts.kind === 'practice')
				say('Lumi', r === 'win' ? ['Wow, you learn fast! Here, keep that card.'] : ['Good try! Watch which zones you can still win and focus there.']);
		};
		if (exp) {
			toastIcon = 'auto_awesome';
			showToast(`+${exp} EXP · upgrade your cards in the Deck menu`);
		}
		if (ids.length) rewards = { title: b.opts.shiny ? '✨ Shiny reward! A rare card and +10 studs' : r === 'win' ? `New card added to your collection!${exp ? ` · +${exp} EXP` : ''}` : '', ids, then: after };
		else after();
	}

	// ---------------- music ----------------
	/** "!" over whoever matters for the current goal, "…" for small talk */
	function updateMarkers() {
		const s = game.save;
		town?.setGoal(!s.hasDeck ? (momTalked ? 'prof' : 'mom') : !s.rivalWins ? 'rival' : s.collection.length < POOL.length ? 'grass' : null);
		town?.setNpcMarkers({
			prof: !s.hasDeck ? '!' : '…',
			rival: s.hasDeck && !s.rivalWins ? '!' : '…',
			mom: !s.hasDeck && !momTalked ? '!' : '…',
			sister: s.hasDeck && !s.wins ? '!' : '…',
			kid: s.golden.length < GOLDEN_STUDS.length && !kidTalked ? '!' : '…'
		});
	}
	let momTalked = false;
	let kidTalked = false;
	$effect(() => {
		// refresh markers whenever progress changes
		void [game.save.hasDeck, game.save.rivalWins, game.save.wins, game.save.golden.length];
		updateMarkers();
	});

	function collectGolden(id: string) {
		const s = game.save;
		if (s.golden.includes(id)) return;
		s.golden.push(id);
		s.studs += 5;
		game.persist();
		sfx.coin();
		const n = s.golden.length;
		toastIcon = 'stars';
		if (n >= GOLDEN_STUDS.length) {
			sfx.fanfare();
			game.giveCard('goldbrick');
			rewards = { title: 'All golden studs found! A legendary card appears…', ids: ['goldbrick'] };
		} else showToast(`Golden stud! ${n}/${GOLDEN_STUDS.length} found · +5 studs`);
	}

	// the 3D village is fully hidden behind a battle or the deck builder: stop rendering it (saves GPU for the cards)
	$effect(() => {
		const hide = (!!battle && !flash) || deckOpen;
		if (town) town.suspended = hide;
	});

	// a won battle: the victory music plays over the result screen (the village music returns afterwards)
	// a lost battle: the fight music fades out, a moment of silence, then the defeat music
	let defeatTimer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const r = battle?.result;
		if (r === 'win' && current !== 'victory') playTrack('victory');
		if (r === 'lose' && current === 'fight' && !defeatTimer) {
			const a = tracks.fight;
			if (a && musicOn) fadeAudio(a, 0, 900);
			defeatTimer = setTimeout(() => {
				defeatTimer = null;
				if (battle?.result === 'lose') playTrack('defeat');
			}, 1200);
		}
	});

	function toggleMusic() {
		musicOn = !musicOn;
		const a = track(current);
		if (musicOn) {
			a.play().catch(() => {});
			fadeAudio(a, VOL);
		} else fadeAudio(a, 0, 250);
	}

	function start() {
		started = true;
		playTrack('home');
		if (!game.save.hasDeck) {
			const swiping = controls === 'swipe';
			setTimeout(() => say('You', ["*yawn* What a beautiful morning in Brickhollow.", isTouch ? swiping ? 'I should go downstairs and talk to Mom in the kitchen. Swipe to walk, tap to stop, and tap the talk button.' : 'I should go downstairs and talk to Mom in the kitchen. Drag the stick to walk and tap the talk button.' : 'I should go downstairs and talk to Mom in the kitchen — WASD / arrow keys to walk, E or Space to talk.']), 400);
		}
	}

	function resetSave() {
		if (confirm('Delete your save and start over?')) {
			game.reset();
			helpOpen = false;
		}
	}

	// ---------------- touch joystick ----------------
	const JOY_R = 48;
	function joyDown(e: PointerEvent) {
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		joy = { id: e.pointerId, cx: e.clientX, cy: e.clientY, dx: 0, dy: 0 };
	}
	function joyMove(e: PointerEvent) {
		if (!joy || e.pointerId !== joy.id) return;
		let dx = e.clientX - joy.cx;
		let dy = e.clientY - joy.cy;
		const d = Math.hypot(dx, dy);
		if (d > JOY_R) {
			dx = (dx / d) * JOY_R;
			dy = (dy / d) * JOY_R;
		}
		joy.dx = dx;
		joy.dy = dy;
		const k = d < 8 ? 0 : 1 / JOY_R;
		town?.setVirtual(dx * k, dy * k);
	}
	function joyUp() {
		joy = null;
		town?.setVirtual(0, 0);
	}
	function zoomBy(d: number) {
		if (town) town.zoom = Math.min(1.7, Math.max(0.55, town.zoom + d));
	}

	onMount(() => {
		let disposed = false;
		game.init();
		album.load();
		loadVoices();
		Promise.all([preloadFigure(), preloadSprites()]).then(() => {
			if (!disposed) createTown();
		});
		// no game audio at all while the tab has no focus (music, voices and sound effects)
		const syncFocus = () => {
			const on = document.visibilityState === 'visible' && document.hasFocus();
			tabFocused = on;
			for (const a of Object.values(tracks)) if (a) a.muted = !on;
			setSfxFocused(on);
			setVoiceFocused(on);
		};
		window.addEventListener('focus', syncFocus);
		window.addEventListener('blur', syncFocus);
		document.addEventListener('visibilitychange', syncFocus);
		syncFocus();
		return () => {
			window.removeEventListener('focus', syncFocus);
			window.removeEventListener('blur', syncFocus);
			document.removeEventListener('visibilitychange', syncFocus);
			disposed = true;
			town?.dispose();
			Object.values(tracks).forEach((a) => a?.pause());
		};
	});

	function createTown() {
		isTouch = matchMedia('(pointer: coarse)').matches;
		town = new Town(host, {
			onNearby: (id) => (nearby = id),
			onInteract: (id) => talk(id),
			onReading: (id) => (reading = id),
			// something rustles, but your Voxelmon can't fight right now
			onGrass: () => {
				const v = game.save.voxelmon;
				toastIcon = 'bedtime';
				showToast(v ? `The grass rustles… but ${CARD_MAP[v].name} is too tired to fight. Let it rest!` : 'The grass rustles… better not wander here without a Voxelmon.');
			},
			onStep: (surface) => sfx.step(surface),
			onGolden: (id) => collectGolden(id),
			onSplash: (loud) => sfx.splash(loud),
			onFishJump: () => sfx.fishSplash(),
			onKick: (s) => sfx.kick(s),
			onQuack: (pitch) => sfx.quack(pitch),
			onDance: (on, crowd) => {
				if (!on) return;
				sfx.dance();
				if (crowd >= 1) {
					game.save.danceParties = (game.save.danceParties ?? 0) + 1;
					game.persist();
					toastIcon = 'celebration';
					showToast(crowd > 1 ? 'Dance party! Everyone joins in.' : 'Dance party! Somebody joins in.');
				}
			},
			onChestSpawn: () => {
				sfx.flutter();
				toastIcon = 'redeem';
				showToast('Something sparkles somewhere in the village…');
			},
			onHideSeek: (ev, text) => {
				if (ev === 'hint') {
					toastIcon = /^(Hot|Uh)/.test(text ?? '') ? 'local_fire_department' : /^(Warm|Getting)/.test(text ?? '') ? 'thermostat' : 'ac_unit';
					showToast(`Pip: "${text}"`);
					return;
				}
				seekEnd = 0;
				if (ev === 'found') {
					const s = game.save;
					s.hideSeekWins = (s.hideSeekWins ?? 0) + 1;
					s.studs += 5;
					game.persist();
					sfx.reward(false);
					toastIcon = 'visibility';
					showToast('You found Pip! +5 studs');
				} else {
					sfx.blip();
					toastIcon = 'sentiment_very_satisfied';
					showToast('Time is up! Pip was hiding really well.');
				}
			},
			onFirefly: () => {
				const s = game.save;
				s.fireflies = (s.fireflies ?? 0) + 1;
				s.studs += 1;
				game.persist();
				sfx.twinkle();
				toastIcon = 'nightlight';
				showToast(`A firefly! (${s.fireflies} caught) · +1 stud`);
			},
			onPuddle: () => {
				game.save.puddleSplashes = (game.save.puddleSplashes ?? 0) + 1;
				game.persist();
			},
			onSit: (seated) => {
				if (!seated) return;
				game.save.benchSits = (game.save.benchSits ?? 0) + 1;
				game.persist();
				toastIcon = 'schedule';
				showToast('Time flies on the bench… (move to get up)');
			},
			onGoal: () => {
				const s = game.save;
				s.goals = (s.goals ?? 0) + 1;
				s.studs += 1;
				game.persist();
				sfx.reward(false);
				toastIcon = 'sports_soccer';
				showToast(`GOAL! (${s.goals} scored) · +1 stud`);
			},
			onRain: (level) => sfx.rain(level),
			onTag: (ev) => {
				if (ev === 'start') {
					sfx.alert();
					toastIcon = 'directions_run';
					showToast('Tag with Pip! Catch him again within 10 seconds (sprint with Shift)');
				} else if (ev === 'caught') {
					game.save.studs += 3;
					game.save.tagWins = (game.save.tagWins ?? 0) + 1;
					game.persist();
					sfx.reward(false);
					toastIcon = 'emoji_events';
					showToast('You tagged Pip! +3 studs');
				} else {
					sfx.blip();
					toastIcon = 'sentiment_very_satisfied';
					showToast('Pip got away! Try again in a bit.');
				}
			},
			onButterfly: () => {
				const s = game.save;
				s.butterflies = (s.butterflies ?? 0) + 1;
				s.studs += 1;
				game.persist();
				sfx.flutter();
				toastIcon = 'flutter_dash';
				showToast(`A butterfly! (${s.butterflies} caught) · +1 stud`);
			},
			onDebugPick: (id) => {
				toastIcon = 'content_copy';
				showToast(`Copied: ${id}`);
			},
			onBlockedRoute: () => gateTalk(),
			onZone: (label) => {
				areaLabel = label;
				toastIcon = label === 'Stonecrest' ? 'castle' : label === 'Route 1' ? 'forest' : 'home_pin';
				showToast(label === 'Stonecrest' ? 'Welcome to Stonecrest!' : label === 'Route 1' ? 'Route 1' : 'Back in Brickhollow');
			},
			onVox: (e) => onVox(e),
			onBlockedGrass: () => (toastIcon = 'grass') && showToast("The grass is rustling... You need a deck first! Visit Prof. Quill's lab."),
			onArea: (_id, label) => {
				areaLabel = label;
				fade++;
				sfx.door();
			}
		});
		town.hasDeck = () => game.save.hasDeck;
		town.routeOpen = () => !!game.save.gateOpen;
		town.vox.setCompanion(game.save.voxelmon ?? null);
		voxHud = town.vox.hud();
		if (import.meta.env.DEV) Object.assign(window, { __town: town, __dev: { talk, startBattle, game, showRewards: (ids: string[]) => (rewards = { title: 'Test pack', ids }) } });
		town.paused = busy;
		applyDebug();
		// continue where the player left off (saved in localStorage)
		try {
			const raw = localStorage.getItem(POSE_KEY);
			const label = raw ? town.restorePose(JSON.parse(raw)) : null;
			if (label) areaLabel = label;
		} catch {
			/* no / broken saved position: start in your room */
		}
		const savePose = () => {
			if (!town || battle) return;
			try {
				localStorage.setItem(POSE_KEY, JSON.stringify(town.getPose()));
			} catch {
				/* ignore */
			}
		};
		setInterval(savePose, 2000);
		window.addEventListener('pagehide', savePose);
		town.setCollected(game.save.golden);
		setInterval(() => {
			if (!town) return;
			tod = town.timeOfDay;
			// your Voxelmon's health recovers slowly between battles
			if (!town.vox.busy) voxHud = town.vox.hud();
		}, 1500);
		updateMarkers();
		ready = true;
	}
</script>

<svelte:window onkeydown={onGlobalKey} bind:innerWidth={vw} bind:innerHeight={vh} />
<Tooltip />

<svelte:head>
	<title>Brickhollow: Snap Beasts</title>
</svelte:head>

<div class="fixed inset-0 overflow-hidden bg-[#bfe6ff]" style="font-family: 'Fredoka', system-ui, sans-serif">
	<div bind:this={host} class="absolute inset-0"></div>

	{#if started}
		<!-- HUD -->
		<div class="hud-left">
			<div class="pill font-bold">
				<span class="material-symbols-rounded text-[#c91a09]">home_pin</span> <span class="truncate max-w-[38vw]">{areaLabel}</span>
			</div>
			<div class="pill" title="Studs (won from battles)">
				<span class="material-symbols-rounded text-[#f0a020]">toll</span>{game.save.studs}
			</div>
		</div>
		<div class="hud-right">
			{#if game.save.hasDeck}
				<button class="ibtn" title="Deck" onclick={() => (deckOpen = true)}><FluentIcon name="deck" /> <span class="hidden sm:inline">Deck</span></button>
			{/if}
			<button class="ibtn" class:on={menuOpen} title="Menu" aria-expanded={menuOpen} onclick={() => (menuOpen = !menuOpen)}><span class="material-symbols-rounded">{menuOpen ? 'close' : 'menu'}</span></button>
		</div>
		{#if menuOpen}
			<!-- the things you rarely need: progress, photo, sound, controls, help -->
			<button class="menu-shade" aria-label="Close menu" onclick={() => (menuOpen = false)}></button>
			<div class="menu" role="menu">
				<div class="menu-stats">
					<span title="Time of day"><span class="material-symbols-rounded text-[#f0a020]">{todInfo.icon}</span>{todInfo.label}</span>
					<span title="Golden studs found"><span class="material-symbols-rounded text-[#e0a800]">stars</span>{game.save.golden.length}/{GOLDEN_STUDS.length}</span>
					{#if game.save.hasDeck}
						<span title="Cards collected"><span class="material-symbols-rounded text-[#3d9bff]">style</span>{game.save.collection.length}/{POOL.length}</span>
						<span title="Wins / Losses"><FluentIcon name="trophy" size={18} />{game.save.wins} / {game.save.losses}</span>
					{/if}
				</div>
				<button class="menu-item" onclick={() => { menuOpen = false; takePhoto(); }}><span class="material-symbols-rounded">photo_camera</span>Take a photo<kbd class="hidden sm:inline">P</kbd></button>
				<button class="menu-item" onclick={() => { voicesOn = !voicesOn; setVoicesEnabled(voicesOn); }}>
					<span class="material-symbols-rounded">{voicesOn ? 'record_voice_over' : 'voice_over_off'}</span>Voices<span class="state">{voicesOn ? 'on' : 'off'}</span>
				</button>
				<button class="menu-item" onclick={toggleMusic}><FluentIcon name={musicOn ? 'speaker' : 'mute'} />Music<span class="state">{musicOn ? 'on' : 'off'}</span></button>
				{#if isTouch}
					<div class="menu-label">Controls</div>
					<div class="seg">
						<button class:on={controls === 'swipe'} onclick={() => setControls('swipe')}><span class="material-symbols-rounded">swipe</span>Swipe</button>
						<button class:on={controls === 'dpad'} onclick={() => setControls('dpad')}><span class="material-symbols-rounded">gamepad</span>D-pad</button>
					</div>
					<div class="menu-hint">{controls === 'swipe' ? 'Swipe once to walk that way, tap to stop.' : 'Hold and drag the stick to walk.'}</div>
				{/if}
				<button class="menu-item" onclick={() => { menuOpen = false; helpOpen = true; }}><FluentIcon name="help" />Help</button>
			</div>
		{/if}

		<div class="quest" title="Current goal">
			<span class="material-symbols-rounded">{quest.icon}</span>
			<span>{quest.text}</span>
		</div>

		{#if nearby && !busy}
			<button class="prompt" onclick={() => town?.interact()}>
				{#if isTouch}<span class="material-symbols-rounded">{promptFor(nearby)[1]}</span>{:else}<kbd>E</kbd>{/if} {promptFor(nearby)[0]} {npcName(nearby)}
			</button>
		{/if}

		{#if debugOn}
			<div class="dbg">
				<button class="dbg-head" onclick={() => (dbgOpen = !dbgOpen)}>
					<span class="material-symbols-rounded">bug_report</span> DEBUG
					<span class="dbg-fps">{dbgStats.fps} fps</span>
					<span class="material-symbols-rounded">{dbgOpen ? 'expand_more' : 'expand_less'}</span>
				</button>
				{#if dbgOpen}
					<label><input type="checkbox" bind:checked={dbgPick} onchange={() => (saveFlag('brickhollow-debug-pick', dbgPick), applyDebug())} /> Pick objects (click copies id)</label>
					<label><input type="checkbox" bind:checked={dbgWire} onchange={() => (saveFlag('brickhollow-debug-wire', dbgWire), applyDebug())} /> Wireframe</label>
					<div class="dbg-stats">
						<span>{dbgStats.calls} draw calls</span>
						<span>{(dbgStats.tris / 1000).toFixed(0)}k triangles / frame</span>
						<span>{dbgStats.geometries} geometries</span>
					</div>
					<button class="dbg-btn" onclick={() => (dbgReport = dbgReport ? null : (town?.polygonReport() ?? null))}>
						<span class="material-symbols-rounded">analytics</span> {dbgReport ? 'Hide' : 'Polygon report'}
					</button>
					{#if dbgReport}
						<div class="dbg-report">
							<div class="dbg-total">scene total: {(dbgReport.total / 1000).toFixed(0)}k triangles</div>
							{#each dbgReport.rows as r}
								<div class="dbg-row"><span class="name" title={r.name}>{r.name}</span><span>{r.n > 1 ? `${r.n}× ${(r.each / 1000).toFixed(1)}k` : ''}</span><b>{(r.tris / 1000).toFixed(0)}k</b></div>
							{/each}
						</div>
					{/if}
					<div class="dbg-hint">F9 turns debug mode off</div>
				{/if}
			</div>
		{/if}

		{#if seekCount > 0}
			<div class="seek-eyes"><span class="material-symbols-rounded">visibility_off</span><b>{seekCount}</b><small>Pip is hiding!</small></div>
		{/if}
		{#if seekEnd && seekNow < seekEnd}
			<div class="seek-timer"><span class="material-symbols-rounded">visibility</span>{Math.ceil((seekEnd - seekNow) / 1000)}s</div>
		{/if}
		{#if voxHud?.wild && voxHud.phase !== 'none'}
			{@const w = voxHud.wild}
			{@const me = voxHud.me}
			<div class="vx-wild" class:shiny={w.shiny}>
				<div class="vx-head"><span class="material-symbols-rounded">{w.shiny ? 'auto_awesome' : 'grass'}</span><b>{w.shiny ? 'Shiny' : 'Wild'} {w.name}</b><small>{w.hp}/{w.max}</small></div>
				<div class="hpbar"><i style="width:{(w.hp / w.max) * 100}%"></i></div>
			</div>
			<div class="vx-me">
				<div class="vx-head"><span class="material-symbols-rounded">pets</span><b>{me.name}</b><small>{me.hp}/{me.max}</small></div>
				<div class="hpbar mine"><i style="width:{(me.hp / me.max) * 100}%"></i></div>
				<div class="vx-skills">
					{#each voxHud.skills as sk, i}
						<button class="vx-skill" style="--c:{sk.color}; --p:{(sk.left / sk.cd) * 100}%" disabled={sk.left > 0 || voxHud.phase !== 'fight'} onclick={() => town?.vox.useSkill(i)}>
							<span class="material-symbols-rounded">{sk.icon}</span>
							<b>{sk.name}</b>
							<kbd>{i + 1}</kbd>
							{#if sk.left > 0}<span class="cd">{sk.left.toFixed(1)}s</span>{/if}
						</button>
					{/each}
					<button class="vx-run" onclick={() => town?.vox.flee()} disabled={voxHud.phase === 'end'}><span class="material-symbols-rounded">directions_run</span>Run</button>
				</div>
			</div>
		{:else if voxHud && voxHud.me.hp < voxHud.me.max}
			<!-- between battles: your Voxelmon is still recovering -->
			<div class="vx-rest" title="Your Voxelmon recovers over time (or sleep in your bed)">
				<span class="material-symbols-rounded">{voxHud.me.tired ? 'bedtime' : 'favorite'}</span>
				{voxHud.me.name}
				<div class="hpbar mine"><i style="width:{(voxHud.me.hp / voxHud.me.max) * 100}%"></i></div>
			</div>
		{/if}
		{#if voxReveal}
			<VoxelReveal id={voxReveal} onClose={() => (voxReveal = null)} />
		{/if}
		{#if toast}
			<div class="toast"><span class="material-symbols-rounded">{toastIcon}</span>{toast}</div>
		{/if}

		{#if isTouch && !busy && controls === 'swipe'}
			<!-- swipe anywhere: the figure keeps walking that way (around obstacles), a tap stops it -->
			<div class="swipe-zone" role="presentation" onpointerdown={swipeDown} onpointermove={swipeMove} onpointerup={swipeUp} onpointercancel={swipeUp}></div>
			{#if swipeMark}
				{#key swipeMark.k}
					<div class="swipe-mark" style="left:{swipeMark.x}px; top:{swipeMark.y}px; rotate:{Math.atan2(swipeMark.dy, swipeMark.dx)}rad"><span class="material-symbols-rounded">arrow_forward</span></div>
				{/key}
			{/if}
		{:else if isTouch && !busy}
			<div
				class="joy-zone"
				role="presentation"
				onpointerdown={joyDown}
				onpointermove={joyMove}
				onpointerup={joyUp}
				onpointercancel={joyUp}
			>
				<div class="joy-base" style={joy ? `left:${joy.cx}px;top:${joy.cy}px;position:fixed` : ''}>
					<div class="joy-knob" style="transform: translate({joy?.dx ?? 0}px, {joy?.dy ?? 0}px)"></div>
				</div>
			</div>
		{/if}
		{#if isTouch && !busy}
			<div class="zoom-btns">
				<button onclick={() => zoomBy(-0.15)} aria-label="Zoom in"><span class="material-symbols-rounded">add</span></button>
				<button onclick={() => zoomBy(0.15)} aria-label="Zoom out"><span class="material-symbols-rounded">remove</span></button>
			</div>
		{/if}
		{#key photoFlash}
			{#if photoFlash > 0}<div class="photo-flash"></div>{/if}
		{/key}
		{#if polaroid}
			{#key polaroid}
				<div class="polaroid"><img src={polaroid} alt="Your photo" /><span>{areaLabel} · saved to your PC</span></div>
			{/key}
		{/if}
		{#key fade}
			{#if fade > 0}<div class="area-fade"></div>{/if}
		{/key}
	{:else}
		<!-- title screen -->
		<div class="absolute inset-0 z-30 grid place-items-center bg-gradient-to-b from-sky-300/40 to-black/40">
			<div class="title-card">
				{#if ready}
					<div class="flex justify-center gap-2 mb-3">
					{#each ['cinderpup', 'drizzlet', 'sproutle', 'lumifox'] as id, i}
						<div style="transform: rotate({(i - 1.5) * 6}deg) translateY({Math.abs(i - 1.5) * 6}px)">
							<Card {id} size="sm" />
						</div>
					{/each}
				</div>
				{/if}
				<h1 class="logo">BRICKHOLLOW</h1>
				<div class="sub">S N A P &nbsp; B E A S T S</div>
				<p class="text-sm opacity-80 mt-2 mb-5">A brick-built village • a card battler with 3 zones</p>
				<button class="start" onclick={start} disabled={!ready}>
					<span class="material-symbols-rounded">play_arrow</span>
					{game.save.hasDeck ? 'Continue' : 'Start'}
				</button>
				<div class="text-xs opacity-70 mt-4">WASD / Arrows: walk • Shift: run • E / Space: talk • P: photo • Q: dance • Mouse wheel: zoom</div>
			</div>
		</div>
	{/if}

	{#if dialog}
		{#key dialogKey}
			<Dialog data={dialog} onClose={() => { stopVoice(); dialog = null; }} />
		{/key}
	{/if}

	{#if pcRect && reading === 'pc:bedroom'}
		<PcDesktop rect={pcRect} onClose={() => town?.stopReading()} />
	{:else if pcRect && reading?.startsWith('tv:')}
		<TvShow rect={pcRect} {tod} rainy={(town?.rain.target ?? 0) > 0} onClose={() => town?.stopReading()} />
	{:else if pcRect && reading?.startsWith('piano:')}
		<PianoKeys rect={pcRect} onClose={() => town?.stopReading()} onSong={playedSong} />
	{:else if pcRect && reading?.startsWith('pc:lab')}
		<LabTerminal rect={pcRect} onClose={() => town?.stopReading()} />
	{/if}

	{#if rewards}
		<div class="fixed inset-0 z-40 bg-black/60 grid place-items-center p-4">
			<div class="reward">
				<h2 class="text-2xl font-bold mb-4 flex items-center gap-2 justify-center"><FluentIcon name="backpack" size={26} /> {rewards.title}</h2>
				<div class="flex flex-wrap gap-3 justify-center max-h-[60vh] p-2 pack">
					{#each rewards.ids as id, i}
						{@const big = packCardW >= 150}
						{@const rare = CARD_MAP[id].cost >= 5 || !!CARD_MAP[id].special}
						<button
							class="deal pack-card"
							class:open={revealed.has(i)}
							class:tearing={tearing.has(i)}
							style="animation-delay:{i * 60}ms; --pw:{packCardW}px"
							onclick={() => reveal(i)}
							onmouseenter={() => !revealed.has(i) && !tearing.has(i) && sfx.foilTouch()}
							aria-label="Tear open the pack"
						>
							<div class="card-in" class:rare>
								<Card {id} size={big ? 'lg' : 'md'} width="{packCardW}px" />
							</div>
							{#if !revealed.has(i) || tearing.has(i)}
								<!-- shiny foil wrapper (like a trading card booster), torn open on click -->
								<div class="foil" class:gold={rare}>
									{#each ['left', 'right'] as half}
										<div class="foil-half {half}">
											<div class="foil-art">
												<div class="crimp top"></div>
												<div class="foil-logo">
													<span class="material-symbols-rounded">{rare ? 'auto_awesome' : 'style'}</span>
													<b>BRICK<br />BEASTS</b>
													<small>{rare ? 'RARE · 1 CARD' : '1 CARD'}</small>
												</div>
												<div class="crimp bottom"></div>
												<div class="holo"></div>
											</div>
										</div>
									{/each}
									<div class="foil-strip"><div class="crimp top"></div><div class="holo"></div></div>
								</div>
							{/if}
						</button>
					{/each}
				</div>
				<div class="text-sm opacity-80 mt-3">{revealed.size < rewards.ids.length ? 'Tap a pack to tear it open!' : ''}</div>
				{#if revealed.size < rewards.ids.length}
					<button class="start mt-3" onclick={revealAll}>Reveal all</button>
				{:else}
					<button
						class="start mt-3"
						onclick={() => {
							const then = rewards?.then;
							rewards = null;
							revealed = new Set();
							then?.();
						}}>Awesome!</button
					>
				{/if}
			</div>
		</div>
	{/if}

	{#if battle}
		<BattleView {battle} onDone={battleDone} />
	{/if}

	{#if deckOpen}
		<DeckBuilder debug={debugOn} onClose={() => (deckOpen = false)} onVoxelmon={(id) => { deckOpen = false; voxReveal = id; }} />
	{/if}

	{#if helpOpen}
		<div class="fixed inset-0 z-40 bg-black/60 grid place-items-center p-4">
			<div class="reward max-w-xl text-left help-box">
				<h2 class="text-2xl font-bold mb-3 flex items-center gap-2"><FluentIcon name="help" size={26} /> How to play</h2>
				<ul class="space-y-2 text-[15px] list-disc pl-5">
					<li><b>Walk</b> with WASD / arrow keys, <b>run</b> with Shift, <b>talk</b> with E or Space.</li>
					<li>Get your deck from <b>Prof. Quill</b> in the lab and battle Dex and Lumi with your cards.</li>
					<li><b>Battles:</b> 6 turns, energy = turn number. Play cards into 3 zones; each zone reveals on turns 1–3 with its own effect.</li>
					<li>Both sides plan in secret. The player winning more zones reveals first.</li>
					<li><b>Win 2 of 3 zones</b> (ties broken by total power). <b>SNAP</b> doubles the stakes, the final turn doubles them again.</li>
					<li>Win studs and new cards, then tune your 12-card deck in the <b>Deck</b> menu.</li>
					<li>Every card battle gives <b>EXP</b>. In the Deck menu, tap a card to upgrade it: <b>Bronze</b>, <b>Silver</b> (glitter), <b>Gold</b> (holo).</li>
					<li>A <b>6-energy card</b> has one more stage: it becomes a real 3D <b>Voxelmon</b> that walks with you. Only then does Prof. Quill let you onto <b>Route 1</b>.</li>
					<li>In the tall grass, wild Voxelmon jump out. Your Voxelmon fights on its own; press <b>1</b> and <b>2</b> for its attacks, which then recharge. Win to get EXP and the wild beast's card.</li>
				</ul>
				<h3 class="text-lg font-bold mt-4 mb-2">Things to do in the village</h3>
				<div class="fun">
					{#each FUN as f}
						<div><span class="material-symbols-rounded">{f[0]}</span><span>{@html f[1]}</span></div>
					{/each}
				</div>
				<p class="mt-3 text-sm opacity-80">
					<span class="material-symbols-rounded align-middle" style="font-size:1.1em">auto_awesome</span>
					Stickers: <b>{earnedStickers(game.save).length}/{STICKERS.length}</b> · see the sticker book on the PC in your room.
				</p>
				<div class="flex gap-2 mt-5 justify-between">
					<button class="ibtn" onclick={resetSave}><FluentIcon name="reset" /> Reset save</button>
					<button class="ibtn" onclick={() => { toggleDebug(); helpOpen = false; }}><span class="material-symbols-rounded">bug_report</span> Debug {debugOn ? 'off' : 'on'}</button>
					<button class="start" onclick={() => (helpOpen = false)}>Got it</button>
				</div>
			</div>
		</div>
	{/if}

	{#if flash}
		<div class="wipe"></div>
	{/if}
</div>

<style>
	/* ---------- Voxelmon battle HUD ---------- */
	.vx-wild,
	.vx-me,
	.vx-rest {
		position: fixed;
		z-index: 30;
		left: 50%;
		translate: -50% 0;
		font-family: 'Fredoka', system-ui, sans-serif;
		color: #fff;
	}
	.vx-wild {
		top: 7.4rem;
		width: min(22rem, 80vw);
		padding: 0.4rem 0.8rem 0.6rem;
		border-radius: 0.9rem;
		background: rgb(27 42 52 / 0.85);
		box-shadow: 0 0.2rem 0 rgb(0 0 0 / 0.3);
		animation: vxin 0.35s ease-out;
	}
	.vx-wild.shiny {
		background: linear-gradient(135deg, rgb(120 90 0 / 0.92), rgb(27 42 52 / 0.9));
		box-shadow: 0 0 1rem rgb(255 210 61 / 0.6);
	}
	.vx-me {
		bottom: 1.2rem;
		width: min(32rem, 94vw);
		padding: 0.5rem 0.8rem 0.7rem;
		border-radius: 1rem;
		background: rgb(27 42 52 / 0.88);
		box-shadow: 0 0.25rem 0 rgb(0 0 0 / 0.3);
		animation: vxup 0.35s ease-out;
	}
	@keyframes vxin {
		from {
			opacity: 0;
			translate: -50% -1rem;
		}
	}
	@keyframes vxup {
		from {
			opacity: 0;
			translate: -50% 1rem;
		}
	}
	.vx-head {
		display: flex;
		align-items: center;
		gap: 0.35rem;
	}
	.vx-head b {
		flex: 1;
	}
	.vx-head small {
		font-weight: 700;
		opacity: 0.85;
	}
	.hpbar {
		height: 0.6rem;
		margin-top: 0.25rem;
		border-radius: 999px;
		background: rgb(255 255 255 / 0.15);
		overflow: hidden;
	}
	.hpbar i {
		display: block;
		height: 100%;
		border-radius: inherit;
		background: linear-gradient(90deg, #ff5a4a, #ff8a3d);
		transition: width 0.25s;
	}
	.hpbar.mine i {
		background: linear-gradient(90deg, #4fc06a, #a6e05a);
	}
	.vx-skills {
		display: flex;
		gap: 0.5rem;
		margin-top: 0.55rem;
	}
	.vx-skill {
		position: relative;
		flex: 1;
		display: flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.5rem 0.7rem;
		border-radius: 0.8rem;
		overflow: hidden;
		background: color-mix(in srgb, var(--c) 35%, #1b2a34);
		border: 0.14rem solid var(--c);
		color: #fff;
		font-weight: 700;
		text-align: left;
		box-shadow: 0 0.2rem 0 rgb(0 0 0 / 0.35);
		transition: transform 0.08s;
	}
	.vx-skill:not(:disabled):hover {
		transform: translateY(-0.1rem);
	}
	.vx-skill:not(:disabled):active {
		transform: translateY(0.1rem);
	}
	/* recharge: a dark curtain that shrinks from the top */
	.vx-skill:disabled::after {
		content: '';
		position: absolute;
		inset: 0;
		background: rgb(0 0 0 / 0.55);
		clip-path: inset(calc(100% - var(--p)) 0 0 0);
	}
	.vx-skill b {
		flex: 1;
	}
	.vx-skill kbd {
		padding: 0 0.35rem;
		border-radius: 0.3rem;
		background: rgb(255 255 255 / 0.2);
		font-size: 0.8rem;
	}
	.vx-skill .cd {
		position: absolute;
		z-index: 1;
		right: 0.5rem;
		bottom: 0.1rem;
		font-size: 0.75rem;
	}
	.vx-run {
		display: flex;
		align-items: center;
		gap: 0.2rem;
		padding: 0.5rem 0.7rem;
		border-radius: 0.8rem;
		background: rgb(255 255 255 / 0.12);
		color: #fff;
		font-weight: 700;
	}
	.vx-rest {
		bottom: 1.2rem;
		display: flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.3rem 0.8rem;
		border-radius: 999px;
		background: rgb(27 42 52 / 0.8);
		font-size: 0.85rem;
	}
	.vx-rest .hpbar {
		width: 6rem;
		margin: 0;
	}
	.reward.help-box {
		max-height: 92vh;
		overflow-y: auto;
		text-align: left;
	}
	.fun {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(13rem, 1fr));
		gap: 0.35rem 0.8rem;
		font-size: 14px;
	}
	.fun > div {
		display: flex;
		align-items: center;
		gap: 0.45rem;
	}
	.fun .material-symbols-rounded {
		flex: none;
		display: grid;
		place-items: center;
		width: 1.8rem;
		height: 1.8rem;
		border-radius: 0.5rem;
		background: #ffe9a8;
		color: #8a5a00;
		font-size: 1.15rem;
	}
	.photo-flash {
		position: fixed;
		inset: 0;
		z-index: 70;
		background: #fff;
		pointer-events: none;
		animation: photoflash 0.45s ease-out forwards;
	}
	@keyframes photoflash {
		from {
			opacity: 0.95;
		}
		to {
			opacity: 0;
		}
	}
	/* the snapshot pops out like an instant photo, then slides away */
	.polaroid {
		position: fixed;
		right: 1rem;
		bottom: 5.5rem;
		z-index: 40;
		width: min(15rem, 42vw);
		padding: 0.45rem 0.45rem 0.3rem;
		background: #fffdf6;
		border-radius: 0.2rem;
		box-shadow: 0 0.4rem 1rem rgb(0 0 0 / 0.35);
		rotate: 4deg;
		pointer-events: none;
		animation: polaroid 2.6s ease-in-out forwards;
		font-family: 'Fredoka', system-ui, sans-serif;
	}
	.polaroid img {
		display: block;
		width: 100%;
		border-radius: 0.1rem;
	}
	.polaroid span {
		display: block;
		margin-top: 0.25rem;
		text-align: center;
		font-size: 0.75rem;
		color: #5a4a3a;
	}
	@keyframes polaroid {
		0% {
			opacity: 0;
			translate: 0 1.5rem;
			scale: 0.7;
		}
		12% {
			opacity: 1;
			translate: 0 0;
			scale: 1;
		}
		82% {
			opacity: 1;
			translate: 0 0;
		}
		100% {
			opacity: 0;
			translate: 120% 0;
		}
	}
	.seek-eyes {
		position: fixed;
		inset: 0;
		z-index: 60;
		display: grid;
		place-content: center;
		justify-items: center;
		gap: 0.3rem;
		color: #fff;
		background: rgb(10 14 20 / 0.94);
		font-family: 'Fredoka', system-ui, sans-serif;
		animation: seekfade 0.3s ease-out;
	}
	.seek-eyes .material-symbols-rounded {
		font-size: 3rem;
		opacity: 0.8;
	}
	.seek-eyes b {
		font-size: 4rem;
		line-height: 1;
	}
	.seek-eyes small {
		font-size: 1.1rem;
		opacity: 0.8;
	}
	@keyframes seekfade {
		from {
			opacity: 0;
		}
	}
	.seek-timer {
		position: fixed;
		top: 7.2rem; /* below the toasts */
		left: 50%;
		translate: -50% 0;
		z-index: 30;
		display: flex;
		align-items: center;
		gap: 0.3rem;
		padding: 0.25rem 0.8rem;
		border-radius: 999px;
		background: #1e8f8f;
		color: #fff;
		font-weight: 700;
		font-family: 'Fredoka', system-ui, sans-serif;
		box-shadow: 0 0.15rem 0 rgb(0 0 0 / 0.3);
	}
	.pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 12px;
		background: #fff;
		color: #1b2a34;
		border: 3px solid #1b2a34;
		box-shadow: inset 0 -3px 0 rgb(0 0 0 / 0.1);
		font-weight: 600;
	}
	.ibtn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		border-radius: 12px;
		background: #f2cd37;
		color: #1b2a34;
		border: 3px solid #1b2a34;
		box-shadow: inset 0 -3px 0 rgb(0 0 0 / 0.18);
		font-weight: 700;
		cursor: pointer;
	}
	.ibtn:hover {
		filter: brightness(1.07);
	}
	.prompt {
		position: absolute;
		left: 50%;
		bottom: 28px;
		transform: translateX(-50%);
		z-index: 20;
		background: #1b2a34;
		color: #fff;
		padding: 8px 18px;
		border-radius: 999px;
		font-weight: 600;
		box-shadow: 0 6px 16px rgb(0 0 0 / 0.3);
		animation: bob 0.9s infinite alternate;
	}
	kbd {
		background: #f2cd37;
		color: #1b2a34;
		border-radius: 6px;
		padding: 0 7px;
		margin-right: 6px;
		font-weight: 800;
	}
	@keyframes bob {
		to {
			transform: translate(-50%, -4px);
		}
	}
	.toast {
		position: absolute;
		top: 70px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 25;
		display: flex;
		gap: 8px;
		align-items: center;
		background: #fff;
		color: #1b2a34;
		border: 3px solid #1b2a34;
		padding: 8px 16px;
		border-radius: 12px;
		font-weight: 600;
	}
	.title-card {
		text-align: center;
		color: #fff;
		background: rgb(27 42 52 / 0.82);
		border: 4px solid rgb(255 255 255 / 0.25);
		border-radius: 24px;
		padding: 28px 44px;
		box-shadow: 0 20px 60px rgb(0 0 0 / 0.4);
	}
	.logo {
		font-size: clamp(2.4rem, 7vw, 4.2rem);
		font-weight: 800;
		letter-spacing: 0.04em;
		line-height: 1;
		color: #f2cd37;
		text-shadow:
			0 4px 0 #c91a09,
			0 8px 0 #1b2a34;
	}
	.sub {
		margin-top: 14px;
		font-weight: 700;
		letter-spacing: 0.1em;
		color: #fff;
	}
	.start {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 12px 34px;
		font-size: 1.25rem;
		font-weight: 800;
		color: #fff;
		border-radius: 14px;
		background: linear-gradient(#e83a20, #c91a09);
		border: 3px solid #1b2a34;
		box-shadow: inset 0 -5px 0 rgb(0 0 0 / 0.25);
		cursor: pointer;
	}
	.start:hover {
		filter: brightness(1.1);
	}
	.reward {
		background: linear-gradient(#2a3a5a, #1b2a34);
		color: #fff;
		border: 4px solid rgb(255 255 255 / 0.3);
		border-radius: 20px;
		padding: 24px 28px;
		text-align: center;
		max-width: 1000px;
	}
	.deal {
		animation: deal 0.4s ease-out both;
		display: block;
		line-height: 0;
	}
	/* ---------- foil pack opening ---------- */
	.pack-card {
		position: relative;
		cursor: pointer;
		font-size: calc(var(--pw) / 10);
	}
	.card-in {
		position: relative;
	}
	.pack-card.open .card-in {
		animation: pop 0.55s cubic-bezier(0.3, 1.6, 0.5, 1);
	}
	.pack-card.open .card-in::after {
		content: '';
		position: absolute;
		inset: -15%;
		pointer-events: none;
		background: radial-gradient(circle, rgb(255 255 255 / 0.95), rgb(255 230 120 / 0.55) 40%, transparent 70%);
		animation: sparkle 0.9s ease-out forwards;
	}
	.pack-card.open .card-in.rare {
		filter: drop-shadow(0 0 14px #ffd23d);
	}
	@keyframes pop {
		0% {
			transform: scale(0.85);
		}
		60% {
			transform: scale(1.1) rotate(-2deg);
		}
	}
	.foil {
		position: absolute;
		inset: -3% -1.5%;
		z-index: 2;
		pointer-events: none;
		filter: drop-shadow(0 6px 10px rgb(0 0 0 / 0.45));
	}
	/* hover: no shaking, the foil just catches a bit more light */
	.foil {
		transition: filter 0.2s;
	}
	.pack-card:not(.tearing):hover .foil {
		filter: drop-shadow(0 6px 10px rgb(0 0 0 / 0.45)) brightness(1.12);
	}
	.foil-half,
	.foil-strip {
		position: absolute;
		inset: 0;
		overflow: hidden;
		border-radius: 0.35em;
	}
	/* the two halves meet along a jagged tear line */
	.foil-half.left {
		clip-path: polygon(0 0, 52% 0, 47% 18%, 54% 34%, 46% 50%, 53% 66%, 47% 82%, 52% 100%, 0 100%);
	}
	.foil-half.right {
		clip-path: polygon(52% 0, 100% 0, 100% 100%, 52% 100%, 47% 82%, 53% 66%, 46% 50%, 54% 34%, 47% 18%);
	}
	/* the strip that rips off first (top 13 %) */
	.foil-strip {
		clip-path: polygon(0 0, 100% 0, 100% 11%, 92% 13%, 84% 11%, 76% 13.5%, 68% 11%, 60% 13%, 52% 11%, 44% 13.5%, 36% 11%, 28% 13%, 20% 11%, 12% 13.5%, 4% 11%, 0 12%);
		background: linear-gradient(115deg, #b9c0cc, #f6f8fb 30%, #98a2b3 50%, #eef1f6 70%, #a9b1c0);
	}
	.foil-art {
		position: absolute;
		inset: 0;
		background:
			radial-gradient(circle at 30% 20%, rgb(255 255 255 / 0.9), transparent 25%),
			linear-gradient(115deg, #b9c0cc, #f6f8fb 30%, #98a2b3 50%, #eef1f6 70%, #a9b1c0);
		display: grid;
		place-items: center;
	}
	.foil.gold .foil-art,
	.foil.gold .foil-strip {
		background:
			radial-gradient(circle at 30% 20%, rgb(255 255 255 / 0.9), transparent 25%),
			linear-gradient(115deg, #c99a2e, #fff2b8 30%, #b8862a 50%, #ffe68a 70%, #b98a2e);
	}
	/* rainbow holo sheen sweeping across the foil */
	.holo {
		position: absolute;
		inset: 0;
		background: linear-gradient(115deg, transparent 20%, rgb(255 80 160 / 0.35) 35%, rgb(80 200 255 / 0.4) 45%, rgb(120 255 140 / 0.35) 55%, rgb(255 230 80 / 0.35) 65%, transparent 80%);
		background-size: 250% 100%;
		mix-blend-mode: overlay;
		animation: holo 2.4s linear infinite;
	}
	@keyframes holo {
		from {
			background-position: 150% 0;
		}
		to {
			background-position: -100% 0;
		}
	}
	.crimp {
		position: absolute;
		left: 0;
		right: 0;
		height: 9%;
		background: repeating-linear-gradient(90deg, rgb(0 0 0 / 0.18) 0 2px, rgb(255 255 255 / 0.35) 2px 4px);
	}
	.crimp.top {
		top: 0;
	}
	.crimp.bottom {
		bottom: 0;
	}
	.foil-logo {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.15em;
		color: #c91a09;
		text-shadow: 0 1px 0 #fff;
		line-height: 1;
		transform: rotate(-8deg);
	}
	.foil-logo b {
		font-size: 1.15em;
		font-weight: 900;
		letter-spacing: 0.04em;
		text-align: center;
	}
	.foil-logo small {
		font-size: 0.55em;
		font-weight: 700;
		color: #1b2a34;
	}
	.foil-logo .material-symbols-rounded {
		font-size: 1.8em;
		color: #ffd23d;
		filter: drop-shadow(0 1px 0 #a87a00);
	}
	.foil.gold .foil-logo {
		color: #8a4b00;
	}
	/* tearing: strip rips off, then the halves fly apart */
	.tearing .foil-strip {
		animation: rip 0.35s ease-in forwards;
	}
	.tearing .foil-half.left {
		animation: split-left 0.55s 0.25s ease-in forwards;
	}
	.tearing .foil-half.right {
		animation: split-right 0.55s 0.25s ease-in forwards;
	}
	@keyframes rip {
		to {
			transform: translate(35%, -120%) rotate(-24deg);
			opacity: 0;
		}
	}
	@keyframes split-left {
		to {
			transform: translateX(-75%) rotate(-16deg);
			opacity: 0;
		}
	}
	@keyframes split-right {
		to {
			transform: translateX(75%) rotate(16deg);
			opacity: 0;
		}
	}
	@keyframes sparkle {
		from {
			opacity: 1;
			transform: scale(0.6);
		}
		to {
			opacity: 0;
			transform: scale(1.4);
		}
	}
	@keyframes deal {
		from {
			opacity: 0;
			transform: translateY(30px) rotate(-8deg) scale(0.8);
		}
	}
	.wipe {
		position: fixed;
		inset: 0;
		z-index: 60;
		pointer-events: none;
		background: radial-gradient(circle, #c91a09 0 30%, transparent 32%) 0 0 / 48px 48px, #1b2a34;
		animation: wipe 0.75s ease-in-out;
	}
	@keyframes wipe {
		0% {
			clip-path: circle(0% at 50% 50%);
		}
		45%,
		60% {
			clip-path: circle(80% at 50% 50%);
		}
		100% {
			clip-path: circle(80% at 50% 50%);
			opacity: 0;
		}
	}
	/* ----- dev / debug panel ----- */
	.dbg {
		position: absolute;
		right: 10px;
		bottom: 10px;
		z-index: 25;
		width: 270px;
		max-width: calc(100vw - 20px);
		background: rgb(20 24 32 / 0.88);
		color: #fff;
		border: 2px solid #ff00aa;
		border-radius: 12px;
		font-size: 0.8rem;
		box-shadow: 0 6px 18px rgb(0 0 0 / 0.35);
		overflow: hidden;
	}
	.dbg-head {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 6px 10px;
		background: #ff00aa;
		font-weight: 700;
	}
	.dbg-head .material-symbols-rounded {
		font-size: 18px;
	}
	.dbg-fps {
		margin-left: auto;
		font-variant-numeric: tabular-nums;
	}
	.dbg label {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 10px 0;
		cursor: pointer;
	}
	.dbg-stats {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 10px;
		padding: 6px 10px;
		opacity: 0.85;
		font-variant-numeric: tabular-nums;
	}
	.dbg-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		margin: 0 10px 6px;
		padding: 3px 8px;
		border-radius: 6px;
		background: rgb(255 255 255 / 0.12);
	}
	.dbg-btn .material-symbols-rounded {
		font-size: 16px;
	}
	.dbg-report {
		padding: 0 10px 6px;
		font-variant-numeric: tabular-nums;
	}
	.dbg-total {
		font-weight: 700;
		margin-bottom: 2px;
	}
	.dbg-row {
		display: grid;
		grid-template-columns: 1fr auto 3.2em;
		gap: 6px;
		opacity: 0.9;
	}
	.dbg-row .name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.dbg-row b {
		text-align: right;
	}
	.dbg-hint {
		padding: 0 10px 6px;
		opacity: 0.55;
		font-size: 0.7rem;
	}
	.quest {
		position: absolute;
		left: 10px;
		top: calc(56px + env(safe-area-inset-top));
		z-index: 20;
		display: flex;
		align-items: center;
		gap: 6px;
		max-width: min(70vw, 420px);
		padding: 5px 12px 5px 8px;
		border-radius: 12px;
		background: rgb(27 42 52 / 0.82);
		color: #fff;
		font-size: 0.85rem;
		font-weight: 600;
		box-shadow: 0 4px 12px rgb(0 0 0 / 0.25);
	}
	.quest .material-symbols-rounded {
		color: #f2cd37;
	}
	@media (max-width: 640px) {
		.quest {
			top: calc(84px + env(safe-area-inset-top));
			font-size: 0.75rem;
		}
	}
	.hud-left {
		position: absolute;
		top: calc(10px + env(safe-area-inset-top));
		left: 10px;
		z-index: 20;
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		max-width: 62vw;
	}
	.hud-right {
		position: absolute;
		top: calc(10px + env(safe-area-inset-top));
		right: 10px;
		z-index: 20;
		display: flex;
		gap: 6px;
	}
	@media (max-width: 640px) {
		.pill {
			padding: 4px 8px;
			font-size: 0.85rem;
			border-width: 2px;
		}
		.ibtn {
			padding: 6px 8px;
			border-width: 2px;
		}
		.prompt {
			bottom: 40px;
			left: auto;
			right: 16px;
			transform: none;
			animation: none;
			padding: 12px 16px;
		}
	}
	.area-fade {
		position: fixed;
		inset: 0;
		z-index: 26;
		background: #0b1218;
		pointer-events: none;
		animation: areafade 0.45s ease-out forwards;
	}
	@keyframes areafade {
		from {
			opacity: 1;
		}
		to {
			opacity: 0;
		}
	}
	/* ---------- menu (top right) ---------- */
	.ibtn.on {
		background: #ffd23d;
	}
	.menu-shade {
		position: fixed;
		inset: 0;
		z-index: 39;
		cursor: default;
	}
	.menu {
		position: absolute;
		top: calc(58px + env(safe-area-inset-top));
		right: 10px;
		z-index: 40;
		width: min(17rem, calc(100vw - 20px));
		padding: 8px;
		border-radius: 16px;
		background: #fff;
		border: 3px solid #1b2a34;
		box-shadow: 0 4px 0 #1b2a34;
		font-family: 'Fredoka', system-ui, sans-serif;
		color: #1b2a34;
		display: flex;
		flex-direction: column;
		gap: 2px;
		animation: menuin 0.15s ease-out;
	}
	@keyframes menuin {
		from {
			opacity: 0;
			translate: 0 -6px;
		}
	}
	.menu-stats {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 4px 10px;
		padding: 4px 6px 8px;
		margin-bottom: 4px;
		border-bottom: 2px solid #e6eaee;
		font-weight: 700;
	}
	.menu-stats > span {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.menu-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 8px;
		border-radius: 10px;
		font-weight: 600;
		text-align: left;
	}
	.menu-item:hover {
		background: #f0f4f7;
	}
	.menu-item .state,
	.menu-item kbd {
		margin-left: auto;
		font-size: 0.8rem;
		opacity: 0.7;
	}
	.menu-label {
		margin: 6px 8px 2px;
		font-size: 0.75rem;
		font-weight: 700;
		text-transform: uppercase;
		opacity: 0.6;
	}
	.seg {
		display: flex;
		gap: 4px;
		margin: 0 4px;
	}
	.seg button {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		padding: 8px 4px;
		border-radius: 10px;
		border: 2px solid #1b2a34;
		font-weight: 700;
	}
	.seg button.on {
		background: #ffd23d;
	}
	.menu-hint {
		margin: 4px 8px 6px;
		font-size: 0.8rem;
		opacity: 0.7;
	}
	/* ---------- swipe controls ---------- */
	.swipe-zone {
		position: absolute;
		inset: 0;
		z-index: 18;
		touch-action: none;
	}
	.swipe-mark {
		position: fixed;
		z-index: 19;
		translate: -50% -50%;
		display: grid;
		place-items: center;
		width: 56px;
		height: 56px;
		border-radius: 50%;
		background: rgb(255 255 255 / 0.35);
		color: #1b2a34;
		pointer-events: none;
		animation: swipemark 0.7s ease-out forwards;
	}
	.swipe-mark .material-symbols-rounded {
		font-size: 34px;
	}
	@keyframes swipemark {
		to {
			opacity: 0;
			scale: 1.3;
		}
	}
	.joy-zone {
		position: absolute;
		left: 0;
		bottom: 0;
		width: 55vw;
		height: 45dvh;
		z-index: 19;
		touch-action: none;
	}
	.joy-base {
		position: absolute;
		left: 80px;
		bottom: 60px;
		width: 110px;
		height: 110px;
		margin: -55px 0 0 -55px;
		border-radius: 50%;
		background: rgb(255 255 255 / 0.35);
		border: 3px solid rgb(27 42 52 / 0.6);
		display: grid;
		place-items: center;
		pointer-events: none;
	}
	.joy-knob {
		width: 50px;
		height: 50px;
		border-radius: 50%;
		background: #f2cd37;
		border: 3px solid #1b2a34;
		box-shadow: inset 0 -4px 0 rgb(0 0 0 / 0.2);
	}
	.zoom-btns {
		position: absolute;
		right: 12px;
		top: 70px;
		z-index: 20;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.zoom-btns button {
		width: 42px;
		height: 42px;
		border-radius: 12px;
		background: rgb(255 255 255 / 0.9);
		border: 2px solid #1b2a34;
		display: grid;
		place-items: center;
	}
</style>
