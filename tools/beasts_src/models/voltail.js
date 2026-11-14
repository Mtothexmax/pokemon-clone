// Voltail, the storm lynx: a stocky yellow cat on four paws with tall tufted ears, a grumpy brow and a
// big purple lightning-bolt tail that zaps enemies. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Voltail',
	palette: {
		body: '#ffc928',
		bodyLight: '#ffdc6a',
		bodyDark: '#e0a818',
		belly: '#fff3b8',
		accent: '#7a4ad6',
		accentLight: '#a47ef0',
		eye: '#ffffff',
		pupil: '#2a1a40',
		brow: '#5a3a10',
		nose: '#3a2a3a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// four sturdy legs with purple paws
		v.side((s) => {
			for (const z of [-3, 3]) {
				v.box(s * 2, 1, z - 1, s * 3, 4, z, 'bodyDark');
				v.box(s * 2, 0, z - 1, s * 3, 0, z + 1, 'accent');
			}
		});

		// long chunky body with a pale belly and purple stripes on the back
		v.ball(0, 6, 0, 4, 3.2, 5.5, 'body');
		v.ball(0, 5, 0.5, 3, 2.2, 4.5, 'belly');
		for (const z of [-2, 1]) v.box(-2, 9, z, 2, 9, z, 'accent');

		// zigzag lightning tail standing up behind it
		v.box(-1, 7, -7, 1, 9, -6, 'accent');
		v.box(-1, 9, -9, 1, 11, -8, 'accent');
		v.box(-1, 11, -7, 1, 13, -6, 'accent');
		v.box(-1, 13, -9, 1, 15, -8, 'accentLight');

		// big round head on the front
		v.ball(0, 10, 5.5, 4.5, 3.8, 3.6, 'body');
		v.dome(0, 12, 5, 3.6, 1.8, 2.8, 'bodyLight');
		v.box(-2, 7, 8, 2, 9, 9, 'belly'); // muzzle

		// tall tufted ears with purple tips
		v.side((s) => {
			v.box(s * 2, 13, 4, s * 4, 14, 5, 'body');
			v.box(s * 3, 15, 4, s * 4, 15, 5, 'body');
			v.box(s * 3, 16, 4, s * 4, 16, 4, 'accent');
		});

		// face: eyes under angry slanted brows, a small nose and a frown
		v.side((s) => {
			for (const [x, y] of [[1, 10], [2, 10], [1, 11], [2, 11]]) v.face(s * x, y, 'eye');
			v.face(s * 1, 10, 'pupil');
			v.face(s * 1, 11, 'pupil');
			v.face(s * 1, 12, 'brow');
			v.face(s * 2, 12, 'brow');
			v.face(s * 3, 13, 'brow');
			v.face(s * 1, 7, 'nose');
		});
		v.face(0, 9, 'nose');
		v.face(0, 8, 'nose');
	}
};
