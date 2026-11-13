<script lang="ts">
	// A card whose beast is a real Voxelmon: the 3D model stands in the picture window and sticks out of
	// the card. Drag to tilt / turn the card (the model turns with it); let go and it springs back.
	// The model is drawn in a WebGL canvas centred on the card; its rotation matches the CSS tilt exactly
	// (same perspective, same pivot), so card and beast move as one object.
	import * as THREE from 'three';
	import Card from './Card.svelte';
	import { buildVoxelmon } from '../three/voxel';

	let { id, width, level = 0 }: { id: string; width: string; level?: number } = $props();

	const PERSPECTIVE = 1000; // px, same for the CSS tilt and the 3D camera
	const BASE_YAW = 0; // the beast looks straight out of the card (it turns when you drag)
	let wrap: HTMLDivElement;
	let canvas: HTMLCanvasElement;
	let rx = $state(0); // card tilt in degrees
	let ry = $state(0);
	let dragging = false;
	let vx = 0, vy = 0; // spring speeds
	let start = { x: 0, y: 0, rx: 0, ry: 0 };

	function down(e: PointerEvent) {
		dragging = true;
		wrap.setPointerCapture(e.pointerId);
		start = { x: e.clientX, y: e.clientY, rx, ry };
	}
	function move(e: PointerEvent) {
		if (!dragging) return;
		ry = Math.max(-75, Math.min(75, start.ry + (e.clientX - start.x) * 0.45));
		rx = Math.max(-45, Math.min(45, start.rx - (e.clientY - start.y) * 0.35));
	}
	function up() {
		dragging = false;
		vx = vy = 0;
	}

	$effect(() => {
		const r = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
		r.setPixelRatio(Math.min(2, devicePixelRatio));
		r.outputColorSpace = THREE.SRGBColorSpace;
		const scene = new THREE.Scene();
		scene.add(new THREE.HemisphereLight('#ffffff', '#5a6a7a', 1.6));
		const sun = new THREE.DirectionalLight('#ffffff', 2.2);
		sun.position.set(300, 800, 900);
		scene.add(sun);
		const cam = new THREE.PerspectiveCamera(40, 1, 1, 6000);
		cam.position.set(0, 0, PERSPECTIVE);
		const pivot = new THREE.Group(); // = the card's centre
		scene.add(pivot);
		let mon: THREE.Group | null = null;

		// sizes: the canvas is the card's box grown on all sides, centred on the card
		const layout = () => {
			const cardEl = wrap.querySelector('.card') as HTMLElement | null;
			const img = wrap.querySelector('.card img') as HTMLElement | null;
			if (!cardEl || !img) return;
			const W = cardEl.offsetWidth, H = cardEl.offsetHeight;
			const cw = W * 1.6, ch = H * 1.7;
			canvas.style.width = `${cw}px`;
			canvas.style.height = `${ch}px`;
			r.setSize(cw, ch, false);
			cam.aspect = cw / ch;
			cam.fov = (2 * Math.atan(ch / 2 / PERSPECTIVE) * 180) / Math.PI;
			cam.updateProjectionMatrix();
			// the picture window's floor, measured from the card centre (y up)
			const cardTop = cardEl.getBoundingClientRect().top, imgRect = img.getBoundingClientRect();
			const floor = H / 2 - (imgRect.bottom - cardTop) + imgRect.height * 0.1;
			if (mon) pivot.remove(mon);
			mon = buildVoxelmon(id, W * 0.62);
			if (mon) {
				mon.position.set(0, floor, W * 0.12);
				mon.rotation.y = BASE_YAW;
				pivot.add(mon);
			}
		};
		const ro = new ResizeObserver(layout);
		ro.observe(wrap);
		layout();

		let raf = 0, last = performance.now();
		const loop = (now: number) => {
			const dt = Math.min(0.25, (now - last) / 1000);
			last = now;
			if (!dragging && (rx || ry || vx || vy)) {
				// spring back to rest (a little overshoot, then settle); small steps, so slow frames stay smooth
				let x = rx, y = ry;
				for (let left = dt; left > 0; left -= 1 / 240) {
					const h = Math.min(left, 1 / 240);
					vx += (-x * 90 - vx * 12) * h;
					vy += (-y * 90 - vy * 12) * h;
					x += vx * h;
					y += vy * h;
				}
				if (Math.abs(x) < 0.01 && Math.abs(vx) < 0.01) x = vx = 0;
				if (Math.abs(y) < 0.01 && Math.abs(vy) < 0.01) y = vy = 0;
				rx = x;
				ry = y;
			}
			// same rotation as the CSS tilt: rotateY(ry) turns +x away, rotateX(rx) tips the top away
			pivot.rotation.set((-rx * Math.PI) / 180, (ry * Math.PI) / 180, 0, 'YXZ');
			if (mon) {
				// breathing and a slow look around while resting
				const t = now / 1000;
				const body = mon.userData.body as THREE.Group;
				body.scale.y = 1 + Math.sin(t * 2.2) * 0.02;
				mon.rotation.y = BASE_YAW + Math.sin(t * 0.6) * 0.12;
			}
			r.render(scene, cam);
			raf = requestAnimationFrame(loop);
		};
		raf = requestAnimationFrame(loop);
		return () => {
			cancelAnimationFrame(raf);
			ro.disconnect();
			r.dispose();
		};
	});
</script>

<div
	class="c3d"
	bind:this={wrap}
	role="img"
	aria-label="3D card: drag to turn it"
	onpointerdown={down}
	onpointermove={move}
	onpointerup={up}
	onpointercancel={up}
>
	<div class="tilt" style="transform: perspective({PERSPECTIVE}px) rotateY({ry}deg) rotateX({rx}deg)">
		<Card {id} size="lg" {width} {level} backdrop />
	</div>
	<canvas bind:this={canvas}></canvas>
	<span class="hint"><span class="material-symbols-rounded">360</span> drag me</span>
</div>

<style>
	.c3d {
		position: relative;
		display: inline-block;
		touch-action: none;
		cursor: grab;
		user-select: none;
	}
	.c3d:active {
		cursor: grabbing;
	}
	.tilt {
		transform-origin: 50% 50%;
	}
	/* the 3D beast: centred on the card, larger than it, so it can stick out */
	canvas {
		position: absolute;
		left: 50%;
		top: 50%;
		translate: -50% -50%;
		pointer-events: none;
	}
	.hint {
		position: absolute;
		right: -0.4rem;
		bottom: -0.4rem;
		display: flex;
		align-items: center;
		gap: 0.2rem;
		padding: 0.15rem 0.55rem;
		border-radius: 999px;
		background: linear-gradient(135deg, #3dbbff, #7a5aff);
		color: #fff;
		font-size: 0.75rem;
		font-weight: 700;
		pointer-events: none;
	}
</style>
