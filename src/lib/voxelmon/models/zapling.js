// Zapling, the little spark critter: a round yellow fuzzball with dark battery stripes on its back,
// tiny feet and one zigzag antenna with a glowing bulb. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Zapling',
	palette: {
		body: '#ffd84a',
		bodyLight: '#ffe682',
		bodyDark: '#e8b820',
		belly: '#fff6c8',
		accent: '#3a3a3a',
		bulb: '#fffbe0',
		eye: '#ffffff',
		pupil: '#222222',
		cheek: '#ff8a4a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two tiny dark feet
		v.side((s) => v.box(s, 0, 0, s * 2, 1, 2, 'accent'));

		// one round fuzzy body with a pale belly
		v.ball(0, 6, 0, 5, 5, 4.5, 'body');
		v.dome(0, 8, -0.5, 4.2, 3, 3.6, 'bodyLight');
		v.ball(0, 4, 2, 3, 2.5, 2.8, 'belly');

		// dark battery stripes around the back, standing out a little
		for (const [z, r] of [[-1, 5.5], [-3, 4.4]]) v.ball(0, 6, z, r, r, 0.6, 'accent');

		// little round ears
		v.side((s) => v.box(s * 3, 10, -1, s * 4, 11, 0, 'bodyDark'));

		// zigzag antenna with a glowing bulb
		v.box(0, 11, 0, 0, 12, 0, 'accent');
		v.box(1, 12, 0, 1, 13, 0, 'accent');
		v.box(0, 13, 0, 0, 14, 0, 'accent');
		v.ball(0, 15.5, 0, 1.3, 1.3, 1.3, 'bulb');

		// face: big round eyes with a shine, orange cheeks, a tiny cat smile
		v.side((s) => {
			for (const [x, y] of [[1, 7], [2, 7], [1, 8], [2, 8]]) v.face(s * x, y, 'pupil');
			v.face(s * 2, 8, 'eye');
			v.face(s * 4, 5, 'cheek');
			v.face(s * 1, 5, 'accent');
		});
		v.face(0, 4, 'accent');
	}
};
