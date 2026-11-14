import { assetUrl } from '../asset-url';
import * as THREE from 'three';
import { VoxelPlay, type VoxEvent } from './voxelmon';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import { RoomEnvironment } from 'three/addons/environments/RoomEnvironment.js';
import { Reflector } from 'three/addons/objects/Reflector.js';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import { Minifig, type FigStyle } from './minifig';
import { buildInteriors, ROOMS, roomById, HOME_STAIRS, BED_WELL, type AreaId } from './interiors';
import { loadModel } from './models';
import { ASSET_VERSION } from './assets-version';

// ---------- LEGO-ish palette ----------
export const C = {
	green: '#4b9f4a',
	darkGreen: '#237841',
	pine: '#1d6630',
	gray: '#a0a5a9',
	darkGray: '#6c6e68',
	red: '#c91a09',
	blue: '#0055bf',
	orange: '#fe8a18',
	tan: '#e4cd9e',
	white: '#f4f4f4',
	brown: '#583927',
	redBrown: '#82422a',
	yellow: '#f2cd37',
	glass: '#9fd8ef',
	water: '#2a7fd4',
	black: '#1b2a34'
};

type Rect = { x0: number; x1: number; z0: number; z1: number };
const inRect = (r: Rect, x: number, z: number) => x >= r.x0 && x <= r.x1 && z >= r.z0 && z <= r.z1;

export interface NpcDef {
	id: string;
	name: string;
	style: FigStyle;
	x: number;
	z: number;
	rot: number;
}

export interface TownCallbacks {
	onNearby: (id: string | null) => void;
	onInteract: (id: string) => void;
	onGrass: () => void;
	onBlockedGrass: () => void;
	/** tried to leave for Route 1 while Prof. Quill still blocks the way */
	onBlockedRoute?: () => void;
	/** graphics quality changed (auto = picked by the frame-rate watch) */
	onQuality?: (q: 'high' | 'medium' | 'low', auto: boolean) => void;
	/** swipe controls: the figure started / stopped walking by itself */
	onAutoWalk?: (on: boolean) => void;
	/** walked into another part of the outside world (Brickhollow / Route 1 / Stonecrest) */
	onZone?: (label: string) => void;
	/** Voxelmon battle events (see voxelmon.ts) */
	onVox?: (e: VoxEvent) => void;
	onArea: (id: AreaId, label: string) => void;
	onGolden: (id: string) => void;
	/** walked into a butterfly: it flies off */
	onButterfly?: () => void;
	/** the player started / stopped dancing (Q); `crowd` = villagers dancing along */
	onDance?: (on: boolean, crowd: number) => void;
	/** a treasure chest appeared somewhere in the village */
	onChestSpawn?: () => void;
	/** hide and seek with Pip: a hot/cold hint, found him, or time ran out */
	onHideSeek?: (ev: 'hint' | 'found' | 'timeout', text?: string) => void;
	/** caught a firefly at night */
	onFirefly?: () => void;
	/** stepped into a (new) puddle */
	onPuddle?: () => void;
	/** a duck quacks (pitch: 1 = mother, higher = duckling) */
	onQuack?: (pitch: number) => void;
	/** sat down on / got up from a bench */
	onSit?: (seated: boolean) => void;
	/** the ball went into the little park goal */
	onGoal?: () => void;
	/** stepped into a puddle */
	onSplash?: (loud: boolean) => void;
	/** the pond fish jumps (one sound covers leaving and falling back into the water) */
	onFishJump?: () => void;
	/** tag mini-game with Pip */
	onTag?: (event: 'start' | 'caught' | 'escaped') => void;
	/** the park ball was kicked (0..1 strength) */
	onKick?: (strength: number) => void;
	/** rain shower strength changed (0..1, outside only) */
	onRain?: (level: number) => void;
	onDebugPick?: (id: string) => void;
	onStep?: (surface: 'road' | 'grass' | 'wood') => void;
	/** sign close-up started (id) or ended (null) */
	onReading?: (id: string | null) => void;
}

export const PLAYER_STYLE: FigStyle = {
	torso: '#1b9e8f', legs: C.darkGray, hair: C.brown, hairStyle: 'cap', cap: C.orange, decal: 'zip', decalColor: C.white
};

export const NPCS: NpcDef[] = [
	{
		id: 'mom', name: 'Mom', x: 193.4, z: -4.4, rot: 0,
		style: { torso: '#a0bcac', legs: C.brown, hair: C.redBrown, hairStyle: 'ponytail', decal: 'apron', decalColor: C.white, female: true }
	},
	{
		id: 'prof', name: 'Prof. Hazel Quill', x: 337, z: -1.2, rot: -Math.PI / 2,
		style: { torso: '#3d6fd1', legs: C.tan, hair: '#c8c8c8', hairStyle: 'bob', coat: true, decalColor: C.red, female: true }
	},
	{
		id: 'rival', name: 'Dex', x: 52.5, z: -3.4, rot: -Math.PI / 2,
		style: { torso: '#7b2fbe', legs: C.black, hair: '#e8e8e8', hairStyle: 'spiky', decal: 'stripe', decalColor: C.yellow }
	},
	{
		id: 'sister', name: 'Lumi', x: 254.5, z: 2, rot: Math.PI / 2,
		style: { torso: '#ff9ecd', legs: C.blue, hair: C.yellow, hairStyle: 'bob', decal: 'star', decalColor: C.white, female: true }
	},
	// Stonecrest
	{
		id: 'knight', name: 'Sir Bricksworth', x: 7, z: -146.2, rot: 0,
		style: { torso: C.gray, legs: C.darkGray, hair: C.darkGray, hairStyle: 'cap', cap: C.gray, decal: 'star', decalColor: C.yellow }
	},
	{
		id: 'baker', name: 'Marla', x: -8, z: -110.6, rot: 0,
		style: { torso: C.white, legs: C.blue, hair: C.redBrown, hairStyle: 'bob', decal: 'apron', decalColor: C.orange, female: true }
	},
	{
		id: 'kid', name: 'Pip', x: -8, z: 13.2, rot: 0,
		style: { torso: C.yellow, legs: C.blue, hair: C.brown, hairStyle: 'short', decal: 'stripe', decalColor: C.red }
	}
];

// you wake up next to your bed, upstairs
const PLAYER_START = new THREE.Vector3(197.4, 0.4, 41.2);

// walkable zones (union) and blockers
const WALK: Rect[] = [
	{ x0: -26.2, x1: 66.2, z0: -25.6, z1: 23.4 },
	{ x0: -3.6, x1: 3.6, z0: -30, z1: -25 },
	{ x0: -8.4, x1: 8.4, z0: -41.6, z1: -28.6 },
	{ x0: -20, x1: 20, z0: -112, z1: -41.6 }, // Route 1 through the forest
	{ x0: -42, x1: 48, z0: -190, z1: -111 }, // Stonecrest
	...ROOMS.map((r) => r.walk)
];
const GRASS: Rect = { x0: -8.6, x1: 8.6, z0: -42, z1: -28.2 };
// ---------- north of the village: Route 1 runs through the forest to the next town, Stonecrest ----------
/** the ground plate of the route and the town (the village plate ends at z -47) */
const NORTH_PLATE: Rect = { x0: -46, x1: 56, z0: -194, z1: -47 };
/** a creek across the route; only the bridge crosses it */
const CREEK: Rect = { x0: -46, x1: 56, z0: -75, z1: -71 };
const BRIDGE: Rect = { x0: -3.4, x1: 3.4, z0: -76.2, z1: -69.8 };
/** the dirt path of the route */
const TRAIL: Rect = { x0: -2.5, x1: 2.5, z0: -111, z1: -42 };
/** all tall grass: the first patch by the village and three more along the route */
const GRASSES: Rect[] = [GRASS, { x0: -17, x1: -4.5, z0: -64, z1: -50 }, { x0: 4.5, x1: 18, z0: -92, z1: -80 }, { x0: -16, x1: -4.5, z0: -106, z1: -94 }];
const inGrass = (x: number, z: number) => GRASSES.some((g) => inRect(g, x, z));
/** Stonecrest starts north of here */
const STONECREST_Z = -111;
/** the castle at the north end of Stonecrest (model from the "lego city" asset, see scripts/voxelize-castle.mjs) */
const CASTLE = { x: 0, z: -166, scale: 1.6 };
const POND: Rect = { x0: -12, x1: -4, z0: 15, z1: 22 };
/** the way north out of the village: closed by Prof. Quill until you have a Voxelmon */
const EXIT_Z = -25.3;
const GATE_PROF = { x: 0, z: -24.4 };

/** Hidden golden studs to collect (some are inside the houses). */
export const GOLDEN_STUDS: { id: string; x: number; z: number }[] = [
	{ id: 'g-nw', x: -22, z: -24 },
	{ id: 'g-pond', x: -14, z: 22.5 },
	{ id: 'g-garden', x: -23, z: 12 },
	{ id: 'g-park', x: 15, z: -12 },
	{ id: 'g-route1', x: 0, z: -40 },
	{ id: 'g-route2', x: -7.5, z: -30 },
	{ id: 'g-route3', x: 6.5, z: -36 },
	{ id: 'g-gap', x: 24.5, z: -14 },
	{ id: 'g-behind', x: 64.5, z: -20 },
	{ id: 'g-yard', x: 60, z: 22.8 },
	{ id: 'g-lamp', x: -25, z: 0.5 },
	{ id: 'g-home', x: 191, z: 6 },
	{ id: 'g-dex', x: 256, z: -5 },
	{ id: 'g-lab', x: 342, z: 8.5 },
	{ id: 'g-bedroom', x: 194.5, z: 44.6 }
];

/** Things you can read with E / tap (signs, mailboxes). */
export const READABLES: { id: string; label: string; x: number; z: number }[] = [
	{ id: 'sign:village', label: 'Village sign', x: 5.9, z: 21.2 },
	{ id: 'sign:route', label: 'Route sign', x: 5, z: -23.2 },
	{ id: 'sign:north', label: 'Route sign', x: 5.5, z: -48 },
	{ id: 'sign:stonecrest', label: 'Town sign', x: 5.5, z: -107.4 },
	{ id: 'sign:garden', label: 'Garden sign', x: -20.6, z: 8.9 },
	{ id: 'sign:lab', label: 'Lab sign', x: 13, z: 19.6 },
	{ id: 'mail:home', label: 'Mailbox', x: -18.2, z: -2.4 },
	{ id: 'bench:park', label: 'the park bench', x: 15, z: -7.7 },
	{ id: 'pond:ducks', label: 'the ducks', x: -2.9, z: 18.5 },
	{ id: 'tv:home', label: 'TV', x: 205, z: -5.2 },
	// Dex & Lumi's house: the gaming TV and Lumi's piano (see interiors.ts)
	{ id: 'tv:dex', label: 'the TV', x: 269, z: -3.4 },
	{ id: 'piano:dex', label: 'the piano', x: 251, z: -3.4 },
	{ id: 'pc:bedroom', label: 'your PC', x: 201.4, z: 36.6 },
	{ id: 'bed:home', label: 'your bed', x: 196.2, z: 38.6 },
	// the LEGO computers on the lab desks (see labComputers / interiors.ts)
	{ id: 'pc:lab1', label: 'the lab computer', x: 343.2, z: -2.2 },
	{ id: 'pc:lab2', label: 'the lab computer', x: 343.2, z: 3.3 }
];

/** Text painted on the signs; read by walking up and zooming in (no menu). */
export const SIGNS: Record<string, { title: string; lines: string[]; note?: string; bg: string; fg: string }> = {
	'sign:village': { title: 'BRICKHOLLOW', lines: ['"A village where every brick has a story."', 'Population: 6 minifigs, lots of Beasts, countless studs.'], bg: '#f4f4f4', fg: '#1b2a34' },
	'sign:route': { title: 'ROUTE 1 ▲', lines: ['Tall grass ahead!', 'Wild Brick Beasts love to hide in it. Keep your deck ready.'], bg: '#fff6c8', fg: '#6b3f1d' },
	'sign:north': { title: 'STONECREST ▲', lines: ['Route 1 goes on through the forest.', 'Mind the creek, use the bridge!'], bg: '#fff6c8', fg: '#6b3f1d' },
	'sign:stonecrest': { title: 'STONECREST', lines: ['"The town below the old castle."', 'Bakery · Inn · Market · Smith'], bg: '#f4f4f4', fg: '#6c6e68' },
	'sign:garden': { title: 'GARDEN', lines: ['Please do not step on the flowers.'], note: 'Pip, that means YOU.', bg: '#e4cd9e', fg: '#6b3f1d' },
	'sign:lab': { title: 'QUILL LAB', lines: ['Brick Beast research since 1978.', 'Opening hours: whenever the Professor remembers to unlock the door.'], bg: '#f4f4f4', fg: '#1e5aa8' }
};

/** a random element */
const pick = <T,>(a: readonly T[]): T => a[Math.floor(Math.random() * a.length)];

export class Town {
	renderer: THREE.WebGLRenderer;
	scene = new THREE.Scene();
	camera: THREE.PerspectiveCamera;
	player: Minifig;
	npcs: { def: NpcDef; fig: Minifig; baseRot: number }[] = [];
	paused = false;
	zoom = 1.15;
	hasDeck = () => false;
	private tmpV2 = new THREE.Vector3();
	/** the part of the outside world the player is in */
	private zone = '';
	/** may the player leave the village for Route 1? (Prof. Quill stands in the way until then) */
	routeOpen = () => true;
	/** your Voxelmon and the real-time battles in the tall grass */
	vox!: VoxelPlay;
	/** Prof. Quill at the village exit while the route is closed */
	private gateProf: Minifig | null = null;

	private cb: TownCallbacks;
	private container: HTMLElement;
	private clock = new THREE.Clock();
	private raf = 0;
	private keys = new Set<string>();
	private virt = { x: 0, z: 0 };
	blockers: Rect[] = [];
	area: AreaId = 'bedroom';
	bushSpots: { x: number; y: number; z: number; s: number }[] = [];
	bambooSpots: { x: number; y: number; z: number; s: number }[] = [];
	flowerSpots: { x: number; y: number; z: number; s: number; r?: number }[] = [];
	flowerPlants: { x: number; z: number; c: string; y?: number }[] = [];
	/** LEGO flower pot (4x4 studs); outdoor pots also block walking */
	flowerBunchSpots: { x: number; y: number; z: number; s: number; r?: number }[] = [];
	/** LEGO computers on the lab desks */
	labComputers: { x: number; y: number; z: number; s: number }[] = [];
	flowerPot(x: number, z: number, y = this.groundY(x, z), block = true) {
		this.flowerSpots.push({ x, y, z, s: 1, r: Math.random() * Math.PI * 2 });
		if (block) this.blockers.push({ x0: x - 1.8, x1: x + 1.8, z0: z - 1.8, z1: z + 1.8 });
	}
	fishes: THREE.Object3D[] = [];
	spinners: THREE.Object3D[] = [];
	private blinkers: THREE.MeshStandardMaterial[] = [];
	private spruceSpots: { x: number; z: number; s: number; r: number }[] = [];
	private roundSpots: { x: number; z: number; s: number; r: number }[] = [];
	private doorLock = 0;
	private matCache = new Map<string, THREE.MeshStandardMaterial>();
	private geoCache = new Map<string, THREE.BufferGeometry>();
	private studs: { x: number; y: number; z: number; c: string }[] = [];
	private sun!: THREE.DirectionalLight;
	private grass!: THREE.InstancedMesh;
	private grassData: { x: number; z: number; s: number; r: number; ph: number }[] = [];
	private lily: THREE.Object3D[] = [];
	private clouds: THREE.Object3D[] = [];
	private nearby: string | null = null;
	private grassMeter = 0;
	private lastBlockedMsg = 0;
	private camPos = new THREE.Vector3();
	private ro: ResizeObserver;

	constructor(container: HTMLElement, cb: TownCallbacks) {
		this.container = container;
		this.cb = cb;
		const r = new THREE.WebGLRenderer({ antialias: true, powerPreference: 'high-performance' });
		const touch = matchMedia('(pointer: coarse)').matches;
		r.setPixelRatio(Math.min(window.devicePixelRatio, touch ? 1.5 : 2));
		r.shadowMap.enabled = true;
		r.shadowMap.type = THREE.PCFSoftShadowMap;
		r.toneMapping = THREE.ACESFilmicToneMapping;
		r.toneMappingExposure = 1.05;
		r.outputColorSpace = THREE.SRGBColorSpace;
		container.appendChild(r.domElement);
		this.renderer = r;

		this.camera = new THREE.PerspectiveCamera(38, 1, 0.5, 400);
		const pmrem = new THREE.PMREMGenerator(r);
		this.scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
		this.scene.environmentIntensity = 0.55;
		this.scene.background = new THREE.Color('#bfe6ff');
		this.scene.fog = new THREE.Fog('#bfe6ff', 90, 190);

		this.section('lights', () => this.buildLights());
		this.section('ground', () => this.buildGround());
		this.section('trees', () => this.buildTrees());
		this.section('house:home', () => this.buildHouse(-15, -11, 12, 10, C.blue, C.white, C.redBrown, 'HOME'));
		this.placeLegoHouse(47, -10, 33);
		this.section('park', () => this.buildPark(15, -11));
		// loose LEGO brick piles (models at true stud scale)
		this.brickPile(0, 45, 16.5, C.red);
		this.brickPile(2, 57.5, 16.5, C.yellow);
		this.brickPile(3, 33.5, 17, C.blue);
		this.brickPile(1, -17, -21.5, C.orange);
		this.section('lab', () => this.buildLab(13, 11, 16, 10));
		this.section('garden', () => this.buildGarden());
		this.section('pond', () => this.buildPond());
		this.section('decor', () => this.buildDecor());
		this.section('north', () => this.buildNorth());
		this.section('tallgrass', () => this.buildTallGrass());
		this.section('golden', () => this.buildGolden());
		this.section('clouds', () => this.buildClouds());
		this.buildButterflies();
		this.buildBall();
		this.buildChest();
		this.buildGoal();
		this.buildRain();
		this.buildPuddles();
		this.buildFireflies();
		this.buildBeacon();
		this.blinkers = buildInteriors(this);
		this.groundStuds();
		this.section('studs', () => this.buildStuds());
		this.loadModels();

		this.player = new Minifig(PLAYER_STYLE);
		this.nameTree(this.player.group, 'player');
		this.player.group.position.copy(PLAYER_START);
		this.player.group.rotation.y = Math.PI;
		this.scene.add(this.player.group);
		for (const def of NPCS) {
			const fig = new Minifig(def.style);
			this.nameTree(fig.group, `npc:${def.id}`);
			fig.group.position.set(def.x, this.groundY(def.x, def.z), def.z);
			fig.group.rotation.y = def.rot;
			this.scene.add(fig.group);
			this.npcs.push({ def, fig, baseRot: def.rot });
		}
		{
			// Prof. Quill guarding the way north (only while the route is closed)
			const def = NPCS.find((n) => n.id === 'prof')!;
			const fig = new Minifig(def.style);
			this.nameTree(fig.group, 'npc:gate-prof');
			fig.group.position.set(GATE_PROF.x, this.groundY(GATE_PROF.x, GATE_PROF.z), GATE_PROF.z);
			this.scene.add(fig.group);
			this.gateProf = fig;
		}
		this.vox = new VoxelPlay({
			scene: this.scene,
			player: this.player.group,
			area: () => this.area,
			groundY: (x, z) => this.groundY(x, z),
			clampToGrass: (x, z, mx = 2) => {
				// the patch you are standing in
				const p = this.player.group.position;
				const g = GRASSES.find((r) => inRect(r, p.x, p.z)) ?? GRASSES.find((r) => inRect(r, x, z)) ?? GRASS;
				const cx = (g.x0 + g.x1) / 2, cz = (g.z0 + g.z1) / 2;
				const hx = Math.max(0, (g.x1 - g.x0) / 2 - mx), hz = Math.max(0, (g.z1 - g.z0) / 2 - 2.5);
				return { x: THREE.MathUtils.clamp(x, cx - hx, cx + hx), z: THREE.MathUtils.clamp(z, cz - hz, cz + hz) };
			},
			emit: (e) => this.cb.onVox?.(e)
		});
		this.applyArea('bedroom', false);

		window.addEventListener('keydown', this.onKeyDown);
		window.addEventListener('keyup', this.onKeyUp);
		container.addEventListener('wheel', this.onWheel, { passive: false });
		this.renderer.domElement.addEventListener('pointerdown', this.onPick);
		this.ro = new ResizeObserver(() => this.resize());
		this.ro.observe(container);
		this.resize();
		this.loop();
	}

	// ---------- helpers ----------
	mat(color: string, extra?: Partial<THREE.MeshStandardMaterialParameters>) {
		const key = color + (extra ? JSON.stringify(extra) : '');
		let m = this.matCache.get(key);
		if (!m) {
			m = new THREE.MeshStandardMaterial({ color, roughness: 0.28, metalness: 0, ...extra });
			this.matCache.set(key, m);
		}
		return m;
	}
	rbox(w: number, h: number, d: number) {
		const key = `${w.toFixed(2)}|${h.toFixed(2)}|${d.toFixed(2)}`;
		let g = this.geoCache.get(key);
		if (!g) {
			g = new RoundedBoxGeometry(w, h, d, 2, Math.min(0.07, w / 4, h / 4, d / 4));
			this.geoCache.set(key, g);
		}
		return g;
	}
	/** Adds a rounded brick; y is the bottom. */
	box(
		w: number, h: number, d: number, color: string | THREE.Material,
		x: number, y: number, z: number,
		opts: { studs?: boolean; parent?: THREE.Object3D; shadow?: boolean } = {}
	) {
		const mat = typeof color === 'string' ? this.mat(color) : color;
		const mesh = new THREE.Mesh(this.rbox(w, h, d), mat);
		mesh.position.set(x, y + h / 2, z);
		mesh.castShadow = opts.shadow !== false;
		mesh.receiveShadow = true;
		(opts.parent ?? this.scene).add(mesh);
		if (opts.studs && typeof color === 'string') this.studRect(x - w / 2, x + w / 2, z - d / 2, z + d / 2, y + h, color);
		return mesh;
	}
	/** Studs on top of a brick: one per full stud of its size, centered on the brick like real LEGO. */
	private studRect(x0: number, x1: number, z0: number, z1: number, y: number, c: string) {
		const nx = Math.max(1, Math.floor(x1 - x0 + 0.05));
		const nz = Math.max(1, Math.floor(z1 - z0 + 0.05));
		// a brick narrower than one stud still gets a single centered stud only if it can hold it
		if (x1 - x0 < 0.7 || z1 - z0 < 0.7) return;
		const ox = x0 + (x1 - x0 - nx) / 2 + 0.5;
		const oz = z0 + (z1 - z0 - nz) / 2 + 0.5;
		for (let i = 0; i < nx; i++) for (let k = 0; k < nz; k++) this.studs.push({ x: ox + i, y, z: oz + k, c });
	}
	private brickTexture(base: string, w: number, h: number) {
		const c = document.createElement('canvas');
		c.width = 128;
		c.height = 77;
		const g = c.getContext('2d')!;
		g.fillStyle = base;
		g.fillRect(0, 0, 128, 77);
		g.strokeStyle = 'rgba(0,0,0,0.13)';
		g.lineWidth = 2;
		g.strokeRect(1, 1, 126, 37);
		g.beginPath();
		g.moveTo(0, 39);
		g.lineTo(128, 39);
		g.moveTo(64, 39);
		g.lineTo(64, 77);
		g.moveTo(0, 76);
		g.lineTo(128, 76);
		g.stroke();
		g.fillStyle = 'rgba(255,255,255,0.12)';
		g.fillRect(2, 3, 124, 5);
		g.fillRect(2, 41, 124, 5);
		const t = new THREE.CanvasTexture(c);
		t.colorSpace = THREE.SRGBColorSpace;
		t.wrapS = t.wrapT = THREE.RepeatWrapping;
		t.repeat.set(w / 4, h / 2.4);
		t.anisotropy = 4;
		return t;
	}
	signTexture(text: string, bg = '#f4f4f4', fg = '#1b2a34') {
		const c = document.createElement('canvas');
		c.width = 512;
		c.height = 128;
		const g = c.getContext('2d')!;
		g.fillStyle = bg;
		g.fillRect(0, 0, 512, 128);
		g.fillStyle = fg;
		g.font = 'bold 64px "Segoe UI", Arial, sans-serif';
		g.textAlign = 'center';
		g.textBaseline = 'middle';
		g.fillText(text, 256, 68);
		const t = new THREE.CanvasTexture(c);
		t.colorSpace = THREE.SRGBColorSpace;
		return t;
	}
	/** sign board with a big title, wrapped body text and an optional scribbled note */
	boardTexture(id: string, w: number, h: number) {
		const d = SIGNS[id];
		const W = 1024, H = Math.round((W * h) / w);
		const c = document.createElement('canvas');
		c.width = W;
		c.height = H;
		const g = c.getContext('2d')!;
		g.fillStyle = d.bg;
		g.fillRect(0, 0, W, H);
		g.strokeStyle = d.fg;
		g.globalAlpha = 0.85;
		g.lineWidth = 10;
		g.strokeRect(22, 22, W - 44, H - 44);
		g.globalAlpha = 1;
		g.fillStyle = d.fg;
		g.textAlign = 'center';
		g.textBaseline = 'alphabetic';
		const titlePx = Math.min(130, H * 0.27);
		g.font = `900 ${titlePx}px "Segoe UI", Arial, sans-serif`;
		let y = 40 + titlePx;
		g.fillText(d.title, W / 2, y);
		y += 22;
		g.fillRect(W / 2 - 200, y, 400, 6);
		const bodyPx = Math.min(48, H * 0.095);
		g.font = `600 ${bodyPx}px "Segoe UI", Arial, sans-serif`;
		y += 18;
		for (const line of d.lines) {
			const words = line.split(' ');
			let cur = '';
			const rows: string[] = [];
			for (const word of words) {
				const test = cur ? cur + ' ' + word : word;
				if (g.measureText(test).width > W - 130 && cur) {
					rows.push(cur);
					cur = word;
				} else cur = test;
			}
			rows.push(cur);
			for (const r of rows) {
				y += bodyPx * 1.25;
				g.fillText(r, W / 2, y);
			}
			y += bodyPx * 0.3;
		}
		if (d.note) {
			g.save();
			g.translate(W * 0.62, H - 62);
			g.rotate(-0.06);
			g.fillStyle = '#c62828';
			g.font = `italic 700 ${bodyPx * 0.95}px "Comic Sans MS", "Segoe Print", cursive`;
			g.fillText(d.note, 0, 0);
			g.restore();
		}
		const t = new THREE.CanvasTexture(c);
		t.colorSpace = THREE.SRGBColorSpace;
		t.anisotropy = 8;
		return t;
	}
	/** any flat thing (front = local +z) that can be looked at up close, e.g. the PC monitor */
	registerView(id: string, mesh: THREE.Mesh, w: number, h: number) {
		this.addSignView(id, mesh, w, h);
	}
	/** screen rectangle (CSS px) of a close-up view once the camera has arrived, else null */
	projectedRect(id: string) {
		const v = this.signViews.get(id);
		if (!v || this.reading !== id || this.focusBlend < 0.999) return null;
		const r = this.renderer.domElement.getBoundingClientRect();
		let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
		for (const [sx, sy] of [[-1, -1], [1, -1], [1, 1], [-1, 1]]) {
			const p = new THREE.Vector3((sx * v.w) / 2, (sy * v.h) / 2, 0).applyMatrix4(v.mesh.matrixWorld).project(this.camera);
			const x = r.left + ((p.x + 1) / 2) * r.width, y = r.top + ((1 - p.y) / 2) * r.height;
			x0 = Math.min(x0, x); x1 = Math.max(x1, x); y0 = Math.min(y0, y); y1 = Math.max(y1, y);
		}
		return { left: x0, top: y0, width: x1 - x0, height: y1 - y0 };
	}
	/** registers a sign mesh (front = local +z) for the close-up view */
	private addSignView(id: string, mesh: THREE.Mesh, w: number, h: number) {
		mesh.name = id;
		mesh.userData.signId = id; // the name may be replaced by the debug naming of a section
		this.signViews.set(id, { mesh, w, h });
	}
	private onRoad(x: number, z: number) {
		return this.roads.some((r) => inRect(r, x, z));
	}
	/** Height of the walkable surface (roads and interior floors are plates). */
	groundY(x: number, z: number) {
		if (inRect(HOME_STAIRS, x, z)) return 0.4 + 0.6 * (Math.min(5, Math.max(0, Math.floor(HOME_STAIRS.z1 - z))) + 1);
		if (this.legoHouse && Math.abs(x - this.legoHouse.x) < 16.5 && Math.abs(z - this.legoHouse.z) < 16.5) return 0.2;
		if (inRect(BRIDGE, x, z)) return 0.5;
		return this.onRoad(x, z) || x > 150 ? 0.4 : 0;
	}

