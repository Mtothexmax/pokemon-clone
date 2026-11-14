// Rockodile, the rocky crocodile: a big, low, mossy-green croc with a huge head, long toothy jaws,
// angry eye bumps and a double row of stone spikes from neck to tail. Built from voxels with the
// commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Rockodile',
	palette: {
		body: '#7d8c5a',
		bodyLight: '#98a872',
		bodyDark: '#5f6d42',
		belly: '#d8d0a0',
		accent: '#4a5530',
		stone: '#8f8e82',
		stoneLight: '#b4b2a4',
		mouth: '#a83a3a',
		teeth: '#fffbe8',
		eye: '#ffffff',
		pupil: '#1a1a10'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// four thick bent legs with pale claws
		v.side((s) => {
			for (const z of [-6, 3]) {
				v.ball(s * 6, 3, z, 2, 3, 2, 'bodyDark');
				v.box(s * 5, 0, z - 1, s * 7, 1, z + 2, 'bodyDark');
				for (const x of [5, 7]) v.set(s * x, 0, z + 3, 'teeth');
			}
		});

		// long low body with a pale belly
		v.ball(0, 6, -2, 6, 4.5, 9, 'body');
		v.ball(0, 4, -2, 5, 2.5, 8, 'belly');

		// heavy tail tapering to the ground
		v.ball(0, 4.5, -12, 3.5, 3, 5, 'body');
		v.ball(0, 3, -18, 2, 2, 4, 'body');

		// big head with a long upper jaw and a lower jaw, the mouth a bit open
		v.ball(0, 8, 8, 5, 4, 4, 'body');
		v.box(-4, 6, 9, 4, 8, 16, 'body');
		v.box(-3, 9, 10, 3, 9, 15, 'bodyLight');
		v.box(-3, 2, 9, 3, 4, 15, 'bodyDark');
		v.box(-3, 5, 10, 3, 5, 15, 'mouth');
		v.side((s) => {
			for (const z of [16, 14, 12, 10]) v.set(s * 4, 5, z, 'teeth');
			for (const z of [15, 13, 11]) v.set(s * 3, 5, z, 'teeth');
			v.set(s * 2, 9, 16, 'accent'); // nostril
		});
		for (const x of [-2, 0, 2]) v.set(x, 5, 16, 'teeth');

		// stone spikes in two rows along the back, one row on the tail
		v.side((s) => {
			for (const z of [-8, -4, 0, 4]) {
				v.box(s * 2, 10, z, s * 2, 11, z + 1, 'stone');
				v.set(s * 2, 12, z, 'stoneLight');
			}
		});
		for (const [z, y] of [[-10, 8], [-14, 7], [-17, 5]]) v.box(0, y, z, 0, y + 1, z + 1, 'stone');

		// face: eyes on bumps on top of the head, angry slanted brows
		v.side((s) => {
			v.ball(s * 2.5, 12, 9, 1.6, 1.5, 1.5, 'body');
			v.face(s * 2, 12, 'pupil');
			v.face(s * 3, 12, 'eye');
			v.face(s * 2, 11, 'eye');
			v.face(s * 3, 11, 'eye');
			v.box(s * 1, 13, 9, s * 2, 13, 10, 'accent');
			v.box(s * 3, 14, 9, s * 4, 14, 10, 'accent');
		});
	}
};
