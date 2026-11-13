// Glaciomoth, the frost moth: a fluffy pale-blue moth standing on little feet, with a snowy white
// fur collar, feathery antennae, sleepy eyes and four big diamond-shaped ice crystal wings spread
// wide behind it. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Glaciomoth',
	palette: {
		body: '#a8e0ff',
		bodyDark: '#78bce8',
		belly: '#ffffff',
		accent: '#3a7ad0',
		accentDark: '#24508c',
		crystal: '#d8f4ff',
		eye: '#ffffff',
		pupil: '#1b2a34',
		cheek: '#ff9cc4'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// a flat diamond (one voxel thick) in the wing plane behind the body
		/** @param {number} cx @param {number} cy @param {number} r @param {string} c */
		const diamond = (cx, cy, r, c) => {
			for (let dy = -r; dy <= r; dy++) v.box(cx - (r - Math.abs(dy)), cy + dy, -2, cx + (r - Math.abs(dy)), cy + dy, -2, c);
		};

		// four crystal wings: blue rim, pale ice inside, a dark blue eye spot in the middle
		v.side((s) => {
			diamond(s * 8, 15, 6, 'accent');
			diamond(s * 8, 15, 5, 'crystal');
			diamond(s * 9, 16, 2, 'accent');
			v.set(s * 9, 16, -2, 'belly');
			diamond(s * 6, 7, 4, 'accent');
			diamond(s * 6, 7, 3, 'crystal');
			diamond(s * 6, 7, 1, 'bodyDark');
		});

		// little dark blue feet
		v.side((s) => v.box(s * 2 - 1, 0, 0, s * 2 + 1, 1, 2, 'accentDark'));

		// round fuzzy body with soft stripes
		v.ball(0, 6.5, 0, 4, 5, 3.6, 'body');
		for (const y of [3, 6]) v.ball(0, y, 0, 3.6, 0.5, 3.6, 'bodyDark');
		v.ball(0, 6.5, 0, 3.8, 4.8, 3.4, 'body');

		// snowy fur collar
		v.ball(0, 11, 0.5, 5.6, 2, 5, 'belly');

		// big round head
		v.ball(0, 15, 1, 5, 4, 4.5, 'body');
		v.dome(0, 17, 0.5, 3.5, 2, 3, 'crystal');

		// feathery antennae curling outwards
		v.side((s) => {
			v.box(s * 2, 19, 1, s * 2, 20, 1, 'accent');
			v.box(s * 3, 21, 1, s * 3, 22, 1, 'accent');
			v.set(s * 4, 23, 1, 'accent');
			for (const y of [21, 22]) v.set(s * 4, y, 1, 'accentDark');
			v.set(s * 2, 22, 1, 'accentDark');
		});

		// face: sleepy half-closed eyes, rosy cheeks, a tiny mouth
		v.side((s) => {
			for (const x of [1, 2, 3]) v.face(s * x, 15, 'accentDark');
			v.face(s * 2, 14, 'eye');
			v.face(s * 3, 14, 'pupil');
			v.face(s * 4, 13, 'cheek');
		});
		v.face(0, 13, 'accentDark');
	}
};
