"""Talking mouths for the minifig heads: two copies of each face print with the mouth half and fully open.

  python tools/mouths.py        assets/minifig/*/head_flat.glb -> head_mouth1.glb, head_mouth2.glb

Each copy is the flat head glTF with only its face image replaced (the game takes the texture from it and swaps the
face material's texture while the villager's voice plays: figure.rs `set_mouth`). The printed mouth is found as the
lowest dark / red blob below the eyes (a moustache above it stays); it is erased (skin) and the open mouth drawn: a dark D shape (flat top
at the mouth's top, round bottom), a red tongue, a strip of teeth, drawn 4x and scaled down (smooth edges).
Needs Pillow.
"""

import glob
import io
import json
import os
import struct

from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(os.path.dirname(HERE), 'assets', 'minifig')


def read_glb(path):
    b = open(path, 'rb').read()
    n = struct.unpack('<I', b[12:16])[0]
    j = json.loads(b[20:20 + n])
    m = struct.unpack('<I', b[20 + n:24 + n])[0]
    return j, bytearray(b[28 + n:28 + n + m])


def write_glb(path, j, bin_):
    js = json.dumps(j, separators=(',', ':')).encode()
    js += b' ' * ((4 - len(js) % 4) % 4)
    bin_ = bytes(bin_) + b'\0' * ((4 - len(bin_) % 4) % 4)
    total = 12 + 8 + len(js) + 8 + len(bin_)
    with open(path, 'wb') as f:
        f.write(struct.pack('<III', 0x46546C67, 2, total))
        f.write(struct.pack('<II', len(js), 0x4E4F534A) + js)
        f.write(struct.pack('<II', len(bin_), 0x004E4942) + bin_)


def mouth_box(img):
    """The printed mouth: rows with dark or red-ish (not skin) pixels in the middle below the eyes."""
    w, h = img.size
    px = img.load()
    skin = px[w // 2, int(h * 0.92)][:3]

    def ink(x, y):
        r, g, b, a = px[x, y]
        if a < 128:
            return False
        dist = abs(r - skin[0]) + abs(g - skin[1]) + abs(b - skin[2])
        lum = 0.3 * r + 0.59 * g + 0.11 * b
        # dark lines, red lips; not the pale pink cheeks (light) or freckles off-centre
        return dist > 90 and (lum < 110 or (r > g + 60 and lum < 150))

    rows = {}
    for y in range(int(h * 0.55), int(h * 0.85)):
        xs = [x for x in range(int(w * 0.36), int(w * 0.64)) if ink(x, y)]
        if len(xs) >= 2:
            rows[y] = (min(xs), max(xs))
    if not rows:
        return None
    ys = sorted(rows)
    # the lowest blob: walk up from the bottom row while rows are (nearly) contiguous
    blob = [ys[-1]]
    for y in reversed(ys[:-1]):
        if blob[-1] - y > 3:
            break
        blob.append(y)
    y0, y1 = min(blob), max(blob)
    # widen to the whole mouth on these rows (the centre window can cut a wide mouth)
    xs = [x for y in range(y0, y1 + 1) for x in range(int(w * 0.25), int(w * 0.75)) if ink(x, y)]
    return min(xs), y0, max(xs), y1


def erase_mouth(img, box):
    """The printed mouth gone (a figure has one mouth at a time: Dex's crooked line and fang stuck out from under the
    open one): every pixel in `box` (2 px more) that is not skin becomes skin."""
    img = img.copy()
    w, h = img.size
    px = img.load()
    skin = px[w // 2, int(h * 0.92)]
    x0, y0, x1, y1 = box
    for y in range(max(0, y0 - 2), min(h, y1 + 3)):
        for x in range(max(0, x0 - 2), min(w, x1 + 3)):
            r, g, b, a = px[x, y]
            if abs(r - skin[0]) + abs(g - skin[1]) + abs(b - skin[2]) > 12:
                px[x, y] = skin
    return img


def open_mouth(img, box, k):
    """The face with the mouth `k` (0..1) open over `box` (the printed one erased first)."""
    img = erase_mouth(img, box)
    w, h = img.size
    S = 4
    big = img.resize((w * S, h * S), Image.LANCZOS)
    d = ImageDraw.Draw(big)
    x0, y0, x1, y1 = box
    cx = (x0 + x1) / 2
    mw = max(x1 - x0 + 4, 26)
    top = y0 - 1
    depth = 6 + 16 * k
    # the D: a flat top edge, the bottom an ellipse half
    left, right = cx - mw / 2, cx + mw / 2
    bbox = [left * S, (top - depth) * S, right * S, (top + depth) * S]
    d.pieslice(bbox, 0, 180, fill=(52, 14, 18, 255))
    # the printed mouth's top part above the D stays hidden under a skin-free dark band
    d.rectangle([left * S, (top - 0.5) * S, right * S, (top + 1.5) * S], fill=(52, 14, 18, 255))
    # teeth along the top, a tongue at the bottom
    d.rectangle([(left + mw * 0.12) * S, (top + 1.2) * S, (right - mw * 0.12) * S, (top + 1.2 + 2.2 + 1.5 * k) * S], fill=(250, 248, 240, 255))
    tw = mw * 0.5
    d.ellipse([(cx - tw / 2) * S, (top + depth * 0.55) * S, (cx + tw / 2) * S, (top + depth * 1.15) * S], fill=(214, 70, 82, 255))
    # clip the tongue to the mouth: redraw the outside of the D's bottom with the original
    mask = Image.new('L', big.size, 0)
    ImageDraw.Draw(mask).pieslice(bbox, 0, 180, fill=255)
    ImageDraw.Draw(mask).rectangle([left * S, (top - 0.5) * S, right * S, (top + 1.5) * S], fill=255)
    orig = img.resize((w * S, h * S), Image.LANCZOS)
    out = Image.composite(big, orig, mask)
    return out.resize((w, h), Image.LANCZOS)


def main():
    for flat in sorted(glob.glob(os.path.join(ROOT, '*', 'head_flat.glb'))):
        style = os.path.basename(os.path.dirname(flat))
        j, bin_ = read_glb(flat)
        im = j['images'][0]
        bv = j['bufferViews'][im['bufferView']]
        off = bv.get('byteOffset', 0)
        img = Image.open(io.BytesIO(bytes(bin_[off:off + bv['byteLength']]))).convert('RGBA')
        box = mouth_box(img)
        if box is None:
            print(f'{style}: no mouth found')
            continue
        for n, k in ((1, 0.45), (2, 1.0)):
            buf = io.BytesIO()
            open_mouth(img, box, k).save(buf, 'PNG', optimize=True)
            png = buf.getvalue()
            # the new image at the end of the buffer, its view pointed there
            jj = json.loads(json.dumps(j))
            b2 = bytearray(bin_)
            b2 += b'\0' * ((4 - len(b2) % 4) % 4)
            jj['bufferViews'][im['bufferView']] = {'buffer': 0, 'byteOffset': len(b2), 'byteLength': len(png)}
            b2 += png
            jj['buffers'] = [{'byteLength': len(b2) + (4 - len(b2) % 4) % 4}]
            out = os.path.join(os.path.dirname(flat), f'head_mouth{n}.glb')
            write_glb(out, jj, b2)
        print(f'{style}: mouth {box} -> head_mouth1.glb, head_mouth2.glb')


if __name__ == '__main__':
    main()
