"""Genera las fuentes de los relojes que van incrustadas en el exe.

Parte de las variables originales de Google Fonts (licencia OFL), fija un peso, deja sólo las cifras,
los dos puntos y el espacio, y le pone un nombre de familia propio para que DirectWrite la encuentre.

Uso: python scripts/fuentes_reloj.py <carpeta con las .ttf originales>
"""

import sys
from pathlib import Path

from fontTools.subset import Options, Subsetter
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ORIGEN = Path(sys.argv[1])
DESTINO = Path(__file__).resolve().parent.parent / "app" / "assets"
TEXTO = "0123456789: "

# (archivo de origen, ejes, familia nueva, archivo de salida)
FUENTES = [
    ("BigShoulders[opsz,wght].ttf", {"opsz": 72, "wght": 150}, "AFR Alto Fino", "reloj-alto-fino.ttf"),
    ("BigShoulders[opsz,wght].ttf", {"opsz": 72, "wght": 600}, "AFR Alto Grueso", "reloj-alto-grueso.ttf"),
    ("Inter[opsz,wght].ttf", {"opsz": 32, "wght": 600}, "AFR Reloj", "reloj-minimal.ttf"),
]


def renombrar(font: TTFont, familia: str) -> None:
    nombres = font["name"]
    nombres.names = [n for n in nombres.names if n.nameID not in (1, 2, 3, 4, 6, 16, 17, 21, 22, 25)]
    ps = familia.replace(" ", "") + "-Regular"
    for nid, valor in ((1, familia), (2, "Regular"), (3, ps), (4, familia), (6, ps)):
        nombres.setName(valor, nid, 3, 1, 0x409)
        nombres.setName(valor, nid, 1, 0, 0)
    font["OS/2"].usWeightClass = 400
    font["OS/2"].fsSelection = (font["OS/2"].fsSelection & ~0b1100001) | 0b1000000  # Regular
    font["head"].macStyle = 0


for origen, ejes, familia, salida in FUENTES:
    font = TTFont(ORIGEN / origen)
    font = instantiateVariableFont(font, ejes)
    opts = Options()
    # La alta va con cifras proporcionales (juntas, como en el iPhone); la minimal, tabulares.
    opts.layout_features = ["kern"] if "Alto" in familia else ["tnum", "kern"]
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    sub = Subsetter(opts)
    sub.populate(text=TEXTO)
    sub.subset(font)
    renombrar(font, familia)
    font.save(DESTINO / salida)
    print(f"{salida}: {(DESTINO / salida).stat().st_size / 1024:.1f} KB")
