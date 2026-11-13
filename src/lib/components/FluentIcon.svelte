<script lang="ts" module>
	import music from '@fluentui/svg-icons/icons/music_note_2_24_filled.svg?raw';
	import mute from '@fluentui/svg-icons/icons/speaker_mute_24_filled.svg?raw';
	import speaker from '@fluentui/svg-icons/icons/speaker_2_24_filled.svg?raw';
	import deck from '@fluentui/svg-icons/icons/style_guide_24_filled.svg?raw';
	import undo from '@fluentui/svg-icons/icons/arrow_undo_24_filled.svg?raw';
	import flash from '@fluentui/svg-icons/icons/flash_24_filled.svg?raw';
	import shield from '@fluentui/svg-icons/icons/shield_24_filled.svg?raw';
	import trophy from '@fluentui/svg-icons/icons/trophy_24_filled.svg?raw';
	import chat from '@fluentui/svg-icons/icons/chat_24_filled.svg?raw';
	import backpack from '@fluentui/svg-icons/icons/backpack_24_filled.svg?raw';
	import flag from '@fluentui/svg-icons/icons/flag_24_filled.svg?raw';
	import dismiss from '@fluentui/svg-icons/icons/dismiss_24_filled.svg?raw';
	import help from '@fluentui/svg-icons/icons/question_circle_24_filled.svg?raw';
	import person from '@fluentui/svg-icons/icons/person_24_filled.svg?raw';
	import reset from '@fluentui/svg-icons/icons/arrow_reset_24_filled.svg?raw';
	import save from '@fluentui/svg-icons/icons/save_24_filled.svg?raw';

	const ICONS = { music, mute, speaker, deck, undo, flash, shield, trophy, chat, backpack, flag, dismiss, help, person, reset, save };
	export type FluentName = keyof typeof ICONS;
</script>

<script lang="ts">
	let { name, size = 22 }: { name: FluentName; size?: number } = $props();
	const svg = $derived(
		ICONS[name].replace('<svg ', `<svg fill="currentColor" width="${size}" height="${size}" `).replace(/ width="24" height="24"/, '')
	);
</script>

<span class="inline-flex shrink-0" aria-hidden="true">{@html svg}</span>
