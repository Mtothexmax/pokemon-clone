// Glimmerling, the sparkly light sprite: a glowing pale-yellow blob with a big round head, two
// star-tipped antennae and little sparkles on its body, standing between two little bricks it made.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Glimmerling',
	palette: {
		body: '#fff0a0',
		bodyLight: '#fffbe0',
		bodyDark: '#f0d860',
		belly: '#ffffff',
		accent: '#f0a020',
		accentLight: '#ffc850',
		brick: '#e8402a',
		brickBlue: '#3a7be0',
		eye: '#ffffff',
		pupil: '#3a2a10',
		cheek: '#ff9a8a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// little round feet
		v.side((s) => v.ball(s * 2, 1, 1, 1.5, 1, 1.8, 'bodyDark'));

		// small round body with a glowing white tummy and stubby arms
		v.ball(0, 4.5, 0, 3.5, 3.5, 3, 'body');
		v.ball(0, 4.5, 1.5, 2.2, 2.2, 2, 'belly');
		v.side((s) => v.ball(s * 3.8, 5, 1, 1, 1, 1, 'bodyDark'));

		// big round head with a light top and a little glow tuft
		v.ball(0, 11, 0.5, 5, 4.5, 4.5, 'body');
		v.dome(0, 13, 0, 3.5, 2.6, 3.5, 'bodyLight');
		v.set(0, 16, 0, 'accentLight');

		// two antennae with orange star tips
		v.side((s) => {
			for (const [x, y] of [[2, 15], [3, 16]]) v.set(s * x, y, 0, 'accent');
			v.set(s * 3, 18, 0, 'accentLight');
			for (const [x, y] of [[3, 17], [3, 19], [2, 18], [4, 18]]) v.set(s * x, y, 0, 'accent');
		});

		// sparkles twinkling on the head and back
		for (const [x, y, z] of [[4, 13, -2], [-4, 13, -2], [0, 12, -4], [3, 8, -3], [-3, 8, -3]]) v.set(x, y, z, 'belly');

		// two little bricks it made, on the ground at its sides (red right, blue left)
		v.side((s) => {
			const c = s > 0 ? 'brick' : 'brickBlue';
			v.box(s * 5, 0, 1, s * 7, 1, 3, c);
			v.set(s * 5, 2, 2, c);
			v.set(s * 7, 2, 2, c);
		});

		// face: big happy eyes with a shine, rosy cheeks, a wide smile
		v.side((s) => {
			for (const [x, y] of [[2, 11], [3, 11], [2, 12], [3, 12]]) v.face(s * x, y, 'eye');
			v.face(s * 2, 11, 'pupil');
			v.face(s * 3, 11, 'pupil');
			v.face(s * 2, 12, 'pupil');
			v.face(s * 4, 10, 'cheek');
			v.face(s * 2, 9, 'pupil');
		});
		for (const x of [-1, 0, 1]) v.face(x, 8, 'pupil');
	}
};
