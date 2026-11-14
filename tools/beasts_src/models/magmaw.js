// Magmaw, the walking volcano: a squat rock beast with a smoking volcano cone on its back, lava
// cracks, a huge underbite jaw full of glowing teeth and spiky rock shoulders.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Magmaw',
	palette: {
		rock: '#5a2a2a',
		rockLight: '#7a3c34',
		rockDark: '#3a1a1a',
		lava: '#ff7a2a',
		lavaHot: '#ffd23d',
		tooth: '#fff1c8',
		eye: '#ffe9a0',
		pupil: '#1a0a0a',
		smoke: '#9a8f8a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// four thick stumpy legs with dark rock feet
		v.side((s) => {
			for (const z of [-4, 5]) {
				v.ball(s * 6, 3, z, 2.6, 3.4, 2.6, 'rock');
				v.box(s * 6 - 2, 0, z - 2, s * 6 + 2, 1, z + 2, 'rockDark');
			}
		});

		// heavy rock body with a glowing lava belly
		v.ball(0, 8, 0, 8, 5.5, 9, 'rock');
		v.ball(0, 6, 1, 6.5, 3, 7.5, 'lava');

		// the volcano cone on the back, getting narrower to a lava crater
		for (let y = 11; y <= 19; y++) v.ball(0, y, -2, 7 - (y - 11) * 0.6, 0.5, 6.5 - (y - 11) * 0.55, y % 3 ? 'rock' : 'rockLight');
		v.box(-1, 18, -3, 1, 19, -1, 'lavaHot');
		v.box(-2, 19, -4, 2, 19, 0, 'lava');
		v.box(-1, 19, -3, 1, 19, -1, 'lavaHot');
		// lava running down the cone
		for (const [x, y0, z] of [[-2, 15, 2], [2, 13, 3], [4, 12, -1], [-4, 13, -5]]) v.box(x, y0, z, x, y0 + 3, z, 'lava');
		// puffs of smoke above the crater
		v.ball(0, 21, -2, 1.6, 1, 1.6, 'smoke');
		v.ball(1, 23, -3, 1.2, 1, 1.2, 'smoke');

		// spiky rock shoulders
		v.side((s) => {
			for (const [z, h] of [[5, 3], [1, 4], [-3, 3]]) v.box(s * 7, 10, z, s * 7, 10 + h, z, 'rockDark');
		});

		// the big head with a heavy brow
		v.ball(0, 10, 9, 6, 5, 4.5, 'rock');
		v.box(-5, 13, 10, 5, 14, 13, 'rockLight');
		// the huge lower jaw sticking out with fangs and a lava glow inside
		v.box(-6, 4, 9, 6, 7, 16, 'rock');
		v.box(-4, 7, 11, 4, 8, 15, 'lava');
		v.box(-3, 8, 12, 3, 8, 14, 'lavaHot');
		v.side((s) => {
			v.box(s * 5, 8, 16, s * 5, 10, 16, 'tooth');
			v.set(s * 3, 8, 16, 'tooth');
			v.set(s * 1, 8, 16, 'tooth');
		});

		// angry face: glowing 2x2 eyes under slanted brows
		v.side((s) => {
			for (const [x, y] of [[2, 11], [3, 11], [2, 12], [3, 12]]) v.face(s * x, y, 'eye');
			v.face(s * 2, 11, 'pupil');
			v.face(s * 2, 12, 'pupil');
			v.face(s * 4, 14, 'rockDark');
			v.face(s * 3, 13, 'rockDark');
			v.face(s * 2, 13, 'rockDark');
			v.face(s * 1, 13, 'rockDark');
		});

		// short spiked tail
		v.box(0, 6, -12, 0, 7, -9, 'rock');
		v.box(0, 8, -12, 0, 9, -12, 'rockDark');
	}
};
