# AFRWIDGETS

Widgets de escritorio para Windows 10 y 11, nativos y muy ligeros. Trece widgets en tres tamaños (S 170×170,
M 360×170, L 360×370) con seis estéticas cada uno y trece colores de acento:

| Estética | Cómo es |
|---|---|
| Suizo | Blanco, negro y un rojo; cifras de grotesca gruesa |
| Noche | Casi negro, cifras finas y ámbar |
| Instrumento | Gris cálido, naranja y tipografía DIN (Bahnschrift) |
| Aire | Sin panel: tipografía fina sobre tu fondo de pantalla |
| Grotesca | Grotesca gruesa (Hanken Grotesk, incrustada), minúscula fina y subrayado fosforito |
| Grotesca negra | La misma grotesca en blanco sobre tinta |

Web y descarga: <https://pedrogomez.dev/proyecto/afrwidgets>

| Widget | Diseños | De dónde salen los datos |
|---|---|---|
| Tiempo | S, M, L | Open-Meteo (sin clave) |
| Disco | S, M | Windows |
| Calendario | S, M, L | Google Calendar (OAuth, sólo lectura) |
| Reloj | Digital S, Analógico S, En palabras M | Windows |
| Husos horarios | M, S | Zonas horarias de Windows |
| Sistema | S, M | CPU y memoria de Windows |
| Red | S, M | Contadores de las interfaces físicas |
| Batería | S, M | Windows |
| Música | S, M | Controles multimedia de Windows (Spotify, navegador…) |
| Foco | S, M | Pomodoro local |
| Notas | Nota S, Lista M | Local |
| Progreso | Año S, Barras M, Cuenta atrás S | Local |
| Cielo | Luna S, Sol M | Calculado en local |

## Consumo medido

24/11/2026, Intel Core i5-8300H y 16 GB de RAM, con los 13 widgets abiertos (repartidos entre las seis estéticas)
durante dos minutos:

- Memoria (conjunto de trabajo privado, la del Administrador de tareas): **6,1 MB**
- Procesador: **0,17 %** de un núcleo de media
- `afrwidgets.exe`: **960 KB** (con la tipografía incrustada) · instalador: **2,3 MB**

## Uso

- Arrastra un widget para moverlo (se imanta a una rejilla de 8 px).
- Clic derecho sobre un widget: estética, diseño, color de acento, tamaño y opciones propias.
- Icono de la bandeja: añadir widgets, bloquear posiciones, iniciar con Windows, recolocar y salir.
- Configuración en `%APPDATA%\AFRWIDGETS\config.json`.

## Google Calendar

El widget usa OAuth 2.0 para apps de escritorio (redirección a `127.0.0.1` con PKCE) y el ámbito
`calendar.readonly`. El token de refresco se guarda cifrado con DPAPI.

Hace falta un cliente OAuth de Google Cloud:

1. En <https://console.cloud.google.com> crea un proyecto y activa la **Google Calendar API**.
2. Configura la pantalla de consentimiento (tipo externo) con el ámbito `.../auth/calendar.readonly`.
3. Crea unas credenciales **ID de cliente de OAuth → App de escritorio**.
4. Para que el instalador lo traiga de serie, compila con:

   ```powershell
   $env:AFR_GOOGLE_CLIENT_ID = "…apps.googleusercontent.com"
   $env:AFR_GOOGLE_CLIENT_SECRET = "…"
   powershell -ExecutionPolicy Bypass -File scripts/build.ps1
   ```

   Si no, la app pide el ID y el secreto la primera vez que conectas el calendario.

Mientras la app de Google no esté verificada, Google enseña el aviso de «app no verificada» y limita el
acceso a 100 usuarios de prueba.

## Compilar

Requisitos: Rust (toolchain `x86_64-pc-windows-gnu`, sin Visual Studio), Inno Setup 6, Node (para `rcedit`) y Python 3.

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build.ps1
```

Deja en `dist/` el exe con icono y manifiesto, los 180 renders de `--render-previews` y
`AFRWIDGETS-Setup-1.0.0.exe`.

## Landing

`site/` es un Worker de Cloudflare sólo con assets, con ruta en la zona pedrogomez.dev. Sigue el lenguaje web de
Apple (referencia en `DESIGN.apple.md`). Para actualizar los renders: `afrwidgets.exe --render-previews`, convertir a
WebP en `site/public/proyecto/afrwidgets/w/` y desplegar:

```powershell
cd site; npx.cmd wrangler deploy
```

## Licencias

AFRWIDGETS: MIT, ver [LICENSE](LICENSE). Tipografía Hanken Grotesk: SIL Open Font License, ver
[app/assets/OFL-HankenGrotesk.txt](app/assets/OFL-HankenGrotesk.txt). Foto de la landing: «Mountain Lake Sunset»,
420 Photography, Wikimedia Commons, CC0.
