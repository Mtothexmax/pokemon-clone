// What the people on TV say. Every line is voiced (scripts/voice-lines.mjs collects them for Fish Audio),
// so keep them fixed text: no numbers or names that change at runtime.

export const TV_ANCHOR = 'BBN Anchor';
export const TV_REPORTER = 'BBN Reporter';
export const TV_NARRATOR = 'Narrator';

/** name shown in the subtitle */
export const TV_NAMES: Record<string, string> = {
	[TV_ANCHOR]: 'Rex Studwell',
	[TV_REPORTER]: 'Penny Plate',
	[TV_NARRATOR]: 'Narrator'
};

export interface TvLine {
	speaker: string;
	text: string;
}

export const TV_SCRIPTS: Record<'news' | 'weatherDay' | 'weatherNight' | 'weatherRain' | 'nature', TvLine[]> = {
	news: [
		{ speaker: TV_ANCHOR, text: "Hello, Brickhollow! This is BBN, the Brick Beast News. I'm Rex Studwell." },
		{ speaker: TV_REPORTER, text: "And I'm Penny Plate. Our top story: Glimmerlings were spotted folding light into tiny bricks!" },
		{ speaker: TV_ANCHOR, text: 'Fascinating. Professor Quill says it is perfectly safe, as long as nobody sneezes.' },
		{ speaker: TV_REPORTER, text: 'In sports: young Dex claims his deck is unbeatable. Local trainers would like a word.' },
		{ speaker: TV_ANCHOR, text: 'The ducks at the pond report record bread deliveries this week. Experts say: more, please.' },
		{ speaker: TV_REPORTER, text: 'And golden studs are still hidden all over the village. If you find one, keep it shiny!' },
		{ speaker: TV_ANCHOR, text: "That's all for now. Stay snappy, Brickhollow!" }
	],
	weatherDay: [
		{ speaker: TV_REPORTER, text: "Here's your Brickhollow weather! Sunny skies all day, perfect for a battle on Route 1." },
		{ speaker: TV_REPORTER, text: 'A little shower may still pop by later. Puddles ahead, so splash responsibly!' }
	],
	weatherNight: [
		{ speaker: TV_REPORTER, text: 'Good evening, Brickhollow. Tonight we expect a clear and starry sky.' },
		{ speaker: TV_REPORTER, text: 'The fireflies are out by the pond. Catch them gently, and let them go again!' }
	],
	weatherRain: [
		{ speaker: TV_REPORTER, text: 'Rain, rain, and a little more rain! Grab your umbrella, Brickhollow.' },
		{ speaker: TV_REPORTER, text: 'The good news: the puddles are perfect for splashing. Pip approves.' }
	],
	nature: [
		{ speaker: TV_NARRATOR, text: 'Here, in the tall grass of Route 1, a young Brick Beast takes its very first steps.' },
		{ speaker: TV_NARRATOR, text: 'Its studs are still soft. Within a week, they will click firmly into place.' },
		{ speaker: TV_NARRATOR, text: 'Look closely. It has found a lost two by four brick. A rare treasure.' },
		{ speaker: TV_NARRATOR, text: 'Brick Beasts talk with a gentle clicking sound. Scientists still argue about what it means.' },
		{ speaker: TV_NARRATOR, text: 'Most of them agree: it means snack time.' },
		{ speaker: TV_NARRATOR, text: 'Nature is patient. And so, it seems, is this little Beast.' }
	]
};
