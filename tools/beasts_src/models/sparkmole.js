// Sparkmole, the digging mole: a stout brown mole with a cream tummy, huge digging paws, a pink nose
// and a little miner's spark lamp on an antenna. Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Sparkmole',
	palette: {
		body: '#7a6a5a',
		bodyLight: '#9a8a78',
		bodyDark: '#5a4c40',
		belly: '#e8d8c0',
		claw: '#fff6e6',
		nose: '#ff8fa0',
		accent: '#ffd23d',
		glow: '#fff6b0',
		metal: '#4a4a52',
		eye: '#ffffff',
		pupil: '#2a2018',
		cheek: '#e89a8a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// wide flat feet with pale claws
		v.side((s) => {
			v.box(s * 2, 0, -1, s * 4, 1, 2, 'bodyDark');
			for (const x of [2, 3, 4]) v.set(s * x, 0, 3, 'claw');
		});

		// stout pear-shaped body, the head is its round top
		v.ball(0, 5.5, 0, 4.8, 5, 4.2, 'body');
		v.ball(0, 9.5, 1, 3.8, 3, 3.4, 'body');
		v.ball(0, 5, 1.8, 3.2, 3.8, 3, 'belly');

		// big digging paws held up in front, with claws
		v.side((s) => {
			v.ball(s * 4.5, 6, 2.5, 1.8, 1.6, 1.6, 'belly');
			for (const y of [5, 6, 7]) v.set(s * 5, y, 4, 'claw');
		});

		// pointy snout with a pink nose
		v.ball(0, 8, 4.5, 2, 1.5, 1.5, 'bodyLight');
		v.box(-1, 8, 6, 1, 8, 6, 'nose');
		v.set(0, 9, 6, 'nose');

		// miner's spark lamp on an antenna
		v.set(0, 12, 1, 'metal');
		v.ball(0, 13.5, 1, 1.6, 1.1, 1.6, 'accent');
		v.box(-1, 15, 0, 1, 15, 2, 'metal');
		v.set(0, 13, 3, 'glow');
		v.set(0, 14, 3, 'glow');

		// face: sleepy squinting eyes (moles don't like light), soft cheeks
		v.side((s) => {
			v.face(s * 2, 11, 'bodyDark');
			v.face(s * 3, 11, 'bodyDark');
			v.face(s * 2, 10, 'pupil');
			v.face(s * 3, 10, 'eye');
			v.face(s * 3, 8, 'cheek');
		});
	}
};
