// Original "Brick Beast" creatures, built as cards.
// Cards are grouped by ability kind (like a Snap-style card game), not by element.
import type { Battle, CardInst } from './battle.svelte';

export type AbilityKind = 'none' | 'reveal' | 'ongoing' | 'activate' | 'endturn' | 'start';

export type Feature =
	| 'ears'
	| 'horns'
	| 'flame'
	| 'leaf'
	| 'fins'
	| 'wings'
	| 'antenna'
	| 'spikes'
	| 'ghost'
	| 'shell'
	| 'tentacles'
	| 'crest';

export interface ArtSpec {
	body: string;
	belly: string;
	accent: string;
	feature: Feature;
	eyes?: 'round' | 'angry' | 'sleepy' | 'happy';
	big?: boolean;
}

export interface CardDef {
	id: string;
	name: string;
	kind: AbilityKind;
	cost: number;
	power: number;
	text: string;
	art: ArtSpec;
	token?: boolean;
	/** special reward, not found in the wild */
	special?: boolean;
	/** On Reveal: triggered when revealed (can trigger twice). */
	onReveal?: (b: Battle, self: CardInst) => void;
	/** Ongoing: power bonus for this card itself. */
	selfBonus?: (b: Battle, self: CardInst) => number;
	/** Ongoing: power this card gives to another card on the board. */
	aura?: (b: Battle, self: CardInst, target: CardInst) => number;
	/** Ongoing: On Reveal abilities of your cards here happen twice. */
	doublesReveal?: boolean;
	/** Activate: once per game, after this card is revealed. */
	activate?: (b: Battle, self: CardInst) => void;
	/** End of Turn: triggers after every turn while revealed. */
	endTurn?: (b: Battle, self: CardInst) => void;
	/** Game Start: triggers once when the battle begins (the card is still in your deck). */
	gameStart?: (b: Battle, self: CardInst) => void;
}

export const KIND_INFO: Record<AbilityKind, { label: string; color: string; dark: string; icon: string }> = {
	none: { label: 'No ability', color: '#b8bcc0', dark: '#5c6166', icon: 'crop_square' },
	reveal: { label: 'On Reveal', color: '#ff8a3d', dark: '#b8400f', icon: 'visibility' },
	ongoing: { label: 'Ongoing', color: '#3d9bff', dark: '#1655b0', icon: 'all_inclusive' },
	activate: { label: 'Activate', color: '#4fc06a', dark: '#1e7a35', icon: 'touch_app' },
	endturn: { label: 'End of Turn', color: '#9a74d6', dark: '#4f2f8a', icon: 'hourglass_bottom' },
	start: { label: 'Game Start', color: '#ffc928', dark: '#a87a00', icon: 'flag' }
};

/** Readable text color (dark or white) for a background color, plus a matching text shadow. */
export function inkOn(hex: string) {
	const n = parseInt(hex.replace('#', '').slice(0, 6), 16);
	const lum = (0.299 * (n >> 16) + 0.587 * ((n >> 8) & 255) + 0.114 * (n & 255)) / 255;
	return lum > 0.62
		? { ink: '#1b2a34', shadow: '0 1px 0 rgb(255 255 255 / 0.5)' }
		: { ink: '#ffffff', shadow: '0 1px 2px rgb(0 0 0 / 0.6)' };
}

const friendsHere = (b: Battle, c: CardInst) => b.cardsAt(c.zone, c.owner).filter((x) => x.uid !== c.uid);
const enemiesHere = (b: Battle, c: CardInst) => b.cardsAt(c.zone, b.other(c.owner));

