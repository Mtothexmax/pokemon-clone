// Goldbrick, the golden reward: a living golden toy brick with six studs on top, a little gold crest,
// stubby feet and arms, shiny highlights and a big happy face.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Goldbrick',
	palette: {
		gold: '#ffcc33',
		goldLight: '#fff3b8',
		goldDark: '#c28e1b',
		shine: '#ffffff',
		eye: '#ffffff',
		pupil: '#3a2a10',
		cheek: '#ff9c7a',
		mouth: '#8a5a10'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// stubby feet
		v.side((s) => v.box(s * 2, 0, -1, s * 4, 1, 3, 'goldDark'));

		// the brick body with a darker bottom edge and shiny highlights on the corner
		v.box(-5, 2, -4, 5, 9, 4, 'gold');
		v.box(-5, 2, -4, 5, 2, 4, 'goldDark');
		v.box(5, 5, 4, 5, 9, 4, 'goldLight');
		v.box(2, 9, 4, 5, 9, 4, 'goldLight');
		v.set(5, 9, 4, 'shine');

		// six round studs on top
		for (const x of [-4, 0, 4]) for (const z of [-2, 2]) v.dome(x, 10, z, 1.2, 1, 1.2, 'gold');

		// a little gold crest running over the middle studs
		v.box(0, 11, -3, 0, 12, 3, 'goldDark');
		v.box(0, 13, -1, 0, 13, 1, 'goldDark');

		// little arms waving at the sides
		v.side((s) => {
			v.box(s * 6, 5, 0, s * 6, 6, 1, 'gold');
			v.box(s * 7, 6, 1, s * 7, 7, 1, 'gold');
		});

		// happy face: 2x2 eyes with a shine, rosy cheeks and a big smile
		v.side((s) => {
			for (const [x, y] of [[2, 6], [3, 6], [2, 7], [3, 7]]) v.face(s * x, y, 'pupil');
			v.face(s * (s > 0 ? 3 : 2), 7, 'eye');
			v.face(s * 4, 5, 'cheek');
			v.face(s * 2, 4, 'mouth');
		});
		v.face(-1, 3, 'mouth');
		v.face(0, 3, 'mouth');
		v.face(1, 3, 'mouth');
	}
};
