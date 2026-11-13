// URL of a file from static/ (e.g. "/voices/x.mp3"). The game may be served from a sub path, like
// GitHub Pages at /pokemon-clone/ (see export.ps1); Vite fills in that base when building.
// Outside of Vite (node tests) it falls back to "/".
export function assetUrl(path: string) {
	const base: string = (import.meta as { env?: { BASE_URL?: string } }).env?.BASE_URL ?? '/';
	return base.replace(/\/$/, '') + path;
}
