"""Writes the vector art of the TV and the computers (assets/brickhollow/screens/art/*.svg).

The engine cooks the folder into glyphs (art.iconset.hzt, `parts true`): every <g> at the root of a file is one part,
drawn on its own with its own colour and transform by st_screens.c. So each file is a stack of one-colour layers;
the colours below say what each part is painted (st_screens.c paints them; the files themselves are black). Every
file is 176 high (the set's em): the TV scenes are 320 x 176 like the three.js TvShow.svelte drawings they come
from, the icons 176 x 176.

Only the big curved shapes are art (a glyph field is too coarse for eyes, studs and thin strokes at TV size: those
are drawn as UI boxes). Shapes sit where they rest; st_screens.c moves them from there. Run from the project root:  python tools/make_screen_art.py
"""
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'assets', 'brickhollow', 'screens', 'art')


def svg(name, w, parts):
    body = '\n'.join(f'  <g fill="{c}">{p}</g>' if not p.startswith('<g') else '  ' + p for c, p in parts)
    # one colour per glyph field: the files are drawn in black (the colours are the code's), so the cook stays quiet
    body = re.sub(r'(fill|stroke)="#[0-9a-fA-F]{6}"', r'\1="#000000"', body)
    open(os.path.join(OUT, name + '.svg'), 'w', newline='\n').write(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} 176">\n{body}\n</svg>\n')
    print(f'{name}: {len(parts)} parts')


def rect(x, y, w, h, r=0.0):
    return f'<rect x="{x:g}" y="{y:g}" width="{w:g}" height="{h:g}" rx="{r:g}"/>'


def circle(x, y, r):
    return f'<circle cx="{x:g}" cy="{y:g}" r="{r:g}"/>'


def ellipse(x, y, rx, ry):
    return f'<ellipse cx="{x:g}" cy="{y:g}" rx="{rx:g}" ry="{ry:g}"/>'


def poly(pts):
    return '<polygon points="' + ' '.join(f'{x:g},{y:g}' for x, y in pts) + '"/>'


def stroke(color, width, inner):
    """a stroke-only part (outlines drawn over their fill, like svg paints the stroke over the fill)"""
    return (color, f'<g fill="none" stroke="{color}" stroke-width="{width:g}" stroke-linecap="round" stroke-linejoin="round">{inner}</g>')


def tv_bbn():
    star = [(0, -22), (6, -6), (22, -6), (9, 4), (14, 20), (0, 10), (-14, 20), (-9, 4), (-22, -6), (-6, -6)]
    svg('tv_bbn', 320, [
        ('#ffd23d', poly([(236 + x, 57 + y) for x, y in star])),                # 0 the star on the studio screen (turns about 236, 57)
    ])


def cloud(x, y):
    return ellipse(x, y, 26, 12) + ellipse(x - 12, y - 6, 14, 11) + ellipse(x + 10, y - 9, 16, 13)


def tv_weather():
    svg('tv_weather', 320, [
        ('#ffffff', cloud(40, 50)),                                             # 0..2 clouds (drift)
        ('#ffffff', cloud(120, 80)),
        ('#ffffff', cloud(280, 110)),
        ('#3e9a32', '<path d="M0 150 Q 60 118 120 146 T 240 140 T 320 136 V176 H0 Z"/>'),  # 3 hills
        ('#4fc06a', '<path d="M0 162 Q 80 140 160 160 T 320 154 V176 H0 Z"/>'),  # 4
    ])


def tv_cartoon():
    svg('tv_cartoon', 320, [
        ('#f2a705', ''.join(f'<path d="M{i * 80 - 10} 150 q 40 -50 80 0"/>' for i in range(7))),  # 0 hills (scroll 160)
    ])


def tv_wild():
    svg('tv_wild', 320, [
        ('#6bb04a', '<path d="M-20 130 Q 80 100 180 126 T 380 118 V176 H-20 Z"/>'),  # 0 the meadow (pans)
    ])


def icons():
    """Lucide-style line icons on a 24 grid, scaled to the 176 em"""
    s = 176 / 24

    def icon(name, inner, width=2.0):
        svg(name, 176, [stroke('#000000', width, f'<g transform="scale({s:g})">{inner}</g>')])

    icon('ic_chevron_left', '<path d="M15 18l-6-6 6-6"/>', 2.6)
    icon('ic_chevron_right', '<path d="M9 18l6-6-6-6"/>', 2.6)
    icon('ic_back', '<path d="M19 12H5"/><path d="M12 19l-7-7 7-7"/>', 2.4)
    icon('ic_power', '<path d="M12 2v10"/><path d="M18.4 6.6a9 9 0 1 1-12.77.04"/>', 2.4)
    icon('ic_mail', '<rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/>')
    icon('ic_chart', '<path d="M3 3v18h18"/><path d="M8 17v-5"/><path d="M13 17V8"/><path d="M18 17v-9"/>', 2.2)
    icon('ic_beast', '<rect x="3" y="8" width="18" height="13" rx="3"/><path d="M7 8V5h3v3M14 8V5h3v3"/>'
                     '<path d="M9 13h.01M15 13h.01"/><path d="M9.5 17c1.5 1 3.5 1 5 0"/>', 2.2)
    icon('ic_sticker', '<path d="M12 2l3 6.5 7 .8-5.2 4.8 1.5 7-6.3-3.7L5.7 21l1.5-7L2 9.3l7-.8z"/>', 2.0)
    icon('ic_coin', '<circle cx="12" cy="12" r="9"/><path d="M12 7v10M9 9.5h4.5a1.5 1.5 0 0 1 0 3h-3a1.5 1.5 0 0 0 0 3H15"/>', 2.0)
    icon('ic_lock', '<rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>', 2.2)
    icon('ic_brick', '<rect x="2" y="9" width="20" height="11" rx="2"/><path d="M6 9V6h4v3M14 9V6h4v3"/>', 2.2)


if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    for f in os.listdir(OUT):
        if f.endswith('.svg'):
            os.remove(os.path.join(OUT, f))
    tv_bbn()
    tv_weather()
    tv_cartoon()
    tv_wild()
    icons()
