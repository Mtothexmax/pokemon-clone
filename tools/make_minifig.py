"""Builds the LEGO minifigures as glb parts for the engine.

Source: the six minifig part meshes of the three.js version (tools/minifig_src/fig-*.bin, feet at y 0, 4.9 studs
tall, facing +z). Every style gets its own folder assets/brickhollow/minifig/<style>/ with head, torso, armL, armR,
legL, legR .glb, each translated to its joint pivot and scaled to metres (STUD_M). The head carries the hair or cap,
the face and the torso print are drawn here and embedded as PNG.

Run from the project root:  python tools/make_minifig.py
"""
import io
import json
import math
import os
import struct

from PIL import Image, ImageDraw

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, 'tools', 'minifig_src')
OUT = os.path.join(ROOT, 'assets', 'brickhollow', 'minifig')

SINGLE_SIDED = True  # the body meshes have inner faces that flicker through when drawn double-sided
STUD_M = 0.38  # metres per stud: the figure is 4.9 studs = 1.86 m, the character controller's size
PARTS = ['head', 'torso', 'armL', 'armR', 'legL', 'legR']
PIVOT = {'head': (0, 3.95, 0), 'torso': (0, 0, 0), 'armL': (-0.86, 3.12, -0.1), 'armR': (0.86, 3.12, -0.1),
         'legL': (-0.4, 1.55, 0), 'legR': (0.4, 1.55, 0)}
SKIN = '#f5c72c'

STYLES = {
    'player': dict(face='player', torso='#1b9e8f', legs='#6c6e68', hair='#583927', hairStyle='cap', cap='#fe8a18', decal='zip', decalColor='#f4f4f4'),
    'dex': dict(face='dex', torso='#7b2fbe', legs='#1b2a34', hair='#e8e8e8', hairStyle='spiky', decal='stripe', decalColor='#f2cd37'),
    'pip': dict(face='pip', torso='#f2cd37', legs='#0055bf', hair='#583927', hairStyle='short', decal='stripe', decalColor='#c91a09'),
    'quill': dict(face='quill', torso='#3d6fd1', legs='#e4cd9e', hair='#c8c8c8', hairStyle='bob', coat=True, decalColor='#c91a09', female=True),
    'lumi': dict(face='lumi', torso='#ff9ecd', legs='#0055bf', hair='#f2cd37', hairStyle='bob', decal='star', decalColor='#f4f4f4', female=True),
    # Stonecrest (the three.js round 2): Sir Bricksworth the castle guard, Marla the baker
    'knight': dict(face='knight', torso='#a0a5a9', legs='#6c6e68', hair='#6c6e68', hairStyle='cap', cap='#a0a5a9', decal='star', decalColor='#f2cd37'),
    'baker': dict(face='baker', torso='#f4f4f4', legs='#0055bf', hair='#82422a', hairStyle='bob', decal='apron', decalColor='#fe8a18', female=True),
    'mom': dict(face='mom', torso='#a0bcac', legs='#583927', hair='#82422a', hairStyle='ponytail', decal='apron', decalColor='#f4f4f4', female=True),
}


def hex_rgb(h):
    h = h.lstrip('#')
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def lin(c):
    c /= 255.0
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def linear_rgba(h):
    r, g, b = hex_rgb(h)
    return [lin(r), lin(g), lin(b), 1.0]


