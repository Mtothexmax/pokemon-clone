// Tickbot, the little clockwork robot: a boxy grey robot with a clock face on its chest, round
// lamp eyes, a red antenna bulb and a golden wind-up key on its back. Built from voxels with the
// commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Tickbot',
	palette: {
		body: '#a0a5a9',
		bodyLight: '#c4c9cc',
		bodyDark: '#70767b',
		belly: '#e8eef2',
		accent: '#c91a09',
		gold: '#e0b040',
		eye: '#ffffff',
		pupil: '#1a1f24',
		cheek: '#ff8a80'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// chunky boots and short legs
		v.side((s) => {
			v.box(s, 0, -1, s * 3, 1, 2, 'bodyDark');
			v.box(s * 2, 2, 0, s * 2, 2, 0, 'bodyLight');
		});

		// boxy body with a clock face on the chest (hands at 12 and 3)
		v.box(-3, 3, -2, 3, 8, 2, 'body');
		v.ball(0, 5.5, 3, 2.3, 2.3, 0.4, 'belly');
		v.set(0, 5, 3, 'bodyDark');
		v.box(0, 6, 3, 0, 7, 3, 'accent');
		v.set(1, 5, 3, 'accent');

		// arms with red grabber hands
		v.side((s) => {
			v.box(s * 4, 7, -1, s * 4, 8, 1, 'bodyDark');
			v.box(s * 4, 4, 0, s * 4, 6, 0, 'bodyLight');
			v.box(s * 4, 3, 0, s * 4, 3, 1, 'accent');
		});

		// golden wind-up key on the back
		v.box(0, 5, -4, 0, 5, -3, 'gold');
		v.side((s) => v.box(s, 4, -5, s * 2, 6, -5, 'gold'));

		// square head with rounded corners and a pale face plate, gold ear bolts
		v.box(-4, 9, -3, 4, 13, 3, 'body');
		v.side((s) => {
			for (const z of [-3, 3]) for (const y of [9, 13]) v.set(s * 4, y, z, null);
			v.box(s * 5, 10, 0, s * 5, 12, 0, 'gold');
		});
		v.box(-3, 10, 4, 3, 12, 4, 'belly');

		// antenna with a red bulb
		v.set(0, 14, 0, 'bodyDark');
		v.ball(0, 15, 0, 1, 0.6, 1, 'accent');

		// face: big round 3x3 lamp eyes with a dark middle, a grille mouth
		v.side((s) => {
			v.ball(s * 2, 11, 4, 1, 1, 0.4, 'eye');
			v.set(s * 2, 11, 4, 'pupil');
			v.face(s * 3, 10, 'cheek');
		});
		v.box(-1, 9, 4, 1, 9, 4, 'bodyDark');
	}
};
