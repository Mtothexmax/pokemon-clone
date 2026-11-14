// Stormaw, the legendary storm beast: a stout navy brute standing on two legs, wrapped in a mane of
// storm clouds, with a zigzag lightning crest, a roaring open jaw and a lightning-bolt tail.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Stormaw',
	palette: {
		body: '#2a2a5a',
		bodyLight: '#3d3d80',
		bodyDark: '#1a1a3a',
		belly: '#ffe060',
		bolt: '#ffd23d',
		boltLight: '#fff6a8',
		cloud: '#c8cce0',
		cloudDark: '#8a90b0',
		eye: '#ffffff',
		pupil: '#101020',
		mouth: '#8a1a3a',
		fang: '#ffffff'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two thick legs with clawed feet
		v.side((s) => {
			v.ball(s * 4, 3, 0, 3, 3.5, 3.5, 'body');
			v.box(s * 4 - 2, 0, -2, s * 4 + 2, 1, 3, 'bodyDark');
			for (const x of [-1, 1]) v.set(s * 4 + x, 0, 4, 'fang');
		});

		// big round body with a yellow belly and a lightning zigzag on it
		v.ball(0, 10, 0, 7, 7, 6, 'body');
		v.ball(0, 9, 2, 5, 5.5, 4.5, 'belly');
		for (const [x, y] of [[1, 13], [0, 12], [-1, 11], [0, 10], [1, 9], [0, 8], [-1, 7], [0, 6]]) v.face(x, y, 'bolt');

		// lightning-bolt tail, zigzagging up behind
		v.box(0, 6, -9, 0, 8, -5, 'body');
		v.box(0, 8, -10, 0, 11, -9, 'bolt');
		v.box(0, 11, -12, 0, 12, -10, 'bolt');
		v.box(0, 12, -13, 0, 16, -12, 'boltLight');

		// stubby arms reaching forward with white claws
		v.side((s) => {
			v.ball(s * 7, 13, 2, 2.6, 2.6, 2.6, 'bodyLight');
			v.box(s * 7 - 1, 10, 3, s * 7 + 1, 12, 6, 'body');
			for (const x of [-1, 1]) v.set(s * 7 + x, 10, 7, 'fang');
		});

		// a mane of storm clouds around the neck
		for (let a = 0; a < 8; a++) {
			const x = Math.round(Math.sin((a * Math.PI) / 4) * 6), z = Math.round(Math.cos((a * Math.PI) / 4) * 5) - 1;
			v.ball(x, 16, z, 2.6, 2.2, 2.6, a % 2 ? 'cloudDark' : 'cloud');
		}

		// the head: a broad block with a heavy snout and a wide roaring jaw
		v.ball(0, 20, 3, 5.5, 4.5, 5, 'body');
		v.box(-3, 19, 7, 3, 20, 10, 'bodyLight');
		v.box(-3, 17, 7, 3, 18, 9, 'mouth');
		v.box(-3, 15, 6, 3, 16, 10, 'body');
		v.side((s) => {
			v.set(s * 2, 18, 10, 'fang');
			v.set(s * 3, 17, 10, 'fang');
			v.set(s * 2, 17, 10, 'fang');
		});

		// zigzag lightning crest from the forehead back over the head
		v.box(0, 24, 5, 0, 25, 6, 'bolt');
		v.box(0, 25, 2, 0, 26, 4, 'bolt');
		v.box(0, 26, 1, 0, 27, 1, 'boltLight');
		v.box(0, 24, -1, 0, 25, 0, 'bolt');
		v.side((s) => {
			v.box(s * 4, 23, 2, s * 4, 25, 2, 'bolt'); // little bolt horns
			v.box(s * 5, 25, 2, s * 5, 26, 2, 'boltLight');
		});

		// angry face: glowing 2x2 eyes under slanted brows
		v.side((s) => {
			for (const [x, y] of [[2, 21], [3, 21], [2, 22], [3, 22]]) v.face(s * x, y, 'eye');
			v.face(s * 2, 21, 'pupil');
			v.face(s * 2, 22, 'pupil');
			v.face(s * 4, 24, 'bodyDark');
			v.face(s * 3, 23, 'bodyDark');
			v.face(s * 2, 23, 'bodyDark');
			v.face(s * 1, 23, 'bodyDark');
		});
	}
};
