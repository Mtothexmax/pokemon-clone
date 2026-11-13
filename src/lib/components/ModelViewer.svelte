<script lang="ts">
	// Debug tool: a card next to its 3D Voxelmon model. Drag to turn, wheel to zoom, arrows to browse all beasts.
	import * as THREE from 'three';
	import { untrack } from 'svelte';
	import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
	import Card from './Card.svelte';
	import { CARD_MAP } from '../game/cards';
	import { buildVoxelmon } from '../three/voxel';

	let { id: startId, ids, onClose }: { id: string; ids: string[]; onClose: () => void } = $props();
	// starts at the tapped card; browsing changes it from then on
	let id = $state(untrack(() => startId));
	let host: HTMLDivElement;
	let spin = $state(true);
	let wire = $state(false);
	let info = $state('');
	const idx = $derived(Math.max(0, ids.indexOf(id)));
	const step = (d: number) => (id = ids[(idx + d + ids.length) % ids.length]);

	// one renderer for the viewer's whole life; the model is swapped when the card changes
	let ctx: { scene: THREE.Scene; mon: THREE.Group | null; controls: OrbitControls } | null = null;
	$effect(() => {
		const r = new THREE.WebGLRenderer({ antialias: true, alpha: true });
		r.setPixelRatio(Math.min(2, devicePixelRatio));
		r.outputColorSpace = THREE.SRGBColorSpace;
		r.shadowMap.enabled = true;
		host.appendChild(r.domElement);
		const scene = new THREE.Scene();
		const cam = new THREE.PerspectiveCamera(35, 1, 0.1, 200);
		cam.position.set(0, 4, 13);
		scene.add(new THREE.HemisphereLight('#ffffff', '#5a6a7a', 1.5));
		const sun = new THREE.DirectionalLight('#ffffff', 2.2);
		sun.position.set(5, 10, 7);
		sun.castShadow = true;
		sun.shadow.mapSize.set(1024, 1024);
		scene.add(sun);
		// a baseplate with studs to stand on
		const plateMat = new THREE.MeshStandardMaterial({ color: '#4fae3f', roughness: 0.6 });
		const plate = new THREE.Mesh(new THREE.BoxGeometry(8, 0.3, 8), plateMat);
		plate.position.y = -0.15;
		plate.receiveShadow = true;
		scene.add(plate);
		const stud = new THREE.CylinderGeometry(0.24, 0.24, 0.18, 12);
		for (let x = -3.5; x <= 3.5; x++)
			for (let z = -3.5; z <= 3.5; z++) {
				const s = new THREE.Mesh(stud, plateMat);
				s.position.set(x, 0.09, z);
				s.receiveShadow = true;
				scene.add(s);
			}
		const controls = new OrbitControls(cam, r.domElement);
		controls.target.set(0, 2.1, 0);
		controls.enableDamping = true;
		controls.minDistance = 4;
		controls.maxDistance = 30;
		controls.addEventListener('start', () => (spin = false));
		ctx = { scene, mon: null, controls };
		const ro = new ResizeObserver(() => {
			const w = host.clientWidth, h = host.clientHeight;
			r.setSize(w, h);
			cam.aspect = w / h;
			cam.updateProjectionMatrix();
		});
		ro.observe(host);
		let raf = 0;
		const loop = () => {
			if (ctx?.mon && spin) ctx.mon.rotation.y += 0.01;
			controls.update();
			r.render(scene, cam);
			raf = requestAnimationFrame(loop);
		};
		loop();
		return () => {
			cancelAnimationFrame(raf);
			ro.disconnect();
			controls.dispose();
			r.dispose();
			r.domElement.remove();
			ctx = null;
		};
	});
	// swap the model when the card changes
	$effect(() => {
		const cur = id;
		if (!ctx) return;
		if (ctx.mon) {
			ctx.scene.remove(ctx.mon);
			(ctx.mon.userData.mat as THREE.Material).dispose();
		}
		const mon = buildVoxelmon(cur, 4.2);
		ctx.mon = mon;
		if (!mon) {
			info = 'No sprite for this card';
			return;
		}
		ctx.scene.add(mon);
		const mesh = (mon.userData.body as THREE.Group).children[0] as THREE.Mesh;
		const geo = mesh.geometry;
		const size = geo.userData.size as { w: number; h: number };
		info = `${size.w}×${size.h} px sprite · ${(geo.getAttribute('position').count / 6).toLocaleString()} voxel faces`;
	});
	$effect(() => {
		const w = wire;
		const m = ctx?.mon?.userData.mat as THREE.MeshStandardMaterial | undefined;
		if (m) m.wireframe = w;
	});

	function onKey(e: KeyboardEvent) {
		// the deck builder underneath listens too: this viewer takes these keys for itself
		if (!['Escape', 'ArrowRight', 'ArrowLeft'].includes(e.key)) return;
		e.stopImmediatePropagation();
		e.preventDefault(); // marks it as handled for the deck builder
		if (e.key === 'Escape') onClose();
		else step(e.key === 'ArrowRight' ? 1 : -1);
	}
