---
name: AFRWIDGETS
description: Widgets nativos y ligeros para el escritorio de Windows, con seis estéticas de diseñador gráfico, y su landing de descarga en lenguaje web de Apple.
colors:
  primary: "#0066cc"
  primary-focus: "#0071e3"
  primary-hover: "#0077ed"
  primary-on-dark: "#2997ff"
  ink: "#1d1d1f"
  ink-80: "#333333"
  ink-48: "#5f5f64"
  muted-dark: "#a1a1a6"
  canvas: "#ffffff"
  parchment: "#f5f5f7"
  tile: "#000000"
  footer-rule: "#d2d2d7"
  chip-ring: "#48484a"
  desk-fallback: "#2a1f1b"
  suizo-panel: "#f7f7f5"
  suizo-ink: "#111111"
  suizo-ink-2: "#5e5e5e"
  suizo-rule: "#d9d9d6"
  suizo-accent: "#e63312"
  noche-panel: "#121214"
  noche-ink: "#f2f2ef"
  noche-ink-2: "#8e8e93"
  noche-rule: "#2a2a2f"
  noche-accent: "#f5a524"
  instrumento-panel: "#e6e4df"
  instrumento-ink: "#232323"
  instrumento-ink-2: "#66635d"
  instrumento-rule: "#cac7bf"
  instrumento-accent: "#ff6a00"
  aire-ink: "#ffffff"
  grotesca-panel: "#ffffff"
  grotesca-ink: "#1f201d"
  grotesca-track: "#ecece8"
  grotesca-negra-track: "#393a35"
  fosforito: "#00ff81"
  warn-light: "#d12b0f"
  warn-dark: "#ff6b4a"
typography:
  display:
    fontFamily: "SF Pro Display, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "clamp(40px, 5.6vw, 72px)"
    fontWeight: 600
    lineHeight: 1.05
    letterSpacing: "-0.025em"
  headline:
    fontFamily: "SF Pro Display, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "clamp(34px, 4.6vw, 56px)"
    fontWeight: 600
    lineHeight: 1.07
    letterSpacing: "-0.015em"
  lead:
    fontFamily: "SF Pro Display, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "clamp(21px, 2.1vw, 28px)"
    fontWeight: 400
    lineHeight: 1.25
    letterSpacing: "0.002em"
  sub-lead:
    fontFamily: "SF Pro Text, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "clamp(19px, 1.8vw, 21px)"
    fontWeight: 400
    lineHeight: 1.38
    letterSpacing: "0.004em"
  figure:
    fontFamily: "SF Pro Display, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "clamp(56px, 7vw, 96px)"
    fontWeight: 600
    lineHeight: 1
    letterSpacing: "-0.03em"
    fontFeature: "tnum"
  title:
    fontFamily: "SF Pro Display, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "21px"
    fontWeight: 600
    lineHeight: 1.19
    letterSpacing: "0.005em"
  title-sm:
    fontFamily: "SF Pro Text, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "17px"
    fontWeight: 600
    lineHeight: 1.24
    letterSpacing: "-0.022em"
  body:
    fontFamily: "SF Pro Text, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "17px"
    fontWeight: 400
    lineHeight: 1.47
    letterSpacing: "-0.022em"
  caption:
    fontFamily: "SF Pro Text, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.43
    letterSpacing: "-0.016em"
  fine:
    fontFamily: "SF Pro Text, -apple-system, BlinkMacSystemFont, Inter, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "-0.01em"
  widget-num-suizo:
    fontFamily: "Segoe UI Variable Display"
    fontWeight: 700
    letterSpacing: "-0.035em"
    fontFeature: "tnum"
  widget-num-noche:
    fontFamily: "Segoe UI Variable Display"
    fontWeight: 250
    letterSpacing: "-0.02em"
    fontFeature: "tnum"
  widget-num-instrumento:
    fontFamily: "Bahnschrift"
    fontWeight: 350
    letterSpacing: "-0.01em"
    fontFeature: "tnum"
  widget-num-aire:
    fontFamily: "Segoe UI Variable Display"
    fontWeight: 200
    letterSpacing: "-0.02em"
    fontFeature: "tnum"
  widget-num-grotesca:
    fontFamily: "Hanken Grotesk"
    fontWeight: 800
    letterSpacing: "-0.045em"
    fontFeature: "tnum"
  widget-label:
    fontFamily: "Bahnschrift"
    fontSize: "10.5px"
    fontWeight: 600
    letterSpacing: "0.12em"
  widget-label-grotesca:
    fontFamily: "Hanken Grotesk"
    fontSize: "12.8px"
    fontWeight: 400
    letterSpacing: "0em"
  widget-text:
    fontFamily: "Segoe UI Variable Text"
    fontWeight: 400
