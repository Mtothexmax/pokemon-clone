// Voxelmon in the world: your 3D beast follows you around, and in the tall grass wild Voxelmon jump out
// for a real-time battle. Both sides fight on their own; you trigger your Voxelmon's 2 attacks, which then
// have to recharge (like Pokémon Quest). The wild one winds up and lunges at yours every few seconds.
import * as THREE from 'three';
import { CARD_MAP, POOL } from '../game/cards';
import { buildVoxelmon } from './voxel';

export type SkillKind = 'dash' | 'volley' | 'strike' | 'wave';
export interface Skill {
	name: string;
	kind: SkillKind;
	power: number;
	/** recharge time in seconds */
	cd: number;
	color: string;
	icon: string;
}

/** the two attacks of each Voxelmon (6-energy cards); other beasts fall back to a generic pair */
const SKILLS: Record<string, [Skill, Skill]> = {
	titanoak: [
		{ name: 'Root Slam', kind: 'dash', power: 14, cd: 2.2, color: '#a07a4a', icon: 'park' },
		{ name: 'Leaf Storm', kind: 'volley', power: 30, cd: 7, color: '#4fc06a', icon: 'eco' }
	],
	stormaw: [
		{ name: 'Thunder Bite', kind: 'dash', power: 13, cd: 2, color: '#ffd23d', icon: 'bolt' },
		{ name: 'Storm Call', kind: 'strike', power: 32, cd: 7.5, color: '#fff27a', icon: 'thunderstorm' }
	],
	abyssquid: [
		{ name: 'Ink Shot', kind: 'volley', power: 12, cd: 2.2, color: '#3a2a6a', icon: 'water_drop' },
		{ name: 'Tidal Whirl', kind: 'wave', power: 30, cd: 7, color: '#3d9bff', icon: 'cyclone' }
	]
};
const GENERIC: [Skill, Skill] = [
	{ name: 'Tackle', kind: 'dash', power: 10, cd: 2, color: '#ffffff', icon: 'sports_mma' },
	{ name: 'Brick Blast', kind: 'volley', power: 22, cd: 6, color: '#ff8a3d', icon: 'grain' }
];
export const skillsOf = (id: string) => SKILLS[id] ?? GENERIC;

export type VoxEvent =
	| { type: 'start'; wild: string; shiny: boolean }
	| { type: 'skill'; who: 'me' | 'wild'; id: string; skill: string }
	| { type: 'hit'; who: 'me' | 'wild'; dmg: number }
	| { type: 'win'; wild: string; shiny: boolean }
	| { type: 'lose'; wild: string }
	| { type: 'fled' };

export interface VoxHost {
	scene: THREE.Scene;
	player: THREE.Object3D;
	area: () => string;
	groundY: (x: number, z: number) => number;
	/** keeps a spawn point inside the tall grass */
	/** keeps a point inside the grass patch you are in; mx = distance to keep from its left/right edge */
	clampToGrass: (x: number, z: number, mx?: number) => { x: number; z: number };
	emit: (e: VoxEvent) => void;
}

interface Fighter {
	id: string;
	g: THREE.Group;
	body: THREE.Group;
	mat: THREE.MeshStandardMaterial;
	hp: number;
	max: number;
	/** home spot during the fight (it returns there after an attack) */
	home: THREE.Vector3;
	act: { kind: SkillKind | 'lunge'; skill?: Skill; t0: number; done: boolean } | null;
	flash: number;
	knock: THREE.Vector3;
	/** normal glow (shiny ones shimmer gold) */
	base: THREE.Color;
}

interface Proj {
	m: THREE.Mesh;
	from: THREE.Vector3;
	to: () => THREE.Vector3;
	t0: number;
	dur: number;
	dmg: number;
	arc: number;
	kind: 'cube' | 'ring' | 'bolt' | 'mark';
}

const tmpV = new THREE.Vector3();
const UP = new THREE.Vector3(0, 1, 0);

