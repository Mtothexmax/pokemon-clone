// Clockcrab, the clockwork crab: a round orange crab whose flat shell is a pocket-watch face with
// hands and hour marks, a golden wind-up key on its back, big claws and round eyes on stalks.
// Built from voxels with the commands in ../draw.js:
//   x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Clockcrab',
	palette: {
		shell: '#e06a3a',
		shellDark: '#b04a24',
		shellLight: '#f08a5a',
		belly: '#ffe0c0',
		gold: '#ffd23d',
		goldDark: '#c29a1b',
		hand: '#3a2a2a',
		eye: '#ffffff',
		pupil: '#1a1a2a',
		cheek: '#ff9c8a'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// three thin legs on each side, stepping outwards
		v.side((s) => {
			for (const z of [-4, -1, 2]) {
				v.box(s * 7, 4, z, s * 9, 4, z, 'shellDark');
				v.box(s * 10, 0, z, s * 10, 4, z, 'shellDark');
			}
		});

		// the round shell, flat on top, with a pale underside
		v.ball(0, 6, 0, 8, 6, 7.5, 'shell');
		v.box(-9, 11, -9, 9, 13, 9, null);
		v.ball(0, 4, 1, 7, 2.5, 6.5, 'belly');

		// the clock face on top: gold rim, cream dial, hour marks and two hands
		v.ball(0, 10, 0, 6.6, 0.5, 6.2, 'gold');
		v.ball(0, 10, 0, 5.4, 0.5, 5, 'belly');
		for (const [x, z] of [[0, -4], [4, 0], [0, 4], [-4, 0]]) v.set(x, 10, z, 'shellDark');
		v.box(0, 10, -3, 0, 10, 0, 'hand'); // hour hand, pointing to 12 (the back)
		v.box(0, 10, 0, 3, 10, 0, 'hand'); // minute hand, pointing to 3
		v.set(0, 11, 0, 'gold');

		// golden wind-up key at the back of the shell
		v.box(0, 9, -8, 0, 15, -8, 'goldDark');
		v.side((s) => {
			v.box(s * 1, 15, -8, s * 4, 19, -8, 'gold');
			v.box(s * 2, 16, -8, s * 3, 18, -8, null);
		});

		// big claws reaching forwards, each with an open pincer
		v.side((s) => {
			v.box(s * 7, 5, 4, s * 8, 6, 7, 'shellDark');
			v.ball(s * 9, 7, 10, 3, 3, 3, 'shell');
			v.box(s * 9 - 1, 7, 11, s * 9 + 1, 7, 13, null);
			v.set(s * 9, 10, 10, 'shellLight');
		});

		// round eyes on stalks, with big pupils
		v.side((s) => {
			v.box(s * 3, 10, 4, s * 3, 14, 4, 'shellDark');
			v.ball(s * 3, 16.5, 4.5, 2.2, 2.2, 1.5, 'eye');
			for (const [x, y] of [[2, 16], [3, 16], [2, 15], [3, 15]]) v.face(s * x, y, 'pupil');
			v.face(s * 4, 8, 'cheek');
		});

		// a little smiling mouth on the front of the shell
		v.face(-1, 6, 'hand');
		v.face(0, 6, 'hand');
		v.face(1, 6, 'hand');
		v.side((s) => v.face(s * 2, 7, 'hand'));
	}
};
