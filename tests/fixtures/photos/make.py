# SPDX-License-Identifier: AGPL-3.0-or-later
"""Makes the photo fixtures (4D), once, on a Mac: python3 -I tests/fixtures/photos/make.py

- geotagged.jpg: a 64 x 48 JPEG (made by sips) given what a phone writes: EXIF with a GPS block (latitude, longitude,
  altitude, and the time), XMP that repeats the location, an ICC profile stub, and a comment. refs.rs must drop all of
  it before a photo is saved or sent.
- angle-a.jpg, angle-b.jpg: two plain images, the photo rung's dry run's stand-ins for a subject's two angles.
"""

import struct
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent


def png(w, h, pixel):
    rows = b"".join(
        b"\0" + b"".join(bytes(pixel(x, y)) for x in range(w)) for y in range(h)
    )
    chunk = lambda t, d: (
        struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d))
    )
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(rows))
        + chunk(b"IEND", b"")
    )


def jpeg(w, h, pixel):
    with tempfile.TemporaryDirectory() as d:
        src, out = Path(d, "in.png"), Path(d, "out.jpg")
        src.write_bytes(png(w, h, pixel))
        subprocess.run(
            ["sips", "-s", "format", "jpeg", str(src), "--out", str(out)],
            check=True,
            capture_output=True,
        )
        return out.read_bytes()


def exif():
    """A little-endian TIFF: IFD0 with a pointer to the GPS IFD, which holds a location in Los Angeles."""
    rational = lambda *pairs: b"".join(struct.pack("<II", n, d) for n, d in pairs)
    gps_values = [
        (1, 2, 2, b"N\0\0\0"),  # GPSLatitudeRef
        (2, 5, 3, rational((34, 1), (3, 1), (2500, 100))),  # GPSLatitude 34°3'25"
        (3, 2, 2, b"W\0\0\0"),  # GPSLongitudeRef
        (4, 5, 3, rational((118, 1), (14, 1), (4800, 100))),  # GPSLongitude 118°14'48"
        (6, 5, 1, rational((89, 1))),  # GPSAltitude
        (29, 2, 11, b"2026:11:04\0"),  # GPSDateStamp
    ]
    head = b"II*\0" + struct.pack("<I", 8)
    ifd0_at = 8
    ifd0_len = 2 + 12 + 4
    gps_at = ifd0_at + ifd0_len
    gps_len = 2 + 12 * len(gps_values) + 4
    data_at = gps_at + gps_len
    ifd0 = (
        struct.pack("<H", 1)
        + struct.pack("<HHII", 0x8825, 4, 1, gps_at)
        + struct.pack("<I", 0)
    )
    entries, data = b"", b""
    for tag, kind, count, value in gps_values:
        if len(value) <= 4:
            entries += struct.pack("<HHI", tag, kind, count) + value.ljust(4, b"\0")
        else:
            entries += struct.pack("<HHII", tag, kind, count, data_at + len(data))
            data += value
    gps = struct.pack("<H", len(gps_values)) + entries + struct.pack("<I", 0)
    return b"Exif\0\0" + head + ifd0 + gps + data


def segment(code, body):
    return bytes([0xFF, code]) + struct.pack(">H", len(body) + 2) + body


def main():
    if sys.platform != "darwin":
        sys.exit("make.py needs sips (macOS)")
    base = jpeg(64, 48, lambda x, y: (x * 4, y * 5, 128))
    xmp = (
        b"http://ns.adobe.com/xap/1.0/\0<x:xmpmeta xmlns:x='adobe:ns:meta/'><rdf:RDF><rdf:Description "
        b"exif:GPSLatitude='34,3.42N' exif:GPSLongitude='118,14.8W'/></rdf:RDF></x:xmpmeta>"
    )
    extra = (
        segment(0xE1, exif())
        + segment(0xE1, xmp)
        + segment(0xE2, b"ICC_PROFILE\0\x01\x01stub")
        + segment(0xFE, b"GPS: the corner of 3rd and Broadway")
    )
    # After SOI, and after a JFIF header if sips wrote one, as a phone's file has them.
    at = 2 + (2 + struct.unpack(">H", base[4:6])[0] if base[2:4] == b"\xff\xe0" else 0)
    (HERE / "geotagged.jpg").write_bytes(base[:at] + extra + base[at:])
    (HERE / "angle-a.jpg").write_bytes(jpeg(48, 32, lambda x, y: (200, 60 + y * 4, 40)))
    (HERE / "angle-b.jpg").write_bytes(jpeg(32, 32, lambda x, y: (40, 60 + x * 4, 200)))


main()
