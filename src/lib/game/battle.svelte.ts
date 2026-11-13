// Snap-style card battle: 3 zones, 6 turns, energy = turn, win 2 of 3 zones.
import { CARD_MAP, ZONES, ZONE_MAP, type ZoneDef } from './cards';

export type Side = 'p' | 'ai';

export interface CardInst {
	uid: number;
	id: string;
	owner: Side;
	zone: number; // -1 = hand / deck
	mod: number; // permanent power changes from abilities
	revealed: boolean;
	staged: boolean;
	order: number;
	pulse: number; // increments to trigger UI animation (reveal)
	fxN: number; // last power change (+/-)
	fxId: number; // increments on every power change -> UI effect
	fxFrom: number; // uid of the card that caused the last change
	activated: boolean;
	/** Activate chosen this turn: it happens during the reveal, in the order you did things (>= 0), -1 = none */
	actOrder?: number;
	/** order in which the card entered the hand: the hand is always kept in this order */
	hseq?: number;
}

export interface ZoneState {
	id: string;
	revealed: boolean;
	p: CardInst[];
	ai: CardInst[];
}

export interface BattleOptions {
	opponent: string;
	kind: 'wild' | 'rival' | 'practice';
	playerDeck: string[];
	aiDeck: string[];
	/** rare golden wild pack */
	shiny?: boolean;
}

export type Phase = 'plan' | 'reveal' | 'over';
export type Result = 'win' | 'lose' | 'draw' | 'retreat' | 'fold';

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
let UID = 1;
let HSEQ = 1;

function shuffle<T>(a: T[]): T[] {
	const r = [...a];
	for (let i = r.length - 1; i > 0; i--) {
		const j = Math.floor(Math.random() * (i + 1));
		[r[i], r[j]] = [r[j], r[i]];
	}
	return r;
}

export class Battle {
	opts: BattleOptions;
	turn = $state(0);
	maxTurns = 6;
	energy = $state(0);
	energyLeft = $state(0);
	phase = $state<Phase>('plan');
	zones = $state<ZoneState[]>([]);
	hand = $state<CardInst[]>([]);
	aiHand = $state<CardInst[]>([]);
	deck = $state<CardInst[]>([]);
	aiDeck = $state<CardInst[]>([]);
	staged = $state<CardInst[]>([]);
	/** your Activates chosen this turn (they happen during the reveal) */
	pendingActs = $state<CardInst[]>([]);
	log = $state<string[]>([]);
	stakes = $state(1);
	pSnapped = $state(false);
	aiSnapped = $state(false);
	pendingSnap = $state(false);
	result = $state<Result | null>(null);
	firstReveal = $state<Side>('p');
	private playCounter = 0;
	speed = 1;
	/** UI hook for power-change effects (sound) */
	onFx?: (n: number, target: CardInst, src: CardInst) => void;
	/** UI hook: a card is about to be destroyed */
	onDestroy?: (target: CardInst, src: CardInst) => void;

	constructor(opts: BattleOptions) {
		this.opts = opts;
		const mk = (id: string, owner: Side): CardInst => ({
			uid: UID++, id, owner, zone: -1, mod: 0, revealed: false, staged: false, order: 0, pulse: 0, activated: false, fxN: 0, fxId: 0, fxFrom: 0
		});
		this.deck = shuffle(opts.playerDeck).map((id) => mk(id, 'p'));
		this.aiDeck = shuffle(opts.aiDeck).map((id) => mk(id, 'ai'));
		this.zones = shuffle(ZONES)
			.slice(0, 3)
			.map((z) => ({ id: z.id, revealed: false, p: [], ai: [] }));
		// Game Start abilities (cards are still in the deck)
		for (const c of [...this.deck, ...this.aiDeck]) {
			const def = CARD_MAP[c.id];
			if (def.gameStart) {
				def.gameStart(this, c);
				if (c.owner === 'p') this.say(`Game Start: ${def.name}`);
			}
		}
		for (let i = 0; i < 3; i++) {
			this.draw('p', true);
			this.draw('ai', true);
		}
		this.nextTurn();
	}

