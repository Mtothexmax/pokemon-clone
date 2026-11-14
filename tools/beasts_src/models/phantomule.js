// Phantomule, the drowsy ghost mule: a purple mule standing on its two front legs, its back half
// fading into a wispy ghost tail that curls down to touch the ground. Long floppy ears, a dark
// shaggy mane, a pale muzzle and sleepy half-closed eyes. Built from voxels with the commands in
// ../draw.js:  x = left/right (0 = middle), y = up (0 = ground), z = forwards (the head looks to +z).
// The 3D Voxelmon AND the card picture are both drawn from this function.
export default {
	name: 'Phantomule',
	palette: {
		body: '#6a5aa0',
		bodyLight: '#8a7cc0',
		bodyDark: '#4a3c7a',
		belly: '#c8c0f0',
		glow: '#e8e4ff',
		accent: '#2a1a4a',
		eye: '#ffffff',
		pupil: '#1b2a34',
		cheek: '#e88ac8'
	},
	/** @param {import('../index').VoxelBuilder} v */
	build(v) {
		// two front legs with dark hooves
		v.side((s) => {
			v.box(s * 2.5 - 1, 2, 3, s * 2.5 + 1, 7, 5, 'body');
			v.box(s * 2.5 - 1, 0, 3, s * 2.5 + 1, 1, 5, 'accent');
		});

		// round body with a pale belly
		v.ball(0, 10, 1, 5, 4.5, 6, 'body');
		v.ball(0, 8.5, 2.5, 3.6, 3, 4.5, 'belly');

		// the back half fades into a wispy ghost tail that curls down to the ground
		v.ball(0, 9.5, -5, 4, 3.5, 3.5, 'body');
		v.ball(0, 7, -8.5, 3, 3, 2.6, 'bodyLight');
		v.ball(0, 4, -10, 2.2, 2.5, 2, 'bodyLight');
		v.ball(0, 1.5, -9, 1.8, 1.6, 2, 'glow');
		v.box(0, 0, -7, 0, 1, -6, 'glow');

		// neck and long-snouted head
		v.ball(0, 13.5, 5.5, 2.6, 3.2, 2.6, 'body');
		v.ball(0, 16.5, 8, 4.5, 4, 4.5, 'body');
		v.box(-2, 12, 10, 2, 15, 14, 'belly');
		v.side((s) => v.set(s * 1, 14, 15, 'bodyDark')); // nostrils

		// long floppy ears, tipping outwards
		v.side((s) => {
			v.box(s * 2, 20, 6, s * 3, 21, 7, 'body');
			v.box(s * 3, 22, 6, s * 4, 23, 7, 'body');
			v.box(s * 3, 21, 7, s * 3, 22, 7, 'cheek');
		});

		// dark shaggy mane from the forehead down the neck
		v.box(-1, 20, 7, 1, 21, 9, 'accent');
		v.box(0, 17, 3, 0, 20, 6, 'accent');
		v.box(0, 13, 1, 0, 16, 3, 'accent');

		// face: sleepy half-closed eyes (dark lid line over a sliver of eye), rosy cheeks, a sleepy smile
		v.side((s) => {
			for (const x of [1, 2, 3]) v.face(s * x, 18, 'accent');
			v.face(s * 2, 17, 'eye');
			v.face(s * 3, 17, 'pupil');
			v.face(s * 4, 16, 'cheek');
		});
		v.box(-1, 12, 15, 1, 12, 15, 'bodyDark');
	}
};
