"""Recomprime los PNG que escribe la app (deflate sin comprimir) y el .ico. Sólo usa la biblioteca estándar."""
import struct, sys, zlib, pathlib

def recompress(data: bytes) -> bytes:
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    out, i, idat = [data[:8]], 8, b""
    chunks = []
    while i < len(data):
        n = struct.unpack(">I", data[i:i+4])[0]; kind = data[i+4:i+8]; body = data[i+8:i+8+n]
        i += 12 + n
        if kind == b"IDAT": idat += body
        else: chunks.append((kind, body))
    raw = zlib.decompress(idat)
    def chunk(k, b): return struct.pack(">I", len(b)) + k + b + struct.pack(">I", zlib.crc32(k + b) & 0xffffffff)
    res = data[:8]
    for k, b in chunks:
        if k == b"IEND":
            res += chunk(b"IDAT", zlib.compress(raw, 9))
        res += chunk(k, b)
    return res

def ico(path: pathlib.Path):
    d = path.read_bytes(); n = struct.unpack("<H", d[4:6])[0]
    entries = [d[6+16*k:6+16*(k+1)] for k in range(n)]
    imgs = []
    for e in entries:
        size, off = struct.unpack("<II", e[8:16]); imgs.append((e[:8], recompress(d[off:off+size])))
    out = d[:6]; off = 6 + 16*n; body = b""
    for head, png in imgs:
        out += head + struct.pack("<II", len(png), off); off += len(png); body += png
    path.write_bytes(out + body)

folder = pathlib.Path(sys.argv[1])
before = sum(p.stat().st_size for p in folder.iterdir())
for p in folder.glob("*.png"): p.write_bytes(recompress(p.read_bytes()))
ico(folder / "afrwidgets.ico")
after = sum(p.stat().st_size for p in folder.iterdir())
print(f"PNG recomprimidos: {before//1024} KB -> {after//1024} KB")
