"""Builds the LEGO plants of the three.js version as glb props for the engine.

Source: tools/props_src/*.bin (the pre-converted LEGO part meshes of the three.js town: the brick round tree and
spruce, the bush and bamboo leaves with their cut-out textures, the flower pot, the flower plant, the "Lego Frog"
plants, cypress, small pine, reeds, fern). Out: assets/brickhollow/props/*.glb, scaled from studs to metres
(STUD_M) - the trees are 1 high and scaled per instance like before. Flower heads come in one glb per colour.

Run from the project root:  python tools/make_props.py
"""
import io
import os
import struct
import sys

from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_minifig import Glb, linear_rgba, STUD_M  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, 'tools', 'props_src')
OUT = os.path.join(ROOT, 'assets', 'brickhollow', 'props')

DARK_GREEN = '#237841'
FLOWER_COLORS = {'red': '#c91a09', 'yellow': '#ffd23d', 'pink': '#ff8ac0', 'white': '#ffffff', 'orange': '#fe8a18',
                 'purple': '#b98ae6'}


def read(name):
    b = open(os.path.join(SRC, name + '.bin'), 'rb').read()
    nv, ni, fl = struct.unpack_from('<3I', b, 0)
    o = 12
    pos = struct.unpack_from(f'<{nv * 3}f', b, o); o += nv * 12
    nrm = struct.unpack_from(f'<{nv * 3}f', b, o); o += nv * 12
    col = None
    if fl & 1:
        col = struct.unpack_from(f'<{nv * 4}B', b, o); o += nv * 4
    uv = None
    if fl & 2:
        uv = struct.unpack_from(f'<{nv * 2}f', b, o); o += nv * 8
    idx = struct.unpack_from(f'<{ni}I', b, o)
    return dict(nv=nv, pos=pos, nrm=nrm, col=col, uv=uv, idx=idx)


class PropGlb(Glb):
    def mesh_prim(self, m, mat, scale=STUD_M):
        nv = m['nv']
        pos = [(m['pos'][i * 3] * scale, m['pos'][i * 3 + 1] * scale, m['pos'][i * 3 + 2] * scale) for i in range(nv)]
        nrm = [tuple(m['nrm'][i * 3:i * 3 + 3]) for i in range(nv)]
        # the three.js textures were flipped on load (flipY); glTF samples the image top-down
        uv = [(m['uv'][i * 2], 1.0 - m['uv'][i * 2 + 1]) for i in range(nv)] if m['uv'] else [(0.0, 0.0)] * nv
        p = self.prim(pos, nrm, uv, list(m['idx']), mat)
        if m['col']:
            # the source bytes are already linear ((4, 47, 13) is LEGO dark green #237841): only normalise them
            cols = [c for i in range(nv) for c in (m['col'][i * 4] / 255.0, m['col'][i * 4 + 1] / 255.0, m['col'][i * 4 + 2] / 255.0, 1.0)]
            p['attributes']['COLOR_0'] = self.acc(struct.pack(f'<{nv * 4}f', *cols), 5126, nv, 'VEC4', target=34962)
        return p

    def cutout_material(self, rgb_png, alpha_png, name):
        rgb = Image.open(os.path.join(SRC, rgb_png)).convert('RGB')
        a = Image.open(os.path.join(SRC, alpha_png)).convert('L').resize(rgb.size)
        rgba = rgb.copy()
        rgba.putalpha(a)
        mi = self.material('#ffffff', rgba, name)
        self.j['materials'][mi].update(alphaMode='MASK', alphaCutoff=0.5)
        return mi


def save(out_name, parts):
    """parts: [(mesh name, colour or None for vertex colours, cutout (rgb, alpha) or None)]"""
    g = PropGlb()
    prims = []
    for k, (mesh, color, cutout) in enumerate(parts):
        m = read(mesh)
        if cutout:
            mat = g.cutout_material(cutout[0], cutout[1], f'{out_name}_{k}')
            g.j['materials'][mat]['pbrMetallicRoughness']['roughnessFactor'] = 0.85
        else:
            mat = g.material(color or '#ffffff', None, f'{out_name}_{k}')
            g.j['materials'][mat]['pbrMetallicRoughness']['roughnessFactor'] = 0.85  # matte: glossy plastic read as metal
        prims.append(g.mesh_prim(m, mat))
    g.save(os.path.join(OUT, out_name + '.glb'), prims, out_name)


