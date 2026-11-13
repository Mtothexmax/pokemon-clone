// Coralyth, the living reef: a plump pink sea critter sitting up on stubby feet, with blue frilly
// fin ears, a crown of branching coral on its head (with tiny blue fish-bubbles) and a wide fish
// tail fin resting on the ground behind it. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Coralyth',
	palette: {
		body: '#ff7a8a',
		bodyDark: '#d85566',
		belly: '#ffe0e4',
		accent: '#3d9bff',
		accentLight: '#8ccaff',
		coral: '#ff9a6a',
		coralTip: '#fff0c8',
		eye: '#ffffff',
		pupil: '#1b2a34',
		cheek: '#ff4f6a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// stubby feet
		v.side((s) => v.box(s * 3 - 1, 0, -1, s * 3 + 1, 1, 3, 'bodyDark'));

		// wide fish tail fin lying on the ground behind
		v.ball(0, 3, -5, 2.5, 2, 2.5, 'body');
		v.box(-1, 1, -9, 1, 2, -7, 'body');
		v.side((s) => {
			v.box(0, 0, -12, s * 4, 1, -10, 'accent');
			v.box(s * 3, 0, -13, s * 5, 1, -13, 'accentLight');
		});

		// plump round body with a pale belly
		v.ball(0, 6, 0, 5, 5, 4.5, 'body');
		v.ball(0, 5.5, 1.5, 3.5, 4, 3.4, 'belly');

		// little fin arms
		v.side((s) => {
			v.ball(s * 5, 6, 1, 1.4, 2, 1.4, 'body');
			v.box(s * 6, 4, 0, s * 7, 5, 1, 'accent');
		});

		// big round head
		v.ball(0, 13, 0.5, 5.5, 4.5, 5, 'body');
		v.ball(0, 11.5, 3, 3.6, 2.2, 2.6, 'belly');

		// frilly blue fin ears, three spikes each
		v.side((s) => {
			v.box(s * 6, 11, -1, s * 6, 15, 1, 'accent');
			for (const [y, z] of [[16, 0], [15, -2], [13, 2]]) v.set(s * 6, y, z, 'accentLight');
			v.box(s * 7, 12, -1, s * 7, 14, 1, 'accent');
			v.set(s * 7, 15, 0, 'accentLight');
		});

		// coral crown: a middle trunk and two side branches that fork, with pale tips
		v.box(0, 17, -1, 0, 21, -1, 'coral');
		v.box(-1, 20, -1, 1, 20, -1, 'coral');
		v.set(0, 22, -1, 'coralTip');
		v.side((s) => {
			v.box(s * 2, 17, 0, s * 3, 18, 0, 'coral');
			v.box(s * 3, 19, 0, s * 3, 20, 0, 'coral');
			v.box(s * 4, 20, 0, s * 5, 20, 0, 'coral');
			v.set(s * 3, 21, 0, 'coralTip');
			v.set(s * 5, 21, 0, 'coralTip');
			v.box(s * 2, 15, -3, s * 2, 19, -3, 'coral');
			v.set(s * 2, 20, -3, 'coralTip');
		});
		v.side((s) => v.set(s * 1, 21, -1, 'accentLight')); // tiny blue polyps

		// little coral nubs down the back
		for (const y of [6, 13]) v.box(0, y, -5, 0, y + 1, -5, 'coral');

		// face: big round 2x2 eyes with a shine, rosy cheeks, a small o-mouth
		v.side((s) => {
			for (const [x, y] of [[2, 13], [3, 13], [2, 14], [3, 14]]) v.face(s * x, y, 'pupil');
			v.face(s * 3, 14, 'eye');
			v.face(s * 4, 12, 'cheek');
		});
		v.face(0, 11, 'bodyDark');
		v.face(0, 10, 'bodyDark');
	}
};
