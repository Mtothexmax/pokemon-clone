import tailwindcss from '@tailwindcss/vite';
import adapterAuto from '@sveltejs/adapter-auto';
import adapterStatic from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

/**
 * Static export (export.ps1 sets SNAP_BASE_PATH, e.g. "/pokemon-clone" for GitHub Pages):
 * adapter-static with 404.html as the SPA fallback, the app under that sub path. Without the variable
 * everything stays as in normal use (adapter-auto, no sub path).
 */
const exportBase = (globalThis as { process?: { env: Record<string, string | undefined> } }).process?.env.SNAP_BASE_PATH;
const isExport = exportBase !== undefined;

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
			// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
			// See https://svelte.dev/docs/kit/adapters for more information about adapters.
			adapter: isExport
				? adapterStatic({ pages: 'build', assets: 'build', fallback: '404.html', precompress: false, strict: false })
				: adapterAuto(),
			paths: { base: (exportBase ?? '').replace(/\/$/, '') as '' | `/${string}` }
		})
	]
});
