// Drizzlet, the drowsy raindrop: a plump water pup with a raindrop curl on its head, fin ears, a fin
// ridge on its back and a fishy tail fin. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Drizzlet',
	palette: {
		body: '#4fb0ff',
		bodyLight: '#86caff',
		bodyDark: '#3590e0',
		belly: '#d4f0ff',
		accent: '#1d6fd1',
		fin: '#2a84e8',
		eye: '#ffffff',
		pupil: '#10284a',
		cheek: '#ff9ab8'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two little flipper feet pointing forwards
		v.side((s) => v.box(s, 0, 0, s * 3, 1, 3, 'accent'));

		// plump round body with a pale belly
		v.ball(0, 4, 0, 4, 3.5, 4.5, 'body');
		v.ball(0, 3.5, 1.5, 3, 2.5, 3.2, 'belly');

		// fishy tail fin at the back, flared at the tip
		v.box(0, 2, -7, 0, 4, -4, 'body');
		v.box(-2, 1, -8, 2, 5, -8, 'fin');
		v.box(-1, 2, -7, 1, 4, -7, 'fin');

		// big round head with a light brow and a raindrop curl on top
		v.ball(0, 8.5, 1, 4.5, 3.8, 4, 'body');
		v.dome(0, 10, 0.5, 3.6, 2.2, 3.2, 'bodyLight');
		v.ball(0, 13, 0, 1.5, 1.5, 1.5, 'bodyLight');
		v.set(0, 15, -1, 'bodyLight');
		v.set(0, 16, -1, 'accent');

		// fin ears on the sides of the head and a fin ridge down the back
		v.side((s) => {
			v.box(s * 5, 8, -1, s * 5, 10, 1, 'fin');
			v.box(s * 6, 9, -2, s * 6, 11, 0, 'fin');
		});
		v.box(0, 7, -3, 0, 9, -2, 'fin');
		v.box(0, 6, -5, 0, 7, -4, 'fin');

		// face: sleepy half-closed eyes (a lid and a line), pink cheeks, a small round mouth
		v.side((s) => {
			v.face(s * 2, 10, 'bodyDark');
			v.face(s * 3, 10, 'bodyDark');
			v.face(s * 2, 9, 'pupil');
			v.face(s * 3, 9, 'pupil');
			v.face(s * 4, 7, 'cheek');
		});
		v.face(0, 7, 'accent');
	}
};