	// ---------------- helpers used by card abilities ----------------
	other(s: Side): Side {
		return s === 'p' ? 'ai' : 'p';
	}
	zoneDef(i: number): ZoneDef {
		return ZONE_MAP[this.zones[i].id];
	}
	/** Revealed cards of a side at a zone. */
	cardsAt(zone: number, side: Side): CardInst[] {
		if (zone < 0) return [];
		return this.zones[zone][side].filter((c) => c.revealed);
	}
	allCards(side: Side): CardInst[] {
		return this.zones.flatMap((z) => z[side].filter((c) => c.revealed));
	}
	capacity(zone: number): number {
		const z = this.zones[zone];
		return z.revealed ? (this.zoneDef(zone).capacity ?? 4) : 4;
	}
	pick<T>(arr: T[]): T | undefined {
		return arr.length ? arr[Math.floor(Math.random() * arr.length)] : undefined;
	}
	name(c: CardInst) {
		return CARD_MAP[c.id].name;
	}
	buff(target: CardInst, n: number, src: CardInst) {
		target.mod += n;
		target.fxN = n;
		target.fxFrom = src.uid;
		target.fxId++;
		this.onFx?.(n, target, src);
		this.say(`${this.name(src)}: ${this.name(target)} ${n > 0 ? '+' : ''}${n} Power`);
	}
	destroy(target: CardInst, src: CardInst) {
		this.onDestroy?.(target, src); // let the UI grab the card's look before it disappears
		const z = this.zones[target.zone];
		z[target.owner] = z[target.owner].filter((c) => c.uid !== target.uid);
		this.say(`${this.name(src)} destroyed ${this.name(target)}!`);
	}
	draw(side: Side, silent = false) {
		const deck = side === 'p' ? this.deck : this.aiDeck;
		const hand = side === 'p' ? this.hand : this.aiHand;
		if (!deck.length || hand.length >= 7) return;
		const c = deck.shift()!;
		c.hseq = HSEQ++;
		hand.push(c);
		if (!silent && side === 'p') this.say(`You drew ${this.name(c)}.`);
	}
	addToHand(side: Side, id: string) {
		const hand = side === 'p' ? this.hand : this.aiHand;
		if (hand.length >= 7) return;
		hand.push({ uid: UID++, id, owner: side, zone: -1, mod: 0, revealed: false, staged: false, order: 0, pulse: 0, activated: false, fxN: 0, fxId: 0, fxFrom: 0, hseq: HSEQ++ });
		this.say(`${side === 'p' ? 'You' : this.opts.opponent} got a ${CARD_MAP[id].name}.`);
	}
	/** Moves a deck card to the top so it ends up in the opening hand. */
	toOpeningHand(c: CardInst) {
		const deck = c.owner === 'p' ? this.deck : this.aiDeck;
		const i = deck.findIndex((x) => x.uid === c.uid);
		if (i > 0) deck.unshift(...deck.splice(i, 1));
	}
	/** How often an On Reveal ability fires for this card right now. */
	revealTimes(c: CardInst) {
		const zd = this.zoneDef(c.zone);
		const zoneOpen = this.zones[c.zone].revealed;
		if (zoneOpen && zd.noOnReveal) return 0;
		let n = 1;
		if (zoneOpen && zd.extraReveal) n++;
		n += this.cardsAt(c.zone, c.owner).filter((x) => x.uid !== c.uid && CARD_MAP[x.id].doublesReveal).length;
		return n;
	}
	canActivate(c: CardInst) {
		const def = CARD_MAP[c.id];
		return !!def.activate && c.revealed && !c.activated && this.queued(c) === false && c.zone >= 0 && (c.owner !== 'p' || this.phase === 'plan');
	}
	/** an Activate chosen this turn, waiting for the reveal */
	queued(c: CardInst) {
		return (c.actOrder ?? -1) >= 0;
	}
	/**
	 * Choose to activate a card. It does not happen right away: like the cards you play,
	 * it happens when the turn is revealed, in the order you did things.
	 */
	activate(c: CardInst) {
		if (!this.canActivate(c)) return false;
		c.actOrder = this.playCounter++;
		c.pulse++;
		if (c.owner === 'p') this.pendingActs.push(c);
		return true;
	}
	/** take back an Activate chosen this turn */
	cancelActivate(c: CardInst) {
		if (this.phase !== 'plan' || !this.queued(c)) return;
		c.actOrder = -1;
		this.pendingActs = this.pendingActs.filter((x) => x.uid !== c.uid);
	}
	/** the Activate really happens (during the reveal) */
	private runActivate(c: CardInst) {
		c.actOrder = -1;
		if (c.activated || c.zone < 0 || !this.zones[c.zone][c.owner].some((x) => x.uid === c.uid)) return false;
		c.activated = true;
		c.pulse++;
		this.say(`${c.owner === 'p' ? 'You' : this.opts.opponent} activated ${this.name(c)}.`);
		CARD_MAP[c.id].activate!(this, c);
		return true;
	}
	say(msg: string) {
		this.log = [msg, ...this.log].slice(0, 30);
	}

