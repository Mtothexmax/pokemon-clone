// All Voxelmon model files (./models/<id>.js), keyed by card id. Used for the 3D model (three/voxel.ts)
// and, for models with a build function, the card picture (game/art.ts).
/** palette key, or null to cut voxels away */
type C = string | null;
/** the commands a model's build function gets (implemented in ./draw.js) */
export interface VoxelBuilder {
	/** fill a block, corners included */
	box(x0: number, y0: number, z0: number, x1: number, y1: number, z1: number, c: C): void;
	/** fill an ellipsoid around (x, y, z) */
	ball(x: number, y: number, z: number, rx: number, ry: number, rz: number, c: C): void;
	/** the upper half of that ellipsoid (shells, caps, tree tops) */
	dome(x: number, y: number, z: number, rx: number, ry: number, rz: number, c: C): void;
	/** one voxel */
	set(x: number, y: number, z: number, c: C): void;
	/** paint the voxel you see first from the front at (x, y): eyes, mouths */
	face(x: number, y: number, c: C): void;
	/** run fn for s = -1 and s = 1 (both sides, mirrored) */
	side(fn: (s: number) => void): void;
}
export type VoxelModel = {
	name?: string;
	palette: Record<string, string>;
	pixels?: string[];
	depth?: string[];
	front?: string[];
	side?: string[];
	build?: (v: VoxelBuilder) => void;
};
export const MODELS: Record<string, VoxelModel> = Object.fromEntries(
	Object.entries(import.meta.glob<{ default: VoxelModel }>('./models/*.js', { eager: true })).map(([path, m]) => [path.split('/').pop()!.replace('.js', ''), m.default])
);