export class VoxelPlay {
	private h: VoxHost;
	/** your Voxelmon */
	me: (Fighter & { tired: boolean; lastArea: string; hopT: number }) | null = null;
	wild: (Fighter & { shiny: boolean; nextAtk: number }) | null = null;
	phase: 'none' | 'intro' | 'fight' | 'end' = 'none';
	/** middle of the current battle (the camera looks there) */
	arena: { x: number; z: number } | null = null;
	private phaseT = 0;
	/** time each of your 2 attacks is ready again */
	ready = [0, 0];
	private projs: Proj[] = [];
	private bits: { m: THREE.Mesh; v: THREE.Vector3; t0: number; life: number }[] = [];
	private nums: { sp: THREE.Sprite; t0: number; base: THREE.Vector3 }[] = [];
	private cube = new THREE.BoxGeometry(1, 1, 1);
	private now = 0;

	constructor(host: VoxHost) {
		this.h = host;
	}

	get busy() {
		return this.phase !== 'none';
	}
	/** can fight now (not knocked out / still resting) */
	get fit() {
		return !!this.me && !this.me.tired;
	}

	// ---------------- your Voxelmon ----------------
	setCompanion(id: string | null) {
		if (this.me?.id === id) return;
		if (this.me) this.h.scene.remove(this.me.g);
		this.me = null;
		if (!id) return;
		const g = buildVoxelmon(id, 4.2);
		if (!g) return;
		const def = CARD_MAP[id];
		const max = 90 + def.power * 3;
		const p = this.h.player.position;
		g.position.set(p.x - 2, this.h.groundY(p.x - 2, p.z + 2), p.z + 2);
		this.h.scene.add(g);
		this.me = { ...this.fighter(id, g, max), tired: false, lastArea: this.h.area(), hopT: 0 };
	}
	private fighter(id: string, g: THREE.Group, max: number): Fighter {
		return { id, g, body: g.userData.body, mat: g.userData.mat, hp: max, max, home: new THREE.Vector3(), act: null, flash: 0, knock: new THREE.Vector3(), base: (g.userData.mat as THREE.MeshStandardMaterial).emissive.clone() };
	}
	/** full health again (a night in your bed) */
	heal() {
		if (!this.me) return;
		this.me.hp = this.me.max;
		this.me.tired = false;
		this.me.body.rotation.set(0, 0, 0);
	}

	// ---------------- wild encounter ----------------
	/** a wild Voxelmon jumps out of the grass in front of you */
	startWild() {
		if (!this.me || this.busy || this.me.tired) return false;
		// mostly small beasts, now and then a big one; never the 6-energy legends
		const pool = POOL.filter((id) => CARD_MAP[id].cost <= 5);
		const weighted = pool.flatMap((id) => Array(6 - CARD_MAP[id].cost).fill(id) as string[]);
		let id = '';
		let g: THREE.Group | null = null;
		const shiny = Math.random() < 1 / 12;
		for (let tries = 0; tries < 8 && !g; tries++) {
			id = weighted[Math.floor(Math.random() * weighted.length)];
			g = buildVoxelmon(id, 2.4 + CARD_MAP[id].cost * 0.25, { shiny });
		}
		if (!g) return false;
		// side by side in front of you (seen from the camera), so nobody stands behind your figure
		const p = this.h.player.position;
		const ry = this.h.player.rotation.y;
		const ahead = Math.cos(ry) > 0 ? 1 : -1; // facing south (+z) or north (-z)
		// the middle of the arena, with room for both beasts (3.4 to each side) inside the patch
		const c = this.h.clampToGrass(p.x, p.z + ahead * 5, 5.6);
		const side = p.x > 0 ? -1 : 1; // the wild one on the roomier side
		const spot = this.h.clampToGrass(c.x + side * 3.4, c.z);
		this.arena = c;
		const cost = CARD_MAP[id].cost;
		const max = Math.round((40 + cost * 13) * (shiny ? 1.4 : 1));
		const w = { ...this.fighter(id, g, max), shiny, nextAtk: 0 };
		w.home.set(spot.x, this.h.groundY(spot.x, spot.z), spot.z);
		g.position.copy(w.home);
		g.scale.setScalar(0.01);
		this.h.scene.add(g);
		this.wild = w;
		// your Voxelmon steps in between
		const me = this.me;
		me.home.set(c.x - side * 3.4, 0, c.z);
		me.home.y = this.h.groundY(me.home.x, me.home.z);
		me.act = null;
		this.projs.forEach((pr) => this.h.scene.remove(pr.m));
		this.projs = [];
		this.phase = 'intro';
		this.phaseT = this.now;
		this.ready = [this.now + 1.2, this.now + 1.2];
		this.h.emit({ type: 'start', wild: id, shiny });
		return true;
	}
	/** run away: the wild one hops back into the grass */
	flee() {
		if (this.phase !== 'fight' && this.phase !== 'intro') return;
		this.endFight('fled');
	}
	/** trigger attack 0 or 1 (keys 1 / 2) */
	useSkill(i: number) {
		const me = this.me, w = this.wild;
		if (this.phase !== 'fight' || !me || !w || me.act || this.now < this.ready[i]) return false;
		const sk = skillsOf(me.id)[i];
		this.ready[i] = this.now + sk.cd;
		me.act = { kind: sk.kind, skill: sk, t0: this.now, done: false };
		this.h.emit({ type: 'skill', who: 'me', id: me.id, skill: sk.name });
		return true;
	}

