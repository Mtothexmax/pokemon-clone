// Emberhorn, the fire ram-bull: a stocky red bull on four dark-hoofed legs, with big curled horns,
// a pale snout, a mane of flames along its back and a little flame on the tip of its tail.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Emberhorn',
	palette: {
		body: '#c8402a',
		bodyDark: '#94291a',
		belly: '#ffc070',
		accent: '#4a2a1a',
		accentLight: '#6e4630',
		flame: '#ff8a2a',
		flameHot: '#ffe070',
		eye: '#ffffff',
		pupil: '#1b2a34',
		nostril: '#7a3a2a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// four sturdy legs with dark hooves
		v.side((s) => {
			for (const z of [-6, 5]) {
				v.box(s * 4 - 1, 2, z - 1, s * 4 + 1, 7, z + 1, 'bodyDark');
				v.box(s * 4 - 1, 0, z - 1, s * 4 + 1, 1, z + 1, 'accent');
			}
		});

		// big barrel body with a warm belly
		v.ball(0, 10, 0, 6.5, 5, 9.5, 'body');
		v.ball(0, 8, 0.5, 5, 3, 8, 'belly');

		// tail with a flame on the tip
		v.box(0, 11, -11, 0, 12, -10, 'bodyDark');
		v.ball(0, 13, -12, 1.2, 1.6, 1.2, 'flame');
		v.set(0, 15, -12, 'flameHot');

		// a mane of flames running down the back, hot yellow at the core
		for (const [z, h] of [[-6, 2], [-2, 3], [2, 4], [6, 5]]) {
			v.ball(0, 14 + h / 2, z, 3, h, 2, 'flame');
			v.ball(0, 14 + h / 2, z + 0.5, 1.5, h - 0.5, 1.2, 'flameHot');
		}

		// the head, low and forward, with a pale snout and dark nostrils
		v.ball(0, 12, 10.5, 5, 4.5, 4, 'body');
		v.box(-3, 7, 13, 3, 11, 16, 'belly');
		v.side((s) => v.set(s * 2, 9, 16, 'nostril'));
		v.box(-1, 7, 16, 1, 7, 16, 'nostril');

		// big curled ram horns
		v.side((s) => {
			v.box(s * 4, 15, 9, s * 7, 16, 11, 'accent');
			v.box(s * 8, 12, 9, s * 9, 16, 11, 'accent');
			v.box(s * 7, 10, 10, s * 8, 11, 12, 'accent');
			v.box(s * 6, 11, 12, s * 6, 12, 13, 'accentLight');
		});

		// face: angry eyes under heavy slanted brows
		v.side((s) => {
			v.face(s * 2, 13, 'eye');
			v.face(s * 3, 13, 'eye');
			v.face(s * 2, 12, 'eye');
			v.face(s * 3, 12, 'pupil');
			v.face(s * 2, 12, 'pupil');
			v.face(s * 1, 14, 'accent');
			v.face(s * 2, 14, 'accent');
			v.face(s * 3, 15, 'accent');
		});
	}
};
