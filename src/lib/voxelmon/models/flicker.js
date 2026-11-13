// Flicker, the little ember sprite: a round glowing ember with stubby arms and a big flickering flame
// tuft on its head that cheers its friends on. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Flicker',
	palette: {
		body: '#ff8a3d',
		bodyDark: '#e06a24',
		belly: '#ffe08a',
		accent: '#d9361b',
		flame: '#ffb02e',
		flameLight: '#fff2b0',
		eye: '#ffffff',
		pupil: '#3a1a10',
		cheek: '#ff5e5e'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two stubby dark-red feet
		v.side((s) => v.box(s * 2 - 1, 0, -1, s * 2 + 1, 1, 2, 'accent'));

		// round ember body with a warm yellow belly
		v.ball(0, 6, 0, 5, 5, 4.5, 'body');
		v.ball(0, 4.5, 1.5, 3.2, 3, 3.2, 'belly');
		v.dome(0, 8.5, -0.5, 4.2, 2.5, 3.6, 'bodyDark');
		v.ball(0, 8, 0.5, 4.6, 2.4, 4.2, 'body');

		// little arms held up in a cheer
		v.side((s) => {
			v.box(s * 5, 5, 0, s * 6, 6, 1, 'body');
			v.set(s * 6, 7, 1, 'accent');
		});

		// the flame tuft: red outside, orange and pale yellow at the heart, leaning back a little
		v.ball(0, 11.5, -0.5, 3, 2.5, 2.5, 'accent');
		v.ball(0, 13.5, -1, 2, 2.2, 1.8, 'accent');
		v.ball(0, 11.5, 0.5, 2.2, 2, 2, 'flame');
		v.box(0, 15, -1, 0, 16, -2, 'accent');
		v.ball(0, 12.5, 1, 1.2, 1.5, 1.2, 'flameLight');
		v.side((s) => v.box(s * 2, 13, 0, s * 2, 14, 0, 'flame'));

		// face: big round eyes with a shine, red cheeks, a small open smile
		v.side((s) => {
			for (const [x, y] of [[1, 7], [2, 7], [1, 8], [2, 8]]) v.face(s * x, y, 'eye');
			v.face(s * 1, 7, 'pupil');
			v.face(s * 2, 7, 'pupil');
			v.face(s * 1, 8, 'pupil');
			v.face(s * 4, 6, 'cheek');
		});
		v.face(0, 5, 'accent');
	}
};
