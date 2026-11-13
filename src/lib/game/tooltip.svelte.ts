// One shared, styled tooltip (instead of the browser's title tooltip). Shown by hovering e.g. a card's ability badge.
export interface Tip {
	title: string;
	text: string;
	icon: string;
	color: string;
	/** anchor rectangle in viewport px */
	x: number;
	y: number;
	w: number;
	h: number;
}

class TooltipState {
	tip = $state<Tip | null>(null);
	show(el: Element, t: Omit<Tip, 'x' | 'y' | 'w' | 'h'>) {
		const r = el.getBoundingClientRect();
		this.tip = { ...t, x: r.left, y: r.top, w: r.width, h: r.height };
	}
	hide() {
		this.tip = null;
	}
}

export const tooltip = new TooltipState();