	/** spots where the grass gets pressed down: under the Voxelmon, and the whole arena during a battle */
	grassPushers() {
		const out: { x: number; z: number; r: number }[] = [];
		if (this.me) out.push({ x: this.me.g.position.x, z: this.me.g.position.z, r: this.phase === 'none' ? 2.2 : 3.4 });
		if (this.wild && this.wild.g.visible) out.push({ x: this.wild.g.position.x, z: this.wild.g.position.z, r: 3.2 });
		return out;
	}
	/** state for the HUD */
	hud() {
		const me = this.me, w = this.wild;
		if (!me) return null;
		return {
			phase: this.phase,
			me: { id: me.id, name: CARD_MAP[me.id].name, hp: Math.ceil(me.hp), max: me.max, tired: me.tired },
			wild: w ? { id: w.id, name: CARD_MAP[w.id].name, hp: Math.ceil(Math.max(0, w.hp)), max: w.max, shiny: w.shiny } : null,
			skills: skillsOf(me.id).map((s, i) => ({ name: s.name, icon: s.icon, color: s.color, left: Math.max(0, this.ready[i] - this.now), cd: s.cd }))
		};
	}

	// ---------------- per frame ----------------
	update(dt: number, t: number) {
		this.now = t;
		const me = this.me;
		if (me) {
			if (this.phase === 'none') this.follow(me, dt, t);
			else this.fight(dt, t);
			// rest: health comes back slowly outside of battles
			if (this.phase === 'none' && me.hp < me.max) {
				me.hp = Math.min(me.max, me.hp + dt * me.max / 90);
				if (me.tired && me.hp > me.max * 0.5) {
					me.tired = false;
					me.body.rotation.set(0, 0, 0);
				}
			}
			this.flashAndKnock(me, dt);
		}
		if (this.wild) this.flashAndKnock(this.wild, dt);
		this.updateProjs(t);
		this.updateBits(dt, t);
		this.updateNums(t);
	}

	private follow(me: NonNullable<VoxelPlay['me']>, dt: number, t: number) {
		const p = this.h.player.position;
		const g = me.g;
		const ry = this.h.player.rotation.y;
		// a little behind and to the side of you
		const tx = p.x - Math.sin(ry) * 3 + Math.cos(ry) * 1.6;
		const tz = p.z - Math.cos(ry) * 3 - Math.sin(ry) * 1.6;
		const area = this.h.area();
		const dx = tx - g.position.x, dz = tz - g.position.z;
		const d = Math.hypot(dx, dz);
		if (area !== me.lastArea || d > 20) {
			// went through a door: it comes along
			me.lastArea = area;
			g.position.set(tx, this.h.groundY(tx, tz), tz);
			return;
		}
		const speed = Math.min(d * 3, me.tired ? 6 : 11);
		const moving = d > 0.4;
		if (moving) {
			g.position.x += (dx / d) * speed * dt;
			g.position.z += (dz / d) * speed * dt;
			this.face(g, dx, dz, dt * 8);
			me.hopT += dt * (me.tired ? 7 : 11);
		} else {
			me.hopT = 0;
			// idle: look at you
			this.face(g, p.x - g.position.x, p.z - g.position.z, dt * 3);
		}
		g.position.y = this.h.groundY(g.position.x, g.position.z) + (moving ? Math.abs(Math.sin(me.hopT)) * 0.45 : 0);
		const breathe = 1 + Math.sin(t * 2.4) * 0.025;
		me.body.scale.set(1 / breathe, breathe, 1 / breathe);
		// a tired Voxelmon drags its feet and leans
		me.body.rotation.z = me.tired ? 0.25 : 0;
	}

