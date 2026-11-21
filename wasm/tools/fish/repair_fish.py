"""The LEGO shark (assets/props/shark.glb) from the user's Tripo model: the simplified OBJ had fallen apart into 122
patches (2871 open edges), so this starts from the full model instead: its glTF export doubles the vertices on UV seams
(merging by distance closes them: 1 piece, no open edges), a collapse decimation to 6000 triangles keeps it closed,
one plain grey material, no UVs. Run: blender -b --factory-startup --python tools/fish/repair_fish.py -- <dir with
orig.gltf> <out dir>  (writes fish.glb and fish.blend)
"""
import bpy, bmesh, sys, math
args = sys.argv[sys.argv.index('--')+1:]
IN, OUT = args[0], args[1]
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=IN + '/orig.gltf')
o = [x for x in bpy.context.scene.objects if x.type == 'MESH'][0]
bpy.context.view_layer.objects.active = o
o.select_set(True)
def report(tag):
    bm = bmesh.new(); bm.from_mesh(o.data)
    bnd = sum(1 for e in bm.edges if e.is_boundary)
    seen=set(); isl=0
    bm.faces.ensure_lookup_table()
    for f in bm.faces:
        if f.index in seen: continue
        isl+=1; st=[f]; seen.add(f.index)
        while st:
            g=st.pop()
            for e in g.edges:
                for h in e.link_faces:
                    if h.index not in seen: seen.add(h.index); st.append(h)
    print(tag, 'verts', len(bm.verts), 'faces', len(bm.faces), 'boundary', bnd, 'islands', isl, flush=True)
    bm.free()
# 1. weld the UV seams (positions duplicated by the glTF export)
bm = bmesh.new(); bm.from_mesh(o.data)
bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
bm.to_mesh(o.data); bm.free()
report('welded')
# 2. decimate (collapse keeps it connected; UVs follow)
target = 6000
m = o.modifiers.new('dec', 'DECIMATE')
m.decimate_type = 'COLLAPSE'
m.ratio = target / len(o.data.polygons)
m.use_collapse_triangulate = True
bpy.ops.object.modifier_apply(modifier='dec')
report('decimated')
# smooth shading with sharp creases kept
for p in o.data.polygons: p.use_smooth = True
# 3. no texture: one plain grey plastic material, no UVs
gray = bpy.data.materials.new('fish grey')
gray.use_nodes = True
bsdf = next(n for n in gray.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
bsdf.inputs['Base Color'].default_value = (0.74, 0.76, 0.8, 1.0)
bsdf.inputs['Roughness'].default_value = 0.4
o.data.materials.clear()
o.data.materials.append(gray)
while o.data.uv_layers:
    o.data.uv_layers.remove(o.data.uv_layers[0])
# 4. export
bpy.ops.export_scene.gltf(filepath=OUT + '/fish.glb', export_format='GLB', use_selection=True, export_normals=True, export_materials='EXPORT')
bpy.ops.wm.save_as_mainfile(filepath=OUT + '/fish.blend')
print('done', flush=True)
