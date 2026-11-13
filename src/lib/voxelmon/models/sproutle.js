// Sproutle, the growing seedling: a round green seed with a pale belly, little leaf arms and a two-leaf
// sprout on its head that grows bigger every turn. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Sproutle',
	palette: {
		body: '#7ad35a',
		bodyLight: '#a3e584',
		bodyDark: '#5bb33f',
		belly: '#e6f7c8',
		accent: '#2f8a26',
		leaf: '#4cb83a',
		stem: '#3f9a2e',
		root: '#8a6a44',
		eye: '#ffffff',
		pupil: '#1f2a14',
		cheek: '#ff9a8a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two rooty brown feet
		v.side((s) => v.box(s, 0, -1, s * 3, 1, 2, 'root'));

		// one round seed body: a big head that is also the body, pale belly in front
		v.ball(0, 6.5, 0, 5, 5, 4.5, 'body');
		v.dome(0, 9, -0.5, 4, 2.5, 3.5, 'bodyLight');
		v.ball(0, 4, 2, 3, 2.5, 2.8, 'belly');

		// little leaf arms
		v.side((s) => {
			v.box(s * 5, 5, 0, s * 6, 6, 1, 'leaf');
			v.set(s * 7, 6, 1, 'accent');
		});

		// the sprout: a short stem with two leaves, one to each side
		v.box(0, 11, 0, 0, 14, 0, 'stem');
		v.side((s) => {
			v.ball(s * 2.5, 14.5, 0, 2.2, 1, 1.4, 'leaf');
			v.box(s * 1, 14, 0, s * 4, 14, 0, 'accent'); // leaf vein
		});
		v.set(0, 15, 0, 'bodyLight'); // new bud

		// face: happy closed eyes (arches), rosy cheeks, a smile
		v.side((s) => {
			v.face(s * 1, 8, 'pupil');
			v.face(s * 2, 9, 'pupil');
			v.face(s * 3, 8, 'pupil');
			v.face(s * 4, 6, 'cheek');
			v.face(s * 1, 6, 'accent');
		});
		v.face(0, 5, 'accent');
	}
};
