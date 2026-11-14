"""Paints the sign boards of the village and the rooms as textured quads (glb) for the engine.

Each board is a quad facing +z, centred on its origin, w x h studs (scaled to metres like the town). The texts are
the three.js version's SIGNS; the same texts are shown in full when you read a sign in the game (st_read.c).

Run from the project root:  python tools/make_signs.py   (uses the Windows Arial fonts when present)
"""
import os
import sys

from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make_minifig import Glb, STUD_M  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'assets', 'brickhollow', 'signs')
FONTS = 'C:/Windows/Fonts/'
PX = 160  # texture pixels per stud

# name: (w, h studs, bg, fg, title, lines, note)
BOARDS = {
    'village': (3.8, 2.0, '#f4f4f4', '#1b2a34', 'BRICKHOLLOW', ['"A village where every brick has a story."',
                                                                'Population: 6 minifigs, lots of Beasts, countless studs.'], None),
    'route': (3.4, 1.8, '#fff6c8', '#6b3f1d', 'ROUTE 1 \u25b2', ['Tall grass ahead!',
                                                                'Wild Brick Beasts love to hide in it. Keep your deck ready.'], None),
    'garden': (2.8, 1.5, '#e4cd9e', '#6b3f1d', 'GARDEN', ['Please do not step on the flowers.'], 'Pip, that means YOU.'),
    'lab': (5.2, 1.9, '#f4f4f4', '#1e5aa8', 'QUILL LAB', ['Brick Beast research since 1978.',
                                                          'Opening hours: whenever the Professor remembers to unlock the door.'], None),
    # house signs and room signs: one word on a strip
    'house_home': (3.4, 0.85, '#f4f4f4', '#0055bf', 'HOME', [], None),
    'house_dex': (3.4, 0.85, '#f4f4f4', '#c91a09', 'DEX & LUMI', [], None),
    'room_dex': (5.0, 1.25, '#7b2fbe', '#f2cd37', 'DEX IS #1', [], None),
    'room_lab': (7.0, 1.75, '#f4f4f4', '#0055bf', 'QUILL LAB', [], None),
    'room_stairs': (2.6, 0.65, '#f2cd37', '#1b2a34', 'MY ROOM \u25b2', [], None),
}


def font(name, size):
    try:
        return ImageFont.truetype(FONTS + name, size)
    except OSError:
        return ImageFont.load_default(size)


def wrap(d, text, f, width):
    words, lines, cur = text.split(), [], ''
    for w in words:
        t = (cur + ' ' + w).strip()
        if d.textlength(t, font=f) <= width or not cur:
            cur = t
        else:
            lines.append(cur)
            cur = w
    if cur:
        lines.append(cur)
    return lines


def draw_text(d, xy, text, f, fill, anchor='mm'):
    """text with the triangle drawn by hand (Arial has no U+25B2)"""
    if '\u25b2' not in text:
        d.text(xy, text, font=f, fill=fill, anchor=anchor)
        return
    head = text.replace('\u25b2', '').rstrip()
    size = f.size
    tw = d.textlength(head, font=f) + size * 0.9
    x0 = xy[0] - tw / 2
    d.text((x0, xy[1]), head, font=f, fill=fill, anchor='lm')
    tx = x0 + d.textlength(head, font=f) + size * 0.5
    s = size * 0.36
    d.polygon([(tx, xy[1] + s), (tx + s * 2, xy[1] + s), (tx + s, xy[1] - s)], fill=fill)


def board_png(w, h, bg, fg, title, lines, note):
    W, H = int(w * PX), int(h * PX)
    im = Image.new('RGB', (W, H), bg)
    d = ImageDraw.Draw(im)
    pad = int(0.08 * H) + 6
    d.rectangle([3, 3, W - 4, H - 4], outline=fg, width=max(3, int(H * 0.025)))
    if not lines and not note:
        f = font('arialbd.ttf', int(H * 0.62))
        while d.textlength(title, font=f) > W - pad * 3 and f.size > 10:
            f = font('arialbd.ttf', f.size - 2)
        draw_text(d, (W / 2, H / 2), title, f, fg)
        return im
    ft = font('arialbd.ttf', int(H * 0.2))
    while d.textlength(title, font=ft) > W - pad * 3:
        ft = font('arialbd.ttf', ft.size - 2)
    draw_text(d, (W / 2, pad + ft.size * 0.6), title, ft, fg)
    y = pad + ft.size * 1.35
    d.line([(W * 0.15, y), (W * 0.85, y)], fill=fg, width=3)
    fb = font('arial.ttf', int(H * 0.095))
    y += fb.size * 0.6
    for para in lines:
        for ln in wrap(d, para, fb, W - pad * 3):
            d.text((W / 2, y), ln, font=fb, fill=fg, anchor='ma')
            y += fb.size * 1.25
        y += fb.size * 0.3
    if note:
        fi = font('ariali.ttf', int(H * 0.085))
        d.text((W / 2, H - pad - fi.size * 0.4), note, font=fi, fill=fg, anchor='md')
    return im


def save(name, w, h, png):
    g = Glb()
    mat = g.material('#ffffff', png, f'sign_{name}')
    g.j['materials'][mat]['doubleSided'] = False
    x, y = w * STUD_M / 2, h * STUD_M / 2
    pos = [(-x, -y, 0.0), (x, -y, 0.0), (x, y, 0.0), (-x, y, 0.0)]
    nrm = [(0.0, 0.0, 1.0)] * 4
    uv = [(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)]
    g.save(os.path.join(OUT, name + '.glb'), [g.prim(pos, nrm, uv, [0, 1, 2, 0, 2, 3], mat)], f'sign_{name}')


if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    for name, (w, h, bg, fg, title, lines, note) in BOARDS.items():
        save(name, w, h, board_png(w, h, bg, fg, title, lines, note))
    print('signs ->', OUT, len(BOARDS))
