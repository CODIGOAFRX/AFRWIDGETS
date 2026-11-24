# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

La landing es web (pedrogomez.dev/proyecto/afrwidgets). La app es nativa de Windows 10/11: sus widgets
son ventanas Win32 dibujadas con Direct2D, no una web empaquetada.

## Stack

- App: Rust (toolchain `x86_64-pc-windows-gnu`), crate `windows` (Win32, Direct2D, DirectWrite, WinHTTP,
  WinRT para la música), `serde_json` para la configuración. Sin webview ni runtime.
- Instalador: Inno Setup 6 (`installer/afrwidgets.iss`), sin permisos de administrador.
- Landing: HTML, CSS y JS sin frameworks, servida por un Worker de Cloudflare con ruta en la zona
  pedrogomez.dev (como `consigna`). El instalador se sirve como asset estático del mismo Worker.

## Users

Gente que personaliza su escritorio de Windows y quiere widgets bonitos sin pagar el peaje de apps
pesadas (Electron, motores de wallpaper). Se bajan el instalador desde la landing, colocan unos cuantos
widgets y se olvidan: la app tiene que desaparecer de su consumo de recursos.

## Product Purpose

Poner widgets en el escritorio (tiempo, disco, calendario de Google y diez más) con varias estéticas por
widget, gastando lo mínimo posible de memoria y CPU. El éxito es que el usuario no note la app en el
Administrador de tareas y que su escritorio se vea mejor que antes.

## Positioning

Nativa de verdad: un único `.exe` pequeño que dibuja con Direct2D y sólo repinta cuando cambia un dato.
Cada widget admite varias estéticas y diseños; no hay que aprender un lenguaje de skins para que quede bien.

## Operating Context

- Vive en la bandeja del sistema; los widgets están pegados al escritorio, detrás de las ventanas, y
  sobreviven a Win+D.
- Se configura con clic derecho sobre cada widget (estética, diseño, tamaño, opciones) y desde el icono
  de la bandeja (añadir widgets, bloquear posiciones, arrancar con Windows).
- El calendario se conecta a Google con OAuth (loopback + PKCE, sólo lectura). Necesita un cliente OAuth
  de tipo «app de escritorio» creado en Google Cloud; el token de refresco se guarda cifrado con DPAPI.
- El tiempo usa Open-Meteo (sin clave). El resto de datos salen del propio Windows.

## Capabilities and Constraints

- 13 widgets: Tiempo, Disco, Calendario, Reloj, Husos horarios, Sistema (CPU y RAM), Red, Batería,
  Música, Foco (pomodoro), Notas, Progreso y Cielo (sol y luna).
- Varias estéticas por widget más variantes de diseño propias de cada uno.
- Consumo: sólo se publican cifras medidas de verdad en el portátil de desarrollo con los 13 widgets
  abiertos. Nunca cifras estimadas.
- Español de España en toda la interfaz y la web.
- El instalador no está firmado: Windows SmartScreen avisará la primera vez. La landing lo dice.
- Decisión abierta: el nombre del repo público de GitHub (aún no existe).

## Brand Commitments

- Nombre: AFRWIDGETS (familia AFR de Pedro Gómez, como AFR-Mind).
- Sin estilo de referencia externo: la estética es propia.

## Evidence on Hand

- Las capturas de la landing son renders reales de los widgets, generados por la propia app
  (`afrwidgets.exe --render-previews <carpeta>`). No se usan maquetas inventadas.
- No hay testimonios, usuarios ni métricas de descargas. No inventarlos.

## Product Principles

1. Lo ligero es la función: cualquier mejora visual que cueste memoria o CPU sostenida se descarta.
2. Repintar sólo cuando cambia algo; nada se anima en bucle.
3. Cada widget tiene que verse bien sobre cualquier fondo de pantalla.
4. Configurar es clic derecho y listo; sin archivos que editar a mano.
5. Las cifras que se enseñan son medidas, no prometidas.

## Accessibility & Inclusion

Texto de los widgets con contraste suficiente sobre fondos variados (cada estética lo resuelve con panel
o con halo). Respeta el escalado de pantalla (DPI por monitor). La landing cumple WCAG 2.2 AA y
`prefers-reduced-motion`.
