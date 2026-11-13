// Titanoak, the ancient tree titan: a chunky bark turtle with a mossy shell, a little tree growing
// on its back and two branch antlers. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Titanoak',
	palette: {
		bark: '#80562f',
		barkLight: '#a0703e',
		barkDark: '#553820',
		belly: '#e2c08a',
		moss: '#4f9a3a',
		mossLight: '#78c04f',
		mossDark: '#356f28',
		leaf: '#5fb544',
		leafLight: '#9be070',
		blossom: '#ff9cc4',
		toe: '#f3e6c4',
		eye: '#ffffff',
		pupil: '#1b2a34',
		cheek: '#ef8f80'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// four round stumpy legs, a bit apart, with pale toes at the front
		v.side((s) => {
			for (const z of [-5, 6]) {
				v.ball(s * 6, 3, z, 2.6, 3.4, 2.6, 'barkDark');
				v.box(s * 6 - 2, 0, z - 2, s * 6 + 2, 1, z + 2, 'barkDark');
				for (const t of [-1, 1]) v.set(s * 6 + t, 0, z + 3, 'toe');
				v.set(s * 6, 1, z + 3, 'toe');
			}
		});

		// round bark body with a light belly underneath
		v.ball(0, 8, 0, 8, 5, 9.5, 'bark');
		v.ball(0, 6, 0.5, 7, 3, 8.5, 'belly');

		// mossy shell on the back: dark rim, moss, a few light patches like plates
		v.dome(0, 10, -1, 9.2, 5.2, 10.5, 'mossDark');
		v.dome(0, 11, -1, 8.6, 5, 10, 'moss');
		for (const [x, z] of [[-4, -5], [4, -5], [0, -1], [-5, 2], [5, 2], [0, -8], [-3, 5], [3, 5]]) v.ball(x, 15, z, 1.6, 1.4, 1.6, 'mossLight');

		// a little tree growing out of the shell, with pink blossoms
		v.box(-1, 15, -3, 1, 20, -1, 'barkDark');
		v.box(-2, 19, -2, 2, 19, -2, 'barkDark');
		v.ball(0, 23, -2, 6, 3.6, 6, 'leaf');
		v.ball(-2, 25.5, -1, 3.2, 2.4, 3.2, 'leafLight');
		v.ball(2.5, 25, -3.5, 2.6, 2, 2.6, 'leafLight');
		for (const [x, y, z] of [[-5, 23, 1], [3, 26, -1], [1, 27, -4], [-3, 26, -5], [6, 22, -2], [-1, 24, 3]]) v.set(x, y, z, 'blossom');

		// the head: a big friendly block on a short neck, with a light snout
		v.ball(0, 9, 9, 4, 3.5, 3, 'bark');
		v.ball(0, 10, 12, 6, 5, 4.2, 'bark');
		v.ball(0, 13, 11.5, 5, 2, 3.5, 'barkLight'); // brow
		v.box(-3, 6, 14, 3, 9, 16, 'belly');
		v.box(-2, 7, 17, 2, 8, 17, 'belly'); // little nose tip

		// branch antlers, forked, with leaves on the tips
		v.side((s) => {
			v.box(s * 3, 14, 11, s * 3, 17, 11, 'barkDark');
			v.box(s * 3, 17, 11, s * 6, 17, 11, 'barkDark');
			v.box(s * 6, 18, 11, s * 6, 19, 11, 'barkDark');
			v.box(s * 4, 18, 11, s * 4, 19, 11, 'barkDark');
			v.ball(s * 6, 20.5, 11, 1.6, 1.2, 1.6, 'leaf');
			v.ball(s * 4, 20, 11, 1.2, 1, 1.2, 'leafLight');
		});

		// face: big 2x2 eyes with a shine, rosy cheeks, a little smile
		v.side((s) => {
			for (const [x, y] of [[2, 11], [3, 11], [2, 12], [3, 12]]) v.face(s * x, y, 'eye');
			v.face(s * 3, 11, 'pupil');
			v.face(s * 2, 11, 'pupil');
			v.face(s * 5, 9, 'cheek');
		});
		v.side((s) => v.face(s * 2, 7, 'pupil'));
		v.face(-1, 6, 'pupil');
		v.face(0, 6, 'pupil');
		v.face(1, 6, 'pupil');

		// stubby tail with a leaf
		v.box(0, 7, -12, 0, 8, -10, 'barkDark');
		v.ball(0, 9, -13, 1.5, 1.2, 1.5, 'leaf');
	}
};
