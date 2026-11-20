"""Card pictures: the Voxelmon front pixel art of the Kaigen port (st_beasts_data.c, generated there by
tools/make_beasts.py from the Voxelmon models) -> src/beasts.bin, read by cards.rs (include_bytes).

Format: for each card in CardId order (cards.rs `CARD_IDS`): u8 width, u8 height, then width*height RGBA
bytes, top row first. Run: python tools/make_beasts.py <path to st_beasts_data.c>
"""
import re, struct, sys, os

IDS = ["pebbit", "flicker", "drizzlet", "sproutle", "zapling", "quickhop", "voltail", "cinderpup", "mosscub",
       "gloomoth", "gustling", "sparkmole", "tickbot", "tidalotl", "rockodile", "fungoo", "zephyrrow", "glimmerling",
       "echowl", "buildbeaver", "thundrake", "coralyth", "emberhorn", "bramblewolf", "phantomule", "boomshroom",
       "glaciomoth", "magmaw", "lumifox", "cragoliath", "clockcrab", "titanoak", "stormaw", "abyssquid", "goldbrick",
       "brickling"]

src = open(sys.argv[1], encoding="utf8").read()
pics = {}
for m in re.finditer(r'cstr_equal\(id, "(\w+)"\)\) \{\s*static const u32 px\[\d+\] = \{([^}]*)\};\s*return \(BeastPicture\)\{(\d+), (\d+), px\}', src):
    name, body, w, h = m.group(1), m.group(2), int(m.group(3)), int(m.group(4))
    px = [int(v.strip().rstrip("u"), 16) for v in body.split(",") if v.strip()]
    assert len(px) == w * h, name
    pics[name] = (w, h, px)
out = bytearray()
for i in IDS:
    w, h, px = pics[i]
    out += bytes([w, h])
    for p in px:
        out += struct.pack("<I", p)  # 0xAABBGGRR little endian -> R G B A
dst = os.path.join(os.path.dirname(__file__), "..", "src", "beasts.bin")
open(dst, "wb").write(out)
print(len(pics), "pictures,", len(out), "bytes ->", dst)