rounded:
  pill: "9999px"
  card: "18px"
  thumb: "12px"
  skip: "8px"
  focus: "6px"
  widget-suizo: "10px"
  widget-noche: "20px"
  widget-instrumento: "26px"
  widget-square: "0px"
spacing:
  gutter: "22px"
  gutter-mobile: "20px"
  band: "96px"
  band-mobile: "72px"
  wrap: "1100px"
  grid-sm: "20px"
  grid-md: "24px"
  grid-lg: "32px"
  widget-pad: "18px"
  widget-pad-aire: "10px"
  widget-s: "170px"
  widget-m: "360px"
  widget-l-height: "370px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.canvas}"
    typography: "{typography.body}"
    rounded: "{rounded.pill}"
    padding: "11px 22px"
    height: "44px"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
    textColor: "{colors.canvas}"
  button-primary-sm:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.canvas}"
    typography: "{typography.fine}"
    rounded: "{rounded.pill}"
    padding: "4px 12px"
    height: "28px"
  chip:
    backgroundColor: "transparent"
    textColor: "{colors.parchment}"
    typography: "{typography.caption}"
    rounded: "{rounded.pill}"
    padding: "12px 18px"
    height: "44px"
  chip-selected:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.parchment}"
    rounded: "{rounded.pill}"
    padding: "11px 17px"
  card:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.ink}"
    rounded: "{rounded.card}"
    padding: "28px 24px"
  nav-global:
    backgroundColor: "{colors.tile}"
    textColor: "{colors.parchment}"
    typography: "{typography.fine}"
    height: "44px"
  nav-sub:
    textColor: "{colors.ink-80}"
    typography: "{typography.fine}"
    height: "52px"
  widget-suizo:
    backgroundColor: "{colors.suizo-panel}"
    textColor: "{colors.suizo-ink}"
    rounded: "{rounded.widget-suizo}"
    padding: "{spacing.widget-pad}"
  widget-noche:
    backgroundColor: "{colors.noche-panel}"
    textColor: "{colors.noche-ink}"
    rounded: "{rounded.widget-noche}"
    padding: "{spacing.widget-pad}"
  widget-instrumento:
    backgroundColor: "{colors.instrumento-panel}"
    textColor: "{colors.instrumento-ink}"
    rounded: "{rounded.widget-instrumento}"
    padding: "{spacing.widget-pad}"
  widget-aire:
    backgroundColor: "transparent"
    textColor: "{colors.aire-ink}"
    rounded: "{rounded.widget-square}"
    padding: "{spacing.widget-pad-aire}"
  widget-grotesca:
    backgroundColor: "{colors.grotesca-panel}"
    textColor: "{colors.grotesca-ink}"
    rounded: "{rounded.widget-square}"
    padding: "{spacing.widget-pad}"
  widget-grotesca-negra:
    backgroundColor: "{colors.grotesca-ink}"
    textColor: "{colors.canvas}"
    rounded: "{rounded.widget-square}"
    padding: "{spacing.widget-pad}"
---

# Design System: AFRWIDGETS

Este sistema tiene dos superficies con lenguajes distintos, y no se mezclan:

1. **Los widgets nativos** (Rust + Direct2D, `app/src/theme.rs`, `app/src/widgets/mod.rs`, `app/src/gfx.rs`): seis estéticas de diseñador gráfico sobre una retícula común. Son el producto.
2. **La landing web** (`site/public/proyecto/afrwidgets/`): lenguaje web de Apple (referencia fijada por el usuario en `DESIGN.apple.md`), con valores propios. Es el escaparate del producto.

Los tokens `widget-*`, `suizo-*`, `noche-*`, `instrumento-*`, `aire-*`, `grotesca-*` y `fosforito` pertenecen a los widgets; el resto, a la landing.

## Overview

