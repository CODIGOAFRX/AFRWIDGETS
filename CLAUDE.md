# AFRWIDGETS

Widgets de escritorio para Windows, nativos y muy ligeros, más su landing de descarga en
pedrogomez.dev/proyecto/afrwidgets. Producto: [PRODUCT.md](PRODUCT.md). Sistema visual: [DESIGN.md](DESIGN.md).
Todo el texto de la app, la web y los comentarios va en español de España.

## Estructura

| Carpeta | Qué hay |
|---|---|
| `app/` | La app en Rust. `src/main.rs` (arranque y `--render-previews`), `src/app.rs` (ventanas, bandeja, menús, arrastre, latido), `src/gfx.rs` (Direct2D/DirectWrite sobre ventanas en capas, texto por línea base, Hanken Grotesk incrustada), `src/theme.rs` (las 6 estéticas y los acentos), `src/widgets/` (retícula S/M/L y los 13 widgets), `src/data/` (sistema, red, tiempo, Google, música) |
| `installer/` | Script de Inno Setup (`afrwidgets.iss`) |
| `site/` | Landing: Worker de Cloudflare con assets estáticos en `site/public/proyecto/afrwidgets/` y ruta en la zona pedrogomez.dev |
| `scripts/` | `build.ps1`: compila, pone icono y manifiesto con rcedit, genera previews y el instalador |

## Comandos

- Rust está en `~/.cargo/bin` (toolchain GNU, sin Visual Studio). Si `cargo` no está en el PATH: `& "$env:USERPROFILE\.cargo\bin\cargo.exe"`.
- Compilar: `cargo build --release` dentro de `app/`.
- Renders de todos los widgets y estéticas a PNG: `afrwidgets.exe --render-previews <carpeta>`.
- Todo (exe + icono + previews + instalador): `powershell -ExecutionPolicy Bypass -File scripts/build.ps1`.
- Inno Setup: `%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe`.
- Landing en local: `wrangler.cmd dev` dentro de `site/`. Despliegue: `wrangler.cmd deploy` dentro de `site/`.

## Dirección visual

- Widgets: limpios y minimalistas, de diseñador gráfico, **sin copiar los widgets de Apple**. Retícula de tres
  módulos, cifras grandes, rótulos en versalitas (o minúscula fina en Grotesca), filetes de 1 px, un acento.
- Landing: lenguaje web de Apple (`DESIGN.apple.md` es la referencia); el sistema real está en `DESIGN.md`.
- Grotesca y Grotesca negra se inspiran en estaciondiseno.es (grotesca gruesa, minúscula fina, acentos fosforito).
- Revisar siempre con hojas de contacto de los renders (`dist/hojas/`) antes de dar un cambio visual por bueno.

## Reglas de la app

- Lo ligero manda: nada de webviews, nada de animaciones en bucle, repintar sólo cuando cambia un dato.
  Render de Direct2D en modo software sobre un DIB (`UpdateLayeredWindow`), sin dispositivo de GPU por widget.
- Las peticiones de red van en hilos aparte y devuelven el resultado con `PostMessage`.
- El token de Google se guarda cifrado con DPAPI. Nunca en claro ni en logs.
- Las cifras de consumo de la landing se miden (Administrador de tareas / `Get-Process`) con los 13 widgets abiertos.

## Skills de este proyecto

Las skills de proyecto viven en `../.claude/skills/` (carpeta compartida de `code/`), como en `consigna/`;
las de usuario en `~/.claude/skills/` y en plugins. No se duplican aquí.

| Herramienta | Dónde | Cuándo usarla |
|---|---|---|
| `impeccable` | proyecto (`../.claude/skills`) | Diseño de las estéticas y de la landing. Ya inicializada (PRODUCT.md). Antes de cerrar algo visual: `/impeccable audit`; pasada final: `/impeccable polish`. |
| `modern-web-guidance` | proyecto | Antes de escribir HTML, CSS o JS de la landing. |
| `emil-design-eng`, `review-animations` | proyecto | Movimiento de la landing (y de la app, si alguna vez se anima algo). |
| `break-ui` | proyecto | Probar los widgets con datos feos: nombres de canción larguísimos, discos de 20 TB, eventos sin título, cero eventos. |
| `untangle-codebase`, `prune-tests` (Caldo AI) | proyecto | Cuando la app crezca sin orden o sobren tests. |
| `taste-skill` | plugin de usuario | Dirección de arte de la landing y reglas anti «hecho por IA». |
| `ui-ux-pro-max` (+ `design`, `design-system`, `ui-styling`) | usuario | Paletas, pares tipográficos y guías de UX antes de decidir estéticas. |
| `web-design-guidelines` | usuario | Auditar la landing contra las Web Interface Guidelines. |
| `playwright-cli` | usuario | Capturas y pruebas de la landing en navegador real (`playwright-cli.cmd`). |
| `find-skills` | usuario | Buscar skills nuevas si falta algo. |
| `cloudflare` | plugin de usuario | Worker de la landing, rutas y despliegue con wrangler. |
| `claude-mem` | plugin de usuario | Memoria entre sesiones. |
| `scroll-world` | plugin de proyecto | Sólo si se quiere una landing con imagen o vídeo generados. Usa servicios de pago: preguntar antes. |
| Subagentes | `.claude/agents/rust-engineer.md` (aquí) y `../.claude/agents/` | `rust-engineer` para la app; `frontend-developer`, `ui-designer`, `accessibility-tester`, `code-reviewer` e `impeccable-*` para la landing y las revisiones. |

Antes de dar por terminado un cambio visual de la landing: escritorio (1440×900), móvil (390×844) y `/impeccable audit`.
Para la app: regenerar los previews y revisarlos todos, y medir la RAM con los 13 widgets abiertos.
