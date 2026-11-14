// Gustling, the little wind bird: a round sky-blue puffball with a white tummy, a curly wind tuft on
// its head, little wings held out to catch the breeze and a teal tail fan. Built from voxels with the
// commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Gustling',
	palette: {
		body: '#9fe6f0',
		bodyLight: '#c8f4f8',
		bodyDark: '#6cc8d6',
		belly: '#ffffff',
		accent: '#2f8f99',
		beak: '#ffb03a',
		feet: '#f08a2a',
		eye: '#ffffff',
		pupil: '#16323a',
		cheek: '#ff9db0'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// little orange legs with three toes
		v.side((s) => {
			v.box(s * 2, 1, 0, s * 2, 2, 0, 'feet');
			v.box(s, 0, 0, s * 3, 0, 1, 'feet');
			v.set(s * 2, 0, 2, 'feet');
		});

		// one round puffball body with a white tummy
		v.ball(0, 7, 0, 5, 5.2, 4.5, 'body');
		v.ball(0, 5.5, 1.5, 3.5, 3.5, 3.2, 'belly');

		// curly wind tuft on top of the head
		for (const [y, z] of [[12, 0], [13, 0], [14, -1], [14, -2], [13, -3]]) v.set(0, y, z, 'accent');
		for (const [y, z] of [[13, 1], [14, 0], [15, -1]]) v.set(0, y, z, 'bodyLight');

		// little wings held out to the sides, teal feather tips
		v.side((s) => {
			v.ball(s * 6.5, 8, -0.5, 2.8, 2.2, 1.2, 'bodyDark');
			v.ball(s * 6, 8.5, 0, 1.8, 1.4, 0.8, 'bodyLight');
			for (const y of [7, 9]) v.set(s * 9, y, -0.5, 'accent');
			v.set(s * 10, 8, -0.5, 'accent');
		});

		// teal tail fan at the back
		v.ball(0, 5, -5, 2.5, 1.5, 1.2, 'accent');
		v.ball(0, 6, -5, 1.2, 1, 0.8, 'bodyDark');

		// face: bright 2x2 eyes with a shine, rosy cheeks, a little orange beak
		v.side((s) => {
			for (const [x, y] of [[2, 9], [3, 9], [2, 10], [3, 10]]) v.face(s * x, y, 'eye');
			v.face(s * 2, 9, 'pupil');
			v.face(s * 3, 9, 'pupil');
			v.face(s * 2, 10, 'pupil');
			v.face(s * 4, 8, 'cheek');
		});
		v.box(-1, 7, 5, 1, 8, 5, 'beak');
		v.set(0, 7, 6, 'beak');
	}
};
