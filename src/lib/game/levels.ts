// Card stages: battles give EXP, EXP upgrades your cards.
// Stages 1-3 only change the look of a card (frame, glitter, holo foil).
// The last stage turns a 6-energy card into a real 3D Voxelmon that walks with you and fights in the tall grass.
import { CARD_MAP } from './cards';

export interface Stage {
	name: string;
	/** EXP needed to reach this stage from the one before */
	cost: number;
	icon: string;
	blurb: string;
}

export const STAGES: Stage[] = [
	{ name: 'Basic', cost: 0, icon: 'style', blurb: 'A fresh card.' },
	{ name: 'Bronze', cost: 15, icon: 'workspace_premium', blurb: 'A shiny bronze frame.' },
	{ name: 'Silver', cost: 30, icon: 'auto_awesome', blurb: 'Silver frame with glitter.' },
	{ name: 'Gold', cost: 45, icon: 'diamond', blurb: 'Gold frame and rainbow holo foil.' },
	{ name: 'Voxelmon', cost: 80, icon: 'view_in_ar', blurb: 'The beast steps out of the card: a real 3D Voxelmon!' }
];
export const VOXEL_LEVEL = 4;

/** only the mighty 6-energy cards can become a real Voxelmon */
export const canBecomeVoxel = (id: string) => (CARD_MAP[id]?.cost ?? 0) >= 6;
export const maxLevel = (id: string) => (canBecomeVoxel(id) ? VOXEL_LEVEL : VOXEL_LEVEL - 1);

/** EXP for a card battle: a win is worth the most, but you always learn something */
export function battleExp(result: 'win' | 'lose' | 'draw' | 'retreat' | 'fold', kind: string) {
	if (result === 'win') return kind === 'rival' ? 40 : 30;
	if (result === 'draw') return 15;
	if (result === 'lose') return 10;
	return 5;
}
/** EXP for beating a wild Voxelmon in the tall grass */
export const WILD_EXP = 20;
