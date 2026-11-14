// Fungoo, the grumpy mushroom: a chubby cream stem body with stubby arms and feet, a sulky face
// tucked under a huge red cap covered in white dots. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Fungoo',
	palette: {
		body: '#d05050',
		bodyLight: '#e87070',
		bodyDark: '#a83838',
		belly: '#fff0e0',
		bellyDark: '#e6d2bc',
		gill: '#d8b89a',
		accent: '#ffffff',
		mouth: '#5a2a2a',
		eye: '#ffffff',
		pupil: '#2a1a14',
		cheek: '#f0a090'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// stubby round feet
		v.side((s) => v.ball(s * 2, 1, 1, 1.6, 1.2, 2, 'bellyDark'));

		// chubby stem body with little arms
		v.ball(0, 5.5, 0, 4, 4.8, 3.6, 'belly');
		v.side((s) => v.ball(s * 4.3, 5, 0.5, 1, 1.8, 1, 'bellyDark'));

		// big cap: gills underneath, a dark rim, red dome, light top
		v.ball(0, 9.5, 0, 6.5, 0.6, 5, 'gill');
		v.dome(0, 10, 0, 7.6, 6, 6.2, 'bodyDark');
		v.dome(0, 11, 0, 7.2, 5.2, 5.8, 'body');
		v.dome(0, 14, 0, 4.5, 2.4, 4.2, 'bodyLight');

		// white dots on the cap
		for (const [x, y, z] of [[0, 16, 0], [4, 14, 3], [-4, 14, -3], [-4, 14, 3], [4, 14, -3], [0, 13, 4.5], [0, 13, -4.5], [6, 12, 0], [-6, 12, 0], [-2, 11, 5.5], [2, 11, 5.5]])
			v.ball(x, y, z, 1, 1, 1, 'accent');

		// face: grumpy sleepy eyes under heavy lids, a frown
		v.side((s) => {
			v.face(s * 1, 8, 'bellyDark');
			v.face(s * 2, 8, 'bellyDark');
			v.face(s * 1, 7, 'pupil');
			v.face(s * 2, 7, 'eye');
			v.face(s * 3, 6, 'cheek');
			v.face(s * 2, 4, 'mouth');
		});
		for (const x of [-1, 0, 1]) v.face(x, 5, 'mouth');
	}
};
