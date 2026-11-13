// Pebbit, the sturdy little pebble bunny: a round stone loaf with a big pebble head, two tall
// stony ears and speckles like gravel. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Pebbit',
	palette: {
		body: '#a58c6a',
		bodyLight: '#c2a985',
		bodyDark: '#857052',
		belly: '#e8dcc4',
		accent: '#6b553a',
		earInner: '#e8b4a0',
		eye: '#ffffff',
		pupil: '#2a2018',
		cheek: '#f0957e'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two big flat feet, stepping a little forwards
		v.side((s) => {
			v.box(s * 3 - 1, 0, -1, s * 3 + 1, 1, 3, 'bodyDark');
			v.set(s * 3, 1, 3, 'belly');
		});

		// round pebble body with a pale belly and a fluffy tail
		v.ball(0, 4, 0, 4, 3.5, 3.5, 'body');
		v.ball(0, 3.5, 1.5, 2.8, 2.5, 2.5, 'belly');
		v.ball(0, 3, -4, 1.5, 1.5, 1.5, 'belly');

		// big round head, a bit lighter on top
		v.ball(0, 9, 0.5, 4.5, 3.8, 4, 'body');
		v.dome(0, 11, 0, 3.5, 1.8, 3, 'bodyLight');

		// tall stony ears with pink insides
		v.side((s) => {
			v.box(s * 2, 12, -1, s * 3, 15, 0, 'body');
			v.box(s * 2, 13, 1, s * 3, 15, 1, 'earInner');
			v.set(s * 2, 16, -1, 'bodyLight');
			v.set(s * 3, 16, -1, 'bodyLight');
		});

		// gravel speckles on the back and head
		for (const [x, y, z] of [[-2, 6, -3], [3, 4, -3], [-3, 3, -2], [1, 12, -2], [-3, 10, -3], [4, 9, -1]]) v.set(x, y, z, 'accent');

		// face: happy closed eyes (little arches), rosy cheeks, a tiny nose and buck tooth
		v.side((s) => {
			v.face(s * 1, 10, 'pupil');
			v.face(s * 2, 11, 'pupil');
			v.face(s * 3, 10, 'pupil');
			v.face(s * 4, 8, 'cheek');
		});
		v.face(0, 8, 'accent');
		v.face(0, 7, 'belly');
	}
};