# ---------------- the source meshes ----------------
def read_bin(path):
    b = open(path, 'rb').read()
    nv, ni, fl = struct.unpack_from('<3I', b, 0)
    o = 12
    pos = struct.unpack_from(f'<{nv * 3}f', b, o); o += nv * 12
    nrm = struct.unpack_from(f'<{nv * 3}f', b, o); o += nv * 12
    if fl & 1:
        o += nv * 4
    uv = None
    if fl & 2:
        uv = struct.unpack_from(f'<{nv * 2}f', b, o); o += nv * 8
    idx = struct.unpack_from(f'<{ni}I', b, o); o += ni * 4
    groups = [(0, ni, 0)]
    if fl & 4:
        gc = struct.unpack_from('<I', b, o)[0]
        g = struct.unpack_from(f'<{gc * 3}I', b, o + 4)
        groups = [(g[i * 3], g[i * 3 + 1], g[i * 3 + 2]) for i in range(gc)]
    return dict(
        pos=[pos[i * 3:i * 3 + 3] for i in range(nv)],
        nrm=[nrm[i * 3:i * 3 + 3] for i in range(nv)],
        uv=[(uv[i * 2], 1.0 - uv[i * 2 + 1]) for i in range(nv)] if uv else [(0.0, 0.0)] * nv,  # canvas flipY -> glTF
        idx=idx, groups=groups)


# ---------------- hair and caps (three.js shapes rebuilt) ----------------
class Geo:
    def __init__(self):
        self.p, self.n, self.i = [], [], []

    def add(self, pts, tris, double=False, outward=True):
        if outward:
            # face away from the piece's centre (a curved shell): flip the winding when the faces point inwards
            c = [sum(q[k] for q in pts) / len(pts) for k in range(3)]
            score = 0.0
            for a, b, cc in tris:
                pa, pb, pc = pts[a], pts[b], pts[cc]
                u = [pb[k] - pa[k] for k in range(3)]
                v = [pc[k] - pa[k] for k in range(3)]
                fn = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]
                score += sum(fn[k] * (pa[k] - c[k]) for k in range(3))
            if score < 0:
                tris = [(a, cc, b) for a, b, cc in tris]
        base = len(self.p)
        nrm = [[0.0, 0.0, 0.0] for _ in pts]
        for a, b, c in tris:
            pa, pb, pc = pts[a], pts[b], pts[c]
            u = [pb[k] - pa[k] for k in range(3)]
            v = [pc[k] - pa[k] for k in range(3)]
            fn = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]
            for t in (a, b, c):
                for k in range(3):
                    nrm[t][k] += fn[k]
        for k, q in enumerate(nrm):
            l = math.sqrt(sum(x * x for x in q)) or 1.0
            nrm[k] = [x / l for x in q]
        self.p += [list(q) for q in pts]
        self.n += nrm
        self.i += [base + t for tri in tris for t in tri]
        if double:
            base2 = len(self.p)
            self.p += [list(q) for q in pts]
            self.n += [[-x for x in q] for q in nrm]
            self.i += [base2 + t for a, b, c in tris for t in (a, c, b)]


def xf(pts, scale=(1, 1, 1), rot=(0, 0, 0), pos=(0, 0, 0)):
    """three.js order: scale, rotate (Euler XYZ = Rx * Ry * Rz), translate"""
    rx, ry, rz = rot
    out = []
    for x, y, z in pts:
        x, y, z = x * scale[0], y * scale[1], z * scale[2]
        x, y = x * math.cos(rz) - y * math.sin(rz), x * math.sin(rz) + y * math.cos(rz)
        x, z = x * math.cos(ry) + z * math.sin(ry), -x * math.sin(ry) + z * math.cos(ry)
        y, z = y * math.cos(rx) - z * math.sin(rx), y * math.sin(rx) + z * math.cos(rx)
        out.append((x + pos[0], y + pos[1], z + pos[2]))
    return out


def sphere_cap(r, ws, hs, theta_len):
    pts, tris = [], []
    for j in range(hs + 1):
        th = theta_len * j / hs
        for i in range(ws + 1):
            ph = 2 * math.pi * i / ws
            pts.append((-r * math.cos(ph) * math.sin(th), r * math.cos(th), r * math.sin(ph) * math.sin(th)))
    for j in range(hs):
        for i in range(ws):
            a = j * (ws + 1) + i
            b = a + ws + 1
            if j > 0:
                tris.append((a, b, a + 1))
            tris.append((a + 1, b, b + 1))
    return pts, tris


