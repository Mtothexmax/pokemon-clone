// Quickhop, the speedy hopper: a sky-blue jumping critter with huge springy feet, long ears swept back
// by the wind and a pink pompom tail. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Quickhop',
	palette: {
		body: '#8fd0ff',
		bodyLight: '#b8e2ff',
		bodyDark: '#68b4ec',
		belly: '#ffffff',
		accent: '#ff6a8a',
		accentLight: '#ffa4b8',
		eye: '#ffffff',
		pupil: '#1c2a40',
		cheek: '#ff8aa4'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// huge springy feet, long and flat, with pink soles at the toes
		v.side((s) => {
			v.box(s * 2, 0, -3, s * 3, 1, 4, 'bodyDark');
			v.box(s * 2, 0, 4, s * 3, 0, 4, 'accent');
		});

		// small leaning body with a white belly
		v.ball(0, 4.5, -0.5, 3.5, 3.5, 3.5, 'body');
		v.ball(0, 4, 1, 2.4, 2.6, 2.4, 'belly');

		// pink pompom tail
		v.ball(0, 4, -4.5, 1.5, 1.5, 1.5, 'accentLight');

		// big round head pushed forwards
		v.ball(0, 9, 1.5, 4.5, 3.6, 3.8, 'body');
		v.dome(0, 10.5, 1, 3.6, 2, 3, 'bodyLight');

		// long ears swept back in the wind, pink inside
		v.side((s) => {
			v.box(s * 2, 12, -1, s * 3, 13, 1, 'body');
			v.box(s * 2, 13, -4, s * 3, 14, -1, 'body');
			v.box(s * 2, 14, -6, s * 3, 14, -3, 'body');
			v.box(s * 2, 14, -2, s * 3, 14, 0, 'accent');
			v.set(s * 2, 13, -5, 'accentLight');
			v.set(s * 3, 13, -5, 'accentLight');
		});

		// face: happy closed eyes (arches), pink cheeks, a pink nose and a grin
		v.side((s) => {
			v.face(s * 1, 9, 'pupil');
			v.face(s * 2, 10, 'pupil');
			v.face(s * 3, 9, 'pupil');
			v.face(s * 3, 7, 'cheek');
			v.face(s * 1, 7, 'pupil');
		});
		v.face(0, 8, 'accent');
		v.face(0, 6, 'pupil');
	}
};
