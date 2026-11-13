// Mosscub, the dozy moss bear: a round bear cub covered in soft moss, with round ears, a pale muzzle,
// a mossy cap on its back with a tiny mushroom and a flower behind one ear. Built from voxels with the
// commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Mosscub',
	palette: {
		body: '#5a9e3a',
		bodyLight: '#78b856',
		bodyDark: '#4a8530',
		belly: '#c8e6a0',
		accent: '#3d6b22',
		cap: '#d9573c',
		capSpot: '#fff4e0',
		flower: '#ffd84a',
		eye: '#ffffff',
		pupil: '#1e2a14',
		cheek: '#e88a7a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two stubby legs with pale pads in front
		v.side((s) => {
			v.box(s * 2, 0, -1, s * 4, 2, 2, 'bodyDark');
			v.box(s * 2, 0, 3, s * 4, 1, 3, 'belly');
		});

		// round tubby body with a pale belly and mossy tufts on the back
		v.ball(0, 5, 0, 4.5, 4, 4, 'body');
		v.ball(0, 4.5, 1.5, 3.2, 3, 3, 'belly');
		for (const [x, y, z] of [[-2, 7, -3], [2, 5, -4], [0, 3, -4], [-3, 4, -3], [3, 8, -2]]) v.ball(x, y, z, 1.2, 1, 1, 'accent');

		// little arms hugging the belly
		v.side((s) => v.box(s * 3, 4, 3, s * 4, 6, 4, 'bodyDark'));

		// big round head with a light mossy top and a pale muzzle
		v.ball(0, 11, 1, 4.8, 3.8, 4, 'body');
		v.dome(0, 13, 0.5, 4, 1.8, 3.2, 'bodyLight');
		v.box(-2, 9, 4, 2, 10, 5, 'belly');
		v.box(-1, 10, 6, 1, 10, 6, 'accent'); // nose

		// round ears, dark inside
		v.side((s) => {
			v.ball(s * 3.5, 14.5, 0, 1.6, 1.6, 1, 'body');
			v.set(s * 3, 14, 1, 'accent');
			v.set(s * 4, 14, 1, 'accent');
		});

		// a little flower behind the right ear and a tiny mushroom on the back
		v.set(6, 14, 0, 'flower');
		v.set(6, 15, 0, 'capSpot');
		v.set(1, 8, -3, 'capSpot');
		v.box(0, 9, -4, 2, 9, -2, 'cap');
		v.set(1, 10, -3, 'cap');
		v.set(0, 9, -4, 'capSpot');

		// face: sleepy half-closed eyes (a lid and a line), cheeks, a small mouth
		v.side((s) => {
			v.face(s * 2, 12, 'bodyDark');
			v.face(s * 3, 12, 'bodyDark');
			v.face(s * 2, 11, 'pupil');
			v.face(s * 3, 11, 'pupil');
			v.face(s * 4, 10, 'cheek');
		});
		v.face(0, 9, 'accent');
	}
};
