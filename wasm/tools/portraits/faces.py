"""The battle's portraits as flat faces: each minifig's face print (the head's `*_face` texture in head.glb and the
talking prints head_mouth1/2.glb) cropped square round the face (one crop for the three, so nothing jumps while it
talks), on its yellow -> assets/portraits/<style>_face_<0|1|2>.png (128 px).
Run: python tools/portraits/faces.py
"""
import io, json, struct
from PIL import Image

STYLES = ["player", "dex", "lumi", "pip", "quill", "mom", "baker", "knight"]
OUT = 128


def face(path):
    b = open(path, "rb").read()
    n = struct.unpack("<I", b[12:16])[0]
    j = json.loads(b[20:20 + n])
    binc = b[20 + n + 8:]
    for img in j.get("images", []):
        if img.get("name", "").endswith("_face"):
            bv = j["bufferViews"][img["bufferView"]]
            o = bv.get("byteOffset", 0)
            return Image.open(io.BytesIO(binc[o:o + bv["byteLength"]])).convert("RGBA")
    raise SystemExit(f"no face print in {path}")


for style in STYLES:
    ims = [face(f"assets/minifig/{style}/{n}.glb") for n in ("head", "head_mouth1", "head_mouth2")]
    bg = ims[0].getpixel((4, 4))
    # the face's extent: what differs from the yellow in any of the three
    x0 = y0 = 10 ** 9
    x1 = y1 = -1
    for im in ims:
        w, h = im.size
        px = im.load()
        for y in range(h):
            for x in range(w):
                p = px[x, y]
                if sum(abs(p[i] - bg[i]) for i in range(3)) > 60:
                    x0, y0, x1, y1 = min(x0, x), min(y0, y), max(x1, x), max(y1, y)
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    side = max(x1 - x0, y1 - y0) * 1.32
    box = (int(cx - side / 2), int(cy - side / 2), int(cx + side / 2), int(cy + side / 2))
    for k, im in enumerate(ims):
        out = Image.new("RGBA", (box[2] - box[0], box[3] - box[1]), bg)
        out.alpha_composite(im.crop(box) if box[0] >= 0 and box[1] >= 0 else im.crop(box))
        out.resize((OUT, OUT), Image.LANCZOS).save(f"assets/portraits/{style}_face_{k}.png")
    print(style, box)
