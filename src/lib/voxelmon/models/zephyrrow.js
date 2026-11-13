// Zephyrrow, the proud sky bird: a tall upright blue bird with a puffed-out pale chest, a swept-back
// golden crest, a hooked gold beak, huge spread wings and long tail feathers. Built from voxels with the
// commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Zephyrrow',
	palette: {
		body: '#5ab8e0',
		bodyLight: '#8ad0f0',
		bodyDark: '#3a90b8',
		belly: '#e8f8ff',
		accent: '#f0b030',
		accentDark: '#c88a18',
		brow: '#1e4a66',
		eye: '#ffffff',
		pupil: '#10202c'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// golden legs with three toes forward and one back
		v.side((s) => {
			v.box(s * 2, 1, 0, s * 2, 3, 0, 'accent');
			v.box(s * 2, 0, -1, s * 2, 0, 2, 'accentDark');
			v.set(s * 1, 0, 1, 'accentDark');
			v.set(s * 3, 0, 1, 'accentDark');
		});

		// upright egg body with a puffed-out chest
		v.ball(0, 7.5, -1, 4.5, 4.8, 4.6, 'body');
		v.ball(0, 7.5, 1.2, 3.5, 4, 3, 'belly');

		// head on top, swept-back golden crest
		v.ball(0, 13.5, 0.5, 3.4, 3.2, 3.4, 'body');
		for (const [y, z] of [[17, 0], [17, -1], [18, -2], [18, -3], [19, -4]]) v.set(0, y, z, 'accent');
		v.side((s) => {
			for (const [y, z] of [[17, -1], [17, -2], [18, -3]]) v.set(s, y, z, 'accentDark');
		});

		// huge wings spread wide: blue on top, a dark feather edge below, gold tips
		v.side((s) => {
			v.ball(s * 8, 9.5, -2, 5.5, 2.6, 0.7, 'bodyDark');
			v.ball(s * 7.5, 10.5, -2, 4.8, 2, 0.7, 'body');
			v.ball(s * 6, 11, -2, 2.5, 1.2, 0.7, 'bodyLight');
			for (const [x, y] of [[13, 11], [13, 10], [12, 8], [10, 7]]) v.set(s * x, y, -2, 'accent');
		});

		// long tail feathers at the back
		v.ball(0, 3.5, -7, 1.5, 1, 3.5, 'bodyDark');
		v.set(0, 3, -10, 'accent');
		v.side((s) => v.set(s, 3, -9, 'accent'));

		// hooked gold beak
		v.box(-1, 12, 4, 1, 13, 4, 'accent');
		v.box(0, 12, 5, 0, 13, 5, 'accent');
		v.set(0, 11, 5, 'accentDark');

		// face: fierce eyes with angry slanted brows
		v.side((s) => {
			v.face(s * 2, 14, 'eye');
			v.face(s * 2, 13, 'eye');
			v.face(s * 1, 14, 'pupil');
			v.face(s * 1, 15, 'brow');
			v.face(s * 2, 16, 'brow');
			v.face(s * 3, 16, 'brow');
		});
	}
};
