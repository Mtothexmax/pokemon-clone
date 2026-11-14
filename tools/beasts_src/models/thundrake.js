// Thundrake, the thunder dragon: a chunky blue dragon standing on two stout legs, with a yellow
// belly, zig-zag lightning horns, little bat wings, a row of bolt spikes down its back and a tail
// that ends in a lightning bolt. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Thundrake',
	palette: {
		body: '#3a4ad0',
		bodyLight: '#5a6af0',
		bodyDark: '#26309a',
		belly: '#ffe060',
		bellyDark: '#e8bc30',
		accent: '#ffd23d',
		wing: '#7d8cff',
		claw: '#f4f4f4',
		eye: '#ffffff',
		pupil: '#1b2a34',
		mouth: '#1a1a40'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two stout legs with big feet and white claws
		v.side((s) => {
			v.ball(s * 4, 4, 0, 2.4, 3.5, 2.8, 'bodyDark');
			v.box(s * 4 - 2, 0, -2, s * 4 + 2, 1, 3, 'bodyDark');
			for (const t of [-1, 1]) v.set(s * 4 + t * 1.5, 0, 4, 'claw');
		});

		// thick tail curling back along the ground, ending in a lightning bolt
		v.ball(0, 5, -6, 3, 2.6, 3, 'body');
		v.ball(0, 3, -9.5, 2.2, 2, 2.6, 'body');
		v.ball(0, 2, -12.5, 1.5, 1.5, 2, 'body');
		for (const [y, z] of [[2, -15], [3, -15], [3, -16], [4, -16], [4, -17], [5, -17], [5, -16], [6, -16], [6, -17], [7, -17], [7, -18], [8, -18]]) v.set(0, y, z, 'accent');

		// big round body with a striped yellow belly
		v.ball(0, 9, 0, 6.5, 6, 5.5, 'body');
		v.ball(0, 8.5, 2, 4.6, 5.2, 4, 'belly');
		for (const y of [5, 8, 11]) v.box(-3, y, 5, 3, y, 6, 'bellyDark');
		v.ball(0, 8.5, 2, 4.6, 5.2, 3.9, 'belly');

		// short arms with claws
		v.side((s) => {
			v.ball(s * 6, 10.5, 2.5, 1.6, 2.6, 1.6, 'body');
			v.set(s * 6, 8, 4, 'claw');
		});

		// little bat wings, spread out to the sides
		v.side((s) => {
			for (let i = 0; i < 6; i++) v.box(s * (5 + i), 11 + i, -4, s * (5 + i), 17 - (i > 3 ? i - 3 : 0), -4, 'wing');
			v.box(s * 5, 17, -4, s * 10, 17, -4, 'bodyDark');
			v.set(s * 11, 18, -4, 'bodyDark');
		});

		// bolt spikes down the back
		for (const [y, z] of [[14, -4], [11, -6], [8, -7]]) {
			v.set(0, y, z, 'accent');
			v.set(0, y + 1, z - 1, 'accent');
		}

		// big head with a wide snout and a pale chin
		v.ball(0, 16.5, 1, 5.5, 4.5, 5, 'body');
		v.ball(0, 14, 5, 3.8, 2.3, 3.4, 'body');
		v.ball(0, 13, 5, 3.2, 1.4, 3.2, 'belly');
		v.side((s) => v.set(s * 1, 15, 8, 'bodyDark')); // nostrils

		// zig-zag lightning horns
		v.side((s) => {
			v.box(s * 3, 20, -1, s * 3, 21, -1, 'accent');
			v.box(s * 4, 21, -2, s * 4, 22, -2, 'accent');
			v.set(s * 5, 23, -3, 'accent');
		});

		// face: angry 2x2 eyes under slanted brows, a toothy frown
		v.side((s) => {
			for (const [x, y] of [[2, 17], [3, 17], [2, 18], [3, 18]]) v.face(s * x, y, 'eye');
			v.face(s * 2, 17, 'pupil');
			v.face(s * 2, 18, 'pupil');
			v.face(s * 1, 19, 'bodyDark');
			v.face(s * 2, 19, 'bodyDark');
			v.face(s * 3, 20, 'bodyDark');
			v.face(s * 4, 20, 'bodyDark');
		});
		v.box(-2, 13, 8, 2, 13, 8, 'mouth');
		v.side((s) => v.set(s * 2, 13, 8, 'claw'));
	}
};
