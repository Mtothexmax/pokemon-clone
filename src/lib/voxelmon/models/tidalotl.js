// Tidalotl, the pink water friend: a chubby pink axolotl standing upright, with a wide flat head,
// three frilly blue gill fronds on each side, a big smile and a finned tail. Built from voxels with
// the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Tidalotl',
	palette: {
		body: '#ff9ec4',
		bodyLight: '#ffc0da',
		bodyDark: '#e07aa4',
		belly: '#ffe0ee',
		accent: '#3d9bff',
		accentLight: '#8cc8ff',
		mouth: '#c0507e',
		eye: '#ffffff',
		pupil: '#2a1a30',
		cheek: '#ff6fa0'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// round stubby feet
		v.side((s) => v.ball(s * 2.5, 1, 1, 1.8, 1.2, 2.2, 'bodyDark'));

		// chubby pear body with a pale tummy and little arms
		v.ball(0, 6, 0, 4.5, 5, 4, 'body');
		v.ball(0, 5.5, 1.5, 3, 3.8, 3, 'belly');
		v.side((s) => v.ball(s * 4.5, 6, 2, 1.2, 2, 1.2, 'bodyDark'));

		// finned tail lying on the ground behind
		v.ball(0, 2.5, -5, 1.8, 2, 3.5, 'body');
		v.ball(0, 2, -8.5, 1.2, 1.4, 2, 'body');
		v.ball(0, 4.5, -6.5, 0.5, 2.2, 3.5, 'bodyLight');

		// wide flat head
		v.ball(0, 13, 1, 6.5, 4, 4.8, 'body');
		v.ball(0, 16, 0, 4.5, 1.2, 3.5, 'bodyLight');

		// three frilly gill fronds on each side, fanning out and up
		v.side((s) => {
			for (const [y, dy] of [[11, -1], [13, 0], [15, 1]])
				for (let t = 0; t < 4; t++) {
					const gy = y + Math.round(t * dy * 0.7);
					v.set(s * (6 + t), gy, 0, 'accent');
					if (t > 0) v.set(s * (6 + t), gy + 1, 0, 'accentLight');
				}
		});

		// face: wide-set happy eyes, rosy cheeks and a big smile
		v.side((s) => {
			for (const [x, y] of [[3, 13], [4, 13], [3, 14], [4, 14]]) v.face(s * x, y, 'eye');
			v.face(s * 3, 13, 'pupil');
			v.face(s * 4, 13, 'pupil');
			v.face(s * 5, 12, 'cheek');
			v.face(s * 2, 12, 'mouth');
		});
		for (const x of [-1, 0, 1]) v.face(x, 11, 'mouth');
	}
};
