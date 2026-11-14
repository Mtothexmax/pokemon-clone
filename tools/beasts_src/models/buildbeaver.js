// Buildbeaver, the busy builder: a chubby beaver standing upright in a white hard hat with an
// orange stripe, big buck teeth, a flat paddle tail and a little hammer in one paw.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Buildbeaver',
	palette: {
		body: '#a0603a',
		bodyDark: '#74421f',
		belly: '#f0c890',
		accent: '#f4f4f4',
		stripe: '#ff9a2a',
		nose: '#3a2418',
		tail: '#5a3a28',
		tailDark: '#3e2618',
		wood: '#c89a5a',
		metal: '#8a96a4',
		eye: '#ffffff',
		pupil: '#1b2a34',
		cheek: '#ef8f80'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two big flat feet with pale toes
		v.side((s) => {
			v.box(s * 2.5 - 1.5, 0, -1, s * 2.5 + 1.5, 1, 3, 'bodyDark');
			for (const t of [-1, 1]) v.set(s * 2.5 + t * 1.5, 0, 4, 'belly');
		});

		// flat paddle tail lying on the ground behind, with a criss-cross pattern
		v.box(-1, 1, -6, 1, 3, -4, 'bodyDark');
		v.box(-3, 0, -12, 3, 1, -6, 'tail');
		for (const [x, z] of [[-2, -7], [0, -7], [2, -7], [-1, -9], [1, -9], [-2, -11], [0, -11], [2, -11]]) v.set(x, 1, z, 'tailDark');

		// chubby pear body with a pale tummy
		v.ball(0, 6, 0, 4.6, 5, 4, 'body');
		v.ball(0, 5.5, 1.5, 3.2, 4, 3, 'belly');

		// stubby arms; the right paw holds a little hammer
		v.side((s) => v.ball(s * 4.5, 7, 1.5, 1.4, 2.4, 1.4, 'body'));
		v.box(5, 3, 3, 5, 9, 3, 'wood');
		v.box(4, 9, 2, 6, 10, 4, 'metal');

		// round head with a pale muzzle and a dark nose
		v.ball(0, 12.5, 0.5, 5, 4, 4.5, 'body');
		v.ball(0, 11, 3.5, 3, 2, 2, 'belly');
		v.box(-1, 12, 6, 1, 12, 6, 'nose');

		// little round ears peeking out under the hat
		v.side((s) => v.ball(s * 5, 14, -0.5, 1.1, 1.2, 1, 'bodyDark'));

		// white hard hat with an orange stripe and a front brim
		v.dome(0, 15, 0.5, 5.4, 3.6, 5, 'accent');
		v.dome(0, 15, 0.5, 0.5, 3.7, 5.2, 'stripe');
		v.box(-4, 15, 4, 4, 15, 7, 'accent');

		// face: happy closed ^ ^ eyes, rosy cheeks, two big buck teeth
		v.side((s) => {
			v.face(s * 2, 13, 'pupil');
			v.face(s * 3, 14, 'pupil');
			v.face(s * 4, 13, 'pupil');
			v.face(s * 4, 11, 'cheek');
		});
		v.box(0, 9, 5, 0, 10, 5, 'accent');
		v.set(0, 10, 4, 'nose');
	}
};