def cylinder(rt, rb, h, seg, t0=0.0, tl=2 * math.pi, caps=False):
    pts, tris = [], []
    for j, (y, r) in enumerate([(h / 2, rt), (-h / 2, rb)]):
        for i in range(seg + 1):
            t = t0 + tl * i / seg
            pts.append((r * math.sin(t), y, r * math.cos(t)))
    for i in range(seg):
        a, b = i, i + seg + 1
        tris += [(a, b, a + 1), (b, b + 1, a + 1)]
    if caps:
        top = len(pts); pts.append((0, h / 2, 0))
        bot = len(pts); pts.append((0, -h / 2, 0))
        for i in range(seg):
            tris.append((top, i, i + 1))
            tris.append((bot, seg + 2 + i, seg + 1 + i))
    return pts, tris


def box(w, h, d):
    x, y, z = w / 2, h / 2, d / 2
    pts, tris = [], []
    faces = [((1, 0, 0), [(x, -y, z), (x, -y, -z), (x, y, -z), (x, y, z)]),
             ((-1, 0, 0), [(-x, -y, -z), (-x, -y, z), (-x, y, z), (-x, y, -z)]),
             ((0, 1, 0), [(-x, y, z), (x, y, z), (x, y, -z), (-x, y, -z)]),
             ((0, -1, 0), [(-x, -y, -z), (x, -y, -z), (x, -y, z), (-x, -y, z)]),
             ((0, 0, 1), [(-x, -y, z), (x, -y, z), (x, y, z), (-x, y, z)]),
             ((0, 0, -1), [(x, -y, -z), (-x, -y, -z), (-x, y, -z), (x, y, -z)])]
    for _, q in faces:
        b = len(pts)
        pts += q
        tris += [(b, b + 1, b + 2), (b, b + 2, b + 3)]
    return pts, tris


def hair_geo(style):
    """hair / cap in head space (head pivot at 0), already through the hair group's (1.04, 1.2, 1.04) and y 0.32"""
    g = Geo()
    band = Geo()  # under a cap: the hair at the back, in the hair colour
    hs = style['hairStyle']
    group = dict(scale=(1.04, 1.2, 1.04), pos=(0, 0.32, 0))

    def put(pts, tris, double=False, into=None, outward=True, **child):
        pts = xf(pts, **child)
        pts = xf(pts, **group)
        # the hair material is double-sided: a second, flipped copy of the faces only fought with the first (grey band)
        (into or g).add(pts, tris, False, outward)

    if hs == 'cap':
        p, t = sphere_cap(0.72, 24, 6, math.pi / 2)
        put(p, t, pos=(0, 0.32, 0), scale=(1, 0.8, 1))
        # brim: a crescent widest at the front (+z)
        # brim: a crescent widest at the front (+z), a top face (normals up) and a bottom face 0.08 below
        R, N = 0.7, 32
        for y, up in ((0.0, True), (-0.08, False)):
            pts, tris = [], []
            for i in range(N + 1):
                a = -math.pi / 2 + i / N * math.pi
                r = R + 0.5 * max(math.cos(a), 0) ** 1.5
                pts.append((math.sin(a) * r, y, math.cos(a) * r))
                pts.append((math.sin(a) * (R - 0.08), y, math.cos(a) * (R - 0.08)))
            for i in range(N):
                a = i * 2
                # outer i, inner i, outer i+1 ...: (a, a+2, a+1) faces up
                tris += [(a, a + 2, a + 1), (a + 1, a + 2, a + 3)] if up else [(a, a + 1, a + 2), (a + 1, a + 3, a + 2)]
            put(pts, tris, outward=False, pos=(0, 0.42, 0))
        p, t = cylinder(0.7, 0.66, 0.5, 16, math.pi * 0.6, math.pi * 0.8)
        put(p, t, double=True, into=band, pos=(0, 0.1, 0))
        return [(g, style['cap']), (band, style['hair'])]
    if hs == 'spiky':
        p, t = sphere_cap(0.74, 20, 6, math.pi / 2)
        put(p, t, pos=(0, 0.3, 0))
        for i in range(9):
            a = i / 9 * math.pi * 2
            p, t = cylinder(0.0, 0.22, 0.7, 6, caps=True)
            put(p, t, pos=(math.sin(a) * 0.5, 0.75, math.cos(a) * 0.45 - 0.1), rot=(math.cos(a) * 0.9, 0, -math.sin(a) * 0.9))
        p, t = cylinder(0.72, 0.68, 0.7, 18, math.pi * 0.55, math.pi * 0.9)
        put(p, t, double=True)
    elif hs in ('bob', 'ponytail'):
        p, t = sphere_cap(0.75, 24, 6, math.pi / 2)
        put(p, t, pos=(0, 0.3, 0))
        bob = hs == 'bob'
        p, t = cylinder(0.76, 0.78, 1.0 if bob else 0.6, 20, math.pi * 0.35, math.pi * 1.3)
        put(p, t, double=True, pos=(0, -0.15 if bob else 0.05, 0))
        if not bob:
            p, t = cylinder(0.25, 0.25, 1.2, 10, caps=True)
            put(p, t, pos=(0, -0.2, -0.85), rot=(0.35, 0, 0))
    elif hs == 'short':
        p, t = sphere_cap(0.73, 24, 6, math.pi / 2)
        put(p, t, pos=(0, 0.32, 0))
        p, t = box(1.0, 0.25, 0.3)
        put(p, t, pos=(0.1, 0.5, 0.55), rot=(0, 0, -0.2))
        p, t = cylinder(0.71, 0.7, 0.55, 18, math.pi * 0.55, math.pi * 0.9)
        put(p, t, double=True, pos=(0, 0.1, 0))
    return [(g, style['hair'])]


