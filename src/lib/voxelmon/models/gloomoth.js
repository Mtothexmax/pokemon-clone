// Gloomoth, the gloomy moth: a fluffy purple moth with a pale collar, feathery antennae and big
// dusky wings with dark eye-spots. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Gloomoth',
	palette: {
		body: '#8a6ad0',
		bodyLight: '#a88ce6',
		bodyDark: '#6a4fb0',
		belly: '#d8c8ff',
		fluff: '#f0e8ff',
		accent: '#3a2a6a',
		eye: '#ffffff',
		pupil: '#1b1530',
		cheek: '#c890e0'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two stubby dark feet
		v.side((s) => v.box(s, 0, 0, s * 2, 1, 2, 'accent'));

		// round fuzzy body with stripes and a pale belly
		v.ball(0, 5, 0, 3.5, 3.5, 3, 'body');
		v.ball(0, 4, 0, 3.7, 0.5, 3.2, 'bodyDark');
		v.ball(0, 6, 0, 3.7, 0.5, 3.2, 'bodyDark');
		v.ball(0, 5, 1.5, 2.2, 2.5, 2, 'belly');

		// fluffy collar and the big round head
		v.ball(0, 8, 0.5, 4, 1.4, 3.6, 'fluff');
		v.ball(0, 11, 1, 3.6, 3, 3.2, 'body');

		// big wings behind the body: dark rim, purple inside, dark eye-spot with a pale dot
		v.side((s) => {
			v.ball(s * 7, 10, -2, 4.6, 4, 0.6, 'bodyDark');
			v.ball(s * 7, 10, -2, 3.6, 3, 0.6, 'bodyLight');
			v.ball(s * 7.5, 10.5, -2, 1.6, 1.6, 0.6, 'accent');
			v.set(s * 8, 11, -2, 'belly');
			v.ball(s * 5.5, 4.5, -2, 3, 2.6, 0.6, 'bodyDark');
			v.ball(s * 5.5, 4.5, -2, 2, 1.6, 0.6, 'body');
		});

		// feathery antennae curling out over the head
		v.side((s) => {
			for (const [x, y] of [[1, 14], [2, 15], [3, 15], [4, 16]]) v.set(s * x, y, 1, 'accent');
			for (const [x, y] of [[2, 16], [3, 16], [5, 16], [5, 15]]) v.set(s * x, y, 1, 'fluff');
		});

		// face: sleepy half-closed eyes under heavy lids, soft cheeks, a tiny mouth
		v.side((s) => {
			v.face(s * 1, 12, 'accent');
			v.face(s * 2, 12, 'accent');
			v.face(s * 1, 11, 'pupil');
			v.face(s * 2, 11, 'eye');
			v.face(s * 3, 10, 'cheek');
		});
		v.face(0, 9, 'accent');
	}
};
