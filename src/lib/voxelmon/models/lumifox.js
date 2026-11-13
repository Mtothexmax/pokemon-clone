// Lumifox, the glowing light fox: a cream-white fox with tall glowing-tipped ears, a star mark on the
// forehead and a huge fluffy tail that curls up and shines at the tip.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Lumifox',
	palette: {
		fur: '#fff4d0',
		furShade: '#f0dca0',
		belly: '#ffffff',
		glow: '#ffb030',
		glowLight: '#ffd877',
		eye: '#ffffff',
		pupil: '#3a2a1a',
		nose: '#5a3a2a',
		cheek: '#ff9c8a',
		earInner: '#ffc8a8'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// four slim legs with glowing paws
		v.side((s) => {
			for (const z of [-4, 4]) {
				v.box(s * 3, 1, z - 1, s * 3 + s, 5, z, 'furShade');
				v.box(s * 3, 0, z - 1, s * 3 + s, 0, z + 1, 'glow');
			}
		});

		// small round body with a white chest
		v.ball(0, 7, 0, 4.5, 3.5, 7, 'fur');
		v.ball(0, 7, 4, 3.5, 3, 3.5, 'belly');

		// big fluffy tail curling up behind, glowing at the tip
		v.ball(0, 8, -8, 3, 3, 3, 'fur');
		v.ball(0, 12, -11, 4, 4, 3.5, 'fur');
		v.ball(0, 17, -11, 3.5, 3.5, 3, 'glowLight');
		v.ball(0, 19, -10.5, 2.2, 2, 2, 'glow');

		// the big head with a pointed white snout and a black nose
		v.ball(0, 13, 7, 6, 5, 4.5, 'fur');
		v.ball(0, 11, 9, 5, 2.5, 3, 'belly');
		v.box(-2, 10, 11, 2, 12, 13, 'belly');
		v.box(-1, 12, 14, 1, 12, 14, 'nose');

		// tall pointed ears with glowing tips
		v.side((s) => {
			v.box(s * 4 - 1, 17, 6, s * 4 + 1, 19, 8, 'fur');
			v.box(s * 4, 20, 6, s * 4, 21, 7, 'fur');
			v.box(s * 4, 22, 7, s * 4, 23, 7, 'glow');
			v.box(s * 4, 18, 9, s * 4, 20, 9, 'earInner');
		});

		// glowing star mark on the forehead
		v.face(0, 16, 'glow');
		v.face(0, 15, 'glow');
		v.face(-1, 16, 'glowLight');
		v.face(1, 16, 'glowLight');
		v.face(0, 17, 'glowLight');

		// happy face: big 2x2 eyes with a shine, rosy cheeks and a smile
		v.side((s) => {
			for (const [x, y] of [[2, 13], [3, 13], [2, 14], [3, 14]]) v.face(s * x, y, 'pupil');
			v.face(s * (s > 0 ? 3 : 2), 14, 'eye');
			v.face(s * 5, 12, 'cheek');
			v.face(s * 2, 10, 'nose');
		});
		v.face(-1, 9, 'nose');
		v.face(0, 9, 'nose');
		v.face(1, 9, 'nose');
	}
};