# ---------------- prints ----------------
def face_png(style):
    """Every villager their own face (the LEGO prints the user showed: lashes, freckles, a plaster, glasses, a smirk)."""
    S = 4  # supersample
    im = Image.new('RGB', (256 * S, 256 * S), hex_rgb(style.get('skin', SKIN)))
    d = ImageDraw.Draw(im)
    ink = (27, 27, 27)
    white = (255, 255, 255)
    face = style.get('face', 'classic')
    E = lambda cx, cy, rx, ry, fill: d.ellipse([(cx - rx) * S, (cy - ry) * S, (cx + rx) * S, (cy + ry) * S], fill=fill)
    L = lambda pts, w, c=ink: d.line([(x * S, y * S) for x, y in pts], fill=c, width=int(w * S), joint='curve')
    A = lambda box, a0, a1, w, c=ink: d.arc([v * S for v in box], a0, a1, fill=c, width=int(w * S))
    P = lambda box, a0, a1, c: d.pieslice([v * S for v in box], a0, a1, fill=c)
    G = lambda pts, c: d.polygon([(x * S, y * S) for x, y in pts], fill=c)

    def eyes(y=120, rx=9, ry=13, iris=None):
        for x in (98, 158):
            if iris:  # a white with a coloured iris (the grown-ups' prints)
                E(x, y, rx + 4, ry + 3, white)
                E(x, y + 1, rx - 1, ry - 2, iris)
                E(x, y + 1, rx - 5, ry - 6, ink)
            else:
                E(x, y, rx, ry, ink)
            E(x + 3, y - 5, 3.5, 3.5, white)

    def lashes(y=120):
        for x, s in ((98, -1), (158, 1)):
            for k in range(3):
                L([(x + s * (6 + k * 3), y - 11 + k * 2), (x + s * (12 + k * 4), y - 17 + k * 3)], 3)

    def freckles(y=150):
        for cx in (76, 180):
            for dx, dy in ((-6, -3), (0, 3), (6, -2), (2, -8)):
                E(cx + dx, y + dy, 2.2, 2.2, (150, 88, 30))

    if face == 'player':  # brave and friendly: strong brows, a big open grin
        eyes()
        L([(80, 92), (112, 84)], 7)
        L([(144, 84), (176, 92)], 7)
        P([96, 140, 160, 192], 0, 180, ink)
        G([(102, 167), (154, 167), (150, 175), (106, 175)], white)  # the teeth
        E(128, 184, 12, 4, (200, 70, 60))                         # the tongue
    elif face == 'dex':  # the rival: angry brows, a lopsided smirk with a fang
        eyes(y=122)
        L([(80, 96), (100, 100), (112, 108)], 7)
        L([(144, 108), (156, 100), (178, 92)], 7)
        L([(102, 172), (130, 170), (152, 162), (160, 154)], 6)
        G([(140, 168), (146, 167), (143, 177)], white)
        L([(156, 150), (166, 160)], 4)  # the smirk's dimple
    elif face == 'pip':  # the little one: freckles, a plaster across the nose, a cheeky smile
        eyes(y=118, rx=10, ry=14)
        A([82, 84, 114, 104], 200, 330, 5)
        A([142, 84, 174, 104], 210, 340, 5)
        freckles(148)
        G([(108, 140), (148, 140), (148, 152), (108, 152)], (238, 150, 120))  # the plaster
        for x in (118, 128, 138):
            E(x, 146, 1.6, 1.6, (200, 110, 90))
        A([104, 150, 152, 186], 25, 155, 6)
    elif face == 'quill':  # the professor: round glasses, kind wrinkles, a small smile
        eyes(y=122, rx=7, ry=9, iris=(70, 120, 190))
        for x in (98, 158):
            A([x - 22, 100, x + 22, 144], 0, 360, 6, (60, 60, 64))
        L([(120, 120), (136, 120)], 5, (60, 60, 64))  # the bridge
        L([(84, 92), (112, 90)], 5, (110, 110, 110))
        L([(144, 90), (172, 92)], 5, (110, 110, 110))
        A([108, 156, 148, 182], 20, 160, 5, (176, 60, 60))
        for x, s in ((68, 1), (188, -1)):  # laugh lines
            L([(x, 116), (x - s * 6, 124)], 3, (180, 140, 40))
            L([(x, 128), (x - s * 6, 134)], 3, (180, 140, 40))
    elif face == 'lumi':  # sunny: lashes, rosy cheeks, freckles, a big happy smile
        eyes(y=118, rx=10, ry=14)
        lashes(118)
        A([84, 88, 112, 102], 200, 340, 4)
        A([144, 88, 172, 102], 200, 340, 4)
        E(74, 150, 12, 7, (240, 150, 120))
        E(182, 150, 12, 7, (240, 150, 120))
        freckles(146)
        P([104, 148, 152, 186], 0, 180, (180, 40, 50))
        L([(104, 167), (152, 167)], 4, (180, 40, 50))
    elif face == 'mom':  # warm: lashes, blue eyes, eye shadow, red lips
        for x in (98, 158):
            E(x, 112, 16, 9, (150, 175, 210))  # eye shadow
        eyes(y=120, rx=7, ry=9, iris=(60, 110, 180))
        lashes(118)
        L([(84, 96), (112, 92)], 4)
        L([(144, 92), (172, 96)], 4)
        P([106, 160, 150, 184], 0, 180, (192, 40, 50))
        G([(106, 172), (128, 166), (150, 172)], (192, 40, 50))
    elif face == 'knight':  # the old guard: bushy brows, a big grey moustache, a scar
        eyes(y=118)
        L([(78, 96), (96, 88), (114, 94)], 9, (90, 90, 90))
        L([(142, 94), (160, 88), (178, 96)], 9, (90, 90, 90))
        G([(96, 150), (128, 142), (160, 150), (170, 166), (150, 160), (128, 156), (106, 160), (86, 166)], (120, 120, 124))
        A([112, 160, 144, 184], 30, 150, 5)
        L([(168, 104), (178, 132)], 3, (190, 110, 60))
    elif face == 'baker':  # warm and busy: lashes, a flour smudge, rosy cheeks, a wide smile
        eyes(y=120, rx=8, ry=11, iris=(110, 70, 40))
        lashes(120)
        L([(84, 96), (112, 92)], 4)
        L([(144, 92), (172, 96)], 4)
        E(76, 152, 11, 6, (240, 150, 120))
        E(180, 152, 11, 6, (240, 150, 120))
        E(170, 82, 9, 5, (250, 248, 240))  # flour on the forehead
        P([104, 150, 152, 186], 0, 180, (170, 40, 50))
        G([(110, 168), (146, 168), (142, 175), (114, 175)], white)
    else:
        eyes()
        L([(82, 90), (112, 86)], 6)
        L([(144, 86), (174, 90)], 6)
        A([96, 118, 160, 182], 0.18 * 180, 0.82 * 180, 7)
    return im.resize((256, 256), Image.LANCZOS)


