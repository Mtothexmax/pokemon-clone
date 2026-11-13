// Snapshots of the village (P key / camera button), kept apart from the save so the save stays small.
const KEY = 'brickhollow-photos';
export const MAX_PHOTOS = 9;

export interface Photo {
	src: string; // small JPEG data URL
	t: number; // when it was taken (ms)
	where: string; // area label
}

class PhotoAlbum {
	list = $state<Photo[]>([]);
	load() {
		try {
			const raw = localStorage.getItem(KEY);
			if (raw) this.list = JSON.parse(raw);
		} catch {
			/* storage unavailable */
		}
	}
	add(p: Photo) {
		this.list = [p, ...this.list].slice(0, MAX_PHOTOS);
		try {
			localStorage.setItem(KEY, JSON.stringify(this.list));
		} catch {
			// storage full: keep fewer photos
			this.list = this.list.slice(0, 4);
			try {
				localStorage.setItem(KEY, JSON.stringify(this.list));
			} catch {
				/* give up silently */
			}
		}
	}
}
export const album = new PhotoAlbum();
