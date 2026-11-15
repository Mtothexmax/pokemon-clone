"""Paints the sign boards of the village and the rooms as textured quads (glb) for the engine.

Each board is a quad facing +z, centred on its origin, w x h studs (scaled to metres like the town). The texts are
the three.js version's SIGNS; the same texts are shown in full when you read a sign in the game (st_read.c).

Each board is designed as an SVG (tools/signs_src/, wood or an enamel plate, little pictures, the Windows Segoe
fonts) and rendered by headless Brave (tools/svg_render.mjs, the engine's Playwright); without a browser the plain
boards of board_png are used.

Run from the project root:  python tools/make_signs.py
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
    # Route 1 north and Stonecrest (the three.js round 2)
    'north': (3.4, 1.8, '#fff6c8', '#6b3f1d', 'STONECREST ▲', ['Route 1 goes on through the forest.',
                                                                    'Mind the creek, use the bridge!'], None),
    'stonecrest': (3.4, 1.8, '#f4f4f4', '#6c6e68', 'STONECREST', ['"The town below the old castle."',
                                                                  'Bakery · Inn · Market · Smith'], None),
    'house_bakery': (3.4, 0.85, '#f4f4f4', '#c91a09', 'BAKERY', [], None),
    'house_inn': (3.4, 0.85, '#f4f4f4', '#0055bf', 'INN', [], None),
    'house_shop': (2.6, 0.65, '#f4f4f4', '#237841', 'SHOP', [], None),
    'house_market': (3.4, 0.85, '#f4f4f4', '#fe8a18', 'MARKET', [], None),
    'house_smith': (3.4, 0.85, '#f4f4f4', '#82422a', 'SMITH', [], None),
    'house_post': (1.6, 0.4, '#f4f4f4', '#0055bf', 'POST', [], None),
    # the Deck-o-Matic's screen in the lab (st_dive.c shows one or the other)
    'machine_missing': (2.2, 1.4, '#140404', '#ff4a3a', 'MISSING PART', ['INSERT STUD CORE'], None),
    'machine_ready': (2.2, 1.4, '#03140a', '#4dff8a', 'READY', ['DECKS PRINTED: 1'], None),
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


# ---------------------------------------------------------------- the SVG designs
# Each board is drawn as an SVG (tools/signs_src/<name>.svg, W x H px at PX per stud) and rendered by headless Brave
# (tools/svg_render.mjs); without a browser the plain PIL board above is used. Text is measured with the same Windows
# fonts the browser uses, so it is sized and wrapped to fit.

def fnt(file, size):
    try:
        return ImageFont.truetype(FONTS + file, int(size))
    except OSError:
        return ImageFont.load_default(int(size))


def fit(text, file, size, width):
    """the largest size <= size at which `text` fits `width` px"""
    d = ImageDraw.Draw(Image.new('RGB', (1, 1)))
    while size > 8 and d.textlength(text, font=fnt(file, size)) > width:
        size -= 1
    return size


def wrap_px(text, file, size, width):
    d = ImageDraw.Draw(Image.new('RGB', (1, 1)))
    return wrap(d, text, fnt(file, size), width)


def esc(s):
    return s.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;').replace('"', '&quot;')


TITLE = ("'Segoe UI Black', 'Arial Black', sans-serif", 'seguibl.ttf')
BODY = ("'Segoe UI', Arial, sans-serif", 'segoeuib.ttf')
HAND = ("'Segoe Print', 'Comic Sans MS', cursive", 'segoeprb.ttf')


def text(x, y, s, fam, size, fill, anchor='middle', extra=''):
    return (f'<text x="{x:.1f}" y="{y:.1f}" font-family="{fam}" font-size="{size:.1f}" font-weight="700" fill="{fill}" '
            f'text-anchor="{anchor}" dominant-baseline="central" {extra}>{esc(s)}</text>')


def screw(x, y, r):
    return (f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.1f}" fill="url(#screw)" stroke="#6b7480" stroke-width="{r * 0.18:.1f}"/>'
            f'<line x1="{x - r * 0.55:.1f}" y1="{y + r * 0.35:.1f}" x2="{x + r * 0.55:.1f}" y2="{y - r * 0.35:.1f}" '
            f'stroke="#59616b" stroke-width="{r * 0.22:.1f}" stroke-linecap="round"/>')


def brick(x, y, s, color, dark):
    """a 2x2 LEGO brick seen from the front-top, s = its width"""
    h = s * 0.62
    out = f'<rect x="{x:.1f}" y="{y + s * 0.18:.1f}" width="{s:.1f}" height="{h:.1f}" rx="{s * 0.06:.1f}" fill="{color}" stroke="{dark}" stroke-width="{s * 0.04:.1f}"/>'
    for k in (0.27, 0.73):
        out += (f'<rect x="{x + s * k - s * 0.15:.1f}" y="{y:.1f}" width="{s * 0.3:.1f}" height="{s * 0.24:.1f}" rx="{s * 0.06:.1f}" '
                f'fill="{color}" stroke="{dark}" stroke-width="{s * 0.04:.1f}"/>')
    return out


def flower(x, y, r, petal, centre='#ffd23d'):
    out = ''
    for k in range(5):
        a = k * 72
        out += f'<ellipse cx="{x:.1f}" cy="{y - r * 0.55:.1f}" rx="{r * 0.36:.1f}" ry="{r * 0.55:.1f}" fill="{petal}" transform="rotate({a} {x:.1f} {y:.1f})"/>'
    return out + f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r * 0.32:.1f}" fill="{centre}"/>'


def leaf(x, y, s, rot, color='#3e8a2a'):
    return f'<path d="M0 0 Q {s * 0.5:.1f} {-s * 0.45:.1f} {s:.1f} 0 Q {s * 0.5:.1f} {s * 0.45:.1f} 0 0 Z" fill="{color}" transform="translate({x:.1f} {y:.1f}) rotate({rot})"/>'


def arrow_up(x, y, s, fill):
    return (f'<path d="M{x:.1f} {y - s * 0.55:.1f} L{x + s * 0.5:.1f} {y:.1f} L{x + s * 0.2:.1f} {y:.1f} L{x + s * 0.2:.1f} {y + s * 0.5:.1f} '
            f'L{x - s * 0.2:.1f} {y + s * 0.5:.1f} L{x - s * 0.2:.1f} {y:.1f} L{x - s * 0.5:.1f} {y:.1f} Z" fill="{fill}"/>')


def defs(W, H):
    return f'''<defs>
  <radialGradient id="screw" cx="0.35" cy="0.3" r="0.8"><stop offset="0" stop-color="#ffffff"/><stop offset="0.5" stop-color="#c3c9d1"/><stop offset="1" stop-color="#7d8692"/></radialGradient>
  <linearGradient id="wood" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#a8692f"/><stop offset="1" stop-color="#7a4520"/></linearGradient>
  <linearGradient id="enamel" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffffff"/><stop offset="1" stop-color="#e9edf1"/></linearGradient>
  <linearGradient id="gloss" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffffff" stop-opacity="0.35"/><stop offset="0.45" stop-color="#ffffff" stop-opacity="0"/></linearGradient>
  <filter id="soft" x="-5%" y="-5%" width="110%" height="120%"><feDropShadow dx="0" dy="{H * 0.012:.1f}" stdDeviation="{H * 0.01:.1f}" flood-opacity="0.35"/></filter>
  <pattern id="scan" width="4" height="4" patternUnits="userSpaceOnUse"><rect width="4" height="2" fill="#ffffff" fill-opacity="0.05"/></pattern>
  <filter id="glow" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="{H * 0.02:.1f}" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
</defs>'''


def wood_grain(x, y, w, h, color='#5c3417', n=7):
    out = ''
    for k in range(n):
        yy = y + h * (k + 0.5) / n
        amp = h * 0.012 * (1 + k % 3)
        out += (f'<path d="M{x:.1f} {yy:.1f} C {x + w * 0.3:.1f} {yy - amp:.1f}, {x + w * 0.6:.1f} {yy + amp:.1f}, {x + w:.1f} {yy - amp * 0.5:.1f}" '
                f'stroke="{color}" stroke-opacity="0.18" stroke-width="{h * 0.008:.1f}" fill="none"/>')
    return out


def board_svg(name, w, h, bg, fg, title, lines, note):
    W, H = int(w * PX), int(h * PX)
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">', defs(W, H)]
    m = H * 0.06  # frame width
    if name.startswith('machine_'):  # a glowing screen: scanlines, a warning sign or a tick, big lit letters
        cx, cy, s0 = W / 2, H * 0.3, H * 0.17
        o.append(f'<rect width="{W}" height="{H}" fill="{bg}"/><rect width="{W}" height="{H}" fill="url(#scan)"/>')
        if name == 'machine_missing':
            o.append(f'<path d="M{cx:.1f} {cy - s0:.1f} L{cx + s0 * 1.15:.1f} {cy + s0 * 0.85:.1f} L{cx - s0 * 1.15:.1f} {cy + s0 * 0.85:.1f} Z" '
                     f'fill="none" stroke="{fg}" stroke-width="{H * 0.025:.1f}" stroke-linejoin="round" filter="url(#glow)"/>')
            o.append(text(cx, cy + s0 * 0.3, '!', TITLE[0], s0 * 1.0, fg, extra='filter="url(#glow)"'))
        else:
            o.append(f'<path d="M{cx - s0 * 0.75:.1f} {cy:.1f} L{cx - s0 * 0.15:.1f} {cy + s0 * 0.6:.1f} L{cx + s0 * 0.85:.1f} {cy - s0 * 0.55:.1f}" '
                     f'fill="none" stroke="{fg}" stroke-width="{H * 0.05:.1f}" stroke-linecap="round" stroke-linejoin="round" filter="url(#glow)"/>')
        ts = fit(title, TITLE[1], H * 0.2, W * 0.86)
        o.append(text(cx, H * 0.64, title, TITLE[0], ts, fg, extra='filter="url(#glow)"'))
        bs = fit(lines[0], BODY[1], H * 0.09, W * 0.8)
        o.append(text(cx, H * 0.83, lines[0], "Consolas, monospace", bs, fg, extra='fill-opacity="0.8"'))
        o.append('</svg>')
        return '\n'.join(o), W, H
    if name in ('garden', 'route', 'north'):  # wood: a frame of planks around a painted panel
        o.append(f'<rect width="{W}" height="{H}" fill="url(#wood)"/>' + wood_grain(0, 0, W, H, '#2e1a0b', 9))
        panel = '#e9d4a2' if name == 'garden' else '#ffd23f'
        o.append(f'<rect x="{m}" y="{m}" width="{W - 2 * m}" height="{H - 2 * m}" rx="{m * 0.6}" fill="{panel}" stroke="#4a2a12" stroke-width="{m * 0.18}"/>')
        o.append(wood_grain(m, m, W - 2 * m, H - 2 * m, '#8a5a20', 6) if name == 'garden' else '')
        for (sx, sy) in ((m * 0.5, m * 0.5), (W - m * 0.5, m * 0.5), (m * 0.5, H - m * 0.5), (W - m * 0.5, H - m * 0.5)):
            o.append(f'<circle cx="{sx:.1f}" cy="{sy:.1f}" r="{m * 0.22:.1f}" fill="#3a2210"/>')
    else:  # enamel plate: rounded, a coloured rim, screws in the corners
        o.append(f'<rect width="{W}" height="{H}" fill="{fg}"/>')  # square: the board in the town is
        o.append(f'<rect x="{m}" y="{m}" width="{W - 2 * m}" height="{H - 2 * m}" rx="{H * 0.05}" fill="{bg if bg != "#f4f4f4" else "url(#enamel)"}"/>')
        o.append(f'<rect x="{m * 1.6}" y="{m * 1.6}" width="{W - 3.2 * m}" height="{H - 3.2 * m}" rx="{H * 0.035}" fill="none" stroke="{fg}" stroke-opacity="0.35" stroke-width="{max(2, H * 0.008):.1f}"/>')
        r = m * 0.42
        for (sx, sy) in ((m * 0.5, m * 0.5), (W - m * 0.5, m * 0.5), (m * 0.5, H - m * 0.5), (W - m * 0.5, H - m * 0.5)):
            o.append(screw(sx, sy, r))
    ix0, ix1, iy0, iy1 = m * 2.2, W - m * 2.2, m * 2.0, H - m * 2.0
    iw, ih = ix1 - ix0, iy1 - iy0
    clean = title.replace('▲', '').strip()
    # ---- one-word strips (houses, rooms): a little picture on the left, the word ----
    if not lines and not note:
        icon = ih * 0.9
        pics = {'house_home': 'house', 'house_dex': 'bolt', 'room_dex': 'stars', 'room_lab': 'atom', 'room_stairs': 'arrow'}
        pic = pics.get(name)
        cx_text = (ix0 + icon * 1.15 + ix1) / 2 if pic and pic != 'stars' else W / 2
        tw = ix1 - ix0 - (icon * 1.25 if pic and pic not in ('stars',) else 0) - (icon * 0.9 if pic == 'arrow' else 0)
        if pic == 'stars':
            tw = iw - icon * 2.2
        size = fit(clean, TITLE[1], ih * 0.78, tw)
        px, py = ix0 + icon * 0.55, H / 2
        if pic == 'house':
            s = icon * 0.82
            o.append(f'<path d="M{px - s * 0.5:.1f} {py:.1f} L{px:.1f} {py - s * 0.45:.1f} L{px + s * 0.5:.1f} {py:.1f} Z" fill="#c91a09"/>'
                     f'<rect x="{px - s * 0.36:.1f}" y="{py - s * 0.02:.1f}" width="{s * 0.72:.1f}" height="{s * 0.46:.1f}" fill="{fg}"/>'
                     f'<rect x="{px - s * 0.08:.1f}" y="{py + s * 0.14:.1f}" width="{s * 0.16:.1f}" height="{s * 0.3:.1f}" fill="#ffd23d"/>')
        elif pic == 'bolt':
            s = icon * 0.85
            o.append(f'<path d="M{px + s * 0.1:.1f} {py - s * 0.5:.1f} L{px - s * 0.3:.1f} {py + s * 0.06:.1f} L{px:.1f} {py + s * 0.06:.1f} '
                     f'L{px - s * 0.12:.1f} {py + s * 0.5:.1f} L{px + s * 0.32:.1f} {py - s * 0.08:.1f} L{px + s * 0.02:.1f} {py - s * 0.08:.1f} Z" fill="#ffd23d" stroke="{fg}" stroke-width="{s * 0.06:.1f}" stroke-linejoin="round"/>')
        elif pic == 'atom':
            s = icon * 0.42
            for rot in (0, 60, 120):
                o.append(f'<ellipse cx="{px:.1f}" cy="{py:.1f}" rx="{s:.1f}" ry="{s * 0.38:.1f}" fill="none" stroke="{fg}" stroke-width="{s * 0.12:.1f}" transform="rotate({rot} {px:.1f} {py:.1f})"/>')
            o.append(f'<circle cx="{px:.1f}" cy="{py:.1f}" r="{s * 0.2:.1f}" fill="#c91a09"/>')
        elif pic == 'arrow':
            o.append(arrow_up(ix1 - icon * 0.45, py, icon * 0.75, fg))
            cx_text = (ix0 + ix1 - icon * 0.9) / 2
        elif pic == 'stars':
            for sx in (ix0 + icon * 0.5, ix1 - icon * 0.5):
                o.append(text(sx, py, '★', TITLE[0], icon * 0.85, fg))
        o.append(text(cx_text, H / 2 + size * 0.04, clean, TITLE[0], size, fg,
                      extra=f'paint-order="stroke" stroke="{"#00000033" if name == "room_dex" else "none"}" stroke-width="{size * 0.08:.1f}"'))
        o.append(f'<rect x="{m}" y="{m}" width="{W - 2 * m}" height="{(H - 2 * m) * 0.5:.1f}" rx="{H * 0.05}" fill="url(#gloss)"/>')
        o.append('</svg>')
        return '\n'.join(o), W, H
    # ---- notice boards: a title with pictures beside it, a rule, the lines, a note in handwriting ----
    tsize = fit(clean + ('  ' if '▲' in title else ''), TITLE[1], ih * 0.24, iw * (0.62 if name == 'lab' else 0.7))
    ty = iy0 + tsize * 0.62
    if name == 'lab':  # a blue header band
        band = tsize * 1.45
        o.append(f'<rect x="{m}" y="{m}" width="{W - 2 * m}" height="{band + m * 0.6:.1f}" rx="{H * 0.05}" fill="{fg}"/>')
        o.append(text(W / 2, m + (band + m * 0.6) / 2, clean, TITLE[0], tsize, '#ffffff'))
        for sx in (ix0 + tsize * 0.6, ix1 - tsize * 0.6):  # atoms either side
            s = tsize * 0.48
            cy = m + (band + m * 0.6) / 2
            for rot in (0, 60, 120):
                o.append(f'<ellipse cx="{sx:.1f}" cy="{cy:.1f}" rx="{s:.1f}" ry="{s * 0.38:.1f}" fill="none" stroke="#ffffff" stroke-width="{s * 0.12:.1f}" transform="rotate({rot} {sx:.1f} {cy:.1f})"/>')
            o.append(f'<circle cx="{sx:.1f}" cy="{cy:.1f}" r="{s * 0.2:.1f}" fill="#ffd23d"/>')
        y = m + band + m * 0.6 + tsize * 0.45
    else:
        title_fill = {'garden': '#2f6b1f', 'route': '#3a2210', 'north': '#3a2210'}.get(name, fg)
        o.append(text(W / 2, ty, clean, TITLE[0], tsize, title_fill))
        tw = ImageDraw.Draw(Image.new('RGB', (1, 1))).textlength(clean, font=fnt(TITLE[1], tsize))
        if name == 'village':
            for sx, col, dk in ((W / 2 - tw / 2 - tsize * 1.25, '#c91a09', '#7a0f05'), (W / 2 + tw / 2 + tsize * 0.35, '#f2cd37', '#a87a00')):
                o.append(brick(sx, ty - tsize * 0.45, tsize * 0.9, col, dk))
        if name in ('route', 'north'):
            o.append(arrow_up(W / 2 + tw / 2 + tsize * 0.75, ty, tsize * 0.9, '#3a2210'))
        if name == 'garden':
            for sx in (W / 2 - tw / 2 - tsize * 0.7, W / 2 + tw / 2 + tsize * 0.7):
                o.append(leaf(sx - tsize * 0.15, ty + tsize * 0.15, tsize * 0.45, 200) + leaf(sx + tsize * 0.15, ty + tsize * 0.15, tsize * 0.45, -20))
                o.append(flower(sx, ty - tsize * 0.05, tsize * 0.42, '#ff6fa8' if sx < W / 2 else '#c94ce0'))
        y = ty + tsize * 0.72
        # a rule: studs in a row
        n = 9
        for k in range(n):
            sx = W / 2 + (k - (n - 1) / 2) * tsize * 0.42
            o.append(f'<circle cx="{sx:.1f}" cy="{y:.1f}" r="{tsize * 0.09:.1f}" fill="{title_fill}" fill-opacity="0.55"/>')
        y += tsize * 0.38
    bsize = ih * 0.12
    body_fill = {'garden': '#4a2a12', 'route': '#3a2210', 'north': '#3a2210', 'stonecrest': '#3d3f3c'}.get(name, '#1b2a34' if name != 'lab' else '#24476f')
    nsize = bsize * 1.05
    room = (iy1 - (nsize * 1.4 if note else 0)) - y
    rows = []
    while True:
        rows = []
        for para in lines:
            rows += [(ln, para) for ln in wrap_px(para, BODY[1], bsize, iw * 0.94)]
        if len(rows) * bsize * 1.3 <= room or bsize < 10:
            break
        bsize -= 1
    y += (room - len(rows) * bsize * 1.3) * 0.35 + bsize * 0.65
    for ln, _ in rows:
        o.append(text(W / 2, y, ln, BODY[0], bsize, body_fill))
        y += bsize * 1.3
    if note:
        ns = fit(note, HAND[1], nsize, iw * 0.9)
        o.append(text(W / 2, iy1 - ns * 0.55, note, HAND[0], ns, '#b0281a', extra='transform="rotate(-2 %.1f %.1f)"' % (W / 2, iy1)))
    if name == 'garden':  # little flowers growing in the bottom corners
        for (fx, col) in ((ix0 + bsize * 0.4, '#ff6fa8'), (ix1 - bsize * 0.4, '#ffd23d')):
            o.append(leaf(fx - bsize * 0.5, iy1 - bsize * 0.1, bsize * 0.8, 210) + leaf(fx + bsize * 0.1, iy1 - bsize * 0.2, bsize * 0.8, -30))
            o.append(flower(fx, iy1 - bsize * 0.75, bsize * 0.55, col, '#ffffff' if col == '#ffd23d' else '#ffd23d'))
    if name in ('route', 'north'):  # tufts of tall grass along the bottom
        for k in range(14):
            gx = m + (W - 2 * m) * (k + 0.5) / 14
            hgt = H * (0.06 + 0.03 * (k % 3))
            o.append(f'<path d="M{gx - 6:.1f} {H - m:.1f} Q {gx:.1f} {H - m - hgt * 0.6:.1f} {gx + 3:.1f} {H - m - hgt:.1f} Q {gx + 2:.1f} {H - m - hgt * 0.4:.1f} {gx + 8:.1f} {H - m:.1f} Z" fill="#3e9a32"/>')
    o.append('</svg>')
    return '\n'.join(o), W, H


def render_svgs(names):
    """writes tools/signs_src/<name>.svg and renders them with headless Brave; {name: PIL image} (empty without a browser)"""
    import subprocess
    import tempfile
    src = os.path.join(ROOT, 'tools', 'signs_src')
    os.makedirs(src, exist_ok=True)
    files = []
    for name, (w, h, bg, fg, title, lines, note) in BOARDS.items():
        svg, W, H = board_svg(name, w, h, bg, fg, title, lines, note)
        p = os.path.join(src, name + '.svg')
        open(p, 'w', encoding='utf-8', newline='\n').write(svg)
        files.append(p)
    out = tempfile.mkdtemp()
    try:
        subprocess.run(['node', os.path.join(ROOT, 'tools', 'svg_render.mjs'), out, *files], check=True)
    except (OSError, subprocess.CalledProcessError) as e:
        print('no browser to render the SVG signs, plain boards instead:', e)
        return {}
    return {n: Image.open(os.path.join(out, n + '.png')).convert('RGB') for n in names}


if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    art = render_svgs(list(BOARDS))
    for name, (w, h, bg, fg, title, lines, note) in BOARDS.items():
        save(name, w, h, art.get(name) or board_png(w, h, bg, fg, title, lines, note))
    print('signs ->', OUT, len(BOARDS), '(SVG designs)' if art else '(plain boards)')
