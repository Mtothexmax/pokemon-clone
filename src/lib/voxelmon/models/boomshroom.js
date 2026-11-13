// Boomshroom, the explosive mushroom: a grumpy little mushroom with a pale stalk body, stubby
// arms and feet, a huge pink cap with white dots and a lit fuse sparking on top.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Boomshroom',
	palette: {
		body: '#e04a8a',
		bodyDark: '#b02e66',
		belly: '#fff0f6',
		bellyDark: '#ecc8d8',
		accent: '#ffffff',
		fuse: '#4a3a3a',
		spark: '#ffb02a',
		sparkHot: '#fff070',
		eye: '#ffffff',
		pupil: '#1b2a34'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// stubby feet
		v.side((s) => v.box(s * 2 - 1, 0, -1, s * 2 + 1, 1, 3, 'bellyDark'));

		// chubby stalk body and little arms
		v.ball(0, 6, 0, 4.5, 5.5, 4, 'belly');
		v.side((s) => v.ball(s * 5, 5.5, 1, 1.3, 2, 1.3, 'bellyDark'));

		// huge cap: dark gills underneath, pink dome on top
		v.dome(0, 10.5, 0, 8.5, 6.5, 8, 'body');
		v.ball(0, 10, 0, 8, 1, 7.5, 'bodyDark');

		// white dots on the cap
		for (const [x, y, z] of [[0, 17, 0], [-5, 14.5, 3.5], [5, 14.5, 3.5], [0, 14, 6], [-6, 13.5, -3.5], [6, 13.5, -3.5], [-3, 16, -3], [3, 16, -3], [0, 13.5, -6.5]]) v.ball(x, y, z, 1.3, 1.1, 1.3, 'accent');

		// a lit fuse sticking out of the top
		v.box(0, 17, 0, 0, 19, 0, 'fuse');
		v.set(0, 19, 1, 'fuse');
		v.set(0, 20, 1, 'spark');
		v.side((s) => v.set(s * 1, 20, 1, 'spark'));
		v.set(0, 21, 1, 'sparkHot');

		// face: angry 2x2 eyes under slanted brows, a grumpy frown
		v.side((s) => {
			for (const [x, y] of [[1, 6], [2, 6], [1, 7], [2, 7]]) v.face(s * x, y, 'eye');
			v.face(s * 1, 6, 'pupil');
			v.face(s * 1, 7, 'pupil');
			v.face(s * 1, 8, 'pupil');
			v.face(s * 2, 9, 'pupil');
			v.face(s * 3, 9, 'pupil');
			v.face(s * 2, 3, 'pupil');
		});
		v.face(-1, 4, 'pupil');
		v.face(0, 4, 'pupil');
		v.face(1, 4, 'pupil');
	}
};