</script>

<svelte:window onkeydowncapture={onKey} />

<div class="viewer" role="dialog" aria-label="3D model of {CARD_MAP[id].name}">
	<header>
		<span class="material-symbols-rounded">view_in_ar</span>
		<b>{CARD_MAP[id].name}</b>
		<small>{idx + 1} / {ids.length} · {info}</small>
		<span class="grow"></span>
		<button class="btn" class:on={spin} onclick={() => (spin = !spin)} title="Turn by itself"><span class="material-symbols-rounded">360</span></button>
		<button class="btn" class:on={wire} onclick={() => (wire = !wire)} title="Wireframe"><span class="material-symbols-rounded">grid_on</span></button>
		<button class="btn" onclick={onClose} title="Close (Esc)"><span class="material-symbols-rounded">close</span></button>
	</header>
	<div class="body">
		<button class="nav" onclick={() => step(-1)} title="Previous (←)"><span class="material-symbols-rounded">chevron_left</span></button>
		<div class="flat">
			<small>2D card</small>
			{#key id}<Card {id} size="lg" width="min(36vw, 52dvh, 280px)" />{/key}
		</div>
		<div class="three">
			<small>3D Voxelmon · drag to turn, wheel to zoom</small>
			<div class="stage" bind:this={host}></div>
		</div>
		<button class="nav" onclick={() => step(1)} title="Next (→)"><span class="material-symbols-rounded">chevron_right</span></button>
	</div>
</div>

<style>
	.viewer {
		position: fixed;
		inset: 0;
		z-index: 60;
		display: flex;
		flex-direction: column;
		background: radial-gradient(circle at 60% 45%, #2a4a6a, #0d1622 75%);
		color: #fff;
		font-family: 'Fredoka', system-ui, sans-serif;
	}
	header {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.6rem 1rem;
		background: rgb(0 0 0 / 0.3);
	}
	header b {
		font-size: 1.2rem;
	}
	header small {
		opacity: 0.7;
	}
	.grow {
		flex: 1;
	}
	.btn.on {
		background: #ffd23d;
		color: #1b2a34;
	}
	.body {
		flex: 1;
		min-height: 0;
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem;
	}
	.flat,
	.three {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.4rem;
	}
	.three {
		flex: 1;
		align-self: stretch;
		min-width: 0;
	}
	.flat small,
	.three small {
		opacity: 0.7;
	}
	.stage {
		flex: 1;
		width: 100%;
		min-height: 0;
		border-radius: 1rem;
		background: rgb(255 255 255 / 0.05);
		overflow: hidden;
		cursor: grab;
	}
	.nav {
		display: grid;
		place-items: center;
		width: 2.6rem;
		height: 2.6rem;
		border-radius: 50%;
		background: rgb(255 255 255 / 0.12);
	}
	@media (max-width: 640px) {
		.body {
			flex-direction: column;
		}
		.nav {
			display: none;
		}
	}
</style>
