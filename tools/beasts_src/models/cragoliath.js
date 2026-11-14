// Cragoliath, the rock golem: a hunched stone giant with huge knuckle-walking fists, a mossy
// boulder grown onto its back and a small grumpy head sunk between its shoulders.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Cragoliath',
	palette: {
		stone: '#8a8a8a',
		stoneLight: '#a8a8a8',
		stoneDark: '#4a4a4a',
		belly: '#c8c8c8',
		boulder: '#6e6a66',
		moss: '#6aa84f',
		eye: '#ffffff',
		pupil: '#1a1a1a',
		gem: '#7fd6e8'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two short thick legs with flat feet
		v.side((s) => {
			v.ball(s * 4, 4, -1, 2.8, 4, 2.8, 'stone');
			v.box(s * 4 - 3, 0, -3, s * 4 + 3, 1, 2, 'stoneDark');
		});

		// broad chunky torso with a pale stone chest
		v.ball(0, 11, 0, 7, 6, 5, 'stone');
		v.ball(0, 10, 2, 5, 4, 3.5, 'belly');
		v.face(0, 12, 'gem'); // a little crystal in the chest
		v.face(0, 11, 'gem');

		// the big boulder on its back, with dark cracks and moss on top
		v.ball(0, 13, -5, 7.5, 7, 5.5, 'boulder');
		v.box(-8, 13, -6, 8, 13, -6, 'stoneDark');
		v.box(0, 8, -10, 0, 18, -10, 'stoneDark');
		v.dome(-2, 19, -5, 4, 1.5, 3, 'moss');

		// huge arms hanging down to giant knuckle fists
		v.side((s) => {
			v.ball(s * 8, 15, 1, 3.2, 3, 3.2, 'stoneLight');
			v.box(s * 9 - 1, 5, 0, s * 9 + 1, 13, 2, 'stone');
			v.ball(s * 9, 3, 2, 3.2, 3, 3.5, 'stoneLight');
			for (const z of [3, 5]) v.set(s * 9, 0, z, 'stoneDark');
		});

		// small grumpy head sunk between the shoulders, with a heavy brow
		v.ball(0, 16, 4, 5, 4.2, 4, 'stone');
		v.box(-4, 19, 5, 4, 19, 7, 'stoneLight');

		// angry face: 2x2 eyes with slanted brows and a flat frowning mouth
		v.side((s) => {
			for (const [x, y] of [[1, 17], [2, 17], [1, 16], [2, 16]]) v.face(s * x, y, 'eye');
			v.face(s * 1, 16, 'pupil');
			v.face(s * 1, 17, 'pupil');
			v.face(s * 3, 19, 'stoneDark');
			v.face(s * 2, 18, 'stoneDark');
			v.face(s * 1, 18, 'stoneDark');
			v.face(s * 2, 13, 'stoneDark');
		});
		v.face(-1, 14, 'stoneDark');
		v.face(0, 14, 'stoneDark');
		v.face(1, 14, 'stoneDark');
	}
};
