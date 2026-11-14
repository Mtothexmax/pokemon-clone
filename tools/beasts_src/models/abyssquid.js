// Abyssquid, the legendary deep-sea squid: a tall navy mantle with fins and glowing pink spots, huge
// round eyes, a ring of curling tentacles to stand on and two raised arms holding glowing lure orbs.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Abyssquid',
	palette: {
		body: '#2a3a8a',
		bodyLight: '#3f55b0',
		bodyDark: '#1a2560',
		belly: '#8ab0ff',
		glow: '#ff5a8a',
		glowLight: '#ffb0c8',
		eye: '#ffffff',
		pupil: '#0a1030',
		beak: '#1a1a3a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// eight tentacles spreading out on the ground, tips curling up, pale undersides
		for (let a = 0; a < 8; a++) {
			const dx = Math.sin(((a + 0.5) * Math.PI) / 4), dz = Math.cos(((a + 0.5) * Math.PI) / 4);
			for (const [r, y, size] of [[3, 6, 2], [5, 3, 1.8], [7, 1, 1.5], [9, 1, 1.1]]) v.ball(dx * r, y, dz * r, size, size, size, 'body');
			v.box(Math.round(dx * 10), 2, Math.round(dz * 10), Math.round(dx * 10), 3, Math.round(dz * 10), 'bodyLight');
			v.set(Math.round(dx * 7), 0, Math.round(dz * 7), 'belly');
			v.set(Math.round(dx * 8), 0, Math.round(dz * 8), 'glow');
		}

		// the round head with a pale face and belly
		v.ball(0, 10, 0, 6, 5, 6, 'body');
		v.ball(0, 9, 2, 5, 4, 4.5, 'belly');

		// the tall mantle on top, with fins and glowing spots
		v.ball(0, 17, -1, 5.5, 7, 5, 'body');
		v.ball(0, 23, -1, 3, 2, 3, 'bodyLight');
		v.side((s) => {
			v.box(s * 5, 18, -2, s * 7, 21, -1, 'bodyLight');
			v.box(s * 8, 20, -2, s * 8, 22, -1, 'bodyLight');
		});
		for (const [x, y] of [[0, 19], [-3, 17], [3, 17], [-2, 21], [2, 21], [0, 15]]) v.face(x, y, 'glow');
		v.face(0, 20, 'glowLight');

		// two arms raised forwards, holding glowing lure orbs
		v.side((s) => {
			v.ball(s * 5, 9, 4, 1.8, 1.8, 1.8, 'body');
			v.ball(s * 6.5, 11.5, 5, 1.8, 1.8, 1.8, 'body');
			v.ball(s * 8, 14, 6, 1.5, 1.5, 1.5, 'bodyLight');
			v.ball(s * 8, 17, 6.5, 1.8, 1.8, 1.8, 'glow');
			v.set(s * 8, 18, 8, 'glowLight');
		});

		// big round eyes (3x3 with a big pupil and a shine) and a small beak
		v.side((s) => {
			for (const x of [2, 3, 4]) for (const y of [10, 11, 12]) v.face(s * x, y, 'eye');
			for (const [x, y] of [[2, 10], [3, 10], [2, 11], [3, 11]]) v.face(s * x, y, 'pupil');
		});
		v.face(0, 7, 'beak');
		v.face(0, 8, 'beak');
	}
};