**Creative North Star: "La vitrina y la pieza"**

La landing es una vitrina blanca, pergamino y negra que se aparta para que la pieza, el widget real, sea lo único con volumen. Bandas a sangre que alternan de tono, titulares de 600 apretados, un único azul para todo lo que se pulsa y aire de sobra. No hay ilustración ni maqueta: las imágenes son renders producidos por la propia app sobre una fotografía de escritorio.

Los widgets son lo contrario de una vitrina: piezas gráficas planas, de diseñador, que viven sobre el fondo de pantalla de cualquiera. Comparten una retícula de tres módulos, un margen interior, texto colocado por línea base, cifras tabulares, filetes de 1 px y un solo color de acento por widget. Cada estética es un lenguaje completo (color, tipografía, interletraje, radio), no un cambio de paleta. El usuario lo dejó dicho: limpios y minimalistas, de diseñador gráfico, y sin copiar los widgets de Apple. La familia Grotesca nace de estaciondiseno.es: grotesca gruesa en mayúsculas, minúscula fina para los rótulos y el acento como rotulador fosforito.

Lo ligero es la función: ninguna decisión visual de los widgets puede costar memoria o CPU sostenida, así que no hay degradados animados, desenfoques ni sombras pintadas por widget (la única excepción es el halo de Aire, que es legibilidad, no decoración).

**Key Characteristics:**
- Landing: bandas a sangre blanco / pergamino / negro, un azul de acción, píldoras, tarjetas de 18 px, una sola sombra (la del producto).
- Widgets: retícula S 170×170, M 360×170, L 360×370 con margen de 18; cifras grandes tabulares; rótulos pequeños; filetes de 1 px; un acento.
- Seis estéticas que cambian tipografía, radio e interletraje, no sólo color.
- Iconos dibujados con formas geométricas puras; nunca fuentes de iconos ni emoji.

## Colors

La landing es neutra con un único azul; los widgets son neutros por estética con un acento por widget elegido de una lista cerrada.

### Primary
- **Azul de acción** (primary): todo lo que se pulsa en la landing: píldoras, enlaces, números de los pasos. Al pasar el ratón la píldora sube a **azul encendido** (primary-hover).
- **Azul de foco** (primary-focus): anillo de foco de 2 px y anillo del chip elegido.
- **Azul sobre negro** (primary-on-dark): enlaces y anillo de foco dentro de las bandas negras y la barra global.

### Neutral (landing)
- **Tinta** (ink): texto principal y fondo del chip elegido.
- **Tinta 80** (ink-80): enlaces de la subbarra.
- **Gris de apoyo** (ink-48): entradillas, pies de tarjeta, letra pequeña, unidades de las cifras. Es el gris más claro permitido sobre blanco y pergamino.
- **Gris sobre negro** (muted-dark): entradillas y párrafos en bandas negras.
- **Lienzo** (canvas), **Pergamino** (parchment), **Negro de banda** (tile): los tres tonos de banda. Las tarjetas son lienzo sobre pergamino.
- **Filete del pie** (footer-rule) y **Anillo de chip** (chip-ring): los dos únicos trazos de la landing.
- **Escritorio de reserva** (desk-fallback): color bajo la foto del lago mientras carga.

### Widgets: neutros por estética
- **Suizo**: panel blanco roto, tinta casi negra, grises 5E/9C, filete D9D9D6, pista E4E4E0. Acento por defecto rojo suizo.
- **Noche**: panel casi negro con borde blanco al 6 %, tinta F2F2EF, grises 8E/55, filete 2A2A2F. Acento por defecto ámbar; un acento demasiado oscuro se aclara un 40 % hacia blanco.
- **Instrumento**: panel gris cálido, tinta 232323, grises cálidos 66635D/9E9A92, filete CAC7BF. Acento por defecto naranja.
- **Aire**: sin panel. Todo en blanco a distintas opacidades (82, 60, 38 y 26 %) con halo para leerse sobre cualquier foto. Acento por defecto blanco.
- **Grotesca**: panel blanco puro, tinta 1F201D, grises como tinta al 68 y 45 %, el filete es la propia tinta. Acento por defecto **fosforito**.
- **Grotesca negra**: la inversa: panel 1F201D, tinta blanca (72 y 46 %), filete blanco al 85 %, pista 393A35. Acento fosforito.