	// ---------------- power ----------------
	power(c: CardInst): number {
		const def = CARD_MAP[c.id];
		let p = def.power + c.mod;
		if (c.zone < 0 || !c.revealed) return p;
		if (def.selfBonus) p += def.selfBonus(this, c);
		const zd = this.zoneDef(c.zone);
		if (this.zones[c.zone].revealed && zd.bonus) p += zd.bonus(this, c.zone, c);
		for (const z of this.zones) {
			for (const s of ['p', 'ai'] as Side[]) {
				for (const o of z[s]) {
					if (!o.revealed) continue;
					const od = CARD_MAP[o.id];
					if (od.aura) p += od.aura(this, o, c);
				}
			}
		}
		return p;
	}
	zonePower(zone: number, side: Side): number {
		return this.cardsAt(zone, side).reduce((s, c) => s + this.power(c), 0);
	}
	zoneWinner(zone: number): Side | null {
		const a = this.zonePower(zone, 'p');
		const b = this.zonePower(zone, 'ai');
		return a > b ? 'p' : b > a ? 'ai' : null;
	}
	winningZones(side: Side) {
		return this.zones.filter((_, i) => this.zoneWinner(i) === side).length;
	}

	// ---------------- turn flow ----------------
	nextTurn() {
		this.turn++;
		if (this.turn <= 3) {
			this.zones[this.turn - 1].revealed = true;
			this.say(`Zone revealed: ${this.zoneDef(this.turn - 1).name}`);
		}
		this.energy = this.turn;
		this.energyLeft = this.turn;
		this.draw('p');
		this.draw('ai');
		if (this.turn === this.maxTurns) {
			this.stakes *= 2;
			this.say('Final turn! Stakes doubled.');
		}
		this.staged = [];
		this.pendingActs = [];
		this.phase = 'plan';
		this.maybeAiSnap();
	}

	canPlay(c: CardInst, zone: number): boolean {
		if (this.phase !== 'plan') return false;
		if (CARD_MAP[c.id].cost > this.energyLeft) return false;
		return this.zoneOpen(zone, 'p');
	}
	zoneOpen(zone: number, side: Side) {
		const z = this.zones[zone];
		if (z[side].length >= this.capacity(zone)) return false;
		const zd = this.zoneDef(zone);
		if (z.revealed && zd.canPlay && !zd.canPlay(this, zone)) return false;
		return true;
	}

	stage(c: CardInst, zone: number) {
		if (!this.canPlay(c, zone)) return false;
		this.hand = this.hand.filter((h) => h.uid !== c.uid);
		c.zone = zone;
		c.staged = true;
		c.order = this.playCounter++;
		this.zones[zone].p.push(c);
		this.staged.push(c);
		this.energyLeft -= CARD_MAP[c.id].cost;
		return true;
	}

