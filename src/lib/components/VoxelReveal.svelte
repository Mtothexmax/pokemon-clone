<script lang="ts">
	// The big moment: a 6-energy card reached its last stage and its beast steps out as a real 3D Voxelmon.
	// A small three.js stage shows the voxel model spinning on a glowing baseplate.
	import * as THREE from 'three';
	import { CARD_MAP } from '../game/cards';
	import { buildVoxelmon } from '../three/voxel';
	import { skillsOf } from '../three/voxelmon';

	let { id, onClose }: { id: string; onClose: () => void } = $props();
	let host: HTMLDivElement;
	const def = $derived(CARD_MAP[id]);
	const skills = $derived(skillsOf(id));

	$effect(() => {
		const W = host.clientWidth, H = host.clientHeight;
		const r = new THREE.WebGLRenderer({ antialias: true, alpha: true });
		r.setPixelRatio(Math.min(2, devicePixelRatio));
		r.setSize(W, H);
		r.outputColorSpace = THREE.SRGBColorSpace;
		host.appendChild(r.domElement);
		const scene = new THREE.Scene();
		const cam = new THREE.PerspectiveCamera(35, W / H, 0.1, 100);
		cam.position.set(0, 3.2, 11);
		cam.lookAt(0, 2, 0);
		scene.add(new THREE.HemisphereLight('#ffffff', '#5a6a7a', 1.6));
		const sun = new THREE.DirectionalLight('#ffffff', 2.2);
		sun.position.set(4, 8, 6);
		scene.add(sun);
		// glowing round baseplate with studs
		const plate = new THREE.Mesh(new THREE.CylinderGeometry(3, 3, 0.3, 40), new THREE.MeshStandardMaterial({ color: '#3d6fd1', emissive: '#1e3a8a', emissiveIntensity: 0.6 }));
		plate.position.y = -0.15;
		scene.add(plate);
		for (let i = 0; i < 12; i++) {
			const a = (i / 12) * Math.PI * 2;
			const s = new THREE.Mesh(new THREE.CylinderGeometry(0.22, 0.22, 0.16, 12), plate.material);
			s.position.set(Math.cos(a) * 2.4, 0.05, Math.sin(a) * 2.4);
			scene.add(s);
		}
		const mon = buildVoxelmon(id, 4.2);
		if (mon) scene.add(mon);
		let raf = 0;
		const t0 = performance.now();
		const loop = () => {
			const t = (performance.now() - t0) / 1000;
			if (mon) {
				// rises out of the card, then turns slowly
				const k = Math.min(1, t / 1.2);
				const e = 1 - Math.pow(1 - k, 3);
				mon.scale.setScalar(Math.max(0.01, e));
				mon.position.y = (1 - e) * -1.5 + Math.abs(Math.sin(t * 2)) * 0.15 * k;
				mon.rotation.y = t * 0.9 + (1 - e) * 6;
			}
			r.render(scene, cam);
			raf = requestAnimationFrame(loop);
		};
		loop();
		return () => {
			cancelAnimationFrame(raf);
			r.dispose();
			r.domElement.remove();
		};
	});
</script>

<div class="reveal" role="dialog" aria-label="{def.name} became a Voxelmon">
	<div class="rays"></div>
	<h2>{def.name} is a real <b>Voxelmon</b> now!</h2>
	<div class="stage" bind:this={host}></div>
	<div class="skills">
		{#each skills as s, i}
			<div class="skill" style="--c:{s.color}"><span class="material-symbols-rounded">{s.icon}</span><b>{s.name}</b><small>Key {i + 1} · recharges in {s.cd}s</small></div>
		{/each}
	</div>
	<p>It walks with you now. Show it to Prof. Quill at the village exit!</p>
	<button class="go" onclick={onClose}><span class="material-symbols-rounded">pets</span> Let's go!</button>
</div>

<style>
	.reveal {
		position: fixed;
		inset: 0;
		z-index: 60;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.6rem;
		padding: 1rem;
		background: radial-gradient(circle at 50% 45%, #2a4a8a, #0d1622 70%);
		color: #fff;
		font-family: 'Fredoka', system-ui, sans-serif;
		text-align: center;
		overflow: hidden;
		animation: fadein 0.4s ease-out;
	}
	@keyframes fadein {
		from {
			opacity: 0;
		}
	}
	.rays {
		position: absolute;
		left: 50%;
		top: 45%;
		width: 160vmax;
		height: 160vmax;
		translate: -50% -50%;
		background: repeating-conic-gradient(rgb(255 255 255 / 0.07) 0 8deg, transparent 8deg 20deg);
		animation: spin 30s linear infinite;
		pointer-events: none;
	}
	@keyframes spin {
		to {
			rotate: 360deg;
		}
	}
	h2 {
		position: relative;
		font-size: clamp(1.3rem, 4vw, 2.2rem);
		font-weight: 800;
	}
	h2 b {
		background: linear-gradient(90deg, #3dbbff, #b06aff, #ffd23d);
		-webkit-background-clip: text;
		background-clip: text;
		color: transparent;
	}
	.stage {
		position: relative;
		width: min(90vw, 26rem);
		height: min(46vh, 22rem);
	}
	.skills {
		position: relative;
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 0.6rem;
	}
	.skill {
		display: grid;
		grid-template-columns: auto 1fr;
		align-items: center;
		column-gap: 0.4rem;
		padding: 0.35rem 0.8rem;
		border-radius: 0.8rem;
		background: rgb(255 255 255 / 0.1);
		border: 0.12rem solid var(--c);
		text-align: left;
	}
	.skill .material-symbols-rounded {
		grid-row: span 2;
		font-size: 1.8rem;
		color: var(--c);
	}
	.skill small {
		opacity: 0.75;
	}
	p {
		position: relative;
		opacity: 0.85;
	}
	.go {
		position: relative;
		display: flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.6rem 1.6rem;
		border-radius: 999px;
		background: linear-gradient(135deg, #ffd23d, #f29b05);
		color: #1b2a34;
		font-weight: 800;
		font-size: 1.1rem;
		box-shadow: 0 0.25rem 0 #a86a00;
	}
</style>