	private face(g: THREE.Object3D, dx: number, dz: number, k: number) {
		if (Math.hypot(dx, dz) < 0.01) return;
		const target = Math.atan2(dx, dz);
		let diff = target - g.rotation.y;
		diff = Math.atan2(Math.sin(diff), Math.cos(diff));
		g.rotation.y += diff * Math.min(1, k);
	}

	private fight(dt: number, t: number) {
		const me = this.me!, w = this.wild!;
		const since = t - this.phaseT;
		if (this.phase === 'intro') {
			// the wild one pops up out of the grass, yours runs into position
			const k = Math.min(1, since / 0.5);
			w.g.scale.setScalar(Math.max(0.01, k < 1 ? k * 1.15 : 1));
			w.g.position.y = w.home.y + Math.sin(Math.min(1, since / 0.7) * Math.PI) * 1.6;
			this.moveTo(me, me.home, dt, 14);
			this.face(w.g, me.g.position.x - w.g.position.x, me.g.position.z - w.g.position.z, 1);
			if (since > 1.1) {
				this.phase = 'fight';
				this.phaseT = t;
				w.nextAtk = t + 1.6;
			}
			return;
		}
		if (this.phase === 'end') {
			// after the result: yours hops happily (or lies down), then back to following
			if (since > 1.6) {
				this.phase = 'none';
				if (this.wild) this.h.scene.remove(this.wild.g);
				this.wild = null;
			} else if (this.wild && this.wild.hp > 0) {
				// fled / you lost: it hops away and shrinks
				const s = Math.max(0.01, 1 - since / 0.9);
				this.wild.g.scale.setScalar(s);
				this.wild.g.position.y = this.wild.home.y + Math.abs(Math.sin(since * 9)) * 0.8;
			} else if (!me.tired) {
				me.g.position.y = this.h.groundY(me.g.position.x, me.g.position.z) + Math.abs(Math.sin(since * 10)) * 0.7;
			}
			return;
		}
		// ----- fight -----
		this.runAct(me, w, t, dt, true);
		this.runAct(w, me, t, dt, false);
		if (!me.act) {
			this.moveTo(me, me.home, dt, 8);
			this.face(me.g, w.g.position.x - me.g.position.x, w.g.position.z - me.g.position.z, dt * 10);
			me.g.position.y = me.home.y + Math.abs(Math.sin(t * 5)) * 0.15;
		}
		if (!w.act) {
			this.moveTo(w, w.home, dt, 8);
			this.face(w.g, me.g.position.x - w.g.position.x, me.g.position.z - w.g.position.z, dt * 10);
			w.g.position.y = w.home.y + Math.abs(Math.sin(t * 4 + 1)) * 0.12;
			// the wild one attacks on its own
			if (t > w.nextAtk) {
				w.act = { kind: 'lunge', t0: t, done: false };
				this.h.emit({ type: 'skill', who: 'wild', id: w.id, skill: 'Lunge' });
			}
		}
	}

	private moveTo(f: Fighter, to: THREE.Vector3, dt: number, speed: number) {
		const g = f.g.position;
		const dx = to.x - g.x, dz = to.z - g.z;
		const d = Math.hypot(dx, dz);
		if (d < 0.05) return;
		const s = Math.min(d, speed * dt);
		g.x += (dx / d) * s;
		g.z += (dz / d) * s;
		g.y = this.h.groundY(g.x, g.z);
	}