	unstage(c: CardInst) {
		if (this.phase !== 'plan' || !c.staged) return;
		const z = this.zones[c.zone];
		z.p = z.p.filter((x) => x.uid !== c.uid);
		this.staged = this.staged.filter((x) => x.uid !== c.uid);
		this.energyLeft += CARD_MAP[c.id].cost;
		c.zone = -1;
		c.staged = false;
		// back into its old slot (no reordering the hand by playing and taking back)
		const at = this.hand.findIndex((h) => (h.hseq ?? 0) > (c.hseq ?? 0));
		if (at < 0) this.hand.push(c);
		else this.hand.splice(at, 0, c);
	}

	snap() {
		if (this.pSnapped || this.phase === 'over') return;
		this.pSnapped = true;
		this.stakes *= 2;
		this.say('You SNAPPED! Stakes doubled.');
	}

	retreat() {
		if (this.phase === 'over') return;
		this.phase = 'over';
		this.result = 'retreat';
		this.say('You retreated.');
	}

	async endTurn() {
		if (this.phase !== 'plan') return;
		this.phase = 'reveal';
		const aiActs = this.allCards('ai').filter((c) => this.canActivate(c));
		aiActs.forEach((c) => this.activate(c));
		const aiPlays = this.aiPlan();
		// whoever is winning more zones reveals first
		const pw = this.winningZones('p');
		const aw = this.winningZones('ai');
		this.firstReveal = pw > aw ? 'p' : aw > pw ? 'ai' : Math.random() < 0.5 ? 'p' : 'ai';
		// each side: the cards it played and the Activates it chose, in the order they were done
		type Step = { c: CardInst; act: boolean; at: number };
		const steps = (plays: CardInst[], acts: CardInst[]): Step[] =>
			[...plays.map((c) => ({ c, act: false, at: c.order })), ...acts.map((c) => ({ c, act: true, at: c.actOrder ?? 0 }))].sort((a, b) => a.at - b.at);
		const mine = steps([...this.staged], [...this.pendingActs]);
		const theirs = steps(aiPlays, aiActs);
		const order: Step[][] = this.firstReveal === 'p' ? [mine, theirs] : [theirs, mine];

		await sleep(350 * this.speed);
		for (const list of order) {
			for (const { c, act } of list) {
				if (act) {
					if (this.runActivate(c)) await sleep(650 * this.speed);
					continue;
				}
				// card might have been destroyed meanwhile
				if (c.zone < 0 || !this.zones[c.zone][c.owner].some((x) => x.uid === c.uid)) continue;
				c.staged = false;
				c.revealed = true;
				c.pulse++;
				const def = CARD_MAP[c.id];
				if (def.onReveal) {
					const times = this.revealTimes(c);
					for (let k = 0; k < times; k++) {
						if (k > 0) await sleep(300 * this.speed);
						def.onReveal(this, c);
					}
				}
				await sleep(650 * this.speed);
			}
		}
		this.staged = [];
		this.pendingActs = [];
		// End of Turn abilities
		for (const side of [this.firstReveal, this.other(this.firstReveal)]) {
			for (let z = 0; z < 3; z++) {
				for (const c of this.cardsAt(z, side)) {
					const def = CARD_MAP[c.id];
					if (!def.endTurn) continue;
					const times = this.zones[z].revealed && this.zoneDef(z).extraEndTurn ? 2 : 1;
					for (let k = 0; k < times; k++) def.endTurn(this, c);
					await sleep(400 * this.speed);
				}
			}
		}
		if (this.turn >= this.maxTurns) {
			this.finish();
		} else {
			this.nextTurn();
		}
	}

