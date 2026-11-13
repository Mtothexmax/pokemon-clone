// Sticker book: little rewards for playing around in the village (shown on the PC in your room).
import { GOLDEN_STUDS } from '../three/town';

export interface StickerSave {
	butterflies: number;
	goals?: number;
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
	golden: string[];
	wins: number;
	rivalWins: number;
}

export interface Sticker {
	id: string;
	icon: string;
	name: string;
	hint: string;
	color: string;
	done: (s: StickerSave) => boolean;
}

export const STICKERS: Sticker[] = [
	{ id: 'butterfly', icon: 'flutter_dash', name: 'Butterfly catcher', hint: 'Catch a butterfly', color: '#ff8ac0', done: (s) => s.butterflies >= 1 },
	{ id: 'butterfly10', icon: 'filter_vintage', name: 'Butterfly fan', hint: 'Catch 10 butterflies', color: '#e96ed1', done: (s) => s.butterflies >= 10 },
	{ id: 'goal', icon: 'sports_soccer', name: 'Goal!', hint: 'Kick the ball into the goal', color: '#4fc06a', done: (s) => (s.goals ?? 0) >= 1 },
	{ id: 'hattrick', icon: 'emoji_events', name: 'Hat-trick', hint: 'Score 3 goals', color: '#2f9e44', done: (s) => (s.goals ?? 0) >= 3 },
	{ id: 'ducks', icon: 'bakery_dining', name: 'Duck friend', hint: 'Feed the ducks', color: '#f2b705', done: (s) => (s.duckFeeds ?? 0) >= 1 },
	{ id: 'ducks10', icon: 'egg', name: 'Duck parent', hint: 'Feed the ducks 10 times', color: '#e8a400', done: (s) => (s.duckFeeds ?? 0) >= 10 },
	{ id: 'firefly', icon: 'nightlight', name: 'Night owl', hint: 'Catch 5 fireflies at night', color: '#3b3f8f', done: (s) => (s.fireflies ?? 0) >= 5 },
	{ id: 'tag', icon: 'directions_run', name: 'Speedy', hint: 'Win a game of tag against Pip', color: '#ff8a1f', done: (s) => (s.tagWins ?? 0) >= 1 },
	{ id: 'seek', icon: 'visibility', name: 'Sharp eyes', hint: 'Find Pip at hide and seek', color: '#1e8f8f', done: (s) => (s.hideSeekWins ?? 0) >= 1 },
	{ id: 'photo', icon: 'photo_camera', name: 'Shutterbug', hint: 'Take 5 photos (P)', color: '#5a6b7c', done: (s) => (s.photos ?? 0) >= 5 },
	{ id: 'puddle', icon: 'water_drop', name: 'Puddle jumper', hint: 'Splash through 5 puddles', color: '#3d9bff', done: (s) => (s.puddleSplashes ?? 0) >= 5 },
	{ id: 'piano', icon: 'piano', name: 'Little maestro', hint: "Play a song on Lumi's piano", color: '#1b2a34', done: (s) => (s.pianoSongs ?? 0) >= 1 },
	{ id: 'dance', icon: 'celebration', name: 'Dance party', hint: 'Dance (Q) next to a villager', color: '#ff5fa2', done: (s) => (s.danceParties ?? 0) >= 1 },
	{ id: 'bench', icon: 'chair', name: 'Time flies', hint: 'Sit on the park bench', color: '#a0522d', done: (s) => (s.benchSits ?? 0) >= 1 },
	{ id: 'chest', icon: 'redeem', name: 'Lucky find', hint: 'Open 3 treasure chests', color: '#8a4b22', done: (s) => (s.chests ?? 0) >= 3 },
	{ id: 'golden5', icon: 'stars', name: 'Treasure hunter', hint: 'Find 5 golden studs', color: '#d4a017', done: (s) => s.golden.length >= 5 },
	{ id: 'goldenAll', icon: 'workspace_premium', name: 'Golden touch', hint: `Find all ${GOLDEN_STUDS.length} golden studs`, color: '#b8860b', done: (s) => s.golden.length >= GOLDEN_STUDS.length },
	{ id: 'win', icon: 'military_tech', name: 'First win', hint: 'Win a battle', color: '#c91a09', done: (s) => s.wins >= 1 },
	{ id: 'rival', icon: 'swords', name: 'Rival beaten', hint: 'Beat Dex', color: '#8a1006', done: (s) => s.rivalWins >= 1 }
];

/** ids of all stickers this save has earned */
export const earnedStickers = (s: StickerSave) => STICKERS.filter((k) => k.done(s)).map((k) => k.id);