	/** attack animations; damage lands at the right moment */
	private runAct(f: Fighter, foe: Fighter, t: number, dt: number, mine: boolean) {
		const a = f.act;
		if (!a) return;
		const age = t - a.t0;
		const g = f.g;
		const toFoe = tmpV.set(foe.g.position.x - g.position.x, 0, foe.g.position.z - g.position.z);
		const dist = toFoe.length();
		toFoe.normalize();
		this.face(g, toFoe.x, toFoe.z, dt * 12);
		const contact = (k: number) => {
			// k: 0 = at home, 1 = touching the foe
			const reach = Math.max(0, dist - 2.2);
			g.position.x = f.home.x + toFoe.x * reach * k;
			g.position.z = f.home.z + toFoe.z * reach * k;
		};
		const kind = a.kind;
		if (kind === 'dash' || kind === 'lunge') {
			const wind = kind === 'lunge' ? 0.55 : 0.18;
			if (age < wind) {
				// wind up: crouch and lean back (the wild one flashes red so you can see it coming)
				f.body.scale.set(1.12, 0.84, 1.12);
				contact(-0.12 * (age / wind));
				if (kind === 'lunge') f.mat.emissive.setRGB(0.45 * Math.abs(Math.sin(age * 20)), 0, 0);
			} else if (age < wind + 0.16) {
				f.body.scale.set(0.9, 1.15, 0.9);
				const k = (age - wind) / 0.16;
				contact(k);
				g.position.y = f.home.y + Math.sin(k * Math.PI) * 0.8;
			} else if (!a.done) {
				a.done = true;
				f.body.scale.set(1, 1, 1);
				if (kind === 'lunge') f.mat.emissive.copy(f.base);
				const dmg = kind === 'lunge' ? this.wildDmg() : a.skill!.power;
				this.damage(foe, dmg, mine, a.skill?.color ?? '#ffffff');
			} else if (age < wind + 0.5) {
				contact(1 - (age - wind - 0.16) / 0.34);
			} else this.finishAct(f, mine, t);
			return;
		}
		const sk = a.skill!;
		// ranged attacks: a short cast, then the effect flies on its own
		if (!a.done) {
			f.body.scale.set(1.08, 0.9, 1.08);
			if (age > 0.2) {
				a.done = true;
				f.body.scale.set(1, 1, 1);
				this.cast(f, foe, sk, mine);
			}
		} else if (age > 0.6) this.finishAct(f, mine, t);
	}
	private finishAct(f: Fighter, mine: boolean, t: number) {
		f.act = null;
		f.body.scale.set(1, 1, 1);
		if (!mine && this.wild) this.wild.nextAtk = t + 2.2 + Math.random() * 1.4;
	}
	private wildDmg() {
		const w = this.wild!;
		return Math.round((6 + CARD_MAP[w.id].cost * 1.6) * (w.shiny ? 1.25 : 1) * (0.85 + Math.random() * 0.3));
	}

	private cast(f: Fighter, foe: Fighter, sk: Skill, mine: boolean) {
		const from = f.g.position.clone().add(new THREE.Vector3(0, 2, 0));
		const target = () => foe.g.position.clone().add(new THREE.Vector3(0, 1.4, 0));
		const mat = (op = 1) => new THREE.MeshStandardMaterial({ color: sk.color, emissive: sk.color, emissiveIntensity: 0.6, transparent: op < 1, opacity: op });
		if (sk.kind === 'volley') {
			for (let i = 0; i < 5; i++) {
				const m = new THREE.Mesh(this.cube, mat());
				m.scale.setScalar(0.45);
				m.visible = false;
				this.h.scene.add(m);
				this.projs.push({ m, from: from.clone().add(new THREE.Vector3((Math.random() - 0.5) * 1.2, Math.random(), (Math.random() - 0.5) * 1.2)), to: target, t0: this.now + i * 0.09, dur: 0.45, dmg: sk.power / 5, arc: 1.5 + Math.random(), kind: 'cube' });
			}
		} else if (sk.kind === 'wave') {
			const m = new THREE.Mesh(new THREE.TorusGeometry(1, 0.22, 6, 24), mat(0.85));
			m.rotation.x = Math.PI / 2;
			this.h.scene.add(m);
			this.projs.push({ m, from: f.g.position.clone().setY(f.g.position.y + 0.5), to: () => foe.g.position.clone().setY(foe.g.position.y + 0.5), t0: this.now, dur: 0.55, dmg: sk.power, arc: 0, kind: 'ring' });
		} else if (sk.kind === 'strike') {
			// a marker under the foe, then a lightning bolt from the sky
			const mark = new THREE.Mesh(new THREE.RingGeometry(1.2, 1.6, 24), new THREE.MeshBasicMaterial({ color: sk.color, transparent: true, opacity: 0.8, side: THREE.DoubleSide }));
			mark.rotation.x = -Math.PI / 2;
			this.h.scene.add(mark);
			this.projs.push({ m: mark, from: foe.g.position.clone(), to: () => foe.g.position.clone(), t0: this.now, dur: 0.5, dmg: 0, arc: 0, kind: 'mark' });
			const bolt = new THREE.Mesh(new THREE.BoxGeometry(0.5, 14, 0.5), new THREE.MeshBasicMaterial({ color: '#fffbd0' }));
			bolt.visible = false;
			this.h.scene.add(bolt);
			this.projs.push({ m: bolt, from: foe.g.position.clone(), to: () => foe.g.position.clone(), t0: this.now + 0.5, dur: 0.25, dmg: sk.power, arc: 0, kind: 'bolt' });
		}
		void mine;
	}