### Acentos de widget
Lista cerrada en `theme.rs`: el de la estética, el de Windows, rojo E63312, naranja FF6A00, ámbar F5A524, verde 2E9E5B, azul 2F6BFF, rosa FF4F79, verde neón 00FF81, amarillo limón FFFF82, salmón FF7A78, rosa chicle FF79F5 y azul cielo 8CE5FF. El texto sobre acento es 111111 si la luminancia del acento pasa de 0,45, y blanco si no. Aviso: **rojo de aviso** (warn-light) en estéticas claras y **coral de aviso** (warn-dark) en oscuras.

### Named Rules
**The One Blue Rule.** En la landing sólo es azul lo que se pulsa. Ningún titular, icono ni fondo es azul.

**The One Accent Rule.** Cada widget tiene un único acento. Todo lo demás sale de tinta, tintas secundarias, filete y pista de su estética.

**The Safe Accent Rule.** Una estética clara sustituye un acento casi blanco por su acento por defecto, y Noche aclara un acento casi negro. Ningún acento puede desaparecer contra su panel.

## Typography

**Landing, titulares:** SF Pro Display en Apple; Inter autoalojada (woff2 variable 100–900, latin y latin-ext) en el resto.
**Landing, texto:** SF Pro Text, con la misma Inter de reserva.
**Widgets:** Segoe UI Variable Display/Text (Suizo, Noche, Aire), Bahnschrift, la DIN de Windows (rótulos de Suizo y todo Instrumento), y Hanken Grotesk incrustada en el exe (OFL) para Grotesca y Grotesca negra, de modo que no depende de lo que tenga instalado el PC.

**Character:** En la landing, una sola familia trabajada por peso e interletraje: 600 apretado para titulares, 400 a 17 px para leer. En los widgets, la tipografía es la estética: la cifra de Noche a 250 es casi un hilo, la de Suizo a 700 es un bloque, la de Grotesca a 800 en mayúsculas con −0,045 em es un cartel.

### Hierarchy (landing)
- **Display** (h1 del hero): una vez por página, equilibrado con `text-wrap: balance`.
- **Headline** (h2 de cada banda): siempre centrado, con su entradilla debajo.
- **Lead** (entradilla del hero, 30 em máx.) y **Sub-lead** (entradilla de banda, 34 em máx.), ambas en gris de apoyo.
- **Figure**: cifras medidas de la sección de ligereza, tabulares, con la unidad a 0,4 em en gris.
- **Title** (21 px) en columnas de datos y pasos; **Title-sm** (17 px) bajo cada miniatura de widget.
- **Body** (17 px / 1,47), **Caption** (14 px) para pies de tarjeta y chips, **Fine** (12 px) para navegación, requisitos, método y pie.

### Hierarchy (widgets)
- **Cifra** (num): el dato principal a tamaño grande, con la unidad detrás al 28 % de su cuerpo (mín. 12) en tinta secundaria. Hay una variante 200 más pesada (tope 800) para el dato principal de mosaicos.
- **Rótulo** (label) a 10,5 px: versalitas espaciadas (0,10–0,14 em) en todas las estéticas salvo Grotesca, donde va en minúscula fina 400, sin interletraje y un 22 % mayor.
- **Texto** y **texto fuerte** (600) para descripciones, eventos y títulos de canción.
- Grotesca pasa a mayúsculas las palabras grandes (cifras y el reloj en palabras).

### Named Rules
**The Baseline Rule.** En los widgets todo texto se coloca por su línea base (`text_base`), nunca por la caja: así una cifra de 64 y un rótulo de 10 se alinean como en una maqueta impresa.

**The Tabular Rule.** Toda cifra es tabular: DirectWrite activa `tnum` en toda la tipografía de los widgets, y las cifras grandes de la landing llevan `font-variant-numeric: tabular-nums`. Un reloj o un contador no baila.

**The Lowercase Grotesca Rule.** En Grotesca los rótulos van en minúscula fina y las palabras grandes en mayúscula gruesa. Nunca versalitas espaciadas en esa familia.

## Layout