	// ---------- world ----------
	private roads: Rect[] = [
		{ x0: -3, x1: 3, z0: -27, z1: 24 },
		{ x0: -26, x1: 29, z0: -2, z1: 3 },
		{ x0: 26, x1: 29, z0: 3, z1: 10 },
		{ x0: 29, x1: 66, z0: 7, z1: 10 },
		{ x0: -16, x1: -14, z0: -6, z1: -2 },
				{ x0: 3, x1: 15, z0: 16.5, z1: 19.5 },
		{ x0: -20, x1: -18, z0: 3, z1: 7 },
		// Stonecrest: main street, cross street, the square with the fountain
		{ x0: -3, x1: 3, z0: -148, z1: -111 },
		{ x0: -30, x1: 40, z0: -127, z1: -122 },
		{ x0: -10, x1: 10, z0: -147, z1: -133 }
	];
	private noStuds: Rect[] = [];

	private buildLights() {
		this.hemi = new THREE.HemisphereLight('#e8f4ff', '#5a7a4a', 1.1);
		this.scene.add(this.hemi);
		const sun = new THREE.DirectionalLight('#fff6e0', 2.4);
		this.sunLight = sun;
		sun.castShadow = true;
		sun.shadow.mapSize.set(2048, 2048);
		const s = sun.shadow.camera;
		s.left = -42;
		s.right = 42;
		s.top = 42;
		s.bottom = -42;
		s.near = 1;
		s.far = 160;
		sun.shadow.bias = -0.0004;
		sun.shadow.normalBias = 0.04;
		this.scene.add(sun, sun.target);
		this.sun = sun;
	}

	private buildGround() {
		// baseplate
		const base = new THREE.Mesh(new THREE.BoxGeometry(104, 0.8, 76), this.mat(C.green));
		base.position.set(20, -0.4, -9);
		base.receiveShadow = true;
		this.scene.add(base);
		// table under the baseplate (diorama feel)
		const table = new THREE.Mesh(new THREE.BoxGeometry(400, 1, 400), this.mat('#e9eef2', { roughness: 0.6 }));
		table.position.y = -1.3;
		table.receiveShadow = true;
		this.scene.add(table);

		for (const r of this.roads) {
			const m = new THREE.Mesh(new THREE.BoxGeometry(r.x1 - r.x0, 0.4, r.z1 - r.z0), this.mat(C.gray));
			m.position.set((r.x0 + r.x1) / 2, 0.2, (r.z0 + r.z1) / 2);
			m.receiveShadow = true;
			this.scene.add(m);
		}
	}

	private groundStuds() {
		// studs on ground: grass cells and road cells
		for (let x = -31.5; x < 72; x++) {
			for (let z = -46.5; z < 29; z++) {
				if (this.noStuds.some((r) => inRect(r, x, z))) continue;
				if (inRect(POND, x, z)) continue;
				const road = this.onRoad(x, z);
				this.studs.push({ x, y: road ? 0.4 : 0, z, c: road ? C.gray : C.green });
			}
		}
		// the route and Stonecrest: green, the dirt path tan, the streets grey; no studs in the creek
		for (let x = NORTH_PLATE.x0 + 0.5; x < NORTH_PLATE.x1; x++) {
			for (let z = NORTH_PLATE.z0 + 0.5; z < NORTH_PLATE.z1; z++) {
				if (inRect(CREEK, x, z) || this.noStuds.some((r) => inRect(r, x, z))) continue;
				const road = this.onRoad(x, z);
				this.studs.push({ x, y: road ? 0.4 : 0, z, c: road ? C.gray : inRect(TRAIL, x, z) ? '#c8a46a' : C.green });
			}
		}
	}

	private buildTrees() {
		const spr = (x: number, z: number) => this.spruceSpots.push({ x, z, s: 8.5 + Math.random() * 2.5, r: Math.random() * 6 });
		const rnd = (x: number, z: number, s = 6 + Math.random() * 1.5) => this.roundSpots.push({ x, z, s, r: Math.random() * 6 });
		const mix = (x: number, z: number) => (Math.random() < 0.25 ? rnd(x, z) : spr(x, z));
		const j = () => (Math.random() - 0.5) * 0.8;
		const step = 3.6;
		// east / west borders (two staggered rows)
		for (let z = -45; z <= 27; z += step) {
			mix(-30.3 + j(), z);
			mix(-27.4 + j(), z + step / 2);
			mix(70.3 + j(), z);
			mix(67.4 + j(), z + step / 2);
		}
		// south border
		// (kept low so they don't hide the village from the camera)
		for (let x = -24; x <= 64; x += step) {
			rnd(x + j(), 27.2, 4 + Math.random() * 0.8);
			this.bushSpots.push({ x: x + step / 2 + j(), y: 0, z: 25, s: 1.6 + Math.random() * 0.5 });
		}
		// separation between village and route
		for (let x = -25; x <= 25; x += step) {
			if (Math.abs(x) > 6) spr(x, -27.6);
			if (Math.abs(x + step / 2) > 11) spr(x + step / 2, -30.2);
		}
		// behind the big house (it reaches further north)
		for (let x = 28; x <= 66; x += step) {
			spr(x + j(), -29.6);
			spr(x + step / 2 + j(), -32.4);
		}
		// route forest
		for (let x = 27 + step; x <= 67; x += step)
			for (let z = -44.5; z <= -35; z += step) mix(x + j(), z + j());
		for (let x = 11.5; x <= 27; x += step)
			for (let z = -44.5; z <= -32; z += step) {
				mix(x + j(), z + j());
				mix(-x + j(), z + j());
			}
		for (let x = -9; x <= 9; x += step) {
			// leave the way north open for the path
			if (Math.abs(x) > 4.5) spr(x, -45);
			if (Math.abs(x + step / 2) > 4.5) mix(x + step / 2, -43.2);
		}
		// LEGO bushes along the north gap and corners
		const bushes = [
			[-7.2, -26.2], [7.2, -26.2], [-24, 22], [24, 22], [-24, -24], [-9, 22.5], [18, 22.5], [-26, 6], [24, 14], [64, 14], [64, -24]
		];
		for (const [x, z] of bushes) {
			this.bushSpots.push({ x, y: 0, z, s: 2.4 });
			this.blockers.push({ x0: x - 1.1, x1: x + 1.1, z0: z - 1.1, z1: z + 1.1 });
		}
	}