def lab_computer():
    """Prof. Quill's lab computer: the LEGO office computer of the three.js version (models-src/office/Computer.obj,
    converted there to lab-computer.bin: grey housing, dark key tops) and its glowing screen panel (lab-computer-
    screen.bin, emissive #40c0ff on #0a1a24). Faces +z like there; the screen sits 0.01 in front (in world units)."""
    from make_minifig import lin
    g = PropGlb()
    body = read('lab-computer')
    mat = g.material('#ffffff', None, 'lab_computer')
    g.j['materials'][mat]['pbrMetallicRoughness']['roughnessFactor'] = 0.35
    p = g.mesh_prim(body, mat)
    # three.js reads these bytes as sRGB (models.ts): the same colours here, so it looks the same
    nv = body['nv']
    cols = [c for i in range(nv) for c in (lin(body['col'][i * 4]), lin(body['col'][i * 4 + 1]), lin(body['col'][i * 4 + 2]), 1.0)]
    p['attributes']['COLOR_0'] = g.acc(struct.pack(f'<{nv * 4}f', *cols), 5126, nv, 'VEC4', target=34962)
    scr = read('lab-computer-screen')
    off = 0.01 / 1.2  # the instance scale is 1.2
    scr['pos'] = [v + (off if k % 3 in (1, 2) else 0.0) for k, v in enumerate(scr['pos'])]
    smat = g.material('#0a1a24', None, 'lab_computer_screen')
    g.j['materials'][smat]['pbrMetallicRoughness']['roughnessFactor'] = 0.15
    g.j['materials'][smat]['emissiveFactor'] = [lin(0x40) * 0.9, lin(0xc0) * 0.9, lin(0xff) * 0.9]
    g.save(os.path.join(OUT, 'lab_computer.glb'), [p, g.mesh_prim(scr, smat)], 'lab_computer')


def castle():
    """The old castle north of Stonecrest (three.js static/models/castle.bin, voxelized from the "lego city" asset):
    vertex colours stored as sRGB bytes (three.js converted them on load), here made linear. Units: studs before
    the town's scale 1.6 (st_town.c places it turned by 90 degrees)."""
    m = read('castle')
    lin = lambda c: (c / 255.0) / 12.92 if c / 255.0 <= 0.04045 else ((c / 255.0 + 0.055) / 1.055) ** 2.4
    table = [round(lin(c) * 255.0) for c in range(256)]
    m['col'] = tuple(table[c] if (i % 4) != 3 else 255 for i, c in enumerate(m['col']))
    g = PropGlb()
    mat = g.material('#ffffff', None, 'castle')
    g.j['materials'][mat]['pbrMetallicRoughness']['roughnessFactor'] = 0.65
    g.save(os.path.join(OUT, 'castle.glb'), [g.mesh_prim(m, mat)], 'castle')


if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    if sys.argv[1:] == ['castle']:
        castle()
        sys.exit(0)
    save('round_tree', [('round-tree', None, None)])
    save('spruce', [('spruce-tree', None, None)])
    save('cypress', [('cypress', None, None)])
    save('pine_small', [('pine-small', None, None)])
    save('flowerpot', [('flowerpot', None, None)])
    save('flower_bunch', [('flowerpot-flowers', None, None)])
    save('bush', [('bush', None, ('bush.png', 'bush_a.png'))])
    save('bamboo', [('bamboo', None, ('bamboo.png', 'bamboo_a.png'))])
    save('reed', [('tall-grass', '#4b9f4a', None)])
    save('fern', [('fern', '#3e9a32', None)])
    for cname, chex in FLOWER_COLORS.items():
        save(f'flower_{cname}', [('flowerplant-leaf', DARK_GREEN, None), ('flowerplant-head', chex, None)])
        for v in ('sprout', 'bouquet', 'stem-a', 'stem-b'):
            save(f'frog_{v.replace("-", "_")}_{cname}', [(f'frog-{v}-green', DARK_GREEN, None), (f'frog-{v}-flower', chex, None)])
    for v in ('leafy-a', 'leafy-b'):
        save(f'frog_{v.replace("-", "_")}', [(f'frog-{v}-green', DARK_GREEN, None)])
    lab_computer()
    castle()
    print('props ->', OUT, len(os.listdir(OUT)), 'files')