**Landing.** Bandas a sangre (96 px arriba y abajo, 22 px a los lados; 72/20 por debajo de 833 px) que alternan lienzo, negro y pergamino; el hero y la sección de ligereza en blanco, estéticas y conexión en negro, widgets e instalación en pergamino. Contenido en un contenedor de 1100 px; titulares y entradillas en una columna centrada de 760 px. Rejillas: miniaturas de widgets en `auto-fill` de 200 px mín. con 20 px de hueco (dos columnas en móvil, 12 px); tres columnas para cifras, datos y pasos que pasan a una sola por debajo de 833 px. La mesa de estéticas es una retícula de 4 columnas con áreas `"a a b c" "d e f f"` que reproduce los módulos M y S; en móvil, 2 columnas. La pantalla del hero es un escritorio 16:9 (2:3 en móvil, con cuatro widgets) en el que cada widget se coloca a su tamaño real relativo sobre 1300×731. Navegación: barra global negra de 44 px y subbarra pegajosa de 52 px; `scroll-padding-top` de 64 px. Cortes: 833 px y 640 px.

**Widgets.** Retícula de tres módulos en DIP: **S** 170×170, **M** 360×170 (dos S más un hueco de 20) y **L** 360×370 (dos M más 30). Todo widget declara sus diseños sobre esos módulos; no hay tamaños libres. Margen interior común de 18 (10 en Aire, que no tiene panel). En el escritorio se imantan a una rejilla. Los M partidos en columnas se separan con filetes verticales de 1 px (husos horarios, previsión del tiempo).

### Named Rules
**The Three Modules Rule.** Un widget nuevo encaja en S, M o L. Si no cabe, se rediseña; no se inventa un cuarto tamaño.

## Elevation & Depth

La landing es plana: profundidad por cambio de tono de banda y por tarjetas blancas sobre pergamino. La única sombra es la del producto, un `drop-shadow` bajo cada render de widget, que sigue la silueta real del PNG (incluida la franja de Grotesca y la ausencia de panel de Aire). La subbarra es esmerilada (blanco al 78 % con `saturate(180%) blur(20px)`) y lleva un filete inferior de negro al 8 %.

Los widgets son planos. No pintan sombra propia: la ventana en capas deja ver el fondo. Noche dibuja un borde de 1 px blanco al 6 % para despegarse de fondos oscuros; Aire, sin panel, dibuja un halo suave bajo texto y trazos para leerse sobre cualquier foto.

### Shadow Vocabulary
- **Sombra de producto** (`filter: drop-shadow(3px 5px 30px rgb(0 0 0 / 0.22))`, token `--shadow-product`): bajo todo render de widget de la landing.
- **Halo de Aire** (sólo en la app): legibilidad sobre fondos fotográficos, no decoración.

### Named Rules
**The Product Shadow Rule.** En la landing sólo hace sombra el producto. Ni tarjetas, ni píldoras, ni chips llevan sombra.

## Shapes

**Landing:** píldora total (9999 px) para todo lo pulsable (botones y chips); 18 px para tarjetas, la pantalla del hero y la mesa; 12 px para el fondo de cada miniatura; 6 px de esquina en el anillo de foco.

**Widgets:** el radio es parte de la estética: Suizo 10, Noche 20, Instrumento 26 y esquina viva en Aire, Grotesca y Grotesca negra. Barras con extremos redondos (radio = media altura); barras segmentadas con huecos de 2–4 px y esquina máxima de 2; anillos finos de trazo redondo; filetes de 1 px; iconos geométricos (sol en disco de acento, nube maciza de un rectángulo redondeado y dos círculos, lluvia en trazos, triángulos y barras para los mandos de música).

**Franja de Grotesca:** una banda vertical de acento de 7 px en el borde derecho del panel, a toda altura.

## Components

### Buttons (landing)
- **Shape:** píldora completa (9999 px), alto mínimo 44 px.
- **Primary:** azul de acción con texto blanco a 17 px, 11×22 de relleno. Hover: azul encendido; pulsación: `scale(0.95)` en 120 ms. Transición de color 150 ms `ease-out`.
- **Pequeña:** misma píldora a 12 px, alto 28, relleno 4×12, sólo en la subbarra.
- **Enlace con flecha:** texto azul con un «›» detrás que avanza 3 px al pasar el ratón; alto mínimo 44 px. Acompaña a una píldora en el mismo grupo de acciones (12×28 de hueco).