export const CARDS: CardDef[] = [
	// ---------- cost 1 ----------
	{
		id: 'pebbit', name: 'Pebbit', kind: 'none', cost: 1, power: 2,
		text: 'A sturdy little pebble bunny.',
		art: { body: '#a58c6a', belly: '#e8dcc4', accent: '#6b553a', feature: 'ears', eyes: 'happy' }
	},
	{
		id: 'flicker', name: 'Flicker', kind: 'reveal', cost: 1, power: 1,
		text: 'On Reveal: Your other cards here get +1 Power.',
		art: { body: '#ff8a3d', belly: '#ffe08a', accent: '#d9361b', feature: 'flame', eyes: 'round' },
		onReveal: (b, c) => friendsHere(b, c).forEach((x) => b.buff(x, 1, c))
	},
	{
		id: 'drizzlet', name: 'Drizzlet', kind: 'reveal', cost: 1, power: 1,
		text: 'On Reveal: Draw a card.',
		art: { body: '#4fb0ff', belly: '#d4f0ff', accent: '#1d6fd1', feature: 'fins', eyes: 'sleepy' },
		onReveal: (b, c) => b.draw(c.owner)
	},
	{
		id: 'sproutle', name: 'Sproutle', kind: 'endturn', cost: 1, power: 1,
		text: 'End of Turn: This card gets +1 Power.',
		art: { body: '#7ad35a', belly: '#e6f7c8', accent: '#2f8a26', feature: 'leaf', eyes: 'happy' },
		endTurn: (b, c) => b.buff(c, 1, c)
	},
	{
		id: 'zapling', name: 'Zapling', kind: 'reveal', cost: 1, power: 2,
		text: 'On Reveal: A random enemy card here gets -1 Power.',
		art: { body: '#ffd84a', belly: '#fff6c8', accent: '#3a3a3a', feature: 'antenna', eyes: 'round' },
		onReveal: (b, c) => {
			const t = b.pick(enemiesHere(b, c));
			if (t) b.buff(t, -1, c);
		}
	},
	{
		id: 'quickhop', name: 'Quickhop', kind: 'start', cost: 1, power: 2,
		text: 'Game Start: Put this card into your opening hand.',
		art: { body: '#8fd0ff', belly: '#ffffff', accent: '#ff6a8a', feature: 'ears', eyes: 'happy' },
		gameStart: (b, c) => b.toOpeningHand(c)
	},
	// ---------- cost 2 ----------
	{
		id: 'voltail', name: 'Voltail', kind: 'reveal', cost: 2, power: 2,
		text: 'On Reveal: A random enemy card here gets -2 Power.',
		art: { body: '#ffc928', belly: '#fff3b8', accent: '#7a4ad6', feature: 'ears', eyes: 'angry' },
		onReveal: (b, c) => {
			const t = b.pick(enemiesHere(b, c));
			if (t) b.buff(t, -2, c);
		}
	},
	{
		id: 'cinderpup', name: 'Cinderpup', kind: 'reveal', cost: 2, power: 2,
		text: 'On Reveal: If an enemy card is here, +3 Power.',
		art: { body: '#e8562a', belly: '#ffd0a0', accent: '#3a2a20', feature: 'ears', eyes: 'angry' },
		onReveal: (b, c) => {
			if (enemiesHere(b, c).length) b.buff(c, 3, c);
		}
	},
	{
		id: 'mosscub', name: 'Mosscub', kind: 'ongoing', cost: 2, power: 1,
		text: 'Ongoing: Your other cards here have +1 Power.',
		art: { body: '#5a9e3a', belly: '#c8e6a0', accent: '#3d6b22', feature: 'ears', eyes: 'sleepy' },
		aura: (b, c, t) => (t.owner === c.owner && t.zone === c.zone && t.uid !== c.uid ? 1 : 0)
	},
	{
		id: 'gloomoth', name: 'Gloomoth', kind: 'ongoing', cost: 2, power: 2,
		text: 'Ongoing: Enemy cards here have -1 Power.',
		art: { body: '#8a6ad0', belly: '#d8c8ff', accent: '#3a2a6a', feature: 'wings', eyes: 'sleepy' },
		aura: (b, c, t) => (t.owner !== c.owner && t.zone === c.zone ? -1 : 0)
	},
	{
		id: 'gustling', name: 'Gustling', kind: 'activate', cost: 2, power: 2,
		text: 'Activate: Your other cards here get +1 Power.',
		art: { body: '#9fe6f0', belly: '#ffffff', accent: '#2f8f99', feature: 'wings', eyes: 'happy' },
		activate: (b, c) => friendsHere(b, c).forEach((x) => b.buff(x, 1, c))
	},
	{
		id: 'sparkmole', name: 'Sparkmole', kind: 'activate', cost: 2, power: 3,
		text: 'Activate: Draw a card.',
		art: { body: '#7a6a5a', belly: '#e8d8c0', accent: '#ffd23d', feature: 'antenna', eyes: 'sleepy' },
		activate: (b, c) => b.draw(c.owner)
	},
	{
		id: 'tickbot', name: 'Tickbot', kind: 'endturn', cost: 2, power: 2,
		text: 'End of Turn: A random other card of yours here gets +1 Power.',
		art: { body: '#a0a5a9', belly: '#e8eef2', accent: '#c91a09', feature: 'antenna', eyes: 'round' },
		endTurn: (b, c) => {
			const t = b.pick(friendsHere(b, c));
			if (t) b.buff(t, 1, c);
		}
	},
	// ---------- cost 3 ----------
	{
		id: 'tidalotl', name: 'Tidalotl', kind: 'reveal', cost: 3, power: 3,
		text: 'On Reveal: Your cards in other zones get +1 Power.',
		art: { body: '#ff9ec4', belly: '#ffe0ee', accent: '#3d9bff', feature: 'tentacles', eyes: 'happy' },
		onReveal: (b, c) => b.allCards(c.owner).filter((x) => x.zone !== c.zone).forEach((x) => b.buff(x, 1, c))
	},
	{
		id: 'rockodile', name: 'Rockodile', kind: 'none', cost: 3, power: 5,
		text: 'Big jaws, no tricks.',
		art: { body: '#7d8c5a', belly: '#d8d0a0', accent: '#4a5530', feature: 'spikes', eyes: 'angry', big: true }
	},
	{
		id: 'fungoo', name: 'Fungoo', kind: 'reveal', cost: 3, power: 2,
		text: 'On Reveal: All enemy cards here get -1 Power.',
		art: { body: '#d05050', belly: '#fff0e0', accent: '#ffffff', feature: 'crest', eyes: 'sleepy' },
		onReveal: (b, c) => enemiesHere(b, c).forEach((x) => b.buff(x, -1, c))
	},
	{
		id: 'zephyrrow', name: 'Zephyrrow', kind: 'ongoing', cost: 3, power: 3,
		text: 'Ongoing: +4 Power if this is your only card here.',
		art: { body: '#5ab8e0', belly: '#e8f8ff', accent: '#f0b030', feature: 'wings', eyes: 'angry' },
		selfBonus: (b, c) => (b.cardsAt(c.zone, c.owner).length === 1 ? 4 : 0)
	},
	{
		id: 'glimmerling', name: 'Glimmerling', kind: 'reveal', cost: 3, power: 3,
		text: 'On Reveal: Add a Brickling (1 cost, 2 Power) to your hand.',
		art: { body: '#fff0a0', belly: '#ffffff', accent: '#f0a020', feature: 'antenna', eyes: 'happy' },
		onReveal: (b, c) => b.addToHand(c.owner, 'brickling')
	},
	{
		id: 'echowl', name: 'Echowl', kind: 'ongoing', cost: 3, power: 2,
		text: 'Ongoing: On Reveal abilities of your cards here happen twice.',
		art: { body: '#8a5a3a', belly: '#f0d8b0', accent: '#ffd23d', feature: 'wings', eyes: 'round' },
		doublesReveal: true
	},
	{
		id: 'buildbeaver', name: 'Buildbeaver', kind: 'start', cost: 3, power: 3,
		text: 'Game Start: This card gets +2 Power for this game.',
		art: { body: '#a0603a', belly: '#f0c890', accent: '#f4f4f4', feature: 'ears', eyes: 'happy' },
		gameStart: (b, c) => {
			c.mod += 2;
		}
	},
	// ---------- cost 4 ----------
	{
		id: 'thundrake', name: 'Thundrake', kind: 'reveal', cost: 4, power: 5,
		text: 'On Reveal: The strongest enemy card here gets -3 Power.',
		art: { body: '#3a4ad0', belly: '#ffe060', accent: '#ffd23d', feature: 'horns', eyes: 'angry', big: true },
		onReveal: (b, c) => {
			const es = enemiesHere(b, c);
			if (!es.length) return;
			const t = es.reduce((a, x) => (b.power(x) > b.power(a) ? x : a));
			b.buff(t, -3, c);
		}
	},
	{
		id: 'coralyth', name: 'Coralyth', kind: 'ongoing', cost: 4, power: 3,
		text: 'Ongoing: Your cards in other zones have +1 Power.',
		art: { body: '#ff7a8a', belly: '#ffe0e4', accent: '#3d9bff', feature: 'crest', eyes: 'round' },
		aura: (b, c, t) => (t.owner === c.owner && t.zone !== c.zone ? 1 : 0)
	},
	{
		id: 'emberhorn', name: 'Emberhorn', kind: 'reveal', cost: 4, power: 7,
		text: 'On Reveal: Your other cards here get -1 Power.',
		art: { body: '#c8402a', belly: '#ffc070', accent: '#4a2a1a', feature: 'horns', eyes: 'angry', big: true },
		onReveal: (b, c) => friendsHere(b, c).forEach((x) => b.buff(x, -1, c))
	},
	{
		id: 'bramblewolf', name: 'Bramblewolf', kind: 'ongoing', cost: 4, power: 3,
		text: 'Ongoing: +1 Power for each other card you have here.',
		art: { body: '#3e7a32', belly: '#b8e090', accent: '#8a3a2a', feature: 'spikes', eyes: 'angry' },
		selfBonus: (b, c) => b.cardsAt(c.zone, c.owner).length - 1
	},
	{
		id: 'phantomule', name: 'Phantomule', kind: 'reveal', cost: 4, power: 3,
		text: 'On Reveal: Destroy the weakest enemy card here.',
		art: { body: '#6a5aa0', belly: '#c8c0f0', accent: '#2a1a4a', feature: 'ghost', eyes: 'sleepy' },
		onReveal: (b, c) => {
			const es = enemiesHere(b, c);
			if (!es.length) return;
			const t = es.reduce((a, x) => (b.power(x) < b.power(a) ? x : a));
			b.destroy(t, c);
		}
	},
	{
		id: 'boomshroom', name: 'Boomshroom', kind: 'activate', cost: 4, power: 4,
		text: 'Activate: All enemy cards here get -2 Power.',
		art: { body: '#e04a8a', belly: '#fff0f6', accent: '#ffffff', feature: 'crest', eyes: 'angry' },
		activate: (b, c) => enemiesHere(b, c).forEach((x) => b.buff(x, -2, c))
	},
	// ---------- cost 5 ----------
	{
		id: 'glaciomoth', name: 'Glaciomoth', kind: 'reveal', cost: 5, power: 4,
		text: 'On Reveal: All enemy cards here get -2 Power.',
		art: { body: '#a8e0ff', belly: '#ffffff', accent: '#3a7ad0', feature: 'wings', eyes: 'sleepy', big: true },
		onReveal: (b, c) => enemiesHere(b, c).forEach((x) => b.buff(x, -2, c))
	},
	{
		id: 'magmaw', name: 'Magmaw', kind: 'none', cost: 5, power: 8,
		text: 'A walking volcano.',
		art: { body: '#5a2a2a', belly: '#ff7a2a', accent: '#ffd23d', feature: 'spikes', eyes: 'angry', big: true }
	},
	{
		id: 'lumifox', name: 'Lumifox', kind: 'reveal', cost: 5, power: 3,
		text: 'On Reveal: Your other cards here get +2 Power.',
		art: { body: '#fff4d0', belly: '#ffffff', accent: '#ffb030', feature: 'ears', eyes: 'happy' },
		onReveal: (b, c) => friendsHere(b, c).forEach((x) => b.buff(x, 2, c))
	},
	{
		id: 'cragoliath', name: 'Cragoliath', kind: 'ongoing', cost: 5, power: 6,
		text: 'Ongoing: +3 Power if your side of this zone is full.',
		art: { body: '#8a8a8a', belly: '#c8c8c8', accent: '#4a4a4a', feature: 'shell', eyes: 'angry', big: true },
		selfBonus: (b, c) => (b.cardsAt(c.zone, c.owner).length >= b.capacity(c.zone) ? 3 : 0)
	},
	{
		id: 'clockcrab', name: 'Clockcrab', kind: 'endturn', cost: 5, power: 4,
		text: 'End of Turn: This card gets +2 Power.',
		art: { body: '#e06a3a', belly: '#ffe0c0', accent: '#ffd23d', feature: 'shell', eyes: 'round', big: true },
		endTurn: (b, c) => b.buff(c, 2, c)
	},
	// ---------- cost 6 ----------
	{
		id: 'titanoak', name: 'Titanoak', kind: 'none', cost: 6, power: 11,
		text: 'An ancient tree-titan.',
		art: { body: '#7a5a3a', belly: '#c8a070', accent: '#3a9a2a', feature: 'leaf', eyes: 'sleepy', big: true }
	},
	{
		id: 'stormaw', name: 'Stormaw', kind: 'reveal', cost: 6, power: 7,
		text: 'On Reveal: ALL enemy cards get -1 Power.',
		art: { body: '#2a2a5a', belly: '#ffe060', accent: '#ffd23d', feature: 'crest', eyes: 'angry', big: true },
		onReveal: (b, c) => b.allCards(b.other(c.owner)).forEach((x) => b.buff(x, -1, c))
	},
	{
		id: 'abyssquid', name: 'Abyssquid', kind: 'ongoing', cost: 6, power: 5,
		text: 'Ongoing: Your other cards here have +2 Power.',
		art: { body: '#2a3a8a', belly: '#8ab0ff', accent: '#ff5a8a', feature: 'tentacles', eyes: 'round', big: true },
		aura: (b, c, t) => (t.owner === c.owner && t.zone === c.zone && t.uid !== c.uid ? 2 : 0)
	},
	// ---------- special rewards ----------
	{
		id: 'goldbrick', name: 'Goldbrick', kind: 'start', cost: 2, power: 3, special: true,
		text: 'Game Start: Put this card into your opening hand. It gets +2 Power for this game.',
		art: { body: '#ffcc33', belly: '#fff3b8', accent: '#c28e1b', feature: 'crest', eyes: 'happy' },
		gameStart: (b, c) => {
			b.toOpeningHand(c);
			c.mod += 2;
		}
	},
	// ---------- tokens ----------
	{
		id: 'brickling', name: 'Brickling', kind: 'none', cost: 1, power: 2, token: true,
		text: 'A tiny sparkle brick.',
		art: { body: '#ffe680', belly: '#ffffff', accent: '#f0a020', feature: 'antenna', eyes: 'happy' }
	}
];

