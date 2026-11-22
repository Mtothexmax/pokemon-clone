"""The battle's portraits: each minifig head (assets/minifig/<style>/head.glb, and its talking prints head_mouth1/2)
rendered small on a transparent background -> assets/portraits/<style>_<0|1|2>.png. The battle shows the opponent's
and yours instead of a letter, and swaps the opponent's by the voice's loudness while it talks.
Run: blender -b --factory-startup --python tools/portraits/render_heads.py -- <repo wasm dir>
"""
import bpy, sys, math, mathutils
root = sys.argv[sys.argv.index('--') + 1]
STYLES = ["player", "dex", "lumi", "pip", "quill", "mom", "baker", "knight"]
for style in STYLES:
    for k, name in enumerate(["head", "head_mouth1", "head_mouth2"]):
        bpy.ops.wm.read_factory_settings(use_empty=True)
        sc = bpy.context.scene
        bpy.ops.import_scene.gltf(filepath=f"{root}/assets/minifig/{style}/{name}.glb")
        objs = [o for o in sc.objects if o.type == 'MESH']
        lo = mathutils.Vector((1e9, 1e9, 1e9)); hi = -lo
        for o in objs:
            for v in o.bound_box:
                w = o.matrix_world @ mathutils.Vector(v)
                lo = mathutils.Vector(map(min, lo, w)); hi = mathutils.Vector(map(max, hi, w))
        c = (lo + hi) / 2
        size = max(hi - lo)
        cam = bpy.data.objects.new('cam', bpy.data.cameras.new('cam')); sc.collection.objects.link(cam); sc.camera = cam
        cam.data.type = 'ORTHO'
        cam.data.ortho_scale = size * 1.12
        # the face looks along glTF +z (Blender -y): from the front, a little turned and from a little above
        d = mathutils.Vector((0.35, -1.0, 0.25)).normalized() * size * 4
        cam.location = c + d
        cam.rotation_euler = (c - cam.location).to_track_quat('-Z', 'Y').to_euler()
        sun = bpy.data.objects.new('sun', bpy.data.lights.new('sun', 'SUN')); sc.collection.objects.link(sun)
        sun.data.energy = 3.0; sun.rotation_euler = (math.radians(55), math.radians(-15), math.radians(-30))
        fill = bpy.data.objects.new('fill', bpy.data.lights.new('fill', 'SUN')); sc.collection.objects.link(fill)
        fill.data.energy = 1.2; fill.rotation_euler = (math.radians(70), math.radians(30), math.radians(140))
        world = bpy.data.worlds.new('w'); sc.world = world; world.use_nodes = True
        world.node_tree.nodes['Background'].inputs[0].default_value = (1, 1, 1, 1)
        world.node_tree.nodes['Background'].inputs[1].default_value = 0.6
        eng = [i.identifier for i in bpy.types.RenderSettings.bl_rna.properties['engine'].enum_items]
        sc.render.engine = 'BLENDER_EEVEE' if 'BLENDER_EEVEE' in eng else 'BLENDER_EEVEE_NEXT'
        sc.render.film_transparent = True
        sc.render.resolution_x = sc.render.resolution_y = 160
        sc.render.image_settings.file_format = 'PNG'
        sc.render.image_settings.color_mode = 'RGBA'
        sc.render.filepath = f"{root}/assets/portraits/{style}_{k}.png"
        bpy.ops.render.render(write_still=True)
print('heads done', flush=True)