	private updateProjs(t: number) {
		if (!this.projs.length) return;
		this.projs = this.projs.filter((pr) => {
			const k = (t - pr.t0) / pr.dur;
			if (k < 0) return true;
			pr.m.visible = true;
			const to = pr.to();
			if (pr.kind === 'cube') {
				pr.m.position.lerpVectors(pr.from, to, Math.min(1, k));
				pr.m.position.y += Math.sin(Math.min(1, k) * Math.PI) * pr.arc;
				pr.m.rotation.x = pr.m.rotation.y = k * 9;
			} else if (pr.kind === 'ring') {
				pr.m.position.lerpVectors(pr.from, to, Math.min(1, k));
				pr.m.scale.setScalar(1 + k * 1.6);
			} else if (pr.kind === 'mark') {
				pr.m.position.set(to.x, to.y + 0.08, to.z);
				pr.m.scale.setScalar(1.3 - Math.min(1, k) * 0.5);
				(pr.m.material as THREE.MeshBasicMaterial).opacity = 0.5 + 0.5 * Math.abs(Math.sin(k * 18));
			} else if (pr.kind === 'bolt') {
				pr.m.position.set(to.x, to.y + 7, to.z);
				pr.m.scale.x = pr.m.scale.z = 1 + Math.random();
			}
			if (k >= 1) {
				this.h.scene.remove(pr.m);
				if (pr.kind === 'ring' || pr.kind === 'bolt' || pr.kind === 'mark') (pr.m.geometry as THREE.BufferGeometry).dispose();
				(pr.m.material as THREE.Material).dispose();
				if (pr.dmg > 0 && this.wild && this.me && this.phase === 'fight') {
					// all ranged attacks are yours (the wild ones only lunge)
					this.damage(this.wild, pr.dmg, true, (pr.m.material as THREE.MeshStandardMaterial).color?.getStyle?.() ?? '#fff');
				}
				return false;
			}
			return true;
		});
	}

	private damage(f: Fighter, dmg: number, byMe: boolean, color: string) {
		if (this.phase !== 'fight') return;
		dmg = Math.max(1, Math.round(dmg));
		f.hp -= dmg;
		f.flash = 0.14;
		const away = tmpV.set(f.g.position.x - (byMe ? this.me! : this.wild!).g.position.x, 0, f.g.position.z - (byMe ? this.me! : this.wild!).g.position.z).normalize();
		f.knock.copy(away).multiplyScalar(0.6);
		this.burst(f.g.position.clone().add(new THREE.Vector3(0, 1.5, 0)), color, 7);
		this.number(f.g.position.clone().add(new THREE.Vector3(0, 4.6, 0)), dmg, byMe);
		this.h.emit({ type: 'hit', who: byMe ? 'wild' : 'me', dmg });
		if (f.hp <= 0) {
			f.hp = 0;
			this.endFight(byMe ? 'win' : 'lose');
		}
	}

	private endFight(r: 'win' | 'lose' | 'fled') {
		const me = this.me!, w = this.wild!;
		this.phase = 'end';
		this.phaseT = this.now;
		me.act = null;
		w.act = null;
		w.mat.emissive.copy(w.base);
		this.projs.forEach((pr) => this.h.scene.remove(pr.m));
		this.projs = [];
		if (r === 'win') {
			// the wild one bursts into its own bricks
			this.burst(w.g.position.clone().add(new THREE.Vector3(0, 1.4, 0)), w.shiny ? '#ffd23d' : '#ffffff', 26, w.g);
			w.g.visible = false;
			this.h.emit({ type: 'win', wild: w.id, shiny: w.shiny });
		} else if (r === 'lose') {
			me.tired = true;
			me.body.rotation.z = Math.PI / 2.4; // knocked over
			this.h.emit({ type: 'lose', wild: w.id });
		} else this.h.emit({ type: 'fled' });
	}

