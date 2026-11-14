// Brickling, the tiny sparkle brick: a little two-stud toy brick on stubby feet, with a bobbing
// antenna that ends in a glowing sparkle star and a happy little face.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Brickling',
	palette: {
		body: '#ffe680',
		bodyDark: '#e0b84a',
		belly: '#ffffff',
		spark: '#f0a020',
		sparkLight: '#fff2b0',
		eye: '#ffffff',
		pupil: '#3a2a10',
		cheek: '#ff9c7a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// stubby feet
		v.side((s) => v.box(s * 2, 0, -1, s * 2, 0, 1, 'bodyDark'));

		// the little brick body with a darker bottom edge and two studs on top
		v.box(-3, 1, -2, 3, 5, 2, 'body');
		v.box(-3, 1, -2, 3, 1, 2, 'bodyDark');
		v.box(3, 4, 2, 3, 5, 2, 'belly'); // a shiny corner
		v.side((s) => v.box(s * 2, 6, 0, s * 2 - s, 6, 0, 'body'));

		// antenna with a sparkle star on the tip
		v.box(0, 6, 0, 0, 6, 0, 'bodyDark');
		v.ball(0, 8, 0, 1, 1, 1, 'spark');
		v.set(0, 8, 0, 'sparkLight');
		v.set(0, 8, 1, 'sparkLight');

		// happy face: eyes with a shine, rosy cheeks and a tiny smile
		v.side((s) => {
			v.face(s * 2, 3, 'pupil');
			v.face(s * 2, 4, 'eye');
			v.face(s * 3, 2, 'cheek');
		});
		v.face(0, 2, 'pupil');
	}
};