def torso_png(style):
    S = 4
    base = '#f4f4f4' if style.get('coat') else style['torso']
    im = Image.new('RGB', (128 * S, 128 * S), hex_rgb(base))
    d = ImageDraw.Draw(im)
    dc = hex_rgb(style.get('decalColor', '#ffffff'))
    R = lambda x, y, w, h, c: d.rectangle([x * S, y * S, (x + w) * S - 1, (y + h) * S - 1], fill=c)
    if style.get('coat'):
        d.polygon([(44 * S, 0), (84 * S, 0), (64 * S, 70 * S)], fill=hex_rgb(style['torso']))
        R(60, 6, 8, 46, dc)
        d.line([(44 * S, 0), (64 * S, 70 * S), (64 * S, 128 * S)], fill=(153, 153, 153), width=3 * S)
        d.line([(84 * S, 0), (64 * S, 70 * S)], fill=(153, 153, 153), width=3 * S)
        R(84, 60, 18, 4, (61, 155, 255))
    decal = style.get('decal')
    if decal == 'zip':
        R(61, 0, 6, 128, dc); R(14, 30, 28, 8, dc); R(86, 30, 28, 8, dc)
    elif decal == 'apron':
        d.rounded_rectangle([30 * S, 36 * S, 98 * S, 128 * S], radius=10 * S, fill=dc)
        for i in range(3):
            cx = 48 + i * 16
            d.ellipse([(cx - 5) * S, 65 * S, (cx + 5) * S, 75 * S], fill=(230, 90, 138))
    elif decal == 'star':
        pts = []
        for i in range(10):
            r = 10 if i % 2 else 24
            a = i / 10 * math.pi * 2 - math.pi / 2
            pts.append(((64 + math.cos(a) * r) * S, (60 + math.sin(a) * r) * S))
        d.polygon(pts, fill=dc)
    elif decal == 'stripe':
        R(0, 44, 128, 18, dc); R(0, 76, 128, 8, dc)
    return im.resize((128, 128), Image.LANCZOS)