	finish() {
		this.phase = 'over';
		const pw = this.winningZones('p');
		const aw = this.winningZones('ai');
		if (pw !== aw) {
			this.result = pw > aw ? 'win' : 'lose';
		} else {
			const tp = this.zones.reduce((s, _, i) => s + this.zonePower(i, 'p'), 0);
			const ta = this.zones.reduce((s, _, i) => s + this.zonePower(i, 'ai'), 0);
			this.result = tp > ta ? 'win' : ta > tp ? 'lose' : 'draw';
		}
		this.say(this.result === 'win' ? 'Victory!' : this.result === 'lose' ? 'Defeat...' : 'Draw.');
	}

	// ---------------- AI ----------------
	private maybeAiSnap() {
		if (this.aiSnapped || this.turn < 3 || this.turn > 5) return;
		const lead = this.zones.reduce((s, _, i) => s + this.zonePower(i, 'ai') - this.zonePower(i, 'p'), 0);
		const strongHand = this.aiHand.filter((c) => CARD_MAP[c.id].power >= 5).length;
		const score = lead / 4 + strongHand * 0.5 + this.winningZones('ai') - 1;
		if (score > 1.8 && Math.random() < 0.6) {
			this.aiSnapped = true;
			this.stakes *= 2;
			this.say(`${this.opts.opponent} SNAPPED! Stakes doubled.`);
		}
	}

	private aiPlan(): CardInst[] {
		const hand = this.aiHand;
		const energy = this.energy;
		// choose subset maximizing energy spent, then power
		let best: CardInst[] = [];
		let bestScore = -1;
		const n = Math.min(hand.length, 8);
		for (let mask = 1; mask < 1 << n; mask++) {
			let cost = 0;
			let pow = 0;
			const set: CardInst[] = [];
			for (let i = 0; i < n; i++)
				if (mask & (1 << i)) {
					const d = CARD_MAP[hand[i].id];
					cost += d.cost;
					pow += d.power;
					set.push(hand[i]);
				}
			if (cost > energy) continue;
			const score = cost * 10 + pow + Math.random();
			if (score > bestScore) {
				bestScore = score;
				best = set;
			}
		}
		// assign zones
		const pending = [0, 0, 0];
		const counts = [0, 1, 2].map((i) => this.zones[i].ai.length);
		const placed: CardInst[] = [];
		best.sort((a, b) => CARD_MAP[b.id].cost - CARD_MAP[a.id].cost);
		for (const c of best) {
			const d = CARD_MAP[c.id];
			let bestZ = -1;
			let bestS = -Infinity;
			for (let z = 0; z < 3; z++) {
				if (counts[z] >= this.capacity(z)) continue;
				const zd = this.zoneDef(z);
				if (this.zones[z].revealed && zd.canPlay && !zd.canPlay(this, z)) continue;
				const diff = this.zonePower(z, 'ai') + pending[z] - this.zonePower(z, 'p');
				let s = -Math.abs(diff + d.power - 3);
				if (diff > 10) s -= 6; // already safe
				if (diff < -14) s -= 6; // hopeless
				if (this.zones[z].revealed && zd.bonus) {
					const fake = { ...c, zone: z, revealed: true } as CardInst;
					s += zd.bonus(this, z, fake) * 2;
				}
				if (d.selfBonus && (d.id === 'gustling' || d.id === 'zephyrrow')) s += counts[z] === 0 ? 6 : -6;
				if (d.onReveal && this.zones[z].revealed && zd.noOnReveal) s -= 4;
				if ((d.id === 'voltail' || d.id === 'thundrake' || d.id === 'phantomule' || d.id === 'cinderpup' || d.id === 'glaciomoth') && this.zones[z].p.some((x) => x.revealed)) s += 4;
				s += Math.random() * 2;
				if (s > bestS) {
					bestS = s;
					bestZ = z;
				}
			}
			if (bestZ < 0) continue;
			counts[bestZ]++;
			pending[bestZ] += d.power;
			this.aiHand = this.aiHand.filter((h) => h.uid !== c.uid);
			c.zone = bestZ;
			c.staged = true;
			c.order = this.playCounter++;
			this.zones[bestZ].ai.push(c);
			placed.push(c);
		}
		return placed;
	}
}
