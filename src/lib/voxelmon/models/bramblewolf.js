// Bramblewolf, the thorny pack hunter: a green leafy wolf on four sturdy legs, with tall pointed
// ears, a pale muzzle, a ridge of brown bramble thorns and red berries along its back and a big
// bushy leaf tail. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Bramblewolf',
	palette: {
		body: '#3e7a32',
		bodyLight: '#5a9a44',
		bodyDark: '#2a5622',
		belly: '#b8e090',
		accent: '#8a3a2a',
		accentDark: '#5e2418',
		berry: '#e0304a',
		eye: '#ffffff',
		pupil: '#1b2a34',
		nose: '#1e2a1a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// four sturdy legs with dark paws
		v.side((s) => {
			for (const z of [-5, 5]) {
				v.box(s * 3 - 1, 1, z - 1, s * 3 + 1, 6, z + 1, 'body');
				v.box(s * 3 - 1, 0, z - 1, s * 3 + 1, 0, z + 2, 'bodyDark');
			}
		});

		// lean body with a pale chest and belly
		v.ball(0, 9, 0, 5, 4.5, 8.5, 'body');
		v.ball(0, 7.5, 1.5, 3.6, 3, 7, 'belly');

		// big bushy tail sweeping up behind, with a light tip
		v.ball(0, 11, -10, 2.4, 2.4, 2.6, 'body');
		v.ball(0, 14, -12, 2.6, 3, 2.4, 'body');
		v.ball(0, 17, -12.5, 1.8, 1.6, 1.8, 'bodyLight');

		// the head: a round skull with a pale muzzle and a dark nose
		v.ball(0, 13.5, 9, 5.2, 4.6, 4.2, 'body');
		v.box(-2, 10, 11, 2, 12, 15, 'belly');
		v.box(-1, 12, 15, 1, 12, 16, 'nose');

		// tall pointed ears with dark insides
		v.side((s) => {
			v.box(s * 2, 17, 8, s * 4, 17, 9, 'body');
			v.box(s * 3, 18, 8, s * 4, 19, 9, 'body');
			v.set(s * 4, 20, 8, 'body');
			v.box(s * 3, 17, 9, s * 3, 18, 9, 'bodyDark');
		});

		// bramble ridge: thorns zig-zagging down the neck and back, with berries in between
		for (const [x, y, z] of [[0, 17, 5], [0, 15, 2], [0, 14, -2], [0, 14, -6], [0, 13, -9]]) {
			v.box(x, y - 1, z, x, y, z, 'accent');
			v.set(x, y + 1, z - 1, 'accentDark');
		}
		v.side((s) => {
			for (const z of [-4, 0, 3]) v.set(s * 1, 13, z, 'berry');
			for (const z of [-5, 2]) v.set(s * 3, 12, z, 'accent');
		});

		// face: angry eyes with slanted brows, a little fang
		v.side((s) => {
			v.face(s * 2, 14, 'eye');
			v.face(s * 3, 14, 'eye');
			v.face(s * 2, 13, 'eye');
			v.face(s * 3, 13, 'eye');
			v.face(s * 2, 13, 'pupil');
			v.face(s * 2, 14, 'pupil');
			v.face(s * 1, 15, 'bodyDark');
			v.face(s * 2, 15, 'bodyDark');
			v.face(s * 3, 16, 'bodyDark');
		});
		v.box(-2, 9, 13, 2, 9, 14, 'belly');
		v.set(1, 9, 15, 'eye');
	}
};