# ---------------- glb ----------------
class Glb:
    def __init__(self):
        self.bin = bytearray()
        self.j = dict(asset=dict(version='2.0', generator='brickhollow make_minifig.py'), buffers=[], bufferViews=[],
                      accessors=[], meshes=[], nodes=[], scenes=[dict(nodes=[0])], scene=0, materials=[], textures=[],
                      images=[], samplers=[dict(magFilter=9729, minFilter=9987, wrapS=33071, wrapT=33071)])

    def view(self, data, target=None):
        while len(self.bin) % 4:
            self.bin.append(0)
        v = dict(buffer=0, byteOffset=len(self.bin), byteLength=len(data))
        if target:
            v['target'] = target
        self.bin += data
        self.j['bufferViews'].append(v)
        return len(self.j['bufferViews']) - 1

    def acc(self, data, comp, count, typ, mn=None, mx=None, target=None):
        a = dict(bufferView=self.view(data, target), componentType=comp, count=count, type=typ)
        if mn is not None:
            a['min'], a['max'] = mn, mx
        self.j['accessors'].append(a)
        return len(self.j['accessors']) - 1

    def material(self, color, png=None, name='m'):
        m = dict(name=name, pbrMetallicRoughness=dict(baseColorFactor=linear_rgba(color), metallicFactor=0.0, roughnessFactor=0.72),
                 doubleSided=True)
        if png is not None:
            buf = io.BytesIO()
            png.save(buf, 'PNG')
            self.j['images'].append(dict(name=name, bufferView=self.view(bytes(buf.getvalue())), mimeType='image/png'))
            # unique names: the cook files embedded images by name, two prints of one style must not collide
            self.j['textures'].append(dict(name=name, source=len(self.j['images']) - 1, sampler=0))
            m['pbrMetallicRoughness']['baseColorTexture'] = dict(index=len(self.j['textures']) - 1)
            m['pbrMetallicRoughness']['baseColorFactor'] = [1, 1, 1, 1]
        self.j['materials'].append(m)
        return len(self.j['materials']) - 1

    def prim(self, pos, nrm, uv, idx, mat):
        n = len(pos)
        P = self.acc(struct.pack(f'<{n * 3}f', *[c for p in pos for c in p]), 5126, n, 'VEC3',
                     [min(p[k] for p in pos) for k in range(3)], [max(p[k] for p in pos) for k in range(3)], 34962)
        N = self.acc(struct.pack(f'<{n * 3}f', *[c for p in nrm for c in p]), 5126, n, 'VEC3', target=34962)
        T = self.acc(struct.pack(f'<{n * 2}f', *[c for p in uv for c in p]), 5126, n, 'VEC2', target=34962)
        I = self.acc(struct.pack(f'<{len(idx)}I', *idx), 5125, len(idx), 'SCALAR', target=34963)
        return dict(attributes=dict(POSITION=P, NORMAL=N, TEXCOORD_0=T), indices=I, material=mat)

    def save(self, path, prims, name):
        self.j['meshes'].append(dict(name=name, primitives=prims))
        self.j['nodes'].append(dict(name=name, mesh=0))
        while len(self.bin) % 4:
            self.bin.append(0)
        self.j['buffers'].append(dict(byteLength=len(self.bin)))
        if not self.j['textures']:
            for k in ('textures', 'images', 'samplers'):
                del self.j[k]
        js = json.dumps(self.j, separators=(',', ':')).encode()
        while len(js) % 4:
            js += b' '
        out = struct.pack('<III', 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(self.bin))
        out += struct.pack('<II', len(js), 0x4E4F534A) + js + struct.pack('<II', len(self.bin), 0x004E4942) + bytes(self.bin)
        open(path, 'wb').write(out)


