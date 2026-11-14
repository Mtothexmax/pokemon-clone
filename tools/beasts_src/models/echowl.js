// Echowl, the wise echo owl: a round brown owl with a pale heart-shaped face, huge round eyes,
// feathery ear tufts and wings half spread, as if it is about to call out and hear itself twice.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Echowl',
	palette: {
		body: '#8a5a3a',
		bodyLight: '#a87650',
		bodyDark: '#5e3b24',
		belly: '#f0d8b0',
		bellyDark: '#d8b888',
		accent: '#ffd23d',
		accentDark: '#d8a020',
		eye: '#ffffff',
		pupil: '#1b2a34',
		cheek: '#ef8f80'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// little yellow feet with three toes
		v.side((s) => {
			v.box(s * 2, 1, 1, s * 2, 2, 1, 'accentDark');
			v.box(s * 2 - 1, 0, 1, s * 2 + 1, 0, 3, 'accent');
		});

		// round body with a pale speckled belly
		v.ball(0, 6, 0, 5, 5, 4.5, 'body');
		v.ball(0, 5.5, 1.5, 3.6, 4, 3.4, 'belly');
		for (const [x, y] of [[-1, 6], [1, 4], [-2, 3], [2, 7], [0, 8]]) v.face(x, y, 'bellyDark');

		// short tail feathers at the back
		v.box(-2, 2, -6, 2, 3, -4, 'bodyDark');

		// wings, half spread, with light feather tips
		v.side((s) => {
			v.ball(s * 6, 7, -0.5, 1.6, 4, 3, 'bodyDark');
			v.box(s * 7, 6, -2, s * 8, 9, 1, 'bodyDark');
			v.box(s * 9, 5, -1, s * 9, 8, 1, 'bodyLight');
		});

		// big head with a pale heart-shaped face disc
		v.ball(0, 12.5, 0.5, 6, 4.5, 5, 'body');
		v.ball(0, 12, 3, 4.6, 3.4, 2.6, 'belly');

		// feathery ear tufts
		v.side((s) => {
			v.box(s * 4, 16, 0, s * 5, 17, 1, 'bodyDark');
			v.set(s * 5, 18, 0, 'bodyDark');
		});

		// face: huge round 3x3 eyes with a centred pupil, rosy cheeks, a little hooked beak
		v.side((s) => {
			v.box(s * 1, 12, 5, s * 3, 14, 5, 'eye');
			v.set(s * 2, 13, 5, 'pupil');
			v.set(s * 4, 11, 5, 'cheek');
		});
		v.set(0, 11, 6, 'accent');
		v.set(0, 10, 6, 'accentDark');
	}
};