	/** Loads the brick tree / bush models and instances them at the collected spots. */
	// ---------- LEGO plants (from the "Lego Frog" scene): leafy plants, a bouquet, flower stems ----------
	private static FROG_VARIANTS = ['frog-leafy-a', 'frog-leafy-b', 'frog-sprout', 'frog-bouquet', 'frog-stem-a', 'frog-stem-b'] as const;
	/** finds free spots (village edge, pond banks, garden) and instances the plants there */
	private async placeFrogPlants() {
		let seed = 1337;
		const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
		const taken: { x: number; z: number; r: number }[] = [
			...this.spruceSpots.map((s) => ({ x: s.x, z: s.z, r: 3 })),
			...this.roundSpots.map((s) => ({ x: s.x, z: s.z, r: 3 })),
			...this.bushSpots.filter((s) => s.x < 150).map((s) => ({ x: s.x, z: s.z, r: 1.6 })),
			...this.bambooSpots.filter((s) => s.x < 150).map((s) => ({ x: s.x, z: s.z, r: 1.6 })),
			...this.flowerSpots.filter((s) => s.x < 150).map((s) => ({ x: s.x, z: s.z, r: 1.6 })),
			...this.flowerBunchSpots.map((s) => ({ x: s.x, z: s.z, r: 1.6 })),
			...this.flowerPlants.filter((s) => s.x < 150).map((s) => ({ x: s.x, z: s.z, r: 1.2 })),
			...READABLES.map((r) => ({ x: r.x, z: r.z, r: 2.5 })),
			...GOLDEN_STUDS.map((g) => ({ x: g.x, z: g.z, r: 1.5 })),
			...NPCS.map((n) => ({ x: n.x, z: n.z, r: 3 })),
			...ROOMS.filter((r) => !r.upstairs).map((r) => ({ x: r.door.x, z: r.door.z + 1.5, r: 3.5 }))
		];
		const free = (x: number, z: number, r: number) => {
			if (!inRect(WALK[0], x, z) || inGrass(x, z) || inRect(POND, x, z) || this.onRoad(x, z)) return false;
			if (this.legoHouse && Math.abs(x - this.legoHouse.x) < 17 && Math.abs(z - this.legoHouse.z) < 17) return false;
			for (const b of this.blockers) if (x > b.x0 - r && x < b.x1 + r && z > b.z0 - r && z < b.z1 + r) return false;
			for (const t of taken) if (Math.hypot(t.x - x, t.z - z) < t.r + r) return false;
			return true;
		};
		const plants: { x: number; z: number; v: number; c: string; rot: number; wallY?: number }[] = [];
		const colors = ['#ff8ac0', '#ffd23d', '#ffffff', '#c91a09', '#b98ae6', '#fe8a18'];
		const add = (x: number, z: number, v: number, r: number) => {
			if (!free(x, z, r)) return false;
			plants.push({ x, z, v, c: colors[Math.floor(rnd() * colors.length)], rot: rnd() * Math.PI * 2 });
			taken.push({ x, z, r });
			return true;
		};
		// pond banks and garden corners first
		for (const [x, z, v] of [[-3, 21.5, 0], [-13.4, 18.4, 3], [-24.2, 16.2, 1], [-24, 10.5, 2]]) add(x, z, v, 1.4);
		// then the outer part of the village: a ring 0.8–7 studs inside the walkable border
		const W = WALK[0];
		const cand: [number, number][] = [];
		for (let x = W.x0 + 1; x < W.x1 - 1; x += 1.3)
			for (let z = W.z0 + 1; z < W.z1 - 1; z += 1.3) {
				const edge = Math.min(x - W.x0, W.x1 - x, z - W.z0, W.z1 - z);
				if (edge > 0.8 && edge < 7) cand.push([x + (rnd() - 0.5) * 0.6, z + (rnd() - 0.5) * 0.6]);
			}
		for (let i = cand.length - 1; i > 0; i--) {
			const j = Math.floor(rnd() * (i + 1));
			[cand[i], cand[j]] = [cand[j], cand[i]];
		}
		for (const [x, z] of cand) {
			if (plants.length >= 34) break;
			const v = Math.floor(rnd() * 4); // the flat flower stems (4, 5) climb walls instead, see below
			add(x, z, v, v < 2 ? 1.9 : 1.3);
		}
		// flat flower stems are climbing plants: mount them upright on house fronts that face the camera.
		// A horizontal ray towards -z finds the exact wall surface (doors are left free).
		const ray = new THREE.Raycaster();
		const statics: THREE.Object3D[] = [];
		this.scene.traverse((o) => {
			if ((o as THREE.Mesh).isMesh && !(o as THREE.InstancedMesh).isInstancedMesh && !o.name.startsWith('fx:')) statics.push(o);
		});
		const wallSpots: [number, number, number][] = [
			// x, start z (in front of the wall), height
			[-20.4, -2, 2.4], [-18, -2, 3.1], [-12, -2, 2.6], [-9.6, -2, 3.2], // home (door at -15)
			[5.3, 20, 2.6], [20.7, 20, 2.9] // lab corners
		];
		for (const [x, z0, y] of wallSpots) {
			ray.set(new THREE.Vector3(x, y, z0), new THREE.Vector3(0, 0, -1));
			ray.far = 9;
			const hit = ray.intersectObjects(statics, false)[0];
			if (!hit || !hit.face || Math.abs(hit.face.normal.clone().transformDirection(hit.object.matrixWorld).z) < 0.7) continue;
			plants.push({ x, z: hit.point.z + 0.12, v: rnd() < 0.5 ? 4 : 5, c: colors[Math.floor(rnd() * colors.length)], rot: rnd() * Math.PI * 2, wallY: y });
		}
		const greenMat = this.mat(C.darkGreen, { side: THREE.DoubleSide });
		const flowerMat = this.mat('#ffffff', { side: THREE.DoubleSide });
		const m4 = new THREE.Matrix4();
		await Promise.all(
			Town.FROG_VARIANTS.map(async (name, v) => {
				const list = plants.filter((p) => p.v === v);
				if (!list.length) return;
				const [green, flower] = await Promise.all([loadModel(`/models/${name}-green.bin`), loadModel(`/models/${name}-flower.bin`)]);
				const G = new THREE.InstancedMesh(green, greenMat, list.length);
				const F = new THREE.InstancedMesh(flower, flowerMat, list.length);
				G.name = `plant:${name}`;
				F.name = `plant:${name}-flowers`;
				list.forEach((p, i) => {
					const s = 0.9 + ((p.rot * 7) % 1) * 0.25;
					if (p.wallY !== undefined) {
						// upright against the wall: the plant's up axis (+y) points away from the wall (+z), turned a bit in the wall plane
						m4.makeRotationZ(p.rot).multiply(new THREE.Matrix4().makeRotationX(Math.PI / 2)).scale(new THREE.Vector3(s, s, s)).setPosition(p.x, p.wallY, p.z);
					} else m4.makeRotationY(p.rot).scale(new THREE.Vector3(s, s, s)).setPosition(p.x, this.groundY(p.x, p.z), p.z);
					G.setMatrixAt(i, m4);
					F.setMatrixAt(i, m4);
					F.setColorAt(i, new THREE.Color(p.c));
				});
				for (const im of [G, F]) {
					im.castShadow = false; // small ground plant: receive only
					im.receiveShadow = true;
					this.scene.add(im);
				}
			})
		);
		this.frogPlantCount = plants.length;

		// ---- "Lego Trees and Plants": tall grass tufts, ferns, slim cypresses, small pines ----
		const extra: { name: string; x: number; z: number; rot: number; s: number }[] = [];
		const addExtra = (name: string, x: number, z: number, r: number, s: number, block = 0) => {
			if (!free(x, z, r)) return false;
			extra.push({ name, x, z, rot: rnd() * Math.PI * 2, s: s * (0.9 + rnd() * 0.2) });
			taken.push({ x, z, r });
			if (block) this.blockers.push({ x0: x - block, x1: x + block, z0: z - block, z1: z + block });
			return true;
		};
		// reeds standing in the water at the four pond corners (no free-spot check: the pond itself is a blocker)
		for (const [x, z] of [[-11.2, 15.8], [-4.6, 15.9], [-11.3, 21.2], [-4.7, 21.3]]) extra.push({ name: 'tall-grass', x, z, rot: rnd() * Math.PI * 2, s: 0.45 + rnd() * 0.1 });
		// everything else: search the whole village, outer areas first
		const W2 = WALK[0];
		const all: [number, number, number][] = [];
		for (let x = W2.x0 + 1; x < W2.x1 - 1; x += 1.1)
			for (let z = W2.z0 + 1; z < W2.z1 - 1; z += 1.1) {
				const edge = Math.min(x - W2.x0, W2.x1 - x, z - W2.z0, W2.z1 - z);
				all.push([x + (rnd() - 0.5) * 0.5, z + (rnd() - 0.5) * 0.5, edge + rnd() * 4]);
			}
		all.sort((a, b) => a[2] - b[2]);
		const want: [string, number, number, number, number][] = [
			// name, how many, radius, scale, blocker half-size
			['cypress', 6, 1.7, 0.8, 0.8],
			['fern', 4, 2.2, 0.3, 0],
			['pine-small', 8, 1.6, 0.75, 0.6],
			['tall-grass', 10, 1.1, 0.5, 0]
		];
		for (const [name, n, r, sc, bl] of want) {
			let k = 0;
			for (const [x, z] of all) {
				if (k >= n) break;
				if (addExtra(name, x, z, r, sc, bl)) k++;
			}
		}
		const extraMats: Record<string, THREE.Material> = {
			'tall-grass': this.mat('#4b9f4a', { side: THREE.DoubleSide }),
			fern: this.mat('#3e9a32', { side: THREE.DoubleSide }),
			cypress: new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.35 }),
			'pine-small': new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.35 })
		};
		await Promise.all(
			Object.keys(extraMats).map(async (name) => {
				const list = extra.filter((p) => p.name === name);
				if (!list.length) return;
				const geo = await loadModel(`/models/${name}.bin`);
				const im = new THREE.InstancedMesh(geo, extraMats[name], list.length);
				im.name = `plant:${name}`;
				list.forEach((p, i) => {
					m4.makeRotationY(p.rot).scale(new THREE.Vector3(p.s, p.s, p.s)).setPosition(p.x, this.groundY(p.x, p.z), p.z);
					im.setMatrixAt(i, m4);
				});
				im.castShadow = name === 'cypress' || name === 'pine-small'; // trees cast, grass and ferns only receive
				im.receiveShadow = true;
				this.scene.add(im);
			})
		);
		this.extraPlantCount = extra.length;
	}
	extraPlantCount = 0;
	frogPlantCount = 0;

	private async loadModels() {
		const [round, spruce, bush, bamboo, flowerpot] = await Promise.all([
			loadModel('/models/round-tree.bin'),
			loadModel('/models/spruce-tree.bin'),
			loadModel('/models/bush.bin'),
			loadModel('/models/bamboo.bin'),
			loadModel('/models/flowerpot.bin')
		]);
		const treeMat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.32 });
		const tex = await new THREE.TextureLoader().loadAsync(assetUrl(`/models/bush.png?v=${ASSET_VERSION}`));
		tex.colorSpace = THREE.SRGBColorSpace;
		tex.anisotropy = 4;
		const loadAlpha = async (url: string) => {
			const t = await new THREE.TextureLoader().loadAsync(url);
			t.colorSpace = THREE.NoColorSpace;
			return t;
		};
		// the plant textures keep their cut-out shape in a separate alpha mask
		const [bushA, bambooA] = await Promise.all([loadAlpha(assetUrl(`/models/bush_a.png?v=${ASSET_VERSION}`)), loadAlpha(assetUrl(`/models/bamboo_a.png?v=${ASSET_VERSION}`))]);
		const bushMat = new THREE.MeshStandardMaterial({ map: tex, alphaMap: bushA, roughness: 0.32, alphaTest: 0.5, side: THREE.DoubleSide });
		const btex = await new THREE.TextureLoader().loadAsync(assetUrl(`/models/bamboo.png?v=${ASSET_VERSION}`));
		btex.colorSpace = THREE.SRGBColorSpace;
		btex.anisotropy = 4;
		const bambooMat = new THREE.MeshStandardMaterial({ map: btex, alphaMap: bambooA, roughness: 0.32, side: THREE.DoubleSide, alphaTest: 0.5 });
		const m4 = new THREE.Matrix4();
		const q = new THREE.Quaternion();
		const up = new THREE.Vector3(0, 1, 0);
		const v = new THREE.Vector3();
		const sc = new THREE.Vector3();
		const place = (name: string, geo: THREE.BufferGeometry, mat: THREE.Material, all: { x: number; y?: number; z: number; s: number; r?: number }[], tint: boolean) => {
			if (!all.length) return;
			// in chunks of 32 x 32 units: chunks out of view (and out of the shadow camera) are skipped
			const chunks = new Map<string, typeof all>();
			for (const p of all) {
				const k = `${Math.floor(p.x / 32)},${Math.floor(p.z / 32)}`;
				if (!chunks.has(k)) chunks.set(k, []);
				chunks.get(k)!.push(p);
			}
			for (const [k, spots] of chunks) {
				const im = new THREE.InstancedMesh(geo, mat, spots.length);
				im.name = `${name}:${k}`;
				const col = new THREE.Color();
				spots.forEach((p, i) => {
					q.setFromAxisAngle(up, p.r ?? Math.random() * 6);
					m4.compose(v.set(p.x, p.y ?? 0, p.z), q, sc.set(p.s, p.s, p.s));
					im.setMatrixAt(i, m4);
					if (tint) im.setColorAt(i, col.setHSL(0, 0, 0.9 + Math.random() * 0.1));
				});
				im.computeBoundingSphere();
				// polygon budget: the shadow pass draws every caster again; small ground plants don't need to cast
				im.castShadow = name.startsWith('tree:') || name === 'plant:flowerpot';
				im.receiveShadow = true;
				this.scene.add(im);
			}
		};
		place('tree:spruce', spruce, treeMat, this.spruceSpots, true);
		place('tree:round', round, treeMat, this.roundSpots, true);
		place('plant:bush', bush, bushMat, this.bushSpots, false);
		place('plant:bamboo', bamboo, bambooMat, this.bambooSpots, false);
		{
			const [fpLeaf, fpHead] = await Promise.all([loadModel('/models/flowerplant-leaf.bin'), loadModel('/models/flowerplant-head.bin')]);
			const L = new THREE.InstancedMesh(fpLeaf, this.mat(C.darkGreen, { side: THREE.DoubleSide }), this.flowerPlants.length);
			const H = new THREE.InstancedMesh(fpHead, this.mat('#ffffff', { side: THREE.DoubleSide }), this.flowerPlants.length);
			L.name = 'plant:flower-leaves';
			H.name = 'plant:flower';
			const m4 = new THREE.Matrix4();
			this.flowerPlants.forEach((f, i) => {
				m4.makeRotationY(Math.random() * Math.PI * 2).setPosition(f.x, f.y ?? this.groundY(f.x, f.z), f.z);
				L.setMatrixAt(i, m4);
				H.setMatrixAt(i, m4);
				H.setColorAt(i, new THREE.Color(f.c));
			});
			for (const im of [L, H]) {
				im.castShadow = false; // tiny plants: receive shadows only
				im.receiveShadow = true;
				this.scene.add(im);
			}
		}
		const potMat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.3, side: THREE.DoubleSide });
		place('plant:flowerpot', flowerpot, potMat, this.flowerSpots, false);
		if (this.flowerBunchSpots.length) place('plant:flower-bunch', await loadModel('/models/flowerpot-flowers.bin'), potMat, this.flowerBunchSpots, false);
		if (this.labComputers.length) {
			const [body, scr] = await Promise.all([loadModel('/models/lab-computer.bin'), loadModel('/models/lab-computer-screen.bin')]);
			// double-sided: some faces of the source part point inwards (LDraw winding), they vanished from the side
			const B = new THREE.InstancedMesh(body, new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.35, side: THREE.DoubleSide }), this.labComputers.length);
			const scrMat = new THREE.MeshStandardMaterial({ color: '#0a1a24', emissive: '#40c0ff', emissiveIntensity: 0.9, roughness: 0.15, side: THREE.DoubleSide });
			const S = new THREE.InstancedMesh(scr, scrMat, this.labComputers.length);
			this.blinkers.push(scrMat);
			B.name = 'lab:computer';
			S.name = 'lab:computer-screen';
			const m = new THREE.Matrix4();
			this.labComputers.forEach((c, i) => {
				m.makeScale(c.s, c.s, c.s).setPosition(c.x, c.y, c.z);
				B.setMatrixAt(i, m);
				// the screen sits a hair in front of the housing's slanted face
				m.makeScale(c.s, c.s, c.s).setPosition(c.x, c.y + 0.01, c.z + 0.01);
				S.setMatrixAt(i, m);
			});
			B.castShadow = B.receiveShadow = true;
			this.scene.add(B, S);
			// an invisible plane on each slanted screen = target of the first-person close-up (HTML terminal is drawn on it)
			this.labComputers.forEach((c, i) => {
				const face = new THREE.Mesh(new THREE.PlaneGeometry(2.2, 1.64), new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false }));
				face.position.set(c.x, c.y + 1.33 * (c.s / 1.2), c.z + 0.03);
				face.rotation.x = -Math.PI / 4; // the screen leans back 45°
				face.userData.tilted = true;
				this.scene.add(face);
				this.registerView(`pc:lab${i + 1}`, face, 2.2, 1.64);
			});
		}
		await this.placeFrogPlants();
		if (this.wireframe) this.setWireframe(true); // models loaded after the switch was turned on
		if (this.legoHouse) {
			const geo = await loadModel('/models/lego-house.bin');
			const { x, z } = this.legoHouse;
			const mesh = new THREE.Mesh(geo, new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.3 }));
			mesh.name = 'house:dex (lego-house model)';
			mesh.rotation.y = Math.PI;
			mesh.position.set(x, 0, z);
			mesh.castShadow = mesh.receiveShadow = true;
			this.scene.add(mesh);
		}
	}

	private windowAt(x: number, y: number, z: number, w: number, h: number, axis: 'x' | 'z', out: number, parent: THREE.Object3D, frame = C.white) {
		const t = 0.3;
		if (axis === 'z') {
			this.box(w + 0.4, h + 0.4, t, frame, x, y - 0.2, z + out * 0.05, { parent });
			const g = new THREE.Mesh(this.rbox(w - 0.2, h - 0.2, t), this.mat(C.glass, { roughness: 0.05, transparent: true, opacity: 0.85 }));
			g.position.set(x, y + h / 2, z + out * 0.2);
			parent.add(g);
			this.box(0.15, h, 0.2, frame, x, y, z + out * 0.3, { parent });
		} else {
			this.box(t, h + 0.4, w + 0.4, frame, x + out * 0.05, y - 0.2, z, { parent });
			const g = new THREE.Mesh(this.rbox(t, h - 0.2, w - 0.2), this.mat(C.glass, { roughness: 0.05, transparent: true, opacity: 0.85 }));
			g.position.set(x + out * 0.2, y + h / 2, z);
			parent.add(g);
			this.box(0.2, h, 0.15, frame, x + out * 0.3, y, z, { parent });
		}
	}

	private wall(w: number, h: number, d: number, color: string, x: number, z: number) {
		const tex = this.brickTexture(color, Math.max(w, d), h);
		const m = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), new THREE.MeshStandardMaterial({ map: tex, roughness: 0.3 }));
		m.position.set(x, 0.4 + h / 2, z);
		m.castShadow = m.receiveShadow = true;
		this.scene.add(m);
	}

	private buildHouse(cx: number, cz: number, w: number, d: number, roof: string, wallC: string, door: string, label: string) {
		const g = new THREE.Group();
		this.scene.add(g);
		const x0 = cx - w / 2, x1 = cx + w / 2, z0 = cz - d / 2, z1 = cz + d / 2;
		this.blockers.push({ x0: x0 - 0.6, x1: x1 + 0.6, z0: z0 - 0.6, z1: z1 + 0.5 });
		this.noStuds.push({ x0: x0 - 0.5, x1: x1 + 0.5, z0: z0 - 0.5, z1: z1 + 0.5 });
		// foundation
		this.box(w + 1, 0.4, d + 1, C.gray, cx, 0, cz, { parent: g });
		const H = 7.2;
		this.wall(w, H, 1, wallC, cx, z1 - 0.5);
		this.wall(w, H, 1, wallC, cx, z0 + 0.5);
		this.wall(1, H, d - 2, wallC, x0 + 0.5, cz);
		this.wall(1, H, d - 2, wallC, x1 - 0.5, cz);
		// floor band
		this.box(w + 0.2, 0.4, d + 0.2, C.white, cx, 3.8, cz, { parent: g });
		// door (front faces +z)
		const dx = cx;
		this.box(2.2, 4.0, 0.4, C.white, dx, 0.4, z1 + 0.1, { parent: g });
		this.box(1.8, 3.6, 0.4, door, dx, 0.4, z1 + 0.25, { parent: g });
		const knob = new THREE.Mesh(new THREE.SphereGeometry(0.15, 10, 8), this.mat(C.yellow, { metalness: 0.6, roughness: 0.2 }));
		knob.position.set(dx + 0.55, 2.2, z1 + 0.5);
		g.add(knob);
		// step
		this.box(3, 0.4, 1.2, C.darkGray, dx, 0.4, z1 + 0.7, { parent: g, studs: true });
		// windows
		for (const wx of [cx - w / 2 + 2.3, cx + w / 2 - 2.3]) this.windowAt(wx, 1.6, z1, 2.4, 1.8, 'z', 1, g);
		// upper windows only at the sides: the middle above the door belongs to the house sign
		for (const wx of [cx - w / 2 + 2.3, cx + w / 2 - 2.3]) this.windowAt(wx, 5.0, z1, 1.9, 1.6, 'z', 1, g);
		for (const s of [-1, 1]) this.windowAt(cx + s * (w / 2), 5.0, cz, 2.4, 1.6, 'x', s, g);
		// stepped brick roof
		const layers = 6;
		const ov = 0.8;
		for (let i = 0; i < layers; i++) {
			const dd = d + ov * 2 - i * 1.9;
			if (dd <= 0.6) break;
			const col = i % 2 === 0 ? roof : new THREE.Color(roof).offsetHSL(0, 0, -0.04).getStyle();
			this.box(w + ov * 2, 0.6, dd, col, cx, 0.4 + H + i * 0.6, cz, { parent: g });
		}
		this.box(w + ov * 2, 0.4, 1.0, roof, cx, 0.4 + H + layers * 0.6, cz, { parent: g, studs: true });
		// chimney
		this.box(1.6, 3.4, 1.6, C.redBrown, cx + w / 2 - 2.2, 0.4 + H + 0.6, cz - 1.5, { parent: g, studs: true });
		// sign above the door, on a little board in front of the wall (nothing in front of it)
		const sw = Math.min(3.4, w - 7.4), sh = sw / 4; // fits between the upper side windows
		this.box(sw + 0.3, sh + 0.3, 0.2, C.white, dx, 5.8 - sh / 2 - 0.15, z1, { parent: g, shadow: false });
		const sign = new THREE.Mesh(new THREE.PlaneGeometry(sw, sh), new THREE.MeshStandardMaterial({ map: this.signTexture(label), roughness: 0.4 }));
		sign.position.set(dx, 5.8, z1 + 0.22);
		g.add(sign);
		// mailbox
		const mx = cx - 3.2, mz = z1 + 2.2;
		this.box(0.4, 1.6, 0.4, C.darkGray, mx, 0, mz, { parent: g });
		this.box(1.0, 0.9, 1.4, label === 'HOME' ? C.blue : C.red, mx, 1.6, mz, { parent: g, studs: true });
		this.blockers.push({ x0: mx - 0.6, x1: mx + 0.6, z0: mz - 0.8, z1: mz + 0.8 });
	}

	/**
	 * A pile of loose LEGO bricks built from real brick sizes (1 unit = 1 stud, brick height 1.2),
	 * so studs match the rest of the world. Low-poly: rounded boxes + one instanced stud mesh.
	 */
	private brickPile(n: number, cx: number, cz: number, color: string) {
		let seed = n * 9973 + 17;
		const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
		const sizes: [number, number][] = [[2, 4], [2, 2], [2, 4], [1, 2], [1, 4], [2, 3], [2, 2], [1, 1]];
		const extra = [C.white, C.yellow, C.blue, C.red, C.green, C.orange].filter((c) => c !== color);
		const R = 3.6;
		const grp = new THREE.Group();
		grp.name = `bricks:pile${n}`;
		grp.position.set(cx, 0, cz);
		this.scene.add(grp);
		const studMat = new Map<string, THREE.Matrix4[]>();
		const placed: { x: number; z: number; r: number; top: number }[] = [];
		const count = 16;
		for (let i = 0; i < count; i++) {
			const [w, d] = sizes[Math.floor(rnd() * sizes.length)];
			const col = rnd() < 0.7 ? color : extra[Math.floor(rnd() * extra.length)];
			// rest on the highest brick it overlaps (bounding circles); a pile is at most two layers high
			const myR = Math.hypot(w, d) / 2;
			let x = 0, z = 0, y = 99;
			const STUD_H = 0.2125;
			for (let tries = 0; tries < 8 && y > 1.6; tries++) {
				const ang = rnd() * Math.PI * 2;
				const dist = Math.sqrt(rnd()) * R;
				x = Math.cos(ang) * dist;
				z = Math.sin(ang) * dist;
				// loose bricks rest on the studs below them (ground studs or the brick underneath)
				y = STUD_H;
				for (const p of placed) if (Math.hypot(p.x - x, p.z - z) < (p.r + myR) * 0.85) y = Math.max(y, p.top);
			}
			if (y > 2) continue;
			const brick = new THREE.Group();
			brick.position.set(x, y, z);
			brick.rotation.y = rnd() * Math.PI;
			// a few bricks lean on others
			if (y > 0 && rnd() < 0.35) brick.rotation.x = (rnd() - 0.5) * 0.2;
			grp.add(brick);
			this.box(w, 1.2, d, col, 0, 0, 0, { parent: brick });
			brick.updateMatrixWorld(true);
			for (let sx = 0; sx < w; sx++)
				for (let sz = 0; sz < d; sz++) {
					const m = new THREE.Matrix4().makeTranslation(sx - (w - 1) / 2, 1.2, sz - (d - 1) / 2);
					m.premultiply(brick.matrix);
					(studMat.get(col) ?? studMat.set(col, []).get(col)!).push(m);
				}
			placed.push({ x, z, r: myR, top: y + 1.2 + STUD_H + (brick.rotation.x ? 0.3 : 0) });
		}
		const studGeo = new THREE.CylinderGeometry(0.3, 0.3, 0.2125, 12);
		studGeo.translate(0, 0.106, 0);
		for (const [col, list] of studMat) {
			const im = new THREE.InstancedMesh(studGeo, this.mat(col), list.length);
			list.forEach((m, i) => im.setMatrixAt(i, m));
			im.castShadow = im.receiveShadow = true;
			im.name = `bricks:pile${n}/studs`;
			grp.add(im);
		}
		this.blockers.push({ x0: cx - R + 0.3, x1: cx + R - 0.3, z0: cz - R + 0.3, z1: cz + R - 0.3 });
	}

	private legoHouse: { x: number; z: number } | null = null;
	/** Dex & Lumi's house: the detailed CAD LEGO house at true stud scale (33 x 33 studs). */
	private placeLegoHouse(x: number, z: number, size: number) {
		this.legoHouse = { x, z };
		const h = size / 2;
		this.noStuds.push({ x0: x - h, x1: x + h, z0: z - h, z1: z + h });
		// house body (the garden and stepping stones stay walkable)
		// house body + garage (measured from a top-down render of the model)
		this.blockers.push({ x0: x - 21, x1: x + 7.8, z0: z - 15.5, z1: z - 1.3 });
		this.blockers.push({ x0: x - 18.5, x1: x - 4.5, z0: z - 1.3, z1: z + 6.2 });
		this.blockers.push({ x0: x + 11, x1: x + 13, z0: z - 2.3, z1: z - 0.3 }); // flower pot
	}

	/** Little park where Dex's old house used to be. */
	private buildPark(cx: number, cz: number) {
		for (const [dx, dz] of [[-4.5, -2.5], [4.5, -3], [0, -4.5]]) {
			this.roundSpots.push({ x: cx + dx, z: cz + dz, s: 5.5 + Math.random(), r: Math.random() * 6 });
			this.blockers.push({ x0: cx + dx - 0.9, x1: cx + dx + 0.9, z0: cz + dz - 0.9, z1: cz + dz + 0.9 });
		}
		for (const [dx, dz] of [[-5.5, 3], [5.5, 3]]) {
			// the flowers from the pot (without the pot, prepared in Blender) instead of the two park bushes
			this.flowerBunchSpots.push({ x: cx + dx, y: 0, z: cz + dz, s: 1, r: Math.random() * Math.PI * 2 });
			this.blockers.push({ x0: cx + dx - 1, x1: cx + dx + 1, z0: cz + dz - 1, z1: cz + dz + 1 });
		}
		// bench
		this.box(3.2, 0.4, 1.2, C.redBrown, cx, 1.0, cz + 1.5, { studs: true });
		this.box(3.2, 1.2, 0.3, C.redBrown, cx, 1.4, cz + 0.95);
		for (const dx of [-1.3, 1.3]) this.box(0.4, 1.0, 1.0, C.darkGray, cx + dx, 0, cz + 1.5);
		this.blockers.push({ x0: cx - 1.7, x1: cx + 1.7, z0: cz + 0.8, z1: cz + 2.2 });
	}

	private buildLab(cx: number, cz: number, w: number, d: number) {
		const g = new THREE.Group();
		this.scene.add(g);
		const x0 = cx - w / 2, x1 = cx + w / 2, z0 = cz - d / 2, z1 = cz + d / 2;
		this.blockers.push({ x0: x0 - 0.6, x1: x1 + 0.6, z0: z0 - 0.6, z1: z1 + 0.5 });
		this.noStuds.push({ x0: x0 - 0.5, x1: x1 + 0.5, z0: z0 - 0.5, z1: z1 + 0.5 });
		this.box(w + 1, 0.4, d + 1, C.gray, cx, 0, cz, { parent: g });
		const H = 7.2;
		this.wall(w, H, 1, C.tan, cx, z1 - 0.5);
		this.wall(w, H, 1, C.tan, cx, z0 + 0.5);
		this.wall(1, H, d - 2, C.tan, x0 + 0.5, cz);
		this.wall(1, H, d - 2, C.tan, x1 - 0.5, cz);
		// front faces +z (towards the camera)
		const fz = z1;
		this.box(2.4, 4.2, 0.4, C.white, cx, 0.4, fz + 0.1, { parent: g });
		this.box(2.0, 3.8, 0.4, C.darkGreen, cx, 0.4, fz + 0.25, { parent: g });
		this.box(0.6, 1.2, 0.2, C.glass, cx, 2.6, fz + 0.45, { parent: g });
		this.box(3.4, 0.4, 1.4, C.darkGray, cx, 0.4, fz + 0.8, { parent: g, studs: true });
		// wide windows with striped awnings
		for (const s of [-1, 1]) {
			const wx = cx + s * 4.6;
			this.windowAt(wx, 1.6, fz, 5, 1.6, 'z', 1, g);
			for (let i = 0; i < 3; i++) this.box(5.4, 0.3, 0.6, i % 2 ? C.white : C.red, wx, 3.6 + i * 0.25, fz + 0.4 + i * 0.35, { parent: g });
		}
		// round windows
		for (const wx of [cx - 6, cx - 3.5, cx + 3.5, cx + 6]) {
			const ring = new THREE.Mesh(new THREE.CylinderGeometry(0.85, 0.85, 0.3, 20), this.mat(C.darkGray));
			ring.rotation.x = Math.PI / 2;
			ring.position.set(wx, 5.8, fz + 0.1);
			const glass = new THREE.Mesh(new THREE.CylinderGeometry(0.65, 0.65, 0.35, 20), this.mat('#40b8d8', { roughness: 0.05 }));
			glass.rotation.x = Math.PI / 2;
			glass.position.set(wx, 5.8, fz + 0.15);
			g.add(ring, glass);
		}
		// flat roof with rim + tiles
		this.box(w + 1, 0.6, d + 1, C.darkGray, cx, 0.4 + H, cz, { parent: g });
		for (let x = x0 + 1; x < x1 - 0.5; x += 2.5)
			for (let z = z0 + 1; z < z1 - 0.5; z += 2.5) this.box(2.3, 0.25, 2.3, C.white, x + 1.15, 0.4 + H + 0.6, z + 1.15, { parent: g, shadow: false });
		this.box(w + 1, 0.8, 0.6, C.gray, cx, 0.4 + H + 0.6, z0 - 0.2, { parent: g, studs: true });
		this.box(w + 1, 0.8, 0.6, C.gray, cx, 0.4 + H + 0.6, z1 + 0.2, { parent: g, studs: true });
		this.box(0.6, 0.8, d, C.gray, x0 - 0.2, 0.4 + H + 0.6, cz, { parent: g, studs: true });
		this.box(0.6, 0.8, d, C.gray, x1 + 0.2, 0.4 + H + 0.6, cz, { parent: g, studs: true });
		// solar panels (back of the roof)
		for (let i = 0; i < 3; i++) {
			const p = new THREE.Mesh(this.rbox(3, 0.2, 2), this.mat('#1d3f8a', { roughness: 0.15, metalness: 0.3 }));
			p.position.set(x0 + 3 + i * 3.4, 0.4 + H + 1.6, z0 + 2);
			p.rotation.x = 0.45;
			p.castShadow = true;
			g.add(p);
			this.box(0.3, 0.9, 0.3, C.darkGray, x0 + 3 + i * 3.4, 0.4 + H + 0.8, z0 + 1.6, { parent: g });
		}
		// chimney with hole
		this.box(2.6, 2.4, 2.6, C.redBrown, x1 - 2.8, 0.4 + H + 0.6, z0 + 2.8, { parent: g });
		this.box(1.4, 0.1, 1.4, C.black, x1 - 2.8, 0.4 + H + 3.0, z0 + 2.8, { parent: g });
		// satellite dish
		const dish = new THREE.Mesh(new THREE.SphereGeometry(1.4, 20, 10, 0, Math.PI * 2, 0, Math.PI / 3), this.mat(C.white, { side: THREE.DoubleSide }));
		dish.position.set(cx + 1, 0.4 + H + 2.6, cz);
		dish.rotation.x = Math.PI - 0.7;
		dish.castShadow = true;
		g.add(dish);
		this.box(0.4, 1.6, 0.4, C.darkGray, cx + 1, 0.4 + H + 0.6, cz, { parent: g });
		// lab sign
		const sign = new THREE.Mesh(new THREE.PlaneGeometry(5.2, 1.9), new THREE.MeshStandardMaterial({ map: this.boardTexture('sign:lab', 5.2, 1.9), roughness: 0.4 }));
		sign.position.set(cx, 5.75, fz + 0.06);
		g.add(sign);
		this.addSignView('sign:lab', sign, 5.2, 1.9);
	}

	private fence(x0: number, z0: number, x1: number, z1: number) {
		const len = Math.hypot(x1 - x0, z1 - z0);
		const n = Math.round(len);
		const ang = Math.atan2(z1 - z0, x1 - x0);
		const g = new THREE.Group();
		g.position.set(x0, 0, z0);
		g.rotation.y = -ang;
		this.scene.add(g);
		for (let i = 0; i <= n; i++) this.box(0.22, 1.5, 0.22, C.white, (i * len) / n, 0, 0, { parent: g, studs: false });
		this.box(len, 0.25, 0.18, C.white, len / 2, 0.5, 0, { parent: g });
		this.box(len, 0.25, 0.18, C.white, len / 2, 1.1, 0, { parent: g });
		this.blockers.push({ x0: Math.min(x0, x1) - 0.35, x1: Math.max(x0, x1) + 0.35, z0: Math.min(z0, z1) - 0.35, z1: Math.max(z0, z1) + 0.35 });
	}

	private buildGarden() {
		const x0 = -25, x1 = -13, z0 = 7, z1 = 17;
		// fence with gap on north (entrance at x -20..-18)
		this.fence(x0, z0, -20.2, z0);
		this.fence(-17.8, z0, x1, z0);
		this.fence(x0, z1, x1, z1);
		this.fence(x0, z0, x0, z1);
		this.fence(x1, z0, x1, z1);
		// gate posts
		for (const gx of [-20.4, -17.6]) this.box(0.8, 2, 0.8, C.white, gx, 0, z0, { studs: true });

		// LEGO flower plants (2x2 leaf base + 1x1 flower, from flowersss.obj) — instanced in loadModels()
		const colors = [C.red, C.red, C.yellow, '#ff8ac0', C.white, C.orange];
		for (let x = x0 + 1.6; x < x1 - 1; x += 2.2)
			for (let z = z0 + 1.9; z < z1 - 1; z += 2.2) {
				if (Math.random() < 0.15) continue;
				// spots kept free: the garden sign (-21.2) and the path from the gate (-19.0)
				if (z < 10 && (Math.abs(x + 21.2) < 0.5 || Math.abs(x + 19) < 0.5)) continue;
				this.flowerPlants.push({ x: x + (Math.random() - 0.5) * 0.3, z: z + (Math.random() - 0.5) * 0.3, c: colors[Math.floor(Math.random() * colors.length)] });
			}
		// scarecrow-ish signpost
		this.box(0.3, 2.6, 0.3, C.brown, -21.35, 0, 8.9);
		const sign = new THREE.Mesh(new THREE.PlaneGeometry(2.8, 1.5), new THREE.MeshStandardMaterial({ map: this.boardTexture('sign:garden', 2.8, 1.5), side: THREE.DoubleSide, roughness: 0.6 }));
		sign.position.set(-21.15, 2.1, 8.9);
		sign.rotation.y = Math.PI / 2; // face points right (east, towards the gate path)
		this.scene.add(sign);
		this.addSignView('sign:garden', sign, 2.8, 1.5);
		this.blockers.push({ x0: -21.55, x1: -21.05, z0: 7.6, z1: 10.2 });
	}

	private buildPond() {
		const { x0, x1, z0, z1 } = POND;
		this.blockers.push({ x0: x0 - 0.4, x1: x1 + 0.4, z0: z0 - 0.4, z1: z1 + 0.4 });
		// brown border
		this.box(x1 - x0 + 1, 0.4, 0.6, C.redBrown, (x0 + x1) / 2, 0, z0 - 0.2, { studs: true });
		this.box(x1 - x0 + 1, 0.4, 0.6, C.redBrown, (x0 + x1) / 2, 0, z1 + 0.2, { studs: true });
		this.box(0.6, 0.4, z1 - z0, C.redBrown, x0 - 0.2, 0, (z0 + z1) / 2, { studs: true });
		this.box(0.6, 0.4, z1 - z0, C.redBrown, x1 + 0.2, 0, (z0 + z1) / 2, { studs: true });
		// a dark pond bed, seen through the water (and the fish swimming above it)
		const bed = new THREE.Mesh(new THREE.PlaneGeometry(x1 - x0, z1 - z0), this.mat('#1d4a5e', { roughness: 0.9 }));
		bed.rotation.x = -Math.PI / 2;
		bed.position.set((x0 + x1) / 2, 0.02, (z0 + z1) / 2);
		bed.receiveShadow = true;
		this.scene.add(bed);
		const water = new THREE.Mesh(
			new THREE.BoxGeometry(x1 - x0, 0.3, z1 - z0),
			new THREE.MeshPhysicalMaterial({ color: C.water, roughness: 0.05, metalness: 0, clearcoat: 0.7, transparent: true, opacity: 0.6, depthWrite: false })
		);
		water.position.set((x0 + x1) / 2, 0.05, (z0 + z1) / 2);
		water.receiveShadow = true;
		this.scene.add(water);
		this.buildPondMirror(x0, x1, z0, z1);
		this.buildDucks();
		for (const [x, z] of [[-10, 17], [-6.5, 20], [-8, 18.8]]) {
			const pad = new THREE.Mesh(new THREE.CylinderGeometry(0.6, 0.6, 0.12, 14, 1, false, 0.3, Math.PI * 1.75), this.mat(C.green));
			pad.position.set(x, 0.26, z);
			this.scene.add(pad);
			this.lily.push(pad);
		}
		// little fish
		const fish = new THREE.Mesh(this.rbox(0.8, 0.3, 0.35), this.mat(C.orange));
		fish.position.set(-7, 0.12, 16.5);
		this.scene.add(fish);
		this.lily.push(fish);
	}

	// ---------- Route 1 north through the forest, and the town of Stonecrest ----------
	/** which part of the outside world a spot belongs to (shown in the HUD) */
	zoneLabel(z: number) {
		return z < STONECREST_Z ? 'Stonecrest' : z < -27 ? 'Route 1' : 'Brickhollow';
	}
	private buildNorth() {
		const R = NORTH_PLATE;
		const plate = new THREE.Mesh(new THREE.BoxGeometry(R.x1 - R.x0, 0.8, R.z1 - R.z0), this.mat(C.green));
		plate.position.set((R.x0 + R.x1) / 2, -0.4, (R.z0 + R.z1) / 2);
		plate.receiveShadow = true;
		plate.name = 'ground:north';
		this.scene.add(plate);
		// the dirt path (tan studs on top, see groundStuds)
		const trail = new THREE.Mesh(new THREE.BoxGeometry(TRAIL.x1 - TRAIL.x0, 0.04, TRAIL.z1 - TRAIL.z0), this.mat('#c8a46a'));
		trail.position.set(0, 0.02, (TRAIL.z0 + TRAIL.z1) / 2);
		trail.receiveShadow = true;
		this.scene.add(trail);

		// ---- the creek: dark bed, see-through water, a wooden bridge ----
		const cz = (CREEK.z0 + CREEK.z1) / 2, cw = CREEK.x1 - CREEK.x0, cd = CREEK.z1 - CREEK.z0;
		const bed = new THREE.Mesh(new THREE.PlaneGeometry(cw, cd), this.mat('#1d4a5e', { roughness: 0.9 }));
		bed.rotation.x = -Math.PI / 2;
		bed.position.set((CREEK.x0 + CREEK.x1) / 2, 0.02, cz);
		this.scene.add(bed);
		const water = new THREE.Mesh(
			new THREE.BoxGeometry(cw, 0.3, cd),
			new THREE.MeshPhysicalMaterial({ color: C.water, roughness: 0.05, clearcoat: 0.7, transparent: true, opacity: 0.6, depthWrite: false })
		);
		water.position.set((CREEK.x0 + CREEK.x1) / 2, 0.05, cz);
		this.scene.add(water);
		// stone banks
		for (const z of [CREEK.z0 - 0.3, CREEK.z1 + 0.3]) this.box(cw, 0.35, 0.6, C.gray, (CREEK.x0 + CREEK.x1) / 2, 0, z, { studs: true });
		this.blockers.push({ x0: CREEK.x0, x1: BRIDGE.x0 + 0.4, z0: CREEK.z0 - 0.6, z1: CREEK.z1 + 0.6 });
		this.blockers.push({ x0: BRIDGE.x1 - 0.4, x1: CREEK.x1, z0: CREEK.z0 - 0.6, z1: CREEK.z1 + 0.6 });
		{
			const bw = BRIDGE.x1 - BRIDGE.x0, bd = BRIDGE.z1 - BRIDGE.z0, bz = (BRIDGE.z0 + BRIDGE.z1) / 2;
			this.box(bw, 0.5, bd, '#8a5a32', 0, 0, bz);
			for (let z = BRIDGE.z0 + 0.5; z < BRIDGE.z1; z += 1) this.box(bw - 0.2, 0.06, 0.12, '#6b4426', 0, 0.5, z, { shadow: false });
			for (const sx of [-1, 1]) {
				const x = sx * (bw / 2 - 0.2);
				for (const z of [BRIDGE.z0 + 0.3, bz, BRIDGE.z1 - 0.3]) this.box(0.35, 1.6, 0.35, '#6b4426', x, 0.5, z);
				this.box(0.3, 0.25, bd, '#8a5a32', x, 1.75, bz);
			}
		}

		// ---- signs ----
		const sign = (id: string, x: number, z: number) => {
			this.box(0.35, 3.2, 0.35, C.brown, x, 0, z - 0.2);
			const m = new THREE.Mesh(new THREE.PlaneGeometry(3.4, 1.8), new THREE.MeshStandardMaterial({ map: this.boardTexture(id, 3.4, 1.8), side: THREE.DoubleSide, roughness: 0.6 }));
			m.position.set(x, 2.2, z);
			this.scene.add(m);
			this.addSignView(id, m, 3.4, 1.8);
			this.blockers.push({ x0: x - 0.5, x1: x + 0.5, z0: z - 0.7, z1: z + 0.3 });
		};
		sign('sign:north', 5.5, -49.2);
		sign('sign:stonecrest', 5.5, -108.6);

		// ---- forest on both sides of the route, a few trees and bushes along the path ----
		const spr = (x: number, z: number) => this.spruceSpots.push({ x, z, s: 8.5 + Math.random() * 2.5, r: Math.random() * 6 });
		const rnd = (x: number, z: number) => this.roundSpots.push({ x, z, s: 6 + Math.random() * 1.5, r: Math.random() * 6 });
		const j = () => (Math.random() - 0.5) * 1.2;
		const nearCreek = (z: number) => z > CREEK.z0 - 2.5 && z < CREEK.z1 + 2.5;
		for (let z = -48.5; z >= -110; z -= 3.6)
			for (let ax = 21.5; ax <= 55; ax += ax < 30 ? 3.6 : 4.6)
				for (const sx of [-1, 1]) {
					const x = sx * ax + j();
					if (x < R.x0 + 1 || nearCreek(z) || (ax > 30 && Math.random() < 0.25)) continue;
					(Math.random() < 0.3 ? rnd : spr)(x, z + j());
				}
		let seed = 7;
		const r01 = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
		for (let z = -50; z >= -108; z -= 5.5)
			for (let x = -17; x <= 17; x += 5.5) {
				const px = x + (r01() - 0.5) * 2, pz = z + (r01() - 0.5) * 2;
				if (Math.abs(px) < 6 || nearCreek(pz) || GRASSES.some((g) => px > g.x0 - 2.5 && px < g.x1 + 2.5 && pz > g.z0 - 2.5 && pz < g.z1 + 2.5)) continue;
				if (r01() < 0.45) continue;
				if (r01() < 0.5) {
					rnd(px, pz);
					this.blockers.push({ x0: px - 1.3, x1: px + 1.3, z0: pz - 1.3, z1: pz + 1.3 });
				} else {
					this.bushSpots.push({ x: px, y: 0, z: pz, s: 2.2 + r01() * 0.6 });
					this.blockers.push({ x0: px - 1.1, x1: px + 1.1, z0: pz - 1.1, z1: pz + 1.1 });
				}
			}
		// around Stonecrest
		for (let z = -112; z >= -192; z -= 3.6) {
			spr(-44.2 + j(), z);
			spr(53.5 + j(), z + 1.8);
		}
		for (let x = -40; x <= 50; x += 3.6) {
			spr(x + j(), -191.5);
			if (x < -24 || x > 24) spr(x + 1.8 + j(), -187.5);
		}

		// ---- Stonecrest: houses along the streets ----
		const houses: [number, number, number, number, string, string, string, string][] = [
			[-17, -116, 12, 8, C.red, C.tan, C.blue, 'BAKERY'],
			[17, -116, 12, 8, C.blue, C.white, C.red, 'INN'],
			[33, -116, 10, 8, C.darkGreen, C.yellow, C.redBrown, 'SHOP'],
			[-22, -137, 12, 9, C.orange, C.white, C.darkGreen, 'MARKET'],
			[22, -137, 12, 9, C.redBrown, C.gray, C.yellow, 'SMITH'],
			[36, -137, 9, 8, C.blue, C.tan, C.red, 'POST']
		];
		for (const [x, z, w, d, roof, wall, door, label] of houses) this.buildHouse(x, z, w, d, roof, wall, door, label);
		this.buildFountain(0, -140);

		// ---- the old castle at the north end ----
		{
			const hx = (27.7 * CASTLE.scale) / 2, hz = (21.9 * CASTLE.scale) / 2; // turned by 90 degrees
			this.blockers.push({ x0: CASTLE.x - hx, x1: CASTLE.x + hx, z0: CASTLE.z - hz, z1: CASTLE.z + hz });
			this.noStuds.push({ x0: CASTLE.x - hx, x1: CASTLE.x + hx, z0: CASTLE.z - hz, z1: CASTLE.z + hz });
			loadModel('/models/castle.bin').then((geo) => {
				const m = new THREE.Mesh(geo, new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.65 }));
				m.name = 'castle (lego city asset)';
				m.scale.setScalar(CASTLE.scale);
				m.rotation.y = Math.PI / 2;
				m.position.set(CASTLE.x, 0, CASTLE.z);
				m.castShadow = true;
				m.receiveShadow = true;
				this.scene.add(m);
			});
		}
	}
	/** a stone fountain with a little spout */
	private buildFountain(x: number, z: number) {
		const stone = this.mat(C.gray);
		const basin = new THREE.Mesh(new THREE.CylinderGeometry(3, 3.2, 1, 28), stone);
		basin.position.set(x, 0.9, z);
		basin.castShadow = basin.receiveShadow = true;
		const water = new THREE.Mesh(new THREE.CylinderGeometry(2.6, 2.6, 0.12, 28), new THREE.MeshPhysicalMaterial({ color: C.water, roughness: 0.05, clearcoat: 0.8, transparent: true, opacity: 0.75 }));
		water.position.set(x, 1.38, z);
		const pillar = new THREE.Mesh(new THREE.CylinderGeometry(0.45, 0.6, 2.6, 14), stone);
		pillar.position.set(x, 2.4, z);
		const bowl = new THREE.Mesh(new THREE.CylinderGeometry(1.3, 0.6, 0.5, 20), stone);
		bowl.position.set(x, 3.8, z);
		const spout = new THREE.Mesh(new THREE.CylinderGeometry(0.18, 0.35, 1.4, 10), new THREE.MeshPhysicalMaterial({ color: '#bfe6ff', transparent: true, opacity: 0.7, roughness: 0.1 }));
		spout.position.set(x, 4.6, z);
		for (const m of [pillar, bowl]) m.castShadow = true;
		this.scene.add(basin, water, pillar, bowl, spout);
		this.blockers.push({ x0: x - 3.3, x1: x + 3.3, z0: z - 3.3, z1: z + 3.3 });
	}


	private buildDecor() {
		// village sign
		const post = (x: number, z: number) => this.box(0.35, 3.2, 0.35, C.brown, x, 0, z);
		post(4.4, 20);
		post(7.4, 20);
		const sign = new THREE.Mesh(this.rbox(3.8, 2, 0.25), [
			this.mat(C.white), this.mat(C.white), this.mat(C.white), this.mat(C.white),
			new THREE.MeshStandardMaterial({ map: this.boardTexture('sign:village', 3.8, 2), roughness: 0.4 }),
			this.mat(C.white)
		]);
		sign.position.set(5.9, 2.3, 20.2);
		this.addSignView('sign:village', sign, 3.8, 2);
		sign.castShadow = true;
		this.scene.add(sign);
		this.blockers.push({ x0: 4, x1: 7.8, z0: 19.6, z1: 20.6 });

		const routeSign = new THREE.Mesh(new THREE.PlaneGeometry(3.4, 1.8), new THREE.MeshStandardMaterial({ map: this.boardTexture('sign:route', 3.4, 1.8), side: THREE.DoubleSide, roughness: 0.6 }));
		post(5, -24.5);
		routeSign.position.set(5, 2.2, -24.3);
		this.scene.add(routeSign);
		this.addSignView('sign:route', routeSign, 3.4, 1.8);
		this.blockers.push({ x0: 4.5, x1: 5.5, z0: -25, z1: -24 });

		// lamp posts
		for (const [x, z] of [[-4, -4], [4, 4], [-4, 10], [4, -14], [-22, 4], [22, -3.5]]) {
			this.box(0.35, 5, 0.35, C.black, x, 0, z);
			this.box(0.9, 0.2, 0.9, C.black, x, 5, z);
			const bulbMat = new THREE.MeshStandardMaterial({ color: '#fff5c0', emissive: '#ffe080', emissiveIntensity: 0.6, transparent: true, opacity: 0.9, roughness: 0.3 });
			const bulb = new THREE.Mesh(this.rbox(0.7, 0.8, 0.7), bulbMat);
			bulb.position.set(x, 5.6, z);
			this.scene.add(bulb);
			const lampLight = new THREE.PointLight('#ffd27a', 0, 16, 1.8);
			lampLight.position.set(x, 5.3, z);
			this.scene.add(lampLight);
			this.lamps.push({ mat: bulbMat, light: lampLight });
			this.box(1.0, 0.25, 1.0, C.black, x, 6.0, z, { studs: true });
			this.blockers.push({ x0: x - 0.4, x1: x + 0.4, z0: z - 0.4, z1: z + 0.4 });
		}
		// a few round brick trees inside the village
		for (const [x, z] of [[22, 20], [-24, -20], [8, -20], [-9, -20], [24, 4], [21, -21]]) {
			this.roundSpots.push({ x, z, s: 6.5, r: Math.random() * 6 });
			this.blockers.push({ x0: x - 0.9, x1: x + 0.9, z0: z - 0.9, z1: z + 0.9 });
		}
		// LEGO 30176 bamboo leaves: around the pond, along house walls, garden and the route edge
		const leaf = (x: number, z: number, s = 1.5 + Math.random() * 0.5) => this.bambooSpots.push({ x, y: 0, z, s });
		for (const [x, z] of [[-13, 15.5], [-13, 21], [-3.4, 17], [-12.6, 23.6], [-8, 23.8]]) leaf(x, z);
		for (const dx of [-5.6, -3.6, 3.4]) leaf(-15 + dx, -5.2, 1.2); // (5.6 removed)
		for (const dx of [-12, -8, 12, 15]) leaf(47 + dx, 8.6 + 2, 1.5);
		for (const dx of [-7, 5, 7]) leaf(13 + dx, 17.0, 1.25); // (-5 removed: overlapped -7)
		for (const [x, z] of [[-24, 8.5], [-24, 15.6], [-14, 15.6]]) leaf(x, z, 1.3);
		for (let z = -41; z <= -29; z += 2.4) {
			leaf(-9.2, z + Math.random());
			leaf(9.2, z + Math.random());
		}
		for (const [x, z] of [[-25.5, -4], [25.5, 8], [-25.5, 18], [25.5, -18], [6.5, -25.6], [-6.5, -25.6]]) leaf(x, z, 1.8);
		// flower beds (small LEGO flower plants)
		const bed = (x0: number, x1: number, z: number, cols: string[]) => {
			for (let x = x0; x <= x1; x += 2.1) this.flowerPlants.push({ x, z, c: cols[Math.floor(Math.random() * cols.length)] });
		};
		bed(4.2, 6.3, 18.6, [C.red, C.yellow]);
		bed(19.7, 21.8, 18.6, [C.red, C.yellow]);
		bed(-11.5, -4.8, 23.7, ['#ff8ac0', C.white, C.red]);
		for (const [x, z] of [[8, -9.5], [8, -16], [22, -16]]) this.flowerPlants.push({ x, z, c: C.orange });
		// LEGO flower pots: by the doors, the lab, the park and the pond
		for (const [x, z] of [[-11.8, -4.3], [-22.5, -4.3], [23.5, 12]]) this.flowerPot(x, z); // (the two in front of the lab were removed)
		// bench near the pond
		this.box(3, 0.3, 1, C.redBrown, -1 - 6.5, 0.9, 25 - 2.6);
		// crates near lab
		this.box(1.6, 1.6, 1.6, C.brown, 22.6, 0, 9, { studs: true });
		this.box(1.6, 1.6, 1.6, C.tan, 22.6, 1.6, 9, { studs: true });
		this.blockers.push({ x0: 21.6, x1: 23.6, z0: 8, z1: 10 });
	}

	private buildTallGrass() {
		// LEGO "plant grass stem" piece: 5 flat blades on a round 1x1 plate
		const parts: THREE.BufferGeometry[] = [];
		const plate = new THREE.CylinderGeometry(0.4, 0.4, 0.3, 10);
		plate.translate(0, 0.15, 0);
		parts.push(plate);
		for (let i = 0; i < 6; i++) {
			const b = new THREE.ConeGeometry(0.2, 2.0, 4);
			b.scale(1, 1, 0.3);
			b.translate(0, 1.0, 0);
			b.rotateZ(0.32 + (i % 2) * 0.12);
			b.rotateY((i / 6) * Math.PI * 2);
			parts.push(b);
		}
		const center = new THREE.ConeGeometry(0.2, 2.4, 4);
		center.scale(1, 1, 0.3);
		center.translate(0, 1.2, 0);
		parts.push(center);
		const geo = mergeGeometries(parts)!;
		this.grassData = [];
		for (const G of GRASSES)
		for (let x = G.x0 + 0.5; x < G.x1; x += 1.05)
			for (let z = G.z0 + 0.5; z < G.z1; z += 1.05) {
				this.grassData.push({ x: x + (Math.random() - 0.5) * 0.4, z: z + (Math.random() - 0.5) * 0.4, s: 0.8 + Math.random() * 0.5, r: Math.random() * 6, ph: Math.random() * 6 });
			}
		const im = new THREE.InstancedMesh(geo, this.mat('#ffffff'), this.grassData.length);
		const greens = ['#58ab41', '#4b9f4a', '#7fc04a', '#3c8c3a'];
		this.grassData.forEach((_, i) => im.setColorAt(i, new THREE.Color(greens[i % greens.length])));
		im.castShadow = true;
		im.receiveShadow = true;
		im.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
		this.scene.add(im);
		this.grass = im;
		im.name = 'tallgrass:tuft';
		this.updateGrass(0);
	}

	private golds: { id: string; obj: THREE.Object3D; taken: boolean; base: number }[] = [];
	private bubble: THREE.Sprite | null = null;
	private bubbleUntil = 0;

	private buildGolden() {
		const gold = new THREE.MeshStandardMaterial({ color: '#ffcc33', metalness: 0.85, roughness: 0.18, emissive: '#7a4a00', emissiveIntensity: 0.45 });
		const stud = new THREE.CylinderGeometry(0.55, 0.55, 0.32, 24);
		const top = new THREE.CylinderGeometry(0.42, 0.42, 0.12, 24);
		top.translate(0, 0.2, 0);
		const glowTex = (() => {
			const c = document.createElement('canvas');
			c.width = c.height = 64;
			const g = c.getContext('2d')!;
			const gr = g.createRadialGradient(32, 32, 0, 32, 32, 32);
			gr.addColorStop(0, 'rgba(255,240,150,0.9)');
			gr.addColorStop(1, 'rgba(255,200,50,0)');
			g.fillStyle = gr;
			g.fillRect(0, 0, 64, 64);
			return new THREE.CanvasTexture(c);
		})();
		for (const gs of GOLDEN_STUDS) {
			const grp = new THREE.Group();
			const a = new THREE.Mesh(stud, gold);
			const b = new THREE.Mesh(top, gold);
			a.rotation.x = b.rotation.x = Math.PI / 2;
			a.castShadow = true;
			const glow = new THREE.Sprite(new THREE.SpriteMaterial({ map: glowTex, depthWrite: false, transparent: true, blending: THREE.AdditiveBlending }));
			glow.scale.setScalar(2.4);
			grp.add(a, b, glow);
			const base = this.groundY(gs.x, gs.z) + 1.4;
			grp.position.set(gs.x, base, gs.z);
			this.scene.add(grp);
			this.golds.push({ id: gs.id, obj: grp, taken: false, base });
		}
	}

	/** Hides studs that were collected in an earlier session. */
	setCollected(ids: string[]) {
		for (const g of this.golds) {
			g.taken = ids.includes(g.id);
			g.obj.visible = !g.taken;
		}
	}

	/** Shows a "!" bubble above the player's head (wild encounter). */
	// ---------- NPC markers ("!" = matters for your goal, "…" = small talk) ----------
	private markers = new Map<string, THREE.Sprite>();
	private markerTex = new Map<string, THREE.Texture>();
	private markerTexture(kind: '!' | '…') {
		let t = this.markerTex.get(kind);
		if (t) return t;
		const c = document.createElement('canvas');
		c.width = c.height = 128;
		const g = c.getContext('2d')!;
		g.fillStyle = kind === '!' ? '#f2cd37' : '#ffffff';
		g.strokeStyle = '#1b2a34';
		g.lineWidth = 8;
		g.beginPath();
		g.roundRect(14, 10, 100, 88, 24);
		g.moveTo(52, 96);
		g.lineTo(64, 120);
		g.lineTo(76, 96);
		g.fill();
		g.stroke();
		g.fillStyle = kind === '!' ? '#c91a09' : '#1b2a34';
		g.font = 'bold 80px Arial';
		g.textAlign = 'center';
		g.fillText(kind === '!' ? '!' : '…', 64, kind === '!' ? 84 : 70);
		t = new THREE.CanvasTexture(c);
		t.colorSpace = THREE.SRGBColorSpace;
		this.markerTex.set(kind, t);
		return t;
	}
	/** Sets a speech marker per NPC id (null hides it). */
	setNpcMarkers(map: Record<string, '!' | '…' | null>) {
		for (const n of this.npcs) {
			const kind = map[n.def.id] ?? null;
			let sp = this.markers.get(n.def.id);
			if (!kind) {
				if (sp) sp.visible = false;
				continue;
			}
			if (!sp) {
				sp = new THREE.Sprite(new THREE.SpriteMaterial({ depthTest: false }));
				sp.renderOrder = 9;
				sp.name = `npc:${n.def.id}/marker`;
				this.scene.add(sp);
				this.markers.set(n.def.id, sp);
			}
			sp.material.map = this.markerTexture(kind);
			sp.material.needsUpdate = true;
			sp.userData.kind = kind;
			sp.visible = true;
		}
	}

	// ---------- little particles: dust when running, leaves in tall grass ----------
	private puffs: { sp: THREE.Sprite; v: THREE.Vector3; life: number; max: number }[] = [];
	private puffTex: THREE.Texture | null = null;
	private leafTex: THREE.Texture | null = null;
	private puffTimer = 0;
	private stepTimer = 0;
	private particleTexture(kind: 'dust' | 'leaf') {
		const c = document.createElement('canvas');
		c.width = c.height = 32;
		const g = c.getContext('2d')!;
		if (kind === 'dust') {
			const gr = g.createRadialGradient(16, 16, 0, 16, 16, 16);
			gr.addColorStop(0, 'rgba(255,255,255,0.9)');
			gr.addColorStop(1, 'rgba(255,255,255,0)');
			g.fillStyle = gr;
			g.fillRect(0, 0, 32, 32);
		} else {
			g.fillStyle = '#58ab41';
			g.beginPath();
			g.ellipse(16, 16, 13, 6, 0.6, 0, Math.PI * 2);
			g.fill();
			g.strokeStyle = '#2f7d2a';
			g.lineWidth = 2;
			g.beginPath();
			g.moveTo(6, 22);
			g.lineTo(26, 10);
			g.stroke();
		}
		const t = new THREE.CanvasTexture(c);
		t.colorSpace = THREE.SRGBColorSpace;
		return t;
	}
	private spawnPuff(kind: 'dust' | 'leaf', at: THREE.Vector3) {
		this.puffTex ??= this.particleTexture('dust');
		this.leafTex ??= this.particleTexture('leaf');
		let p = this.puffs.find((q) => q.life <= 0);
		if (!p) {
			if (this.puffs.length > 60) return;
			const sp = new THREE.Sprite(new THREE.SpriteMaterial({ transparent: true, depthWrite: false }));
			sp.name = 'fx:particle';
			this.scene.add(sp);
			p = { sp, v: new THREE.Vector3(), life: 0, max: 1 };
			this.puffs.push(p);
		}
		p.sp.material.map = kind === 'dust' ? this.puffTex : this.leafTex;
		p.sp.material.rotation = Math.random() * 6;
		p.sp.material.needsUpdate = true;
		p.sp.position.copy(at).add(new THREE.Vector3((Math.random() - 0.5) * 0.8, kind === 'dust' ? 0.3 : 1.2, (Math.random() - 0.5) * 0.8));
		p.v.set((Math.random() - 0.5) * 1.5, kind === 'dust' ? 0.8 : 2.4 + Math.random(), (Math.random() - 0.5) * 1.5);
		p.max = p.life = kind === 'dust' ? 0.55 : 0.9;
		p.sp.userData.kind = kind;
		p.sp.visible = true;
	}
	private updateParticles(dt: number, moving: boolean, running: boolean) {
		const p = this.player.group.position;
		const grassy = inGrass(p.x, p.z);
		this.puffTimer -= dt;
		if (moving && this.puffTimer <= 0 && (running || grassy)) {
			this.puffTimer = grassy ? 0.09 : 0.12;
			this.spawnPuff(grassy ? 'leaf' : 'dust', p);
		}
		for (const q of this.puffs) {
			if (q.life <= 0) continue;
			q.life -= dt;
			const k = Math.max(0, q.life / q.max);
			q.sp.position.addScaledVector(q.v, dt);
			if (q.sp.userData.kind === 'leaf') {
				q.v.y -= 6 * dt;
				q.sp.material.rotation += dt * 6;
			}
			q.sp.material.opacity = k;
			q.sp.scale.setScalar(q.sp.userData.kind === 'dust' ? 0.6 + (1 - k) * 1.2 : 0.55);
			if (q.life <= 0) q.sp.visible = false;
		}
	}

	exclaim(ms = 700) {
		if (!this.bubble) {
			const c = document.createElement('canvas');
			c.width = c.height = 128;
			const g = c.getContext('2d')!;
			g.fillStyle = '#fff';
			g.strokeStyle = '#1b2a34';
			g.lineWidth = 8;
			g.beginPath();
			g.roundRect(14, 8, 100, 92, 22);
			g.moveTo(54, 98);
			g.lineTo(64, 122);
			g.lineTo(76, 98);
			g.fill();
			g.stroke();
			g.fillStyle = '#c91a09';
			g.font = 'bold 84px Arial';
			g.textAlign = 'center';
			g.fillText('!', 64, 86);
			const tex = new THREE.CanvasTexture(c);
			tex.colorSpace = THREE.SRGBColorSpace;
			this.bubble = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, depthTest: false }));
			this.bubble.scale.setScalar(3.2);
			this.bubble.renderOrder = 10;
			this.scene.add(this.bubble);
		}
		this.bubble.visible = true;
		this.bubbleUntil = this.clock.elapsedTime + ms / 1000;
	}

	private updateExtras(t: number) {
		for (const n of this.npcs) {
			const sp = this.markers.get(n.def.id);
			if (!sp || !sp.visible) continue;
			const g = n.fig.group.position;
			const bob = sp.userData.kind === '!' ? Math.abs(Math.sin(t * 4)) * 0.35 : Math.sin(t * 2) * 0.1;
			sp.position.set(g.x, g.y + 6.4 + bob, g.z);
			sp.scale.setScalar(sp.userData.kind === '!' ? 2.1 : 1.7);
		}
		const p = this.player.group.position;
		for (const g of this.golds) {
			if (g.taken) continue;
			g.obj.rotation.y = t * 2.2;
			g.obj.position.y = g.base + Math.sin(t * 3 + g.obj.position.x) * 0.2;
			if (Math.hypot(g.obj.position.x - p.x, g.obj.position.z - p.z) < 1.4) {
				g.taken = true;
				g.obj.visible = false;
				this.cb.onGolden(g.id);
			}
		}
		if (this.bubble) {
			this.bubble.visible = t < this.bubbleUntil;
			this.bubble.position.set(p.x, p.y + 6.6 + Math.abs(Math.sin(t * 12)) * 0.25, p.z);
		}
	}

	// ---------- Pip walks a loop around the pond ----------
	private kidPath = [
		[-8, 13.2], [-3, 13.8], [-2.8, 23], [-11, 23.2], [-2.8, 23], [-3, 13.8]
	];
	private kidTarget = 1;
	private kidPause = 0;
	/** Pip loves the ball: when it lies near him and you're around, he runs over and passes it to you (or shoots at the goal) */
	private pipBall = { cool: 0, chasing: false, since: 0, side: 1 };
	private pipFootball(n: { def: NpcDef; fig: Minifig }, distToPlayer: number, dt: number, t: number) {
		const b = this.ball, pb = this.pipBall;
		if (!b || this.paused || this.area !== 'outside' || t < pb.cool) return (pb.chasing = false);
		const g = n.fig.group;
		const bp = b.m.position, p = this.player.group.position;
		const G = this.soccerGoal;
		const inGoal = bp.x > G.x0 - 0.5 && bp.z > G.z0 - 0.5 && bp.z < G.z1 + 0.5;
		const dBall = Math.hypot(bp.x - g.position.x, bp.z - g.position.z);
		const playerToBall = Math.hypot(bp.x - p.x, bp.z - p.z);
		const slow = Math.hypot(b.vx, b.vz) < 2.5;
		// only when the ball is close-ish to Pip, the player is around to play with, and nobody else has it
		if (!pb.chasing) {
			if (!(slow && !inGoal && dBall < 16 && distToPlayer < 20 && playerToBall > 3)) return false;
			pb.chasing = true;
			pb.since = t;
			this.sayBark('kid', pick(['My turn!', 'Ball! Ball!', "I've got it!"]), t);
		}
		if (t - pb.since > 8 || playerToBall < 1.8 || inGoal) {
			// give up (stuck, or the player got there first)
			pb.chasing = false;
			pb.cool = t + 4;
			return false;
		}
		if (dBall < 1.35) {
			// kick: pass to the player, or (sometimes, when there is a clear line) shoot at the goal
			const shoot = Math.random() < 0.35 && bp.x > 12;
			const tx = shoot ? (G.x0 + G.x1) / 2 : p.x, tz = shoot ? (G.z0 + G.z1) / 2 : p.z;
			const dx = tx - bp.x, dz = tz - bp.z, l = Math.hypot(dx, dz) || 1;
			const power = shoot ? 11 : Math.min(10, 3.5 + l * 0.55);
			b.vx = (dx / l) * power;
			b.vz = (dz / l) * power;
			b.lastKick = t;
			this.cb.onKick?.(Math.min(1, power / 14));
			this.sayBark('kid', shoot ? pick(['Shoot!', 'Watch this!']) : pick(['Pass!', 'Your turn!', 'Catch!']), t);
			pb.chasing = false;
			pb.cool = t + 5;
			return true;
		}
		// run to the ball (aim slightly behind it so the kick goes the right way)
		const dx = bp.x - g.position.x, dz = bp.z - g.position.z;
		const base = Math.atan2(dx, dz);
		const sp = 6.5 * dt;
		let moved = false;
		// keep a steering side once picked, so he walks around posts instead of jittering in front of them
		const offs = pb.side > 0 ? [0, 0.6, 1.2, 1.8, 2.4, -0.6, -1.2] : [0, -0.6, -1.2, -1.8, -2.4, 0.6, 1.2];
		for (const off of offs) {
			const a = base + off;
			const nx = g.position.x + Math.sin(a) * sp, nz = g.position.z + Math.cos(a) * sp;
			if (!this.walkableFor(nx, nz, n)) continue;
			if (off !== 0) pb.side = Math.sign(off);
			g.position.x = nx;
			g.position.z = nz;
			let diff = a - g.rotation.y;
			diff = Math.atan2(Math.sin(diff), Math.cos(diff));
			g.rotation.y += diff * Math.min(1, dt * 10);
			moved = true;
			break;
		}
		if (!moved) {
			pb.chasing = false;
			pb.cool = t + 4;
			return false;
		}
		n.def.x = g.position.x;
		n.def.z = g.position.z;
		g.position.y = this.groundY(g.position.x, g.position.z);
		n.fig.animate(dt * 1.4, true);
		return true;
	}

	/** returns true when the NPC was moved/animated this frame */
	// ---------- sitting on the park bench: watch time fly by ----------
	private seated: { x: number; z: number; y: number; rot: number; standX: number; standZ: number } | null = null;
	private static BENCHES: Record<string, { x: number; z: number; y: number; rot: number }> = {
		// seat top is at y 1.4; the figure's hips sit on it (the legs pivot 1.55 above the feet)
		'bench:park': { x: 15, z: -9.4, y: 1.4 - 1.15, rot: 0 }
	};
	/** sits the player down on a bench; returns false if that's not a bench */
	sit(id: string) {
		const b = Town.BENCHES[id];
		if (!b || this.seated) return false;
		const p = this.player.group.position;
		this.seated = { ...b, standX: p.x, standZ: p.z };
		p.set(b.x, b.y, b.z);
		this.player.group.rotation.y = b.rot;
		this.clearKeys();
		this.cb.onSit?.(true);
		return true;
	}
	get isSeated() {
		return !!this.seated;
	}
	private standUp() {
		const s = this.seated;
		if (!s) return;
		this.seated = null;
		const p = this.player.group.position;
		p.set(s.standX, this.groundY(s.standX, s.standZ), s.standZ);
		this.player.legL.rotation.x = this.player.legR.rotation.x = 0;
		this.cb.onSit?.(false);
	}
	private sitPose(dt: number) {
		const f = this.player, k = Math.min(1, dt * 8);
		// legs forward, hands resting on the knees, and a slow happy foot swing
		const swing = Math.sin(this.clock.elapsedTime * 2.2) * 0.12;
		f.legL.rotation.x += (-Math.PI / 2 + swing - f.legL.rotation.x) * k;
		f.legR.rotation.x += (-Math.PI / 2 - swing - f.legR.rotation.x) * k;
		for (const a of [f.armL, f.armR]) a.rotation.x += (-0.9 - a.rotation.x) * k;
		this.updateParticles(dt, false, false);
	}

	// ---------- a duck family on the pond: mother leads, ducklings follow; they paddle over when you come by ----------
	private ducks: { g: THREE.Group; s: number; trail: THREE.Vector3[] }[] = [];
	private duckGoal = new THREE.Vector2(-8, 18.5);
	private duckNextGoal = 0;
	private duckQuack = 0;
	private makeDuck(body: string, s: number) {
		const g = new THREE.Group();
		const add = (w: number, h: number, d: number, c: string, x: number, y: number, z: number) => {
			const m = new THREE.Mesh(this.rbox(w, h, d), this.mat(c));
			m.position.set(x, y, z);
			m.castShadow = true;
			g.add(m);
			return m;
		};
		add(0.9, 0.5, 1.3, body, 0, 0.25, 0); // body
		add(0.6, 0.35, 0.4, body, 0, 0.55, -0.55); // tail up
		add(0.55, 0.55, 0.55, body, 0, 0.85, 0.5); // head
		add(0.35, 0.14, 0.35, C.orange, 0, 0.8, 0.9); // beak
		for (const x of [-0.2, 0.2]) add(0.1, 0.1, 0.06, '#1b2a34', x, 0.95, 0.78); // eyes
		g.scale.setScalar(s);
		g.name = 'animal:duck';
		g.traverse((o) => o.layers.enable(Town.REFLECT)); // mirrored in the water like the figures
		this.scene.add(g);
		return g;
	}
	private buildDucks() {
		const start = new THREE.Vector3(-8, 0.12, 18.5);
		this.ducks.push({ g: this.makeDuck(C.white, 0.95), s: 0.95, trail: [] });
		for (let i = 0; i < 3; i++) this.ducks.push({ g: this.makeDuck(C.yellow, 0.52), s: 0.52, trail: [] });
		this.ducks.forEach((d, i) => d.g.position.copy(start).add(new THREE.Vector3(0, 0, -i * 0.9)));
	}
	private updateDucks(dt: number, t: number) {
		if (!this.ducks.length || this.area !== 'outside') return;
		const { x0, x1, z0, z1 } = POND;
		const m = 1.0; // keep away from the border
		const p = this.player.group.position;
		const mom = this.ducks[0];
		// the player near the pond: swim to the closest water spot ("bread?!")
		const nearX = THREE.MathUtils.clamp(p.x, x0 + m, x1 - m), nearZ = THREE.MathUtils.clamp(p.z, z0 + m, z1 - m);
		const playerNear = Math.hypot(p.x - nearX, p.z - nearZ) < 4.5;
		if (playerNear) this.duckGoal.set(nearX, nearZ);
		else if (t > this.duckNextGoal) {
			this.duckGoal.set(x0 + m + Math.random() * (x1 - x0 - 2 * m), z0 + m + Math.random() * (z1 - z0 - 2 * m));
			this.duckNextGoal = t + 5 + Math.random() * 6;
		}
		if (playerNear && t > this.duckQuack) {
			this.duckQuack = t + 2.5 + Math.random() * 3;
			this.cb.onQuack?.(1);
			if (Math.random() < 0.6) setTimeout(() => this.cb.onQuack?.(1.6 + Math.random() * 0.3), 220);
		}
		// mother: steer toward the goal, slow down near it
		const mp = mom.g.position;
		const dx = this.duckGoal.x - mp.x, dz = this.duckGoal.y - mp.z;
		const dist = Math.hypot(dx, dz);
		if (dist > 0.15) {
			const sp = Math.min(dist, (playerNear ? 2.2 : 1.1) * dt);
			const want = Math.atan2(dx, dz);
			let diff = want - mom.g.rotation.y;
			diff = Math.atan2(Math.sin(diff), Math.cos(diff));
			mom.g.rotation.y += diff * Math.min(1, dt * 3);
			mp.x += Math.sin(mom.g.rotation.y) * sp;
			mp.z += Math.cos(mom.g.rotation.y) * sp;
		} else if (playerNear) {
			// face the player and wait for treats
			const want = Math.atan2(p.x - mp.x, p.z - mp.z);
			let diff = want - mom.g.rotation.y;
			diff = Math.atan2(Math.sin(diff), Math.cos(diff));
			mom.g.rotation.y += diff * Math.min(1, dt * 3);
		}
		mp.x = THREE.MathUtils.clamp(mp.x, x0 + 0.6, x1 - 0.6);
		mp.z = THREE.MathUtils.clamp(mp.z, z0 + 0.6, z1 - 0.6);
		// bread in the water: everybody races to the nearest crumb and gobbles it up
		if (this.updateCrumbs(dt, t)) return this.bobDucks(t);
		// ducklings: each follows the one in front at a short distance, in a little line
		for (let i = 1; i < this.ducks.length; i++) {
			const lead = this.ducks[i - 1].g.position, d = this.ducks[i].g;
			const fx = lead.x - d.position.x, fz = lead.z - d.position.z;
			const fd = Math.hypot(fx, fz);
			const gap = i === 1 ? 1.15 : 0.75;
			if (fd > gap) {
				const k = Math.min(1, ((fd - gap) / fd) * dt * 4);
				d.position.x += fx * k;
				d.position.z += fz * k;
				let diff = Math.atan2(fx, fz) - d.rotation.y;
				diff = Math.atan2(Math.sin(diff), Math.cos(diff));
				d.rotation.y += diff * Math.min(1, dt * 6);
			}
		}
		this.bobDucks(t);
	}
	private bobDucks(t: number) {
		// gentle bobbing on the water
		this.ducks.forEach((d, i) => {
			d.g.position.y = 0.16 + Math.sin(t * 2.4 + i * 1.3) * 0.03;
			d.g.rotation.z = Math.sin(t * 1.8 + i) * 0.05;
		});
	}
	// ---------- feeding the ducks ----------
	private crumbs: { m: THREE.Mesh; from: THREE.Vector3; to: THREE.Vector3; t0: number; landed: boolean }[] = [];
	private lastFeed = -99;
	/** throws a few bread crumbs into the pond (returns false while still on cooldown) */
	feedDucks() {
		const t = this.clock.elapsedTime;
		if (t - this.lastFeed < 3 || this.crumbs.length) return false;
		this.lastFeed = t;
		const { x0, x1, z0, z1 } = POND;
		const p = this.player.group.position;
		const cx = THREE.MathUtils.clamp(p.x, x0 + 1, x1 - 1), cz = THREE.MathUtils.clamp(p.z, z0 + 1, z1 - 1);
		// face the water and toss
		this.player.group.rotation.y = Math.atan2(cx - p.x, cz - p.z);
		this.player.armR.rotation.x = -2.2;
		const geo = new THREE.BoxGeometry(0.3, 0.16, 0.24);
		const mat = this.mat('#d9a35b');
		for (let i = 0; i < 5; i++) {
			const m = new THREE.Mesh(geo, mat);
			m.name = 'fx:bread';
			const to = new THREE.Vector3(
				THREE.MathUtils.clamp(cx + (Math.random() - 0.5) * 3.5 - Math.sign(p.x - cx) * 1.2, x0 + 0.6, x1 - 0.6),
				0.24,
				THREE.MathUtils.clamp(cz + (Math.random() - 0.5) * 3.5 - Math.sign(p.z - cz) * 1.2, z0 + 0.6, z1 - 0.6)
			);
			const from = new THREE.Vector3(p.x, p.y + 2.6, p.z);
			m.position.copy(from);
			this.scene.add(m);
			this.crumbs.push({ m, from, to, t0: t + i * 0.08, landed: false });
		}
		this.cb.onQuack?.(1);
		return true;
	}
	/** moves crumbs and ducks while there is bread; returns true while the ducks are busy eating */
	private updateCrumbs(dt: number, t: number) {
		if (!this.crumbs.length) return false;
		for (const c of this.crumbs) {
			const k = THREE.MathUtils.clamp((t - c.t0) / 0.6, 0, 1);
			if (!c.landed) {
				c.m.position.lerpVectors(c.from, c.to, k);
				c.m.position.y += Math.sin(k * Math.PI) * 1.6; // little arc
				c.m.rotation.x += dt * 9;
				if (k >= 1) {
					c.landed = true;
					c.m.rotation.set(0, Math.random() * 3, 0);
					this.splash(c.to.x, c.to.z, t, 0.24);
				}
			} else c.m.position.y = 0.22 + Math.sin(t * 3 + c.to.x) * 0.02;
		}
		// each duck swims to its nearest floating crumb (the little ones are quicker!)
		this.ducks.forEach((d, i) => {
			const dp = d.g.position;
			let best: (typeof this.crumbs)[number] | null = null, bd = Infinity;
			for (const c of this.crumbs) {
				if (!c.landed) continue;
				const dd = Math.hypot(c.m.position.x - dp.x, c.m.position.z - dp.z);
				if (dd < bd) (bd = dd), (best = c);
			}
			if (!best) return;
			const dx = best.m.position.x - dp.x, dz = best.m.position.z - dp.z;
			if (bd < 0.45 * (d.s + 0.4)) {
				// gobble!
				this.scene.remove(best.m);
				this.crumbs.splice(this.crumbs.indexOf(best), 1);
				this.cb.onQuack?.(i === 0 ? 1 : 1.6 + Math.random() * 0.3);
				d.g.userData.gulp = t;
				return;
			}
			let diff = Math.atan2(dx, dz) - d.g.rotation.y;
			diff = Math.atan2(Math.sin(diff), Math.cos(diff));
			d.g.rotation.y += diff * Math.min(1, dt * 7);
			const sp = Math.min(bd, (i === 0 ? 2.4 : 2.9) * dt);
			dp.x += (dx / bd) * sp;
			dp.z += (dz / bd) * sp;
		});
		// a happy little hop after each gulp
		for (const d of this.ducks) {
			const age = t - (d.g.userData.gulp ?? -9);
			if (age < 0.3) d.g.rotation.x = -Math.sin((age / 0.3) * Math.PI) * 0.5;
			else d.g.rotation.x = 0;
		}
		return true;
	}

	// ---------- hide and seek with Pip ----------
	hideSeek = { phase: 'idle' as 'idle' | 'hiding' | 'resting', until: 0, nextHint: 0, x: 0, z: 0 };
	/** spots on the village lawn the player can walk to from where they stand (flood fill on a 1-stud grid) */
	private reachableSpots() {
		const R = WALK[0];
		const p = this.player.group.position;
		const W = Math.floor(R.x1 - R.x0), H = Math.floor(R.z1 - R.z0);
		const seen = new Uint8Array(W * H);
		const sx = Math.floor(p.x - R.x0), sz = Math.floor(p.z - R.z0);
		const q: number[] = [];
		if (sx >= 0 && sz >= 0 && sx < W && sz < H) {
			seen[sz * W + sx] = 1;
			q.push(sx, sz);
		}
		const out: { x: number; z: number }[] = [];
		while (q.length) {
			const cz = q.pop()!, cx = q.pop()!;
			out.push({ x: R.x0 + cx + 0.5, z: R.z0 + cz + 0.5 });
			for (const [dx, dz] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
				const nx = cx + dx, nz = cz + dz;
				if (nx < 0 || nz < 0 || nx >= W || nz >= H || seen[nz * W + nx]) continue;
				seen[nz * W + nx] = 1;
				if (this.walkable(R.x0 + nx + 0.5, R.z0 + nz + 0.5)) q.push(nx, nz);
			}
		}
		return out;
	}
	/** Pip runs off and hides: tucked against a house, tree or fence, far from you, never on the road */
	startHideSeek() {
		const kid = this.npcs.find((n) => n.def.id === 'kid');
		if (!kid || this.area !== 'outside' || this.hideSeek.phase === 'hiding') return false;
		const p = this.player.group.position;
		const near = (x: number, z: number) => this.blockers.some((b) => x > b.x0 - 1.6 && x < b.x1 + 1.6 && z > b.z0 - 1.6 && z < b.z1 + 1.6);
		const spots = this.reachableSpots().filter((s) => {
			const d = Math.hypot(s.x - p.x, s.z - p.z);
			return d > 16 && d < 45 && !this.onRoad(s.x, s.z) && near(s.x, s.z) && this.walkableFor(s.x, s.z, kid);
		});
		if (!spots.length) return false;
		const s = pick(spots);
		const t = this.clock.elapsedTime;
		Object.assign(this.hideSeek, { phase: 'hiding', until: t + 63, nextHint: t + 9, x: s.x, z: s.z });
		this.tag.phase = 'idle';
		this.pipBall.chasing = false;
		const g = kid.fig.group;
		g.position.set(s.x, this.groundY(s.x, s.z), s.z);
		kid.def.x = s.x;
		kid.def.z = s.z;
		return true;
	}
	private updateHideSeek(n: { def: NpcDef; fig: Minifig }, d: number, t: number) {
		const h = this.hideSeek;
		if (h.phase === 'idle') return false;
		const g = n.fig.group;
		if (h.phase === 'resting') {
			// once you've walked away, he quietly goes back to the pond
			if (d > 24 || this.area !== 'outside') {
				h.phase = 'idle';
				const [x, z] = this.kidPath[0];
				g.position.set(x, this.groundY(x, z), z);
				n.def.x = x;
				n.def.z = z;
				this.kidTarget = 1;
				return false;
			}
			n.fig.wave(t);
			return true;
		}
		// hiding: crouched, peeking around
		g.rotation.y = Math.sin(t * 0.7) * 1.2;
		g.position.y = this.groundY(h.x, h.z) - 0.5 + Math.abs(Math.sin(t * 3)) * 0.08;
		if (this.paused) return true;
		if (d < 3.2) {
			h.phase = 'resting';
			g.position.y = this.groundY(h.x, h.z);
			this.sayBark('kid', 'Aww, you found me!', t);
			this.cb.onHideSeek?.('found');
			return true;
		}
		if (t > h.until) {
			h.phase = 'resting';
			g.position.y = this.groundY(h.x, h.z);
			this.sayBark('kid', 'Here I am! Hehe!', t);
			this.cb.onHideSeek?.('timeout');
			return true;
		}
		if (t > h.nextHint) {
			h.nextHint = t + 8;
			const hint = d < 8 ? pick(['Hot! HOT!', 'Uh oh, so close!', 'Hot!!']) : d < 16 ? pick(['Warm...', 'Getting warmer!', 'Warm!']) : pick(['Cold!', 'Brrr, ice cold!', 'Freezing!']);
			this.sayBark('kid', hint, t);
			this.cb.onHideSeek?.('hint', hint);
		}
		return true;
	}

	private wander(n: { def: NpcDef; fig: Minifig; baseRot: number }, distToPlayer: number, dt: number) {
		if (distToPlayer < 6 || this.paused) return false; // stand still and look at the player
		const g = n.fig.group;
		if (this.kidPause > 0) {
			this.kidPause -= dt;
			n.fig.wave(this.clock.elapsedTime);
			return true;
		}
		const [tx, tz] = this.kidPath[this.kidTarget];
		const dx = tx - g.position.x, dz = tz - g.position.z;
		const dist = Math.hypot(dx, dz);
		if (dist < 0.3) {
			this.kidTarget = (this.kidTarget + 1) % this.kidPath.length;
			if (Math.random() < 0.5) this.kidPause = 1.5 + Math.random() * 2;
			return true;
		}
		const sp = Math.min(dist, 3.2 * dt);
		g.position.x += (dx / dist) * sp;
		g.position.z += (dz / dist) * sp;
		n.def.x = g.position.x;
		n.def.z = g.position.z;
		const want = Math.atan2(dx, dz);
		let diff = want - g.rotation.y;
		diff = Math.atan2(Math.sin(diff), Math.cos(diff));
		g.rotation.y += diff * Math.min(1, dt * 8);
		g.position.y = this.groundY(g.position.x, g.position.z);
		n.fig.animate(dt, true);
		return true;
	}

	// ---------- butterflies ----------
	private butterflies: { sp: THREE.Sprite; cx: number; cz: number; r: number; ph: number; sp2: number; caught?: number; back?: number; from?: THREE.Vector3 }[] = [];
	private buildButterflies() {
		const colors = ['#ff8ac0', '#f2cd37', '#6fb6ff', '#ffffff', '#fe8a18'];
		const tex = colors.map((c) => {
			const cv = document.createElement('canvas');
			cv.width = cv.height = 64;
			const g = cv.getContext('2d')!;
			g.fillStyle = c;
			g.strokeStyle = '#1b2a34';
			g.lineWidth = 3;
			for (const s of [-1, 1]) {
				g.beginPath();
				g.ellipse(32 + s * 14, 24, 13, 15, s * 0.4, 0, Math.PI * 2);
				g.fill();
				g.stroke();
				g.beginPath();
				g.ellipse(32 + s * 11, 44, 9, 10, -s * 0.3, 0, Math.PI * 2);
				g.fill();
				g.stroke();
			}
			g.fillStyle = '#1b2a34';
			g.fillRect(30, 14, 4, 38);
			const t = new THREE.CanvasTexture(cv);
			t.colorSpace = THREE.SRGBColorSpace;
			return t;
		});
		const spots = [
			[-19, 12, 4], [-17, 11, 3], [15, -11, 4], [0, -36, 5], [-4, -32, 4], [5, -38, 4], [45, 16, 5], [-8, 19, 3]
		];
		spots.forEach(([cx, cz, r], i) => {
			const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex[i % tex.length], transparent: true, depthWrite: false }));
			sp.name = `fx:butterfly#${i}`;
			sp.scale.set(0.9, 0.9, 1);
			this.scene.add(sp);
			this.butterflies.push({ sp, cx, cz, r, ph: Math.random() * 6, sp2: 0.4 + Math.random() * 0.4 });
		});
	}
	private updateButterflies(t: number) {
		const p = this.player.group.position;
		for (const b of this.butterflies) {
			const mat = b.sp.material;
			// caught: spiral up and away, stay gone for a while, then flutter back in
			if (b.caught !== undefined) {
				const k = t - b.caught;
				if (k < 1.6) {
					const f = b.from!;
					b.sp.position.set(f.x + Math.sin(k * 9) * k * 1.2, f.y + k * k * 3.5, f.z + Math.cos(k * 9) * k * 1.2);
					b.sp.scale.x = 0.9 * (0.25 + Math.abs(Math.sin(t * 30 + b.ph)) * 0.75);
					mat.opacity = 1 - k / 1.6;
					continue;
				}
				if (t < b.back!) {
					b.sp.visible = false;
					continue;
				}
				b.caught = undefined;
				b.sp.visible = true;
			}
			const a = t * b.sp2 + b.ph;
			b.sp.position.set(b.cx + Math.sin(a) * b.r, 2.2 + Math.sin(a * 2.3) * 0.8, b.cz + Math.sin(a * 1.7) * b.r * 0.6);
			mat.opacity = Math.min(1, mat.opacity + 0.03); // fades back in after returning
			// flapping: squash the sprite horizontally
			b.sp.scale.x = 0.9 * (0.25 + Math.abs(Math.sin(t * 14 + b.ph)) * 0.75);
			// walk into it to "catch" it
			if (!this.paused && this.area === 'outside' && mat.opacity > 0.9 && Math.hypot(b.sp.position.x - p.x, b.sp.position.z - p.z) < 1.3) {
				b.caught = t;
				b.back = t + 25 + Math.random() * 10;
				b.from = b.sp.position.clone();
				this.cb.onButterfly?.();
			}
		}
	}

	// ---------- day / night ----------
	private hemi!: THREE.HemisphereLight;
	private lamps: { mat: THREE.MeshStandardMaterial; light: THREE.PointLight }[] = [];
	private fireflies: { sp: THREE.Sprite; x: number; z: number; ph: number; caught?: number }[] = [];
	/** length of a full day in seconds */
	dayLength = 360;
	/** how many in-game midnights have passed this session */
	dayCount = 0;
	/** 0..1 time of day (0.25 = noon, 0.75 = midnight); starts mid-morning */
	timeOfDay = 0.15;
	/** 0 = night, 1 = full day */
	daylight = 1;
	private skyDay = new THREE.Color('#bfe6ff');
	private skyDusk = new THREE.Color('#ffb98a');
	private skyNight = new THREE.Color('#1c2d52');
	private tmpC = new THREE.Color();
	private buildFireflies() {
		const c = document.createElement('canvas');
		c.width = c.height = 32;
		const g = c.getContext('2d')!;
		const gr = g.createRadialGradient(16, 16, 0, 16, 16, 16);
		gr.addColorStop(0, 'rgba(255,255,200,1)');
		gr.addColorStop(0.3, 'rgba(220,255,120,0.8)');
		gr.addColorStop(1, 'rgba(180,255,80,0)');
		g.fillStyle = gr;
		g.fillRect(0, 0, 32, 32);
		const tex = new THREE.CanvasTexture(c);
		const spots: [number, number][] = [];
		for (let i = 0; i < 14; i++) spots.push([-8 + Math.random() * 16, -40 + Math.random() * 11]); // tall grass
		for (let i = 0; i < 10; i++) spots.push([-13 + Math.random() * 10, 14 + Math.random() * 9]); // pond
		for (let i = 0; i < 6; i++) spots.push([10 + Math.random() * 10, -15 + Math.random() * 7]); // park
		for (const [x, z] of spots) {
			const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, transparent: true, depthWrite: false, blending: THREE.AdditiveBlending }));
			sp.scale.setScalar(1.1);
			sp.name = 'fx:firefly';
			sp.visible = false;
			this.scene.add(sp);
			this.fireflies.push({ sp, x, z, ph: Math.random() * 10 });
		}
	}
	private updateDayNight(dt: number, t: number) {
		const tod = this.timeOfDay + dt / this.dayLength;
		if (tod >= 1) this.dayCount++; // a new day begins (at midnight)
		this.timeOfDay = tod % 1;
		const sunH = Math.sin(this.timeOfDay * Math.PI * 2); // 1 at noon, -1 at midnight
		const day = THREE.MathUtils.clamp(sunH * 1.6 + 0.35, 0, 1);
		this.daylight = day;
		const outside = this.area === 'outside';
		if (outside) {
			// sky: night -> dusk -> day
			const dusk = 1 - Math.min(1, Math.abs(sunH) * 3.5); // peaks around sunrise / sunset
			this.tmpC.copy(this.skyNight).lerp(this.skyDay, day).lerp(this.skyDusk, dusk * 0.55);
			// a rain shower greys the sky a little
			if (this.rain.level > 0) this.tmpC.lerp(this.rainSky, this.rain.level * 0.45 * (0.3 + 0.7 * day));
			(this.scene.background as THREE.Color).copy(this.tmpC);
			(this.scene.fog as THREE.Fog).color.copy(this.tmpC);
			this.sun.intensity = (0.15 + 2.25 * day) * (1 - this.rain.level * 0.45);
			// moonlight is cool blue, daylight warm
			this.sun.color.setHSL(day > 0.3 ? 0.1 : 0.6, 0.6, 0.55 + 0.4 * day);
			this.hemi.intensity = 0.28 + 0.82 * day;
			this.hemi.color.setHSL(day > 0.3 ? 0.58 : 0.62, 0.6, 0.6 + 0.3 * day);
			this.scene.environmentIntensity = 0.1 + 0.45 * day;
		} else {
			// rooms keep their indoor lighting regardless of the time of day
			this.sun.intensity = 2.4;
			this.sun.color.set('#fff6e0');
			this.hemi.intensity = 1.1;
			this.scene.environmentIntensity = 0.55;
		}
		const night = 1 - day;
		for (const l of this.lamps) {
			l.light.intensity = outside ? night * 90 : 0;
			l.mat.emissiveIntensity = 0.4 + night * 2.2;
		}
		const pp = this.player.group.position;
		const walking = this.keys.size > 0 || !!this.virt.x || !!this.virt.z;
		for (const f of this.fireflies) {
			f.sp.visible = outside && night > 0.25;
			if (!f.sp.visible) continue;
			// caught: it zips into your hands, glows bright and is gone; a new one shows up after a while
			if (f.caught !== undefined) {
				const age = t - f.caught;
				if (age < 0.6) {
					f.sp.position.lerp(this.tmpV.set(pp.x, pp.y + 3.2, pp.z), Math.min(1, age * 0.35 + 0.08));
					f.sp.scale.setScalar(1.1 + age * 2.5);
					f.sp.material.opacity = 1 - age / 0.6;
					continue;
				}
				if (age < 30) {
					f.sp.visible = false;
					continue;
				}
				f.caught = undefined;
				f.sp.scale.setScalar(1.1);
			}
			const a = t * 0.6 + f.ph;
			f.sp.position.set(f.x + Math.sin(a) * 1.4, 1.2 + Math.sin(a * 1.7) * 0.6, f.z + Math.cos(a * 0.8) * 1.4);
			f.sp.material.opacity = night * (0.4 + 0.6 * Math.abs(Math.sin(t * 2.5 + f.ph)));
			if (walking && !this.paused && night > 0.5 && Math.hypot(f.sp.position.x - pp.x, f.sp.position.z - pp.z) < 1.2) {
				f.caught = t;
				this.cb.onFirefly?.();
			}
		}
	}

	// ---------- goal beacon: a light pillar with a bouncing arrow over the current objective ----------
	private beacon: THREE.Group | null = null;
	private goal: string | null = null;
	/** where each goal is, depending on which area the player is in */
	private goalPos(goal: string): { x: number; z: number; y: number } | null {
		const npc = (id: string) => {
			const n = this.npcs.find((q) => q.def.id === id);
			return n ? { x: n.fig.group.position.x, z: n.fig.group.position.z, y: n.fig.group.position.y + 7.2 } : null;
		};
		const door = (id: AreaId) => {
			const r = roomById(id);
			return { x: r.door.x, z: r.door.z + 1.2, y: 6.5 };
		};
		// inside a room that isn't the goal: point at the way out
		const exit = () => {
			if (this.area === 'outside') return null;
			if (this.area === 'bedroom') return { x: (BED_WELL.x0 + BED_WELL.x1) / 2, z: (BED_WELL.z0 + BED_WELL.z1) / 2, y: 4 };
			const r = roomById(this.area);
			return { x: r.ox, z: r.oz + r.D / 2 - 0.8, y: 4 };
		};
		switch (goal) {
			case 'mom':
				return this.area === 'home' ? npc('mom') : this.area === 'outside' ? door('home') : exit();
			case 'prof':
				return this.area === 'lab' ? npc('prof') : this.area === 'outside' ? door('lab') : exit();
			case 'rival':
				return this.area === 'outside' ? npc('rival') : exit();
			case 'grass':
				return this.area === 'outside' ? { x: 0, z: -34, y: 4 } : exit();
		}
		return null;
	}
	setGoal(goal: string | null) {
		this.goal = goal;
	}
	private buildBeacon() {
		const g = new THREE.Group();
		g.name = 'fx:goal-beacon';
		// light pillar that fades out upwards (alpha gradient along the cylinder's v coordinate)
		const H = 9;
		const grad = document.createElement('canvas');
		grad.width = 4;
		grad.height = 128;
		const gc = grad.getContext('2d')!;
		const lg = gc.createLinearGradient(0, 128, 0, 0); // canvas bottom = v 0 = foot of the pillar
		lg.addColorStop(0, '#ffffff');
		lg.addColorStop(0.1, '#a0a0a0');
		lg.addColorStop(0.32, '#2a2a2a');
		lg.addColorStop(0.6, '#000000');
		gc.fillStyle = lg;
		gc.fillRect(0, 0, 4, 128);
		const alphaMap = new THREE.CanvasTexture(grad);
		const pillar = new THREE.Mesh(
			new THREE.CylinderGeometry(0.75, 0.75, H, 24, 1, true),
			new THREE.MeshBasicMaterial({ color: '#ffe066', transparent: true, opacity: 0.28, alphaMap, side: THREE.DoubleSide, depthWrite: false, blending: THREE.AdditiveBlending })
		);
		pillar.position.y = H / 2;
		const arrowMat = new THREE.MeshStandardMaterial({ color: '#ffd23d', emissive: '#ffb000', emissiveIntensity: 0.8, roughness: 0.3, transparent: true });
		this.beaconMats = { pillar: pillar.material as THREE.MeshBasicMaterial, arrow: arrowMat };
		const head = new THREE.Mesh(new THREE.ConeGeometry(0.7, 1.2, 4), arrowMat);
		head.rotation.x = Math.PI;
		const shaft = new THREE.Mesh(new THREE.BoxGeometry(0.4, 1.0, 0.4), arrowMat);
		shaft.position.y = 1.05;
		const arrow = new THREE.Group();
		arrow.add(head, shaft);
		arrow.name = 'arrow';
		g.add(pillar, arrow);
		g.visible = false;
		this.scene.add(g);
		this.beacon = g;
	}
	private beaconMats: { pillar: THREE.MeshBasicMaterial; arrow: THREE.MeshStandardMaterial } | null = null;
	private beaconAlpha = 0;
	private beaconT = 0;
	private updateBeacon(t: number) {
		if (!this.beacon || !this.beaconMats) return;
		const pos = this.goal ? this.goalPos(this.goal) : null;
		const p = this.player.group.position;
		// fades out smoothly as you walk up to the goal and back in when you walk away
		const d = pos ? Math.hypot(pos.x - p.x, pos.z - p.z) : 0;
		const k = pos ? THREE.MathUtils.smoothstep(d, 2.5, 8) : 0;
		const dt = Math.min(0.05, t - (this.beaconT || t));
		this.beaconT = t;
		this.beaconAlpha += (k - this.beaconAlpha) * Math.min(1, dt * 6);
		this.beaconMats.pillar.opacity = 0.28 * this.beaconAlpha;
		this.beaconMats.arrow.opacity = this.beaconAlpha;
		this.beaconMats.arrow.depthWrite = this.beaconAlpha > 0.95;
		this.beacon.visible = !!pos && this.beaconAlpha > 0.01;
		if (!pos) return;
		this.beacon.position.set(pos.x, 0, pos.z);
		const arrow = this.beacon.getObjectByName('arrow')!;
		arrow.position.y = pos.y + Math.abs(Math.sin(t * 3)) * 0.8;
		arrow.rotation.y = t * 1.5;
	}

	// ---------- sign close-up ----------
	private signViews = new Map<string, { mesh: THREE.Mesh; w: number; h: number }>();
	/** id of the sign currently read in close-up */
	reading: string | null = null;
	private focusPos = new THREE.Vector3();
	private focusLook = new THREE.Vector3();
	private focusBlend = 0;
	private focusDist = 3;
	private camLook = new THREE.Vector3();
	/** zooms the camera right in front of a sign (first person) so its text is readable */
	readSign(id: string) {
		const v = this.signViews.get(id);
		if (!v) return false;
		v.mesh.updateWorldMatrix(true, false);
		const center = v.mesh.getWorldPosition(new THREE.Vector3());
		// signs: look at them horizontally; tilted screens (lab computers): look straight along their normal
		const normal = new THREE.Vector3(0, 0, 1).transformDirection(v.mesh.matrixWorld);
		if (!v.mesh.userData.tilted) normal.setY(0);
		normal.normalize();
		const vt = Math.tan(THREE.MathUtils.degToRad(this.camera.fov / 2));
		const ht = vt * this.camera.aspect;
		const dist = Math.max((v.w * 0.56) / ht, (v.h * 0.66) / vt);
		this.focusDist = dist;
		this.focusLook.copy(center);
		this.focusPos.copy(center).addScaledVector(normal, dist);
		// the player turns to face the sign
		this.player.group.rotation.y = Math.atan2(-normal.x, -normal.z);
		this.reading = id;
		this.clearKeys();
		this.cb.onReading?.(id);
		return true;
	}
	stopReading() {
		if (!this.reading) return;
		this.reading = null;
		this.cb.onReading?.(null);
	}

	// ---------- pond reflection ----------
	// A small mirror texture that only renders the minifigs (layer REFLECT) with a transparent
	// background, blended over the water with a slight ripple: cheap, because the world is not drawn twice.
	private static REFLECT = 3;
	private mirrorTime: { value: number } = { value: 0 };
	private buildPondMirror(x0: number, x1: number, z0: number, z1: number) {
		const base = (Reflector as unknown as { ReflectorShader: { uniforms: Record<string, THREE.IUniform>; vertexShader: string } }).ReflectorShader;
		const shader = {
			name: 'PondMirror',
			uniforms: { ...THREE.UniformsUtils.clone(base.uniforms), time: this.mirrorTime },
			vertexShader: base.vertexShader,
			fragmentShader: /* glsl */ `
				uniform vec3 color;
				uniform sampler2D tDiffuse;
				uniform float time;
				varying vec4 vUv;
				#include <logdepthbuf_pars_fragment>
				void main() {
					#include <logdepthbuf_fragment>
					vec4 uv = vUv;
					float v = uv.y / uv.w;
					uv.x += sin(v * 90.0 + time * 2.2) * 0.0035 * uv.w;
					vec4 refl = texture2DProj(tDiffuse, uv);
					gl_FragColor = vec4(mix(refl.rgb, color, 0.3), refl.a * 0.62);
					#include <tonemapping_fragment>
					#include <colorspace_fragment>
				}`
		};
		const mirror = new Reflector(new THREE.PlaneGeometry(x1 - x0, z1 - z0), {
			textureWidth: 512,
			textureHeight: 512,
			color: new THREE.Color('#2f6f99'),
			clipBias: 0.003,
			shader
		});
		const mat = mirror.material as THREE.ShaderMaterial;
		mat.transparent = true;
		mat.depthWrite = false;
		mirror.rotation.x = -Math.PI / 2;
		mirror.position.set((x0 + x1) / 2, 0.205, (z0 + z1) / 2);
		mirror.name = 'pond:reflection';
		mirror.renderOrder = 2;
		this.scene.add(mirror);
		mirror.getReflectionCamera(this.camera).layers.set(Town.REFLECT);
		// render only the figures, on a transparent background
		const renderMirror = mirror.onBeforeRender;
		const clear = new THREE.Color();
		let lightsOn = false; // lights must be on the reflection layer too (enabled once)
		mirror.onBeforeRender = (renderer, scene, camera, ...rest) => {
			const figs = [this.player, ...this.npcs.map((n) => n.fig)];
			for (const f of figs) f.group.traverse((o) => !(o as THREE.Sprite).isSprite && o.layers.enable(Town.REFLECT)); // no speech bubbles in the water
			if (!lightsOn) {
				lightsOn = true;
				scene.traverse((o) => {
					if ((o as THREE.Light).isLight) o.layers.enable(Town.REFLECT);
				});
			}
			const bg = scene.background;
			renderer.getClearColor(clear);
			const alpha = renderer.getClearAlpha();
			scene.background = null;
			renderer.setClearColor(0x000000, 0);
			renderMirror.call(mirror, renderer, scene, camera, ...rest);
			renderer.setClearColor(clear, alpha);
			scene.background = bg;
		};
	}

	// ---------- ambient speech bubbles: NPCs say little things when you walk past ----------
	private static BARKS: Record<string, string[]> = {
		mom: ["Don't forget your jacket!", 'Have you eaten breakfast?', 'Dinner is at six, sweetie!', 'So proud of you!'],
		kid: ['Sprint into me to play tag!', 'Catch me if you can!', 'I saw a fish jump!', 'Golden studs are SO shiny!', 'Wheee!'],
		prof: ['Hmm, fascinating…', 'Where are my glasses?', 'Brick Beasts love music!', 'Eureka! …no, wait.'],
		rival: ['Hey, slowpoke!', "Bet you can't beat me!", 'My deck is unbeatable!', 'SNAP!'],
		knight: ['Halt! ...just kidding.', 'This castle is older than bricks!', 'My armour squeaks.', 'Stonecrest welcomes you!'],
		baker: ['Fresh brick bread!', 'Mind the crumbs!', 'The ducks in Brickhollow love my bread.', 'Hungry, traveller?'],
		sister: ['Want to practice?', 'Dex is such a show-off.', 'Win 2 of 3 locations!', 'Ongoing cards are cozy.']
	};
	private barks = new Map<string, { sp: THREE.Sprite; until: number; next: number; last: number }>();
	private bubbleTexture(text: string) {
		const c = document.createElement('canvas');
		const g = c.getContext('2d')!;
		const font = '700 44px "Fredoka", "Segoe UI", Arial, sans-serif';
		g.font = font;
		const w = Math.ceil(g.measureText(text).width) + 56;
		c.width = w;
		c.height = 112;
		g.font = font;
		// bubble with a little tail
		g.fillStyle = '#ffffff';
		g.strokeStyle = '#1b2a34';
		g.lineWidth = 5;
		g.beginPath();
		g.roundRect(3, 3, w - 6, 80, 30);
		g.moveTo(w / 2 - 14, 82);
		g.lineTo(w / 2, 106);
		g.lineTo(w / 2 + 14, 82);
		g.fill();
		g.stroke();
		g.fillStyle = '#ffffff';
		g.fillRect(w / 2 - 11, 76, 22, 8);
		g.fillStyle = '#1b2a34';
		g.textAlign = 'center';
		g.textBaseline = 'middle';
		g.fillText(text, w / 2, 44);
		const tex = new THREE.CanvasTexture(c);
		tex.colorSpace = THREE.SRGBColorSpace;
		return { tex, aspect: w / 112 };
	}
	private updateBark(n: { def: NpcDef; fig: Minifig }, d: number, t: number) {
		let b = this.barks.get(n.def.id);
		if (!b) {
			const sp = new THREE.Sprite(new THREE.SpriteMaterial({ transparent: true, depthWrite: false, opacity: 0 }));
			sp.name = `fx:bark:${n.def.id}`;
			sp.visible = false;
			sp.renderOrder = 5;
			n.fig.group.add(sp);
			b = { sp, until: 0, next: t + 4 + Math.random() * 6, last: -1 };
			this.barks.set(n.def.id, b);
		}
		const lines = n.def.id === 'kid' && this.rain.level > 0.5 ? [...Town.BARKS.kid, 'Rain! Puddle time!', 'I love the rain!'] : Town.BARKS[n.def.id];
		// close enough to notice you, but not standing right in front (then you can talk)
		if (lines && !this.paused && t > b.next && d < 9 && d > 3 && !(n.def.id === 'kid' && this.tag.phase === 'fleeing')) {
			let i = Math.floor(Math.random() * lines.length);
			if (i === b.last) i = (i + 1) % lines.length;
			b.last = i;
			this.sayBark(n.def.id, lines[i], t);
			b.next = t + 18 + Math.random() * 10;
		}
		const left = b.until - t;
		const show = left > 0 && !this.paused && !this.reading;
		b.sp.visible = show;
		if (!show) return;
		const age = 3.2 - left;
		const a = Math.min(1, age / 0.2, left / 0.35);
		b.sp.material.opacity = a;
		b.sp.position.set(0, 7.2 + Math.sin(t * 3) * 0.06 + (1 - Math.min(1, age / 0.2)) * -0.4, 0);
	}
	/** shows a specific line in an NPC's speech bubble right away */
	private sayBark(id: string, text: string, t: number) {
		const b = this.barks.get(id);
		if (!b) return;
		{
			const { tex, aspect } = this.bubbleTexture(text);
			const m = b.sp.material;
			m.map?.dispose();
			m.map = tex;
			m.needsUpdate = true;
			const h = 1.25;
			b.sp.scale.set(h * aspect, h, 1);
			b.sp.center.set(0.5, 0);
			b.until = t + 3.2;
			b.next = Math.max(b.next, t + 8);
		}
	}

	// ---------- tag with Pip: sprint into him, he runs, catch him again within 10 s ----------
	tag = { phase: 'idle' as 'idle' | 'fleeing', start: 0, until: 0, cool: 0, wobble: 0 };
	private updateTag(n: { def: NpcDef; fig: Minifig }, d: number, dt: number, t: number) {
		const tg = this.tag;
		const g = n.fig.group;
		const p = this.player.group.position;
		const sprinting = this.keys.has('shift') && (this.keys.size > 1 || !!this.virt.x || !!this.virt.z);
		if (tg.phase === 'idle') {
			// the player can't get closer than ~1.6 (Pip blocks), so "touching" is < 2.3
			if (!this.paused && this.area === 'outside' && sprinting && d < 2.3 && t > tg.cool) {
				Object.assign(tg, { phase: 'fleeing', start: t, until: t + 10, wobble: Math.random() * 6 });
				this.sayBark('kid', "Tag! You're it!", t);
				this.cb.onTag?.('start');
			} else return false;
		}
		if (this.paused) return true;
		if (t - tg.start > 0.8 && d < 2.3) {
			tg.phase = 'idle';
			tg.cool = t + 25;
			this.sayBark('kid', 'No fair! You got me!', t);
			this.cb.onTag?.('caught');
			return false;
		}
		if (t > tg.until) {
			tg.phase = 'idle';
			tg.cool = t + 15;
			this.sayBark('kid', 'Too slow! Hehe!', t);
			this.cb.onTag?.('escaped');
			return false;
		}
		// run away from the player (a bit slower than the player's sprint), dodging obstacles
		let ax = g.position.x - p.x, az = g.position.z - p.z;
		const l = Math.hypot(ax, az) || 1;
		ax /= l;
		az /= l;
		const base = Math.atan2(ax, az) + Math.sin(t * 2.3 + tg.wobble) * 0.5;
		const sp = 9.6 * dt;
		for (const off of [0, 0.6, -0.6, 1.2, -1.2, 1.9, -1.9, 2.6, -2.6]) {
			const a = base + off;
			const nx = g.position.x + Math.sin(a) * sp, nz = g.position.z + Math.cos(a) * sp;
			if (!this.walkableFor(nx, nz, n)) continue;
			g.position.x = nx;
			g.position.z = nz;
			let diff = a - g.rotation.y;
			diff = Math.atan2(Math.sin(diff), Math.cos(diff));
			g.rotation.y += diff * Math.min(1, dt * 10);
			break;
		}
		n.def.x = g.position.x;
		n.def.z = g.position.z;
		g.position.y = this.groundY(g.position.x, g.position.z);
		n.fig.animate(dt * 1.6, true);
		return true;
	}
	/** like walkable() but ignores the NPC itself (and keeps Pip away from the tall grass) */
	private walkableFor(x: number, z: number, self: { def: NpcDef }) {
		if (!WALK.some((r) => inRect(r, x, z)) || inGrass(x, z)) return false;
		const pr = 0.7;
		for (const b of this.blockers) if (x > b.x0 - pr && x < b.x1 + pr && z > b.z0 - pr && z < b.z1 + pr) return false;
		for (const o of this.npcs) if (o.def !== self.def && Math.hypot(o.def.x - x, o.def.z - z) < 1.6) return false;
		return true;
	}

	// ---------- the pond fish jumps now and then when you are close (splash rings + sound) ----------
	private jump = { next: 8, start: -1, landed: true };
	private splashes: { m: THREE.Mesh; t0: number }[] = [];
	private fishJump(fish: THREE.Object3D, t: number) {
		const j = this.jump;
		const p = this.player.group.position;
		const near = this.area === 'outside' && Math.hypot(p.x + 8, p.z - 18.5) < 14;
		if (j.start < 0 && near && t > j.next) {
			j.start = t;
			j.landed = false;
			this.splash(fish.position.x, fish.position.z, t);
			this.cb.onFishJump?.();
		}
		if (j.start < 0) {
			fish.rotation.z = 0;
			return;
		}
		const k = (t - j.start) / 0.9; // 0..1 over the jump
		if (k >= 1) {
			if (!j.landed) {
				j.landed = true;
				this.splash(fish.position.x, fish.position.z, t);
			}
			j.start = -1;
			j.next = t + 6 + Math.random() * 8;
			return;
		}
		fish.position.y = 0.08 + Math.sin(k * Math.PI) * 1.9;
		fish.rotation.z = (0.5 - k) * 2.4; // nose up, then down
	}
	private splash(x: number, z: number, t: number, y = 0.24) {
		const m = new THREE.Mesh(new THREE.RingGeometry(0.3, 0.42, 24), new THREE.MeshBasicMaterial({ color: '#ffffff', transparent: true, opacity: 0.9, depthWrite: false }));
		m.rotation.x = -Math.PI / 2;
		m.position.set(x, y, z);
		m.name = 'fx:splash';
		this.scene.add(m);
		this.splashes.push({ m, t0: t });
	}
	private updateSplashes(t: number) {
		this.splashes = this.splashes.filter((s) => {
			const k = (t - s.t0) / 0.8;
			if (k >= 1) {
				this.scene.remove(s.m);
				s.m.geometry.dispose();
				(s.m.material as THREE.Material).dispose();
				return false;
			}
			s.m.scale.setScalar(1 + k * 4);
			(s.m.material as THREE.MeshBasicMaterial).opacity = 0.9 * (1 - k);
			return true;
		});
	}

	// ---------- a kickable LEGO ball in the park ----------
	private ball: { m: THREE.Mesh; vx: number; vz: number; r: number; lastKick: number; prev: THREE.Vector3 } | null = null;
	private buildBall() {
		const r = 0.6;
		// classic two-colour ball: red/white segments via a tiny canvas texture
		const c = document.createElement('canvas');
		c.width = 64;
		c.height = 32;
		const g = c.getContext('2d')!;
		for (let i = 0; i < 8; i++) {
			g.fillStyle = i % 2 ? '#f4f4f4' : '#c91a09';
			g.fillRect(i * 8, 0, 8, 32);
		}
		g.fillStyle = '#f2cd37';
		g.fillRect(0, 14, 64, 4); // yellow belt
		const tex = new THREE.CanvasTexture(c);
		tex.colorSpace = THREE.SRGBColorSpace;
		const m = new THREE.Mesh(new THREE.SphereGeometry(r, 24, 16), new THREE.MeshStandardMaterial({ map: tex, roughness: 0.35 }));
		m.name = 'toy:ball';
		m.castShadow = true;
		m.position.set(15, r, -6.2); // on the lawn in front of the park bench
		this.scene.add(m);
		this.ball = { m, vx: 0, vz: 0, r, lastKick: 0, prev: new THREE.Vector3(NaN, 0, 0) }; // (the player doesn't exist yet here)
	}
	private ballFree(x: number, z: number, r: number) {
		if (!inRect(WALK[0], x, z) || inRect(POND, x, z)) return false;
		for (const b of this.blockers) if (x > b.x0 - r && x < b.x1 + r && z > b.z0 - r && z < b.z1 + r) return false;
		return true;
	}
	private updateBall(dt: number, t: number) {
		const b = this.ball;
		if (!b || dt <= 0) return;
		const p = this.player.group.position;
		// player velocity from the last frame
		if (Number.isNaN(b.prev.x)) b.prev.copy(p);
		const pvx = (p.x - b.prev.x) / dt, pvz = (p.z - b.prev.z) / dt;
		b.prev.copy(p);
		const bp = b.m.position;
		const dx = bp.x - p.x, dz = bp.z - p.z;
		const d = Math.hypot(dx, dz);
		const pspeed = Math.hypot(pvx, pvz);
		if (this.area === 'outside' && d < b.r + 0.75 && d > 0.01) {
			// kick: away from the player, a bit in the walking direction, harder when running
			const nx = dx / d, nz = dz / d;
			const power = Math.min(14, 3 + pspeed * 0.9);
			if (pspeed > 1 && t - b.lastKick > 0.25) {
				const mx = pvx / pspeed, mz = pvz / pspeed;
				const kx = nx * 0.6 + mx * 0.4, kz = nz * 0.6 + mz * 0.4;
				const kl = Math.hypot(kx, kz) || 1;
				b.vx = (kx / kl) * power;
				b.vz = (kz / kl) * power;
				b.lastKick = t;
				this.cb.onKick?.(Math.min(1, power / 14));
			}
			// never overlap the player
			const push = b.r + 0.75 - d;
			if (this.ballFree(bp.x + nx * push, bp.z + nz * push, b.r)) {
				bp.x += nx * push;
				bp.z += nz * push;
			}
		}
		const sp = Math.hypot(b.vx, b.vz);
		if (sp < 0.02) {
			b.vx = b.vz = 0;
			return;
		}
		// move, bounce off obstacles (keep 70 % of the speed), roll out
		let nx = bp.x + b.vx * dt, nz = bp.z + b.vz * dt;
		if (!this.ballFree(nx, nz, b.r)) {
			if (this.ballFree(bp.x, nz, b.r)) b.vx = -b.vx * 0.7;
			else if (this.ballFree(nx, bp.z, b.r)) b.vz = -b.vz * 0.7;
			else {
				b.vx = -b.vx * 0.7;
				b.vz = -b.vz * 0.7;
			}
			if (sp > 2) this.cb.onKick?.(Math.min(0.6, sp / 20));
			nx = bp.x + b.vx * dt;
			nz = bp.z + b.vz * dt;
			if (!this.ballFree(nx, nz, b.r)) return;
		}
		bp.x = nx;
		bp.z = nz;
		bp.y = this.groundY(bp.x, bp.z) + b.r;
		// rolling: rotate around the axis perpendicular to the motion
		const axis = new THREE.Vector3(b.vz, 0, -b.vx).normalize();
		b.m.rotateOnWorldAxis(axis, (sp * dt) / b.r);
		const f = Math.exp(-1.4 * dt);
		b.vx *= f;
		b.vz *= f;
	}

	// ---------- a little LEGO goal on the park lawn: kick the ball in ----------
	private soccerGoal = { x0: 24.5, x1: 27, z0: -7.6, z1: -4.4, scoredAt: -99, armed: true };
	private buildGoal() {
		const G = this.soccerGoal;
		const white = new THREE.MeshStandardMaterial({ color: '#f4f4f4', roughness: 0.4 });
		const red = new THREE.MeshStandardMaterial({ color: '#c91a09', roughness: 0.4 });
		const grp = new THREE.Group();
		grp.name = 'toy:goal';
		const H = 2.4, T = 0.32;
		const add = (w: number, h: number, d: number, x: number, y: number, z: number, m: THREE.Material) => {
			const b = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), m);
			b.position.set(x, y, z);
			b.castShadow = true;
			grp.add(b);
		};
		// front posts (red/white striped like a toy) + crossbar
		for (const z of [G.z0, G.z1]) for (let i = 0; i < 4; i++) add(T, H / 4, T, G.x0, (i + 0.5) * (H / 4), z, i % 2 ? white : red);
		add(T, T, G.z1 - G.z0 + T, G.x0, H, (G.z0 + G.z1) / 2, white);
		// back posts and top frame
		for (const z of [G.z0, G.z1]) add(T * 0.8, H * 0.7, T * 0.8, G.x1, H * 0.35, z, white);
		add(T * 0.8, T * 0.8, G.z1 - G.z0, G.x1, H * 0.7, (G.z0 + G.z1) / 2, white);
		for (const z of [G.z0, G.z1]) add(G.x1 - G.x0, T * 0.6, T * 0.6, (G.x0 + G.x1) / 2, H * 0.85, z, white);
		// net: a see-through grid texture on the back and sides
		const c = document.createElement('canvas');
		c.width = c.height = 64;
		const g = c.getContext('2d')!;
		g.strokeStyle = '#ffffff';
		g.lineWidth = 3;
		for (let i = 0; i <= 64; i += 16) {
			g.beginPath(); g.moveTo(i, 0); g.lineTo(i, 64); g.stroke();
			g.beginPath(); g.moveTo(0, i); g.lineTo(64, i); g.stroke();
		}
		const tex = new THREE.CanvasTexture(c);
		tex.wrapS = tex.wrapT = THREE.RepeatWrapping;
		const net = (w: number, h: number) => {
			const t2 = tex.clone();
			t2.repeat.set(w * 1.5, h * 1.5);
			t2.needsUpdate = true;
			return new THREE.MeshBasicMaterial({ map: t2, transparent: true, opacity: 0.75, side: THREE.DoubleSide, depthWrite: false, alphaTest: 0.3 });
		};
		const back = new THREE.Mesh(new THREE.PlaneGeometry(G.z1 - G.z0, H * 0.7), net(G.z1 - G.z0, H * 0.7));
		back.rotation.y = Math.PI / 2;
		back.position.set(G.x1, H * 0.35, (G.z0 + G.z1) / 2);
		grp.add(back);
		for (const z of [G.z0, G.z1]) {
			const s = new THREE.Mesh(new THREE.PlaneGeometry(G.x1 - G.x0, H), net(G.x1 - G.x0, H));
			s.position.set((G.x0 + G.x1) / 2, H / 2, z);
			grp.add(s);
		}
		const top = new THREE.Mesh(new THREE.PlaneGeometry(G.x1 - G.x0, G.z1 - G.z0), net(G.x1 - G.x0, G.z1 - G.z0));
		top.rotation.x = -Math.PI / 2;
		top.position.set((G.x0 + G.x1) / 2, H * 0.85, (G.z0 + G.z1) / 2);
		grp.add(top);
		grp.position.y = this.groundY(G.x0, G.z0);
		this.scene.add(grp);
		// the sides and back block the ball and the player; the mouth (west side) stays open
		this.blockers.push({ x0: G.x0 - 0.2, x1: G.x1 + 0.3, z0: G.z0 - 0.35, z1: G.z0 + 0.05 });
		this.blockers.push({ x0: G.x0 - 0.2, x1: G.x1 + 0.3, z0: G.z1 - 0.05, z1: G.z1 + 0.35 });
		this.blockers.push({ x0: G.x1 - 0.05, x1: G.x1 + 0.35, z0: G.z0, z1: G.z1 });
	}
	private updateGoal(dt: number, t: number) {
		const b = this.ball, G = this.soccerGoal;
		if (!b) return;
		const bp = b.m.position;
		const inside = bp.x > G.x0 + 0.5 && bp.z > G.z0 && bp.z < G.z1 && bp.x < G.x1;
		if (!inside && bp.x < G.x0 - 0.3) G.armed = true; // only counts again once the ball has left the goal
		if (G.armed && inside) {
			G.armed = false;
			G.scoredAt = t;
			this.confetti((G.x0 + G.x1) / 2, (G.z0 + G.z1) / 2, t);
			this.sayBark('kid', 'GOOOAL!', t);
			this.cb.onGoal?.();
		}
		// after a goal the ball rolls back out to the kick-off spot
		if (!G.armed && bp.x > G.x0 - 1 && t - G.scoredAt > 2 && Math.hypot(b.vx, b.vz) < 1.5) {
			b.vx = -9;
			b.vz = (Math.random() - 0.5) * 2;
		}
	}
	private confettiBits: { m: THREE.Mesh; v: THREE.Vector3; spin: THREE.Vector3; t0: number }[] = [];
	/** a burst of colourful LEGO tiles flying up and fluttering down */
	private confetti(x: number, z: number, t: number) {
		const cols = ['#c91a09', '#f2cd37', '#0055bf', '#237841', '#ff8a1f', '#e96ed1', '#ffffff'];
		const geo = new THREE.BoxGeometry(0.28, 0.06, 0.18);
		for (let i = 0; i < 46; i++) {
			const m = new THREE.Mesh(geo, new THREE.MeshStandardMaterial({ color: cols[i % cols.length], roughness: 0.5 }));
			m.position.set(x, 1.5, z);
			m.name = 'fx:confetti';
			this.scene.add(m);
			const a = Math.random() * Math.PI * 2, s = 2 + Math.random() * 5;
			this.confettiBits.push({ m, v: new THREE.Vector3(Math.cos(a) * s, 7 + Math.random() * 6, Math.sin(a) * s), spin: new THREE.Vector3(Math.random() * 10, Math.random() * 10, Math.random() * 10), t0: t });
		}
	}
	private updateConfetti(dt: number, t: number) {
		if (!this.confettiBits.length) return;
		this.confettiBits = this.confettiBits.filter((c) => {
			const age = t - c.t0;
			if (age > 4.5) {
				this.scene.remove(c.m);
				(c.m.material as THREE.Material).dispose();
				return false;
			}
			c.v.y -= 14 * dt;
			if (c.v.y < -2.5) c.v.y = -2.5; // flutter down slowly
			c.v.x *= Math.exp(-1.5 * dt);
			c.v.z *= Math.exp(-1.5 * dt);
			c.m.position.addScaledVector(c.v, dt);
			const gy = this.groundY(c.m.position.x, c.m.position.z) + 0.25;
			if (c.m.position.y < gy) {
				c.m.position.y = gy;
				c.v.set(0, 0, 0);
			} else c.m.rotation.set(c.m.rotation.x + c.spin.x * dt, c.m.rotation.y + c.spin.y * dt, c.m.rotation.z + c.spin.z * dt);
			c.m.scale.setScalar(Math.min(1, (4.5 - age) / 0.6));
			return true;
		});
	}

	// ---------- light rain showers now and then (outside only) ----------
	rain = { level: 0, target: 0, next: 150 + Math.random() * 90, until: 0 };
	private rainSky = new THREE.Color('#8794a0');
	private rainDrops: THREE.InstancedMesh | null = null;
	private drops: { x: number; y: number; z: number; v: number }[] = [];
	private lastRainSent = -1;
	private buildRain() {
		const N = 450;
		const im = new THREE.InstancedMesh(new THREE.BoxGeometry(0.05, 0.9, 0.05), new THREE.MeshBasicMaterial({ color: '#e4f2ff', transparent: true, opacity: 0, depthWrite: false }), N);
		im.name = 'fx:rain';
		im.frustumCulled = false;
		im.visible = false;
		this.scene.add(im);
		this.rainDrops = im;
		for (let i = 0; i < N; i++) this.drops.push({ x: (Math.random() - 0.5) * 40, y: Math.random() * 20, z: (Math.random() - 0.5) * 32, v: 18 + Math.random() * 6 });
	}
	/** a small JPEG snapshot of the current view (rendered right now, so it works without preserveDrawingBuffer) */
	snapPhoto(width = 360) {
		this.renderer.render(this.scene, this.camera);
		const src = this.renderer.domElement;
		if (!src.width || !src.height) return null;
		const c = document.createElement('canvas');
		c.width = width;
		c.height = Math.round((width * src.height) / src.width);
		const g = c.getContext('2d');
		if (!g) return null;
		g.drawImage(src, 0, 0, c.width, c.height);
		return c.toDataURL('image/jpeg', 0.72);
	}
	// ---------- a treasure chest: once a day it shows up somewhere in the village ----------
	chest = { state: 'none' as 'none' | 'waiting' | 'open', x: 0, z: 0, next: 50, openedAt: 0, day: -1 };
	private chestGroup: THREE.Group | null = null;
	private chestLid: THREE.Group | null = null;
	private chestGlint: THREE.Sprite | null = null;
	private buildChest() {
		const g = new THREE.Group();
		g.name = 'toy:chest';
		const wood = this.mat('#8a4b22'), dark = this.mat('#5b2f14'), gold = this.mat(C.yellow, { metalness: 0.6, roughness: 0.3 });
		const add = (parent: THREE.Object3D, w: number, h: number, d: number, m: THREE.Material, x: number, y: number, z: number) => {
			const b = new THREE.Mesh(this.rbox(w, h, d), m);
			b.position.set(x, y, z);
			b.castShadow = true;
			parent.add(b);
			return b;
		};
		add(g, 1.8, 1.0, 1.2, wood, 0, 0.5, 0);
		add(g, 1.86, 0.16, 1.26, dark, 0, 0.12, 0);
		for (const x of [-0.7, 0.7]) add(g, 0.16, 1.02, 1.26, gold, x, 0.5, 0);
		// the lid hinges at the back edge
		const lid = new THREE.Group();
		lid.position.set(0, 1.0, -0.6);
		g.add(lid);
		add(lid, 1.8, 0.45, 1.2, wood, 0, 0.22, 0.6);
		for (const x of [-0.7, 0.7]) add(lid, 0.16, 0.47, 1.26, gold, x, 0.22, 0.6);
		add(lid, 0.32, 0.36, 0.1, gold, 0, 0.05, 1.24); // lock
		// treasure inside (seen when it opens)
		for (let i = 0; i < 6; i++) {
			const c = new THREE.Mesh(new THREE.CylinderGeometry(0.16, 0.16, 0.08, 12), gold);
			c.position.set(-0.5 + (i % 3) * 0.5, 0.95, -0.2 + Math.floor(i / 3) * 0.4);
			g.add(c);
		}
		// a glint above it, visible from afar
		const cv = document.createElement('canvas');
		cv.width = cv.height = 64;
		const gg = cv.getContext('2d')!;
		const grad = gg.createRadialGradient(32, 32, 0, 32, 32, 32);
		grad.addColorStop(0, 'rgba(255,250,200,1)');
		grad.addColorStop(0.3, 'rgba(255,215,80,0.7)');
		grad.addColorStop(1, 'rgba(255,215,80,0)');
		gg.fillStyle = grad;
		gg.fillRect(0, 0, 64, 64);
		gg.fillStyle = 'rgba(255,255,255,0.95)';
		gg.fillRect(30, 4, 4, 56);
		gg.fillRect(4, 30, 56, 4);
		const glint = new THREE.Sprite(new THREE.SpriteMaterial({ map: new THREE.CanvasTexture(cv), transparent: true, depthWrite: false, blending: THREE.AdditiveBlending }));
		glint.scale.setScalar(2.2);
		glint.position.y = 2.6;
		g.add(glint);
		g.visible = false;
		this.scene.add(g);
		this.chestGroup = g;
		this.chestLid = lid;
		this.chestGlint = glint;
	}
	/** puts a chest somewhere reachable, away from the player (also handy for testing) */
	spawnChest() {
		const g = this.chestGroup;
		if (!g || this.area !== 'outside') return false;
		const p = this.player.group.position;
		const spots = this.reachableSpots().filter((s) => {
			const d = Math.hypot(s.x - p.x, s.z - p.z);
			if (d < 12 || d > 50 || this.onRoad(s.x, s.z)) return false;
			// room for the chest itself
			return [[-1.2, 0], [1.2, 0], [0, -1.2], [0, 1.2]].every(([dx, dz]) => this.walkable(s.x + dx, s.z + dz));
		});
		if (!spots.length) return false;
		const s = pick(spots);
		Object.assign(this.chest, { state: 'waiting', x: s.x, z: s.z });
		g.position.set(s.x, this.groundY(s.x, s.z), s.z);
		g.rotation.y = Math.atan2(p.x - s.x, p.z - s.z); // lock facing the player's side
		this.chestLid!.rotation.x = 0;
		g.scale.setScalar(1.4);
		g.visible = true;
		this.cb.onChestSpawn?.();
		return true;
	}
	/** opens the chest the player stands at; returns the studs found (0 if there is none) */
	openChest() {
		if (this.chest.state !== 'waiting') return 0;
		const t = this.clock.elapsedTime;
		this.chest.state = 'open';
		this.chest.openedAt = t;
		this.confetti(this.chest.x, this.chest.z, t);
		return 3 + Math.floor(Math.random() * 6);
	}
	private updateChest(dt: number, t: number) {
		const c = this.chest, g = this.chestGroup;
		if (!g) return;
		// a new one every in-game day (the first one shortly after you start)
		const day = this.dayCount;
		if (c.state === 'none' && t > c.next && c.day !== day && this.area === 'outside' && !this.paused) {
			if (this.spawnChest()) c.day = day;
			else c.next = t + 20;
		}
		if (c.state === 'waiting') {
			this.chestGlint!.material.opacity = 0.6 + 0.4 * Math.sin(t * 4);
			this.chestGlint!.material.rotation = t * 0.8;
			g.position.y = this.groundY(c.x, c.z) + Math.max(0, Math.sin(t * 2.2)) * 0.06; // a little jiggle
		} else if (c.state === 'open') {
			const age = t - c.openedAt;
			this.chestLid!.rotation.x = -Math.min(1, age / 0.4) * 1.9;
			this.chestGlint!.material.opacity = Math.max(0, 1 - age);
			if (age > 3) {
				const k = Math.max(0, 1 - (age - 3) / 0.6);
				g.scale.setScalar(1.4 * k);
				if (k <= 0) {
					g.visible = false;
					c.state = 'none';
					c.next = t + 30;
				}
			}
		}
	}
	// ---------- dancing (Q): the figure hops and spins, music notes float up, villagers nearby join in ----------
	dance = { on: false, nextNote: 0 };
	private notes: { sp: THREE.Sprite; t0: number; x: number; z: number; drift: number }[] = [];
	private noteTex: THREE.Texture[] = [];
	toggleDance() {
		if (this.seated || this.reading) return;
		this.dance.on = !this.dance.on;
		const p = this.player.group.position;
		const crowd = this.dance.on ? this.npcs.filter((n) => Math.hypot(n.fig.group.position.x - p.x, n.fig.group.position.z - p.z) < 10).length : 0;
		if (!this.dance.on) {
			const f = this.player;
			f.armL.rotation.x = f.armR.rotation.x = -0.22;
		}
		this.cb.onDance?.(this.dance.on, crowd);
	}
	private dancePose(dt: number, t: number) {
		const f = this.player;
		f.group.rotation.y += dt * 2.6;
		const s = Math.sin(t * 6);
		f.armL.rotation.x = -2.5 + s * 0.6;
		f.armR.rotation.x = -2.5 - s * 0.6;
		f.legL.rotation.x = Math.max(0, s) * 0.5;
		f.legR.rotation.x = Math.max(0, -s) * 0.5;
		this.updateParticles(dt, false, false);
		// music notes
		if (t > this.dance.nextNote) {
			this.dance.nextNote = t + 0.45;
			if (!this.noteTex.length) {
				for (const [ch, col] of [['♪', '#ff5fa2'], ['♫', '#3d9bff'], ['♪', '#f2b705'], ['♬', '#4fc06a']]) {
					const c = document.createElement('canvas');
					c.width = c.height = 64;
					const g = c.getContext('2d')!;
					g.font = 'bold 52px sans-serif';
					g.textAlign = 'center';
					g.textBaseline = 'middle';
					g.lineWidth = 6;
					g.strokeStyle = '#ffffff';
					g.strokeText(ch, 32, 34);
					g.fillStyle = col;
					g.fillText(ch, 32, 34);
					const tex = new THREE.CanvasTexture(c);
					tex.colorSpace = THREE.SRGBColorSpace;
					this.noteTex.push(tex);
				}
			}
			const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: pick(this.noteTex), transparent: true, depthWrite: false }));
			sp.scale.setScalar(1.7);
			sp.name = 'fx:note';
			const p = f.group.position;
			const a = Math.random() * Math.PI * 2;
			this.scene.add(sp);
			this.notes.push({ sp, t0: t, x: p.x + Math.cos(a) * 1.2, z: p.z + Math.sin(a) * 1.2, drift: Math.random() * 6 });
		}
	}
	private updateNotes(t: number) {
		if (!this.notes.length) return;
		const base = this.player.group.position.y;
		this.notes = this.notes.filter((n) => {
			const age = t - n.t0;
			if (age > 1.8) {
				this.scene.remove(n.sp);
				n.sp.material.dispose();
				return false;
			}
			n.sp.position.set(n.x + Math.sin(age * 4 + n.drift) * 0.3, base + 4 + age * 2.2, n.z);
			n.sp.material.opacity = Math.min(1, age * 5, (1.8 - age) / 0.5);
			return true;
		});
	}
	/**
	 * In a Voxelmon battle you don't stand between the two beasts: you walk back a few steps (still looking
	 * at the fight) until you are well away from the middle of the arena. Returns true while walking.
	 */
	private stepAside(dt: number) {
		const a = this.vox.arena;
		if (!a || this.area !== 'outside') return false;
		const p = this.player.group.position;
		const g = this.player.group;
		let dx = p.x - a.x, dz = p.z - a.z;
		let d = Math.hypot(dx, dz);
		// look at the fight
		const look = Math.atan2(-dx, -dz);
		let diff = look - g.rotation.y;
		diff = Math.atan2(Math.sin(diff), Math.cos(diff));
		g.rotation.y += diff * Math.min(1, dt * 8);
		const KEEP = 8.5; // distance from the middle of the arena
		if (d >= KEEP - 0.1) return false;
		// standing right in the middle: back off towards the camera (south), so the view stays free
		if (d < 0.5) [dx, dz, d] = [0, 1, 1];
		const step = Math.min(KEEP - d, 5 * dt);
		const nx = p.x + (dx / d) * step, nz = p.z + (dz / d) * step;
		let moved = false;
		if (this.walkable(nx, p.z)) (p.x = nx), (moved = true);
		if (this.walkable(p.x, nz)) (p.z = nz), (moved = true);
		if (!moved) return false;
		this.player.animate(dt, true, 0.8);
		this.updateParticles(dt, true, false);
		return true;
	}

	private updateGateProf(dt: number, t: number) {
		const f = this.gateProf;
		if (!f) return;
		const show = this.area === 'outside' && !this.routeOpen();
		f.group.visible = show;
		if (!show) return;
		const p = this.player.group.position;
		// she keeps an eye on you
		f.group.rotation.y = Math.atan2(p.x - GATE_PROF.x, p.z - GATE_PROF.z);
		f.animate(dt, false);
		void t;
	}
	/** start a shower right now (also handy for testing) */
	startRain(seconds = 40) {
		const t = this.clock.elapsedTime;
		Object.assign(this.rain, { target: 1, until: t + seconds });
	}
	private updateRain(dt: number, t: number) {
		const r = this.rain;
		if (r.target === 0 && t > r.next) Object.assign(r, { target: 1, until: t + 35 + Math.random() * 20 });
		if (r.target === 1 && t > r.until) {
			Object.assign(r, { target: 0, next: t + 180 + Math.random() * 120 });
		}
		r.level += (r.target - r.level) * Math.min(1, dt * 0.35);
		if (r.level < 0.003 && r.target === 0) r.level = 0;
		const outside = this.area === 'outside';
		const heard = outside ? r.level : 0;
		if (Math.abs(heard - this.lastRainSent) > 0.02 || (heard === 0 && this.lastRainSent !== 0)) {
			this.lastRainSent = heard;
			this.cb.onRain?.(heard);
		}
		const im = this.rainDrops;
		if (!im) return;
		im.visible = outside && r.level > 0.01;
		if (!im.visible) return;
		(im.material as THREE.MeshBasicMaterial).opacity = 0.75 * r.level;
		// drops fall around the player and wrap around (only as many as the shower is strong)
		const p = this.player.group.position;
		const n = Math.floor(this.drops.length * r.level);
		im.count = n;
		for (let i = 0; i < n; i++) {
			const d = this.drops[i];
			d.y -= d.v * dt;
			if (d.y < 0) {
				d.y += 20;
				d.x = (Math.random() - 0.5) * 40;
				d.z = (Math.random() - 0.5) * 32;
			}
			this.tmpM.makeTranslation(p.x + d.x, d.y, p.z + d.z - 4);
			im.setMatrixAt(i, this.tmpM);
		}
		im.instanceMatrix.needsUpdate = true;
	}

	// ---------- puddles: form on the roads while it rains, dry up slowly, splash when you walk through ----------
	private puddles: { m: THREE.Mesh; x: number; z: number; r: number }[] = [];
	private wet = 0;
	private lastPuddleSplash = 0;
	private lastPuddle: unknown = null;
	private buildPuddles() {
		let seed = 4242;
		const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
		const mat = new THREE.MeshStandardMaterial({ color: '#5b87b8', roughness: 0.04, metalness: 0.5, transparent: true, opacity: 0, depthWrite: false });
		const doors = ROOMS.filter((r) => !r.upstairs).map((r) => r.door);
		for (let tries = 0; tries < 600 && this.puddles.length < 14; tries++) {
			const road = this.roads[Math.floor(rnd() * this.roads.length)];
			const x = road.x0 + 1 + rnd() * (road.x1 - road.x0 - 2), z = road.z0 + 1 + rnd() * (road.z1 - road.z0 - 2);
			if (!this.onRoad(x, z) || x > 100) continue;
			if (this.puddles.some((q) => Math.hypot(q.x - x, q.z - z) < 5)) continue;
			if (doors.some((d) => Math.hypot(d.x - x, d.z + 1.5 - z) < 3.5)) continue;
			const r = 0.9 + rnd() * 0.9;
			const sx = r * (1.1 + rnd() * 0.6);
			// the whole puddle has to lie on the road, not spill onto grass
			const e = sx + 0.3;
			if (![[e, 0], [-e, 0], [0, e], [0, -e], [e * 0.7, e * 0.7], [-e * 0.7, -e * 0.7], [e * 0.7, -e * 0.7], [-e * 0.7, e * 0.7]].every(([dx, dz]) => this.onRoad(x + dx, z + dz))) continue;
			const m = new THREE.Mesh(new THREE.CircleGeometry(1, 28), mat);
			m.rotation.x = -Math.PI / 2;
			m.rotation.z = rnd() * Math.PI;
			m.scale.set(sx, r, 1); // a bit oval
			m.position.set(x, this.groundY(x, z) + 0.215, z); // just above the road studs' bases
			m.name = 'fx:puddle';
			m.visible = false;
			m.renderOrder = 1;
			this.scene.add(m);
			this.puddles.push({ m, x, z, r });
		}
	}
	private updatePuddles(dt: number, t: number) {
		// fill while it rains (about 25 s to full), dry up slowly afterwards (about a minute)
		this.wet = THREE.MathUtils.clamp(this.wet + (this.rain.level > 0.3 ? dt * 0.04 * this.rain.level : -dt * 0.016), 0, 1);
		const w = this.wet;
		const show = this.area === 'outside' && w > 0.01;
		for (const q of this.puddles) {
			q.m.visible = show;
			if (!show) continue;
			const g = 0.35 + 0.65 * w; // grow while filling
			q.m.scale.z = 1;
			q.m.position.y = this.groundY(q.x, q.z) + 0.215;
			(q.m.material as THREE.MeshStandardMaterial).opacity = 0.6 * Math.min(1, w * 1.6);
			q.m.userData.g = g;
			q.m.scale.x = (q.m.userData.sx ??= q.m.scale.x) * g;
			q.m.scale.y = (q.m.userData.sy ??= q.m.scale.y) * g;
		}
		if (!show || w < 0.25 || t - this.lastPuddleSplash < 0.35) return;
		// walking through one: splash ring + sound
		const p = this.player.group.position;
		const moving = this.keys.size > 0 || !!this.virt.x || !!this.virt.z;
		if (!moving || this.paused) return;
		const inside = this.puddles.find((q) => Math.hypot(q.x - p.x, q.z - p.z) < q.r * (0.35 + 0.65 * w) * 0.9) ?? null;
		if (inside !== this.lastPuddle && inside) this.cb.onPuddle?.(); // counts each puddle you step into once
		this.lastPuddle = inside;
		for (const q of this.puddles) {
			if (q === inside) {
				this.lastPuddleSplash = t;
				this.splash(p.x, p.z, t, q.m.position.y + 0.02);
				this.cb.onSplash?.(false);
				this.wetSteps = 14; // wet shoes: the next steps leave footprints
				break;
			}
		}
	}

	// ---------- wet footprints after stepping into a puddle ----------
	private wetSteps = 0;
	private prints: { m: THREE.Mesh; t0: number }[] = [];
	private printIdx = 0;
	private lastPrint = new THREE.Vector3(NaN, 0, 0);
	private printSide = 1;
	private updateFootprints(t: number) {
		const p = this.player.group.position;
		if (this.wetSteps > 0 && this.area === 'outside') {
			if (Number.isNaN(this.lastPrint.x)) this.lastPrint.copy(p);
			if (Math.hypot(p.x - this.lastPrint.x, p.z - this.lastPrint.z) > 0.85) {
				const dx = p.x - this.lastPrint.x, dz = p.z - this.lastPrint.z;
				const a = Math.atan2(dx, dz);
				this.lastPrint.copy(p);
				if (!this.onRoad(p.x, p.z)) {
					this.wetSteps--; // grass soaks it up
					return;
				}
				if (this.prints.length < 28) {
					const m = new THREE.Mesh(
						new THREE.CircleGeometry(0.5, 12),
						new THREE.MeshBasicMaterial({ color: '#2d3a48', transparent: true, opacity: 0, depthWrite: false })
					);
					m.rotation.x = -Math.PI / 2;
					m.name = 'fx:footprint';
					m.renderOrder = 1;
					this.scene.add(m);
					this.prints.push({ m, t0: -99 });
				}
				const fp = this.prints[this.printIdx++ % 28] ?? this.prints[this.prints.length - 1];
				// a little oval shoe print, left and right of the walking line
				this.printSide = -this.printSide;
				const ox = Math.cos(a) * 0.28 * this.printSide, oz = -Math.sin(a) * 0.28 * this.printSide;
				fp.m.position.set(p.x + ox, this.groundY(p.x, p.z) + 0.225, p.z + oz);
				fp.m.rotation.z = a;
				fp.m.scale.set(0.42, 0.7, 1);
				fp.t0 = t;
				fp.m.userData.strength = Math.min(1, this.wetSteps / 8);
				this.wetSteps--;
			}
		} else this.lastPrint.x = NaN;
		for (const f of this.prints) {
			const age = t - f.t0;
			f.m.visible = age < 8 && this.area === 'outside';
			if (f.m.visible) (f.m.material as THREE.MeshBasicMaterial).opacity = 0.38 * (f.m.userData.strength ?? 1) * (1 - age / 8);
		}
	}

	private tmpM = new THREE.Matrix4();
	private tmpQ = new THREE.Quaternion();
	private tmpE = new THREE.Euler();
	private tmpV = new THREE.Vector3();
	private tmpS = new THREE.Vector3();
	private updateGrass(t: number) {
		const px = this.player?.group.position.x ?? 999;
		const pz = this.player?.group.position.z ?? 999;
		// Voxelmon flatten the grass around them (in a battle that makes a little arena)
		const push = this.vox?.grassPushers() ?? [];
		this.grassData.forEach((g, i) => {
			let rx = Math.sin(t * 1.6 + g.ph) * 0.06;
			let rz = Math.cos(t * 1.3 + g.ph) * 0.06;
			let sy = 1;
			const dx = g.x - px, dz = g.z - pz;
			const d = Math.hypot(dx, dz);
			if (d < 1.8) {
				const k = (1.8 - d) / 1.8;
				rz -= (dx / (d || 1)) * 0.7 * k;
				rx += (dz / (d || 1)) * 0.7 * k;
			}
			for (const q of push) {
				const ex = g.x - q.x, ez = g.z - q.z;
				const e = Math.hypot(ex, ez);
				if (e >= q.r) continue;
				const k = (q.r - e) / q.r;
				rz -= (ex / (e || 1)) * 1.1 * k;
				rx += (ez / (e || 1)) * 1.1 * k;
				sy = Math.min(sy, 1 - 0.6 * k);
			}
			this.tmpE.set(rx, g.r, rz);
			this.tmpQ.setFromEuler(this.tmpE);
			this.tmpM.compose(this.tmpV.set(g.x, 0, g.z), this.tmpQ, this.tmpS.set(g.s, g.s * sy, g.s));
			this.grass.setMatrixAt(i, this.tmpM);
		});
		this.grass.instanceMatrix.needsUpdate = true;
	}

	private buildClouds() {
		// a little see-through, like real clouds (own material: the shared white one stays solid)
		const cloud = new THREE.MeshStandardMaterial({ color: C.white, roughness: 0.9, transparent: true, opacity: 0.72, depthWrite: false });
		for (let i = 0; i < 7; i++) {
			const g = new THREE.Group();
			const n = 3 + Math.floor(Math.random() * 3);
			for (let j = 0; j < n; j++) {
				const w = 4 + Math.random() * 5;
				this.box(w, 1.6, 3 + Math.random() * 2, cloud, j * 3 - n * 1.5, Math.random() * 1.2, (Math.random() - 0.5) * 2, { parent: g, shadow: true });
			}
			g.position.set(-80 + Math.random() * 160, 34 + Math.random() * 10, -70 + Math.random() * 100);
			this.scene.add(g);
			this.clouds.push(g);
		}
	}

	private buildStuds() {
		// real LEGO stud: 4.8 mm wide, 1.7 mm tall on an 8 mm pitch -> 0.6 x 0.2125 studs
		// polygon budget: 12 segments and no bottom cap (it sits on the ground and is never seen): 36 instead of 64 triangles
		const geo = new THREE.CylinderGeometry(0.3, 0.3, 0.2125, 12);
		const keep = geo.groups[0].count + geo.groups[1].count; // sides + top cap
		geo.setIndex(Array.from(geo.index!.array.slice(0, keep)));
		geo.clearGroups();
		geo.translate(0, 0.106, 0);
		// in chunks of 24 x 24 studs: chunks out of view are skipped (the world got big)
		const chunks = new Map<string, typeof this.studs>();
		for (const s of this.studs) {
			const k = `${Math.floor(s.x / 24)},${Math.floor(s.z / 24)}`;
			if (!chunks.has(k)) chunks.set(k, []);
			chunks.get(k)!.push(s);
		}
		const mat = this.mat('#ffffff');
		const m4 = new THREE.Matrix4();
		const col = new THREE.Color();
		for (const [k, list] of chunks) {
			const im = new THREE.InstancedMesh(geo, mat, list.length);
			im.name = `ground:stud:${k}`;
			list.forEach((s, i) => {
				m4.makeTranslation(s.x, s.y, s.z);
				im.setMatrixAt(i, m4);
				im.setColorAt(i, col.set(s.c));
			});
			im.computeBoundingSphere();
			im.receiveShadow = true;
			im.castShadow = false;
			this.scene.add(im);
		}
	}

	// ---------- input ----------
	private onKeyDown = (e: KeyboardEvent) => {
		const k = e.key.toLowerCase();
		if (this.reading) {
			// at the piano the keys play notes; only Escape gets up again (handled by the piano overlay)
			if (this.reading.startsWith('piano:')) return;
			if (!['shift', 'control', 'alt', 'meta', 'f9', '`'].includes(k)) {
				e.preventDefault();
				this.stopReading();
			}
			return;
		}
		if (['arrowup', 'arrowdown', 'arrowleft', 'arrowright', ' '].includes(k) && !this.paused) e.preventDefault();
		this.keys.add(k);
		// Voxelmon battle: 1 / 2 trigger the attacks
		if (this.vox.busy) {
			if ((k === '1' || k === '2') && !e.repeat && !this.paused) this.vox.useSkill(+k - 1);
			return;
		}
		if (!this.paused && (k === 'e' || k === ' ' || k === 'enter') && this.nearby && !e.repeat) {
			this.cb.onInteract(this.nearby);
		}
		if (k === 'q' && !e.repeat && !this.paused) this.toggleDance();
	};
	private onKeyUp = (e: KeyboardEvent) => this.keys.delete(e.key.toLowerCase());
	private onWheel = (e: WheelEvent) => {
		e.preventDefault();
		if (this.reading) return this.stopReading();
		this.zoom = THREE.MathUtils.clamp(this.zoom + e.deltaY * 0.001, 0.55, 1.7);
	};
	setVirtual(x: number, z: number) {
		this.virt.x = x;
		this.virt.z = z;
	}
	interact() {
		if (this.nearby && !this.paused) this.cb.onInteract(this.nearby);
	}
	clearKeys() {
		this.keys.clear();
		this.virt.x = this.virt.z = 0;
		this.stopAutoWalk();
	}

	// ---------- swipe controls: one swipe and the figure keeps walking; tap stops ----------
	/** the direction the player swiped (x right, z down the screen), null = standing */
	private auto: { x: number; z: number; side: number; best: number; gained: number } | null = null;
	/** start walking in this direction until tapped, or until there is no way on */
	autoWalk(x: number, z: number) {
		const l = Math.hypot(x, z);
		if (l < 1e-3) return this.stopAutoWalk();
		const p = this.player.group.position;
		// side 0 = not decided yet; best / gained: how far it got in the swipe direction, and when it last got further
		this.auto = { x: x / l, z: z / l, side: 0, best: (p.x * x + p.z * z) / l, gained: this.clock.elapsedTime };
	}
	stopAutoWalk() {
		if (!this.auto) return;
		this.auto = null;
		this.cb.onAutoWalk?.(false);
	}
	get autoWalking() {
		return !!this.auto;
	}
	/**
	 * The way to go now: straight on if it is free, otherwise the closest free direction to the side
	 * (keeps the side it chose last, so it walks around an obstacle instead of wobbling). null = stuck.
	 */
	private autoStep() {
		const a = this.auto!;
		const p = this.player.group.position;
		// the next step must be free (the movement itself slides along walls; a longer look ahead gives up too early in narrow spots)
		const free = (x: number, z: number) => this.walkable(p.x + x * 0.9, p.z + z * 0.9);
		// no progress in the swipe direction for 1.5 s: it is a wall, stop instead of searching back and forth
		const along = p.x * a.x + p.z * a.z;
		if (along > a.best + 0.3) {
			a.best = along;
			a.gained = this.clock.elapsedTime;
		} else if (this.clock.elapsedTime - a.gained > 1.5) return null; // (long enough to walk around a house)
		if (free(a.x, a.z)) return { x: a.x, z: a.z };
		// around the obstacle: decide on one side the first time (towards a door nearby, if there is one), then keep it
		const door = this.nearDoor();
		let order = [1, -1];
		if (door) {
			// the side whose turn points more towards the door
			const tx = door.x - p.x, tz = door.z - p.z;
			const turn = (sd: number) => (a.x * Math.cos(sd) - a.z * Math.sin(sd)) * tx + (a.x * Math.sin(sd) + a.z * Math.cos(sd)) * tz;
			if (turn(-1) > turn(1)) order = [-1, 1];
		}
		for (const ang of [0.45, 0.8, 1.15, Math.PI / 2]) {
			for (const sd of a.side ? [a.side] : order) {
				const c = Math.cos(ang * sd), s = Math.sin(ang * sd);
				const x = a.x * c - a.z * s, z = a.x * s + a.z * c;
				if (free(x, z)) {
					a.side = sd;
					return { x, z };
				}
			}
		}
		return null;
	}

	// ---------- movement ----------
	private walkable(x: number, z: number) {
		if (!WALK.some((r) => inRect(r, x, z))) return false;
		const pr = 0.7;
		for (const b of this.blockers) if (x > b.x0 - pr && x < b.x1 + pr && z > b.z0 - pr && z < b.z1 + pr) return false;
		for (const n of this.npcs) if (Math.hypot(n.def.x - x, n.def.z - z) < 1.6) return false;
		if (this.gateProf?.group.visible && Math.hypot(GATE_PROF.x - x, GATE_PROF.z - z) < 1.6) return false;
		return true;
	}

	private camOffset() {
		return new THREE.Vector3(0, 32, 17).multiplyScalar(this.zoom * (this.area === 'outside' ? 1 : 0.78));
	}

	/** Switches between the village and the (pre-built) interiors. Instant, no loading. */
	/** where the player is (for "continue where you left off") */
	getPose() {
		const p = this.player.group.position;
		return { area: this.area, x: +p.x.toFixed(2), z: +p.z.toFixed(2), rot: +this.player.group.rotation.y.toFixed(3) };
	}
	/** puts the player back to a saved pose; returns the area label, or null if the pose was unusable */
	restorePose(pose: { area: string; x: number; z: number; rot?: number }) {
		const areas: AreaId[] = ['outside', ...ROOMS.map((r) => r.id)];
		if (!pose || !areas.includes(pose.area as AreaId) || !Number.isFinite(pose.x) || !Number.isFinite(pose.z)) return null;
		const area = pose.area as AreaId;
		const p = this.player.group.position;
		p.set(pose.x, 0, pose.z);
		this.player.group.rotation.y = pose.rot ?? 0;
		this.applyArea(area, false);
		this.nudgeFree(); // e.g. something was built there since
		p.y = this.groundY(p.x, p.z);
		this.camPos.copy(p).add(this.camOffset());
		return area === 'outside' ? this.zoneLabel(p.z) : roomById(area).label;
	}

	applyArea(id: AreaId, notify = true) {
		this.stopAutoWalk(); // through a door: stand still on the other side
		this.area = id;
		const inside = id !== 'outside';
		const bg = inside ? '#151f27' : '#bfe6ff';
		(this.scene.background as THREE.Color).set(bg);
		const fog = this.scene.fog as THREE.Fog;
		fog.color.set(bg);
		fog.near = inside ? 45 : 90;
		fog.far = inside ? 80 : 190;
		this.camPos.copy(this.player.group.position).add(this.camOffset());
		this.doorLock = this.clock.elapsedTime + 0.6;
		if (notify) this.cb.onArea(id, inside ? roomById(id).label : 'Brickhollow');
	}

	/** If the player stands inside furniture (e.g. after a teleport), move them to the nearest free spot. */
	private nudgeFree() {
		const p = this.player.group.position;
		if (this.walkable(p.x, p.z)) return;
		for (let r = 0.5; r <= 5; r += 0.5)
			for (let a = 0; a < Math.PI * 2; a += Math.PI / 8) {
				const x = p.x + Math.sin(a) * r, z = p.z - Math.cos(a) * r;
				if (this.walkable(x, z)) {
					p.x = x;
					p.z = z;
					this.camPos.copy(p).add(this.camOffset());
					return;
				}
			}
	}

	/** the closest way out / in near the player: a room's exit, the stairwell, or a house door outside (within 14) */
	private nearDoor(): { x: number; z: number } | null {
		const p = this.player.group.position;
		if (this.area === 'bedroom') return { x: (BED_WELL.x0 + BED_WELL.x1) / 2, z: BED_WELL.z0 + 1 };
		if (this.area !== 'outside') {
			const r = roomById(this.area);
			return { x: r.ox, z: r.oz + r.D / 2 };
		}
		let best: { x: number; z: number } | null = null, bd = 14;
		for (const r of ROOMS) {
			if (r.upstairs) continue;
			const d = Math.hypot(r.door.x - p.x, r.door.z + 1 - p.z);
			if (d < bd) [bd, best] = [d, { x: r.door.x, z: r.door.z + 1 }];
		}
		return best;
	}
	private checkDoors(mx: number, mz: number, t: number) {
		if (t < this.doorLock) return;
		const p = this.player.group.position;
		if (this.area === 'outside') {
			if (mz > -0.3) return;
			for (const r of ROOMS) {
				if (r.upstairs) continue;
				const { x, z } = r.door;
				if (Math.abs(p.x - x) < 1.3 && p.z < z + 2.1 && p.z > z + 0.2) {
					p.set(r.ox, 0.4, r.oz + r.D / 2 - 2.4);
					this.player.group.rotation.y = Math.PI;
					this.nudgeFree();
					this.applyArea(r.id);
					return;
				}
			}
		} else if (this.area === 'home' && mz < -0.3 && inRect(HOME_STAIRS, p.x, p.z) && p.z < HOME_STAIRS.z0 + 1.2) {
			// top of the stairs -> your room
			const r = roomById('bedroom');
			p.set((BED_WELL.x0 + BED_WELL.x1) / 2, 0.4, BED_WELL.z0 - 1.2);
			this.player.group.rotation.y = Math.PI;
			this.applyArea(r.id);
		} else if (this.area === 'bedroom') {
			if (mz > 0.3 && p.x > BED_WELL.x0 && p.x < BED_WELL.x1 && p.z > BED_WELL.z0 + 0.3) {
				p.set((HOME_STAIRS.x0 + HOME_STAIRS.x1) / 2, 0.4, HOME_STAIRS.z1 + 1.2);
				this.player.group.rotation.y = 0;
				this.applyArea('home');
			}
		} else {
			if (mz < 0.3) return;
			const r = roomById(this.area);
			if (Math.abs(p.x - r.ox) < 1.7 && p.z > r.oz + r.D / 2 - 1.7) {
				p.set(r.door.x, 0.4, r.door.z + 2.7);
				this.player.group.rotation.y = 0;
				this.nudgeFree();
				this.applyArea('outside');
			}
		}
	}

	private update(dt: number, t: number) {
		const p = this.player.group.position;
		let mx = 0, mz = 0;
		if (!this.paused && !this.vox.busy) {
			if (this.keys.has('w') || this.keys.has('arrowup')) mz -= 1;
			if (this.keys.has('s') || this.keys.has('arrowdown')) mz += 1;
			if (this.keys.has('a') || this.keys.has('arrowleft')) mx -= 1;
			if (this.keys.has('d') || this.keys.has('arrowright')) mx += 1;
			mx += this.virt.x;
			mz += this.virt.z;
			// swipe controls: keep walking (keys / stick take over while used)
			if (this.auto && !mx && !mz) {
				const d = this.autoStep();
				if (d) {
					mx = d.x;
					mz = d.z;
				} else this.stopAutoWalk();
			}
		}
		const len = Math.hypot(mx, mz);
		const moving = len > 0.05;
		if (this.seated && moving) this.standUp();
		if (this.dance.on && moving) this.toggleDance();
		if (this.dance.on && !this.seated) {
			this.dancePose(dt, t);
		} else if (this.seated) {
			this.sitPose(dt);
		} else if (this.vox.busy && this.stepAside(dt)) {
			// walking back out of the Voxelmon battle (see stepAside)
		} else if (moving) {
			mx /= Math.max(1, len);
			mz /= Math.max(1, len);
			const run = this.keys.has('shift') ? 1.6 : 1;
			const sp = 8.5 * run * dt;
			const nx = p.x + mx * sp;
			const nz = p.z + mz * sp;
			const before = { x: p.x, z: p.z };
			const wantsGrass = inGrass(nx, nz) && !inGrass(p.x, p.z);
			// crossing the exit line, or walking north into Prof. Quill standing there
			const intoProf = !!this.gateProf?.group.visible && mz < 0 && Math.hypot(GATE_PROF.x - nx, GATE_PROF.z - nz) < 2.3;
			const leaving = this.area === 'outside' && ((nz < EXIT_Z && p.z >= EXIT_Z && Math.abs(nx) < 5) || intoProf);
			if (leaving && !this.routeOpen()) {
				// Prof. Quill steps in the way
				if (t - this.lastBlockedMsg > 1.5) {
					this.lastBlockedMsg = t;
					this.cb.onBlockedRoute?.();
				}
			} else if (wantsGrass && !this.hasDeck()) {
				if (t - this.lastBlockedMsg > 1.5) {
					this.lastBlockedMsg = t;
					this.cb.onBlockedGrass();
				}
			} else {
				if (this.walkable(nx, p.z)) p.x = nx;
				if (this.walkable(p.x, nz)) p.z = nz;
			}
			const target = Math.atan2(mx, mz);
			let diff = target - this.player.group.rotation.y;
			diff = Math.atan2(Math.sin(diff), Math.cos(diff));
			this.player.group.rotation.y += diff * Math.min(1, dt * 14);
			const moved = Math.hypot(p.x - before.x, p.z - before.z);
			if (inGrass(p.x, p.z) && moved > 0) {
				this.grassMeter += moved;
				if (this.grassMeter > 3) {
					this.grassMeter = 0;
					if (Math.random() < 0.22) {
						this.clearKeys();
						// with a Voxelmon: a real-time battle right here; otherwise the page explains why not
						if (!this.vox.startWild()) this.cb.onGrass();
					}
				}
			}
			this.player.animate(dt, moved > 0.001, run);
			// footsteps: two per walk cycle
			if (moved > 0.001) {
				this.stepTimer -= dt * run;
				if (this.stepTimer <= 0) {
					this.stepTimer = 0.32;
					this.cb.onStep?.(this.area !== 'outside' ? 'wood' : this.onRoad(p.x, p.z) ? 'road' : 'grass');
				}
			}
			this.updateParticles(dt, moved > 0.001, run > 1);
			// swipe walking: doors count by the direction you swiped, also while sliding sideways along the wall
			const intent = this.auto ?? { x: mx, z: mz };
			this.checkDoors(intent.x, intent.z, t);
		} else {
			this.player.animate(dt, false);
			this.updateParticles(dt, false, false);
		}
		p.y = this.seated ? this.seated.y : this.groundY(p.x, p.z) + (this.dance.on ? Math.abs(Math.sin(t * 6)) * 0.35 : 0);

		// npcs: look at player when close
		let best: string | null = null;
		let bestD = 4.2;
		for (const n of this.npcs) {
			const g = n.fig.group;
			const d = Math.hypot(g.position.x - p.x, g.position.z - p.z);
			this.updateBark(n, d, t);
			if (n.def.id === 'kid' && this.updateHideSeek(n, d, t)) continue;
			if (n.def.id === 'kid' && this.updateTag(n, d, dt, t)) continue;
			if (n.def.id === 'kid' && this.pipFootball(n, d, dt, t)) continue;
			if (n.def.id === 'kid' && this.wander(n, d, dt)) continue;
			let target = n.baseRot;
			if (d < 7) target = Math.atan2(p.x - g.position.x, p.z - g.position.z);
			let diff = target - g.rotation.y;
			diff = Math.atan2(Math.sin(diff), Math.cos(diff));
			g.rotation.y += diff * Math.min(1, dt * 5);
			g.position.y = this.groundY(g.position.x, g.position.z) + Math.abs(Math.sin(t * 2 + n.def.x)) * 0.04;
			n.fig.head.rotation.z = Math.sin(t * 1.3 + n.def.z) * 0.05;
			if (d < bestD) {
				bestD = d;
				best = n.def.id;
			}
			if (this.dance.on && d < 10) {
				// dance along: hop to the beat, arms up, Pip even spins
				const ph = n.def.x * 0.7;
				g.position.y += Math.abs(Math.sin(t * 6 + ph)) * 0.3;
				n.fig.armL.rotation.x = -2.4 + Math.sin(t * 6 + ph) * 0.5;
				n.fig.armR.rotation.x = -2.4 - Math.sin(t * 6 + ph) * 0.5;
				if (n.def.id === 'kid') g.rotation.y = t * 4;
				continue;
			}
			if (n.def.id === 'kid') n.fig.wave(t);
			else n.fig.animate(dt, false);
		}
		for (const r of READABLES) {
			if (this.seated && r.id.startsWith('bench:')) continue;
			const d = Math.hypot(r.x - p.x, r.z - p.z);
			if (d < Math.min(bestD, 2.6)) {
				bestD = d;
				best = r.id;
			}
		}
		if (this.gateProf?.group.visible) {
			const d = Math.hypot(GATE_PROF.x - p.x, GATE_PROF.z - p.z);
			if (d < Math.min(bestD, 3.4)) {
				bestD = d;
				best = 'gate:prof';
			}
		}
		if (this.chest.state === 'waiting' && this.area === 'outside') {
			const d = Math.hypot(this.chest.x - p.x, this.chest.z - p.z);
			if (d < Math.min(bestD, 2.8)) {
				bestD = d;
				best = 'chest:treasure';
			}
		}
		if (best !== this.nearby) {
			this.nearby = best;
			this.cb.onNearby(best);
		}

		this.updateGrass(t);
		this.updateExtras(t);
		this.updateButterflies(t);
		this.updateBall(dt, t);
		this.updateNotes(t);
		this.vox.update(dt, t);
		if (this.area === 'outside') {
			const zone = this.zoneLabel(this.player.group.position.z);
			if (zone !== this.zone) {
				if (this.zone) this.cb.onZone?.(zone);
				this.zone = zone;
			}
		}
		this.updateGateProf(dt, t);
		this.updateChest(dt, t);
		this.updateGoal(dt, t);
		this.updateConfetti(dt, t);
		this.updateRain(dt, t);
		this.updatePuddles(dt, t);
		this.updateFootprints(t);
		this.updateDucks(dt, t);
		// sitting on a bench: time flies (about a full day in 15 s)
		this.updateDayNight(this.seated ? dt * 24 : dt, t);
		this.mirrorTime.value = t;
		this.updateBeacon(t);
		this.lily.forEach((l, i) => {
			l.position.y = (i === this.lily.length - 1 ? 0.08 : 0.26) + Math.sin(t * 1.5 + i) * 0.03;
			if (i === this.lily.length - 1) {
				l.position.x = -8 + Math.sin(t * 0.6) * 3;
				l.position.z = 18.5 + Math.cos(t * 0.6) * 2;
				l.rotation.y = -t * 0.6;
				this.fishJump(l, t);
			} else l.rotation.y += dt * 0.1;
		});
		this.updateSplashes(t);
		this.blinkers.forEach((m, i) => (m.emissiveIntensity = 0.55 + 0.45 * Math.sin(t * (2 + (i % 3)) + i)));
		this.fishes.forEach((f, i) => {
			f.position.x = f.userData.x ??= f.position.x;
			f.position.z = (f.userData.z ??= f.position.z) + Math.sin(t * 0.8 + i * 2) * 1.6;
			f.rotation.y = Math.cos(t * 0.8 + i * 2) > 0 ? Math.PI / 2 : -Math.PI / 2;
		});
		this.spinners.forEach((s, i) => {
			s.rotation.y = t * 0.8 + i;
			s.position.y = 4.6 + Math.sin(t * 2 + i) * 0.15;
		});
		for (const c of this.clouds) {
			c.position.x += dt * 1.2;
			if (c.position.x > 90) c.position.x = -90;
		}

		// camera
		if (this.reading && (this.virt.x || this.virt.z)) this.stopReading();
		// in a Voxelmon battle the camera looks at the arena (between you and the two beasts)
		const arena = this.vox.arena;
		const want = this.tmpV.copy(arena && this.vox.busy ? this.tmpV2.set((p.x + arena.x * 2) / 3, p.y, (p.z + arena.z * 2) / 3) : p).add(this.camOffset());
		this.camPos.lerp(want, Math.min(1, dt * 4));
		const off = this.camOffset();
		this.camLook.set(this.camPos.x, this.camPos.y - off.y + 1.5, this.camPos.z - off.z);
		this.focusBlend = THREE.MathUtils.clamp(this.focusBlend + (this.reading ? dt : -dt) * 1.7, 0, 1);
		const fb = this.focusBlend * this.focusBlend * (3 - 2 * this.focusBlend); // smoothstep
		this.camera.position.copy(this.camPos).lerp(this.focusPos, fb);
		this.camLook.lerp(this.focusLook, fb);
		this.camera.lookAt(this.camLook);
		// clip everything between the eye and the sign (grass, flowers, posts)
		const near = 0.5 + Math.max(0, this.focusDist - 0.7 - 0.5) * fb;
		if (Math.abs(this.camera.near - near) > 1e-3) {
			this.camera.near = near;
			this.camera.updateProjectionMatrix();
		}
		// first person: hide the own figure while the camera is in front of the sign
		this.player.group.visible = fb < 0.35;
		{
			const ang = this.timeOfDay * Math.PI * 2;
			const h = Math.max(0.25, Math.sin(ang));
			this.sun.position.set(p.x + Math.cos(ang) * 45, 12 + h * 42, p.z + 18);
		}
		this.sun.target.position.set(p.x, 0, p.z);
	}

	/** true while a full-screen overlay (battle, deck builder) hides the world: nothing is updated or drawn */
	suspended = false;
	// ---------- graphics quality: high / medium / low, or auto (measures the frame rate and steps down) ----------
	quality: 'high' | 'medium' | 'low' = 'high';
	private sunLight: THREE.DirectionalLight | null = null;
	autoQuality = true;
	private perf = { since: 0, frames: 0, last: 0, warm: 0 };
	setQuality(q: 'high' | 'medium' | 'low', auto = this.autoQuality) {
		this.autoQuality = auto;
		this.perf = { since: 0, frames: 0, last: 0, warm: performance.now() + 5000 };
		if (q === this.quality && this.qualityApplied) return;
		this.quality = q;
		this.qualityApplied = true;
		const r = this.renderer;
		const touch = matchMedia('(pointer: coarse)').matches;
		const dpr = window.devicePixelRatio;
		// sharpness: full resolution on high, less on medium, below 1 on low
		r.setPixelRatio(q === 'high' ? Math.min(dpr, touch ? 1.5 : 2) : q === 'medium' ? Math.min(dpr, 1) : 0.75);
		// shadows: off on low, a smaller shadow map on medium
		const shadows = q !== 'low';
		if (r.shadowMap.enabled !== shadows) {
			r.shadowMap.enabled = shadows;
			this.scene.traverse((o) => {
				const m = (o as THREE.Mesh).material;
				if (m) for (const mm of Array.isArray(m) ? m : [m]) mm.needsUpdate = true;
			});
		}
		if (this.sunLight) {
			const size = q === 'high' ? 2048 : 1024;
			if (this.sunLight.shadow.mapSize.x !== size) {
				this.sunLight.shadow.mapSize.set(size, size);
				this.sunLight.shadow.map?.dispose();
				this.sunLight.shadow.map = null;
			}
		}
		// the pond reflection draws the figures a second time: only on high
		const mirror = this.scene.getObjectByName('pond:reflection');
		if (mirror) mirror.visible = q === 'high';
		this.resize();
		this.cb.onQuality?.(q, this.autoQuality);
	}
	private qualityApplied = false;
	/** auto quality: average frame rate over 4 s; below ~38 fps it steps one level down (never up, so it can't flicker) */
	private watchPerf(now: number) {
		const P = this.perf;
		const gap = now - P.last;
		P.last = now;
		// skip loading, a hidden tab, or a pause (no fair measurement)
		if (!this.autoQuality || document.hidden || now < P.warm || gap > 250 || this.paused) {
			P.since = now;
			P.frames = 0;
			return;
		}
		P.frames++;
		if (now - P.since < 4000) return;
		const fps = (P.frames * 1000) / (now - P.since);
		P.since = now;
		P.frames = 0;
		if (fps < 38 && this.quality !== 'low') this.setQuality(this.quality === 'high' ? 'medium' : 'low', true);
	}

	private loop = () => {
		this.raf = requestAnimationFrame(this.loop);
		if (this.suspended) {
			this.clock.getDelta(); // keep the clock from jumping when the world comes back
			this.perf.last = 0;
			return;
		}
		this.watchPerf(performance.now());
		const dt = Math.min(0.05, this.clock.getDelta());
		this.update(dt, this.clock.elapsedTime);
		this.renderer.info.autoReset = false; // the pond mirror renders mid-frame and would reset the counters
		this.renderer.info.reset(); // count this frame only (main view + shadows + mirror)
		this.renderer.render(this.scene, this.camera);
		if (this.debug) {
			this.frameTimes.push(performance.now());
			while (this.frameTimes.length > 60) this.frameTimes.shift();
		}
	};
	private frameTimes: number[] = [];

	private resize() {
		const w = this.container.clientWidth || window.innerWidth;
		const h = this.container.clientHeight || window.innerHeight;
		this.renderer.setSize(w, h);
		this.camera.aspect = w / h;
		// keep a usable horizontal field of view on portrait phones
		this.camera.fov = w / h < 1 ? Math.min(70, THREE.MathUtils.radToDeg(2 * Math.atan(Math.tan(THREE.MathUtils.degToRad(22)) / (w / h)))) : 38;
		this.camera.updateProjectionMatrix();
	}

	// ---------- debug: name everything, click to copy ids ----------
	debug = false;
	private sectionStart = -1;
	private sectionTag = '';
	private debugBox: THREE.Box3Helper | null = null;
	private raycaster = new THREE.Raycaster();

	/** Names every object a build step adds to the scene, e.g. "lab/box#12". */
	section(tag: string, fn: () => void) {
		this.beginSection(tag);
		fn();
		this.endSection();
	}
	beginSection(tag: string) {
		this.sectionTag = tag;
		this.sectionStart = this.scene.children.length;
	}
	endSection() {
		const added = this.scene.children.slice(this.sectionStart);
		added.forEach((o, i) => this.nameTree(o, `${this.sectionTag}/${o.type.toLowerCase()}#${i}`));
		this.sectionStart = -1;
	}
	nameTree(root: THREE.Object3D, base: string) {
		if (!root.name) root.name = base;
		let i = 0;
		root.traverse((o) => {
			if (o === root) return;
			if (!o.name) o.name = `${root.name}/${o.type.toLowerCase()}#${i++}`;
			else if (!o.name.includes('/')) o.name = `${root.name}/${o.name}`;
		});
	}

	setDebug(on: boolean) {
		this.debug = on;
		if (!on && this.debugBox) this.debugBox.visible = false;
		if (!on) this.setWireframe(false);
	}
	/** debug option: clicking objects copies their id (otherwise clicks behave like in the game, e.g. read signs) */
	pickObjects = true;
	/** debug option: draw every mesh as wireframe */
	wireframe = false;
	setWireframe(on: boolean) {
		this.wireframe = on;
		this.scene.traverse((o) => {
			const m = (o as THREE.Mesh).material;
			if (!m || (o as THREE.Sprite).isSprite) return;
			for (const mat of Array.isArray(m) ? m : [m]) if ('wireframe' in mat) (mat as THREE.MeshStandardMaterial).wireframe = on;
		});
	}
	/** live numbers for the debug panel */
	debugStats() {
		const f = this.frameTimes;
		const fps = f.length > 1 ? ((f.length - 1) * 1000) / (f[f.length - 1] - f[0]) : 0;
		const r = this.renderer.info.render;
		return { fps: Math.round(fps), calls: r.calls, tris: r.triangles, geometries: this.renderer.info.memory.geometries };
	}
	/** which meshes cost the most triangles (instanced meshes count every instance) */
	polygonReport(top = 8) {
		const rows = new Map<string, { tris: number; each: number; n: number }>();
		this.scene.traverse((o) => {
			const mesh = o as THREE.Mesh;
			if (!mesh.isMesh || !mesh.geometry) return;
			const g = mesh.geometry;
			const each = Math.round((g.index ? g.index.count : g.attributes.position?.count ?? 0) / 3);
			const n = (mesh as THREE.InstancedMesh).isInstancedMesh ? (mesh as THREE.InstancedMesh).count : 1;
			const key = (mesh.name || mesh.type).replace(/#\d+$/, '').replace(/\[\d+\]$/, '');
			const row = rows.get(key) ?? { tris: 0, each, n: 0 };
			row.tris += each * n;
			row.n += n;
			rows.set(key, row);
		});
		const all = [...rows.entries()].map(([name, r]) => ({ name, ...r }));
		const total = all.reduce((s, r) => s + r.tris, 0);
		return { total, rows: all.sort((a, b) => b.tris - a.tris).slice(0, top) };
	}

	private onPick = (e: PointerEvent) => {
		if (this.reading) {
			this.stopReading();
			return;
		}
		if (!this.debug || !this.pickObjects) {
			// click / tap on a sign (when close enough) -> read it
			if (this.paused || this.area !== 'outside') return;
			const r = this.renderer.domElement.getBoundingClientRect();
			const ndc = new THREE.Vector2(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
			this.raycaster.setFromCamera(ndc, this.camera);
			const hit = this.raycaster.intersectObjects([...this.signViews.values()].map((v) => v.mesh), false)[0];
			if (!hit) return;
			const id = hit.object.userData.signId as string;
			const p = this.player.group.position;
			const rd = READABLES.find((x) => x.id === id);
			if (rd && Math.hypot(rd.x - p.x, rd.z - p.z) < 14) this.cb.onInteract(id);
			return;
		}
		const r = this.renderer.domElement.getBoundingClientRect();
		const ndc = new THREE.Vector2(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
		this.raycaster.setFromCamera(ndc, this.camera);
		const hit = this.raycaster
			.intersectObjects(this.scene.children, true)
			.find((h) => h.object.visible && !(h.object as THREE.Sprite).isSprite && h.object.type !== 'Box3Helper');
		if (!hit) return;
		const o = hit.object;
		const inst = hit.instanceId !== undefined ? `[${hit.instanceId}]` : '';
		const pt = hit.point;
		const id = `${o.name || o.type}${inst} @ (${pt.x.toFixed(1)}, ${pt.y.toFixed(1)}, ${pt.z.toFixed(1)})`;
		// highlight it
		const box = new THREE.Box3();
		if ((o as THREE.InstancedMesh).isInstancedMesh && hit.instanceId !== undefined) {
			const im = o as THREE.InstancedMesh;
			const m = new THREE.Matrix4();
			im.getMatrixAt(hit.instanceId, m);
			if (!im.geometry.boundingBox) im.geometry.computeBoundingBox();
			box.copy(im.geometry.boundingBox!).applyMatrix4(m).applyMatrix4(im.matrixWorld);
		} else box.setFromObject(o);
		if (!this.debugBox) {
			this.debugBox = new THREE.Box3Helper(box, new THREE.Color('#ff00aa'));
			(this.debugBox.material as THREE.LineBasicMaterial).depthTest = false;
			this.debugBox.renderOrder = 20;
			this.scene.add(this.debugBox);
		}
		this.debugBox.box.copy(box);
		this.debugBox.visible = true;
		navigator.clipboard?.writeText(id).catch(() => {});
		this.cb.onDebugPick?.(id);
	};

	dispose() {
		this.renderer.domElement.removeEventListener('pointerdown', this.onPick);
		cancelAnimationFrame(this.raf);
		window.removeEventListener('keydown', this.onKeyDown);
		window.removeEventListener('keyup', this.onKeyUp);
		this.container.removeEventListener('wheel', this.onWheel);
		this.ro.disconnect();
		this.renderer.dispose();
		this.renderer.domElement.remove();
	}
}