def compact(src, idx):
    """the vertices a group uses, re-indexed from 0"""
    remap, pos, nrm, uv, out = {}, [], [], [], []
    for i in idx:
        if i not in remap:
            remap[i] = len(pos)
            pos.append(src['pos'][i]); nrm.append(src['nrm'][i]); uv.append(src['uv'][i])
        out.append(remap[i])
    return pos, nrm, uv, out


def build_style(name, style, meshes):
    folder = os.path.join(OUT, name)
    os.makedirs(folder, exist_ok=True)
    for part in PARTS:
        src = meshes[part]
        g = Glb()
        colors = [style['torso'] if not style.get('coat') else '#f4f4f4', style['legs'], style.get('skin', SKIN)]
        mats = {}
        prims = []
        px, py, pz = PIVOT[part]
        leg = part in ('legL', 'legR')
        for start, count, mi in src['groups']:
            if mi not in mats:
                if mi == 3:
                    mats[mi] = g.material('#ffffff', torso_png(style), f'{name}_torso_print')
                elif mi == 4:
                    mats[mi] = g.material('#ffffff', face_png(style), f'{name}_face')
                else:
                    mats[mi] = g.material(colors[mi], None, ['torso', 'legs', 'skin'][mi])
                if SINGLE_SIDED:
                    g.j['materials'][mats[mi]]['doubleSided'] = False
            pos, nrm, uv, idx = compact(src, src['idx'][start:start + count])
            # legs a hair narrower than the hips around their pivot: their faces no longer fight with the hip piece
            k = 0.975 if leg else 1.0

            def tuck(x, y, z):
                # the part of a leg inside the hip piece is rounded around the hip pin (y 1.55, along x), like a real
                # minifig leg, so a swinging leg never pokes through the hip's grooved front
                if not leg or y < 1.22:
                    return x, y, z
                dy = y - 1.55
                d = math.sqrt(dy * dy + z * z)
                R = 0.38
                w = min(1.0, (y - 1.22) / 0.12)
                if d <= R:
                    return x, y, z
                k2 = 1.0 + (R / d - 1.0) * w
                return x, 1.55 + dy * k2, z * k2
            pos = [(lambda tx, ty, tz: ((tx - px) * k * STUD_M, (ty - py) * STUD_M, (tz - pz) * k * STUD_M))(*tuck(x, y, z)) for x, y, z in pos]
            prims.append(g.prim(pos, nrm, uv, idx, mats[mi]))
        if leg:
            # the source legs are open at the top: a round hip joint along x closes them (a swinging leg showed its inside)
            hg = Geo()
            cx = -0.34 if part == 'legL' else 0.34
            p, t = cylinder(0.36, 0.36, 0.84, 18, caps=True)
            hg.add(xf(p, rot=(0.0, 0.0, math.pi / 2), pos=(cx, 1.55, 0.0)), t)
            pos = [((x - px) * STUD_M, (y - py) * STUD_M, (z - pz) * STUD_M) for x, y, z in hg.p]
            prims.append(g.prim(pos, hg.n, [(0.0, 0.0)] * len(pos), hg.i, mats.get(1) or g.material(style['legs'], None, 'legs')))
        if part == 'head':
            for k, (hg, color) in enumerate(hair_geo(style)):
                mat = g.material(color, None, f'{name}_hair{k}')
                pos = [(x * STUD_M, y * STUD_M, z * STUD_M) for x, y, z in hg.p]
                prims.append(g.prim(pos, hg.n, [(0.0, 0.0)] * len(pos), hg.i, mat))
        g.save(os.path.join(folder, part + '.glb'), prims, f'{name}_{part}')
    print('minifig', name, '->', folder)


def mirror(src):
    """the left part mirrored to the right: x and normal x negated, winding flipped (the source's right arm and leg
    are mirrored copies whose normals point inwards, they caught light and sky from the wrong side)"""
    idx = list(src['idx'])
    for t in range(0, len(idx), 3):
        idx[t + 1], idx[t + 2] = idx[t + 2], idx[t + 1]
    return dict(pos=[(-x, y, z) for x, y, z in src['pos']], nrm=[(-x, y, z) for x, y, z in src['nrm']], uv=src['uv'],
                idx=idx, groups=src['groups'])


if __name__ == '__main__':
    meshes = {p: read_bin(os.path.join(SRC, f'fig-{p}.bin')) for p in PARTS}
    meshes['armR'] = mirror(meshes['armL'])
    meshes['legR'] = mirror(meshes['legL'])
    for name, style in STYLES.items():
        build_style(name, style, meshes)
    print('pivots (m):', {p: tuple(round(c * STUD_M, 4) for c in PIVOT[p]) for p in PARTS})
