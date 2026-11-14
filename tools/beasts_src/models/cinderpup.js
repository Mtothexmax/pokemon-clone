// Cinderpup, the hot-headed puppy: a chunky red-orange pup with sooty floppy ears and paws, a stubby
// snout, a smouldering flame tuft on its head and a flame on the tip of its tail. Built from voxels with
// the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Cinderpup',
	palette: {
		body: '#e8562a',
		bodyLight: '#f47a48',
		bodyDark: '#c4421c',
		belly: '#ffd0a0',
		accent: '#3a2a20',
		accentLight: '#5a4434',
		flame: '#ffb040',
		flameLight: '#ffe08a',
		eye: '#ffffff',
		pupil: '#1e1410',
		tongue: '#ff7a8a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// four short legs with sooty paws
		v.side((s) => {
			for (const z of [-3, 2]) {
				v.box(s * 2, 1, z - 1, s * 3, 3, z, 'bodyDark');
				v.box(s * 2, 0, z - 1, s * 3, 0, z + 1, 'accent');
			}
		});

		// round chunky body with a peach belly
		v.ball(0, 5, -0.5, 4, 3, 4.5, 'body');
		v.ball(0, 4, 0, 3, 2, 3.5, 'belly');

		// a stubby tail curling up, burning at the tip
		v.box(0, 6, -6, 0, 8, -5, 'body');
		v.ball(0, 9.5, -6, 1.3, 1.5, 1.3, 'flame');
		v.set(0, 11, -6, 'flameLight');

		// big round head with a peach snout and a dark nose
		v.ball(0, 9, 3.5, 4.5, 3.8, 3.6, 'body');
		v.dome(0, 11, 3, 3.6, 1.8, 2.8, 'bodyLight');
		v.box(-2, 6, 6, 2, 8, 8, 'belly');
		v.box(-1, 8, 8, 1, 8, 8, 'accent');

		// sooty floppy ears hanging down the sides of the head
		v.side((s) => {
			v.box(s * 4, 10, 2, s * 5, 12, 4, 'accent');
			v.box(s * 5, 7, 2, s * 5, 9, 4, 'accentLight');
		});

		// smouldering flame tuft on top of the head
		v.ball(0, 13, 3, 1.6, 1.4, 1.6, 'flame');
		v.set(0, 14, 2, 'flameLight');
		v.set(0, 15, 2, 'flame');

		// face: eyes under angry slanted brows, a grumpy mouth with a little tongue
		v.side((s) => {
			for (const [x, y] of [[1, 9], [2, 9], [1, 10], [2, 10]]) v.face(s * x, y, 'eye');
			v.face(s * 1, 9, 'pupil');
			v.face(s * 1, 10, 'pupil');
			v.face(s * 1, 11, 'accent');
			v.face(s * 2, 11, 'accent');
			v.face(s * 2, 12, 'accent');
			v.face(s * 1, 6, 'accent');
		});
		v.face(0, 7, 'accent');
		v.face(0, 6, 'tongue');
	}
};