export const CARD_MAP: Record<string, CardDef> = Object.fromEntries(CARDS.map((c) => [c.id, c]));
export const POOL = CARDS.filter((c) => !c.token && !c.special).map((c) => c.id);

// ---------- Zones (locations) ----------
export interface ZoneDef {
	id: string;
	name: string;
	text: string;
	color: string;
	icon: string;
	capacity?: number;
	noOnReveal?: boolean;
	/** On Reveal abilities here happen one extra time. */
	extraReveal?: boolean;
	/** End of Turn abilities here happen one extra time. */
	extraEndTurn?: boolean;
	bonus?: (b: Battle, zone: number, c: CardInst) => number;
	canPlay?: (b: Battle, zone: number) => boolean;
}

const kindBoost = (k: AbilityKind, n: number) => (_b: Battle, _z: number, c: CardInst) => (CARD_MAP[c.id].kind === k ? n : 0);

export const ZONES: ZoneDef[] = [
	{ id: 'meadow', name: 'Sunny Meadow', text: 'No effect. Just a nice day.', color: '#7ccf5a', icon: 'wb_sunny' },
	{ id: 'stage', name: 'Spotlight Stage', text: 'On Reveal abilities here happen twice.', color: '#ff8a3d', icon: 'theater_comedy', extraReveal: true },
	{ id: 'pond', name: 'Calm Pond', text: 'Ongoing cards here have +2 Power.', color: '#3d9bff', icon: 'waves', bonus: kindBoost('ongoing', 2) },
	{ id: 'grove', name: 'Old Grove', text: 'Cards with no ability have +2 Power here.', color: '#3e9a32', icon: 'forest', bonus: kindBoost('none', 2) },
	{ id: 'ledge', name: 'Sky Ledge', text: 'Only 2 cards per side fit here.', color: '#8fe0e6', icon: 'cloud', capacity: 2 },
	{ id: 'barn', name: 'Haunted Barn', text: 'All cards here have -1 Power.', color: '#7a5aa0', icon: 'nights_stay', bonus: () => -1 },
	{ id: 'library', name: 'Quiet Library', text: 'On Reveal abilities do not trigger here.', color: '#b08a5a', icon: 'menu_book', noOnReveal: true },
	{ id: 'gym', name: 'Brick Gym', text: 'All cards here have +1 Power.', color: '#e04040', icon: 'fitness_center', bonus: () => 1 },
	{
		id: 'toybox', name: 'Toy Box', text: 'Cards that cost 1 or 2 have +2 Power here.', color: '#ffb030', icon: 'toys',
		bonus: (_b, _z, c) => (CARD_MAP[c.id].cost <= 2 ? 2 : 0)
	},
	{ id: 'clock', name: 'Clock Tower', text: 'End of Turn abilities here happen twice.', color: '#9a74d6', icon: 'schedule', extraEndTurn: true },
	{ id: 'tower', name: 'Thunder Tower', text: 'Closed on the final turn.', color: '#ffd23d', icon: 'cell_tower', canPlay: (b) => b.turn < b.maxTurns },
	{ id: 'workshop', name: 'Workshop', text: 'Activate cards here have +2 Power.', color: '#4fc06a', icon: 'construction', bonus: kindBoost('activate', 2) }
];
export const ZONE_MAP: Record<string, ZoneDef> = Object.fromEntries(ZONES.map((z) => [z.id, z]));

