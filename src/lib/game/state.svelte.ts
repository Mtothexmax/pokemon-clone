import { CARD_MAP, DECK_SIZE, POOL, STARTER_COLLECTION, STARTER_DECK } from './cards';
import { STAGES, VOXEL_LEVEL, maxLevel } from './levels';

const KEY = 'brickhollow-save-v1';

export interface DeckDef {
	name: string;
	cards: string[];
}

interface Save {
	hasDeck: boolean;
	collection: string[];
	/** the deck used in battles: the last complete version of the active deck */
	deck: string[];
	/** all decks the player built (like Marvel Snap) */
	decks: DeckDef[];
	active: number;
	studs: number;
	wins: number;
	losses: number;
	rivalWins: number;
	metRival: boolean;
	golden: string[];
	/** butterflies caught in the village */
	butterflies: number;
	/** goals scored with the ball in the park */
	goals?: number;
	/** how often the ducks got bread */
	duckFeeds?: number;
	tagWins?: number;
	danceParties?: number;
	pianoSongs?: number;
	chests?: number;
	photos?: number;
	hideSeekWins?: number;
	fireflies?: number;
	rainbows?: number;
	benchSits?: number;
	puddleSplashes?: number;
	/** EXP from battles, spent on card upgrades (see levels.ts) */
	exp?: number;
	/** upgrade stage per card id (0 = basic … 4 = Voxelmon) */
	cardLevels?: Record<string, number>;
	/** the Voxelmon walking with you (a card id at the Voxelmon stage) */
	voxelmon?: string;
	/** Prof. Quill let you pass to Route 1 */
	gateOpen?: boolean;
	/** wild Voxelmon beaten in the tall grass */
	voxWins?: number;
	/** stickers already announced (see stickers.ts) */
	stickers?: string[];
}

function fresh(): Save {
	return {
		hasDeck: false,
		collection: [],
		deck: [],
		decks: [],
		active: 0,
		studs: 0,
		wins: 0,
		losses: 0,
		rivalWins: 0,
		metRival: false,
		golden: [],
		butterflies: 0
	};
}

function load(): Save {
	try {
		const raw = localStorage.getItem(KEY);
		if (raw) {
			const s: Save = { ...fresh(), ...JSON.parse(raw) };
			// older saves had a single deck
			if (!s.decks.length && s.deck.length) s.decks = [{ name: 'Starter Deck', cards: [...s.deck] }];
			if (s.active >= s.decks.length) s.active = 0;
			return s;
		}
	} catch {
		/* storage unavailable */
	}
	return fresh();
}

class GameState {
	save = $state<Save>(fresh());
	/** true once the save was loaded from storage (never persist before that) */
	loaded = $state(false);

	init() {
		this.save = load();
		this.loaded = true;
	}
	persist() {
		try {
			localStorage.setItem(KEY, JSON.stringify($state.snapshot(this.save)));
		} catch {
			/* ignore */
		}
	}
	reset() {
		this.save = fresh();
		this.persist();
	}
	giveStarter() {
		this.save.hasDeck = true;
		this.save.collection = [...STARTER_COLLECTION];
		this.save.deck = [...STARTER_DECK];
		this.save.decks = [{ name: 'Starter Deck', cards: [...STARTER_DECK] }];
		this.save.active = 0;
		this.persist();
	}
	setDeck(ids: string[]) {
		if (ids.length !== DECK_SIZE) return;
		this.save.deck = [...ids];
		this.persist();
	}
	/** saves the cards of deck i (may be incomplete); the active one also becomes the battle deck once it is full */
	saveDeck(i: number, cards: string[]) {
		const d = this.save.decks[i];
		if (!d) return;
		d.cards = [...cards];
		if (i === this.save.active && cards.length === DECK_SIZE) this.save.deck = [...cards];
		this.persist();
	}
	renameDeck(i: number, name: string) {
		const d = this.save.decks[i];
		if (!d) return;
		d.name = name.trim().slice(0, 24) || d.name;
		this.persist();
	}
	newDeck(): number {
		this.save.decks.push({ name: `Deck ${this.save.decks.length + 1}`, cards: [] });
		this.persist();
		return this.save.decks.length - 1;
	}
	/** makes a complete deck the one used in battles */
	useDeck(i: number) {
		const d = this.save.decks[i];
		if (!d || d.cards.length !== DECK_SIZE) return;
		this.save.active = i;
		this.save.deck = [...d.cards];
		this.persist();
	}
	deleteDeck(i: number) {
		if (this.save.decks.length <= 1) return;
		this.save.decks.splice(i, 1);
		if (this.save.active === i) {
			const full = this.save.decks.findIndex((d) => d.cards.length === DECK_SIZE);
			this.save.active = Math.max(0, full);
			if (full >= 0) this.save.deck = [...this.save.decks[full].cards];
		} else if (this.save.active > i) this.save.active--;
		this.persist();
	}
	level(id: string) {
		return this.save.cardLevels?.[id] ?? 0;
	}
	addExp(n: number) {
		this.save.exp = (this.save.exp ?? 0) + n;
		this.persist();
	}
	/** EXP needed for the next stage of a card (null: already at its last stage) */
	upgradeCost(id: string) {
		const lv = this.level(id);
		return lv >= maxLevel(id) ? null : STAGES[lv + 1].cost;
	}
	/** spends EXP on the next stage; returns the new stage or null */
	upgrade(id: string) {
		const cost = this.upgradeCost(id);
		if (cost === null || (this.save.exp ?? 0) < cost) return null;
		this.save.exp = (this.save.exp ?? 0) - cost;
		const lv = this.level(id) + 1;
		this.save.cardLevels = { ...(this.save.cardLevels ?? {}), [id]: lv };
		// the first Voxelmon walks with you right away
		if (lv === VOXEL_LEVEL && !this.save.voxelmon) this.save.voxelmon = id;
		this.persist();
		return lv;
	}
	/** cards that became real Voxelmon */
	voxelmons() {
		return Object.entries(this.save.cardLevels ?? {})
			.filter(([, lv]) => lv >= VOXEL_LEVEL)
			.map(([id]) => id);
	}
	/** Adds a specific card (e.g. a special reward) if not owned yet. */
	giveCard(id: string) {
		if (!this.save.collection.includes(id)) this.save.collection.push(id);
		this.persist();
	}
	/** A random unowned card, preferring rare ones (cost 5+) — the shiny reward. */
	rewardRareCard(): string | null {
		const missing = POOL.filter((id) => !this.save.collection.includes(id));
		if (!missing.length) return null;
		const rare = missing.filter((id) => CARD_MAP[id].cost >= 5);
		const from = rare.length ? rare : missing;
		const id = from[Math.floor(Math.random() * from.length)];
		this.save.collection.push(id);
		this.persist();
		return id;
	}
	/** Gives a random card the player doesn't own yet. */
	rewardCard(): string | null {
		const missing = POOL.filter((id) => !this.save.collection.includes(id));
		if (!missing.length) return null;
		const id = missing[Math.floor(Math.random() * missing.length)];
		this.save.collection.push(id);
		this.persist();
		return id;
	}
}

export const game = new GameState();