	private flashAndKnock(f: Fighter, dt: number) {
		if (f.flash > 0) {
			f.flash -= dt;
			if (f.flash > 0) f.mat.emissive.setRGB(0.9, 0.9, 0.9);
			else f.mat.emissive.copy(f.base);
		}
		if (f.knock.lengthSq() > 0.0001) {
			f.g.position.addScaledVector(f.knock, dt * 8);
			f.knock.multiplyScalar(Math.max(0, 1 - dt * 8));
		}
	}

	// ---------------- little effects ----------------
	/** cubes flying apart (hits, and a beaten wild Voxelmon falling to pieces) */
	private burst(at: THREE.Vector3, color: string, n: number, src?: THREE.Group) {
		// colours from the beast itself when it breaks apart
		const cols: THREE.Color[] = [];
		const geo = src ? ((src.userData.body as THREE.Group).children[0] as THREE.Mesh).geometry : null;
		const ca = geo?.getAttribute('color');
		for (let i = 0; i < n; i++) {
			const c = new THREE.Color(color);
			if (ca) c.fromBufferAttribute(ca, Math.floor(Math.random() * ca.count));
			cols.push(c);
		}
		for (let i = 0; i < n; i++) {
			const m = new THREE.Mesh(this.cube, new THREE.MeshStandardMaterial({ color: cols[i], transparent: true }));
			m.scale.setScalar(src ? 0.35 + Math.random() * 0.3 : 0.2 + Math.random() * 0.15);
			m.position.copy(at);
			this.h.scene.add(m);
			const v = new THREE.Vector3(Math.random() - 0.5, 0.6 + Math.random(), Math.random() - 0.5).normalize().multiplyScalar((src ? 7 : 5) + Math.random() * 4);
			this.bits.push({ m, v, t0: this.now, life: src ? 1.4 : 0.7 });
		}
	}
	private updateBits(dt: number, t: number) {
		if (!this.bits.length) return;
		this.bits = this.bits.filter((b) => {
			const age = t - b.t0;
			if (age > b.life) {
				this.h.scene.remove(b.m);
				(b.m.material as THREE.Material).dispose();
				return false;
			}
			b.v.y -= 22 * dt;
			b.m.position.addScaledVector(b.v, dt);
			const gy = this.h.groundY(b.m.position.x, b.m.position.z) + 0.1;
			if (b.m.position.y < gy) {
				b.m.position.y = gy;
				b.v.y *= -0.35;
				b.v.x *= 0.6;
				b.v.z *= 0.6;
			}
			b.m.rotation.x += dt * 8;
			b.m.rotation.z += dt * 6;
			(b.m.material as THREE.MeshStandardMaterial).opacity = Math.min(1, (b.life - age) / 0.3);
			return true;
		});
	}
	/** a damage number popping up over the one who got hit */
	private number(at: THREE.Vector3, n: number, byMe: boolean) {
		const c = document.createElement('canvas');
		c.width = 128;
		c.height = 64;
		const g = c.getContext('2d')!;
		g.font = '900 48px Fredoka, Arial, sans-serif';
		g.textAlign = 'center';
		g.textBaseline = 'middle';
		g.lineWidth = 8;
		g.strokeStyle = '#1b2a34';
		g.strokeText(`-${n}`, 64, 34);
		g.fillStyle = byMe ? '#ffd23d' : '#ff5a4a';
		g.fillText(`-${n}`, 64, 34);
		const tex = new THREE.CanvasTexture(c);
		tex.colorSpace = THREE.SRGBColorSpace;
		const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, transparent: true, depthTest: false }));
		sp.scale.set(2.4, 1.2, 1);
		sp.renderOrder = 9;
		this.h.scene.add(sp);
		this.nums.push({ sp, t0: this.now, base: at.clone().add(new THREE.Vector3((Math.random() - 0.5) * 0.8, 0, 0)) });
	}
	private updateNums(t: number) {
		if (!this.nums.length) return;
		this.nums = this.nums.filter((n) => {
			const age = t - n.t0;
			if (age > 0.9) {
				this.h.scene.remove(n.sp);
				n.sp.material.map?.dispose();
				n.sp.material.dispose();
				return false;
			}
			n.sp.position.copy(n.base).addScaledVector(UP, age * 1.8);
			n.sp.material.opacity = Math.min(1, (0.9 - age) / 0.3);
			n.sp.scale.set(2.4, 1.2, 1).multiplyScalar(age < 0.1 ? 0.6 + age * 6 : 1);
			return true;
		});
	}
}