### Chips (landing)
- **Style:** píldoras transparentes sobre banda negra, texto pergamino a 14 px, anillo de 1 px gris (chip-ring) que se aclara a 86868B al pasar el ratón.
- **State:** el elegido (`aria-pressed="true"`) pasa a anillo de 2 px azul de foco sobre tinta, compensando el relleno para no saltar. Cambian a la vez la mesa y la rejilla de widgets, que bajan a opacidad 0,35 durante 220 ms mientras se cambian.

### Cards / Containers (landing)
- **Corner Style:** 18 px.
- **Background:** lienzo sobre banda pergamino.
- **Shadow Strategy:** ninguna (ver Elevation).
- **Border:** ninguno.
- **Internal Padding:** 28×24 en pasos; 20 20 24 en miniaturas (12 12 16 en móvil). Las miniaturas muestran el render a 340 px sobre un recorte de la foto del lago con 14 % de relleno.

### Navigation (landing)
- Barra global negra de 44 px con enlaces de 12 px en E8E8ED (blanco al pasar). Subbarra pegajosa esmerilada de 52 px: nombre del producto a 21 px y 600, enlaces de 12 px en tinta 80 que se vuelven azules al pasar, y la píldora pequeña «Descargar». En móvil (≤833 px) quedan sólo el nombre y la píldora.

### Widget panel (signature)
Cada widget empieza igual: panel de su estética con su radio (o nada en Aire), franja de 7 px en Grotesca, borde de 1 px en Noche, y el contenido a 18 del borde. Arriba a la izquierda, un rótulo; debajo, la cifra o palabra grande; abajo, una línea de texto secundario o una barra.

### Barras, segmentos y anillos (signature)
- **Barra fina** con pista y extremos redondos, para disco, progreso y la previsión.
- **Barra segmentada**: n piezas llenas según la fracción, en acento sobre pista; es la firma de batería, sistema, foco y año.
- **Anillo fino** para el pomodoro; **línea de historial** con relleno de acento al 14 % para CPU y red.

### Subrayado fosforito (Grotesca)
Cuando una cifra o palabra grande iría en acento, Grotesca no tiñe el texto: lo deja en tinta y pinta detrás una banda de acento de 0,42 del cuerpo, desde media altura hasta la línea base, como un rotulador. Grotesca negra la pinta a toda altura (0,98 del cuerpo) y pone el texto en el color legible sobre acento. Los rótulos que irían en acento quedan en tinta.

## Do's and Don'ts

### Do:
- **Do** usar en la landing sólo renders reales generados por la app (`--render-previews`), con la sombra de producto.
- **Do** alternar bandas blanco, negro y pergamino a sangre, y centrar titular y entradilla en 760 px.
- **Do** reservar el azul de acción (y el azul sobre negro en bandas negras) para lo pulsable.
- **Do** encajar todo widget en S 170×170, M 360×170 o L 360×370 con margen de 18.
- **Do** colocar el texto de los widgets por línea base y con cifras tabulares.
- **Do** separar zonas de un widget con filetes de 1 px en el color de filete de su estética.
- **Do** dibujar iconos con formas geométricas (círculos, rectángulos redondeados, triángulos, trazos).
- **Do** usar un único acento por widget, de la lista cerrada de `theme.rs`.
- **Do** en Grotesca, mayúscula gruesa para lo grande, minúscula fina para los rótulos y el acento como subrayado fosforito o franja de 7 px.

### Don't:
- **Don't** copiar los widgets de Apple: ni su panel translúcido ni su tipografía; los widgets son piezas de diseñador gráfico y cada estética tiene su propio radio.
- **Don't** poner sombra a tarjetas, píldoras o chips de la landing; sólo el producto la lleva.
- **Don't** añadir un segundo color de acción a la landing ni teñir titulares de azul.
- **Don't** usar fuentes de iconos, emoji ni imágenes para los iconos de los widgets.
- **Don't** animar nada en bucle en los widgets ni añadir efectos que repinten sin que cambie un dato.
- **Don't** usar versalitas espaciadas en Grotesca ni teñir texto de fosforito: el fosforito siempre va detrás del texto.
- **Don't** inventar un cuarto tamaño de widget fuera de la retícula.
- **Don't** usar maquetas, testimonios o cifras no medidas en la landing.