// ---------- Decks ----------
export const STARTER_COLLECTION = [
	'pebbit', 'flicker', 'drizzlet', 'sproutle', 'zapling', 'quickhop', 'voltail', 'cinderpup', 'mosscub',
	'gustling', 'tidalotl', 'rockodile', 'fungoo', 'thundrake', 'emberhorn', 'bramblewolf', 'titanoak', 'echowl'
];
export const STARTER_DECK = [
	'quickhop', 'flicker', 'drizzlet', 'zapling', 'voltail', 'cinderpup',
	'gustling', 'tidalotl', 'echowl', 'thundrake', 'emberhorn', 'titanoak'
];
export const RIVAL_DECK = [
	'flicker', 'zapling', 'cinderpup', 'gloomoth', 'sparkmole', 'tickbot',
	'tidalotl', 'phantomule', 'thundrake', 'boomshroom', 'lumifox', 'stormaw'
];
/** Dex's rematch deck once you've beaten him: bigger threats and Ongoing synergies */
export const RIVAL_DECK_2 = [
	'flicker', 'voltail', 'cinderpup', 'tickbot', 'echowl', 'coralyth',
	'thundrake', 'phantomule', 'glaciomoth', 'lumifox', 'stormaw', 'abyssquid'
];
export const PRACTICE_DECK = [
	'pebbit', 'sproutle', 'quickhop', 'mosscub', 'gustling', 'rockodile',
	'buildbeaver', 'bramblewolf', 'coralyth', 'clockcrab', 'cragoliath', 'titanoak'
];
export const DECK_SIZE = 12;
