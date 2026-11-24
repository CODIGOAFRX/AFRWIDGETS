//! Estéticas. Cada una es un lenguaje gráfico completo: color, tipografía, interletraje y radio.
//! Cada widget elige la suya por separado, más un color de acento.

use crate::gfx::{Color, Font};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeId {
    Suizo,
    Noche,
    Instrumento,
    Aire,
    Grotesca,
    GrotescaNegra,
}

pub const THEMES: [(ThemeId, &str, &str); 6] = [
    (ThemeId::Suizo, "suizo", "Suizo · blanco, negro y un acento"),
    (ThemeId::Noche, "noche", "Noche · casi negro, cifras finas"),
    (ThemeId::Instrumento, "instrumento", "Instrumento · gris cálido, tipografía DIN"),
    (ThemeId::Aire, "aire", "Aire · sin panel, sobre tu fondo"),
    (ThemeId::Grotesca, "grotesca", "Grotesca · grotesca gruesa y subrayado fosforito"),
    (ThemeId::GrotescaNegra, "grotesca-negra", "Grotesca negra · grotesca gruesa sobre tinta"),
];

impl ThemeId {
    pub fn from_key(k: &str) -> ThemeId {
        THEMES.iter().find(|t| t.1 == k).map(|t| t.0).unwrap_or(ThemeId::Noche)
    }
}

/// (clave, nombre en el menú, color). 0 = el de la estética, 1 = el de Windows.
pub const ACCENTS: [(&str, &str, u32); 13] = [
    ("auto", "El de la estética", 0),
    ("windows", "Color de Windows", 1),
    ("rojo", "Rojo", 0xE63312),
    ("naranja", "Naranja", 0xFF6A00),
    ("ambar", "Ámbar", 0xF5A524),
    ("verde", "Verde", 0x2E9E5B),
    ("azul", "Azul", 0x2F6BFF),
    ("rosa", "Rosa", 0xFF4F79),
    ("neon", "Verde neón", 0x00FF81),
    ("limon", "Amarillo limón", 0xFFFF82),
    ("salmon", "Salmón", 0xFF7A78),
    ("chicle", "Rosa chicle", 0xFF79F5),
    ("cielo", "Azul cielo", 0x8CE5FF),
];

pub fn accent_color(key: &str, theme: ThemeId, system: Color) -> Color {
    match ACCENTS.iter().find(|a| a.0 == key) {
        Some((_, _, 1)) => system,
        Some((_, _, v)) if *v > 1 => Color::hex(*v),
        _ => match theme {
            ThemeId::Suizo => Color::hex(0xE63312),
            ThemeId::Noche => Color::hex(0xF5A524),
            ThemeId::Instrumento => Color::hex(0xFF6A00),
            ThemeId::Aire => Color::hex(0xFFFFFF),
            ThemeId::Grotesca | ThemeId::GrotescaNegra => Color::hex(0x00FF81),
        },
    }
}

const SEGOE_DISPLAY: &str = "Segoe UI Variable Display";
const SEGOE_TEXT: &str = "Segoe UI Variable Text";
const DIN: &str = "Bahnschrift";
const GROT: &str = crate::gfx::GROTESK;

#[derive(Clone, Copy)]
pub struct Style {
    pub id: ThemeId,
    pub panel: Option<Color>,
    pub border: Option<Color>,
    pub radius: f32,
    /// Tinta principal, secundaria y terciaria.
    pub ink: Color,
    pub ink2: Color,
    pub ink3: Color,
    /// Filete de 1 px.
    pub rule: Color,
    /// Pista de barras y anillos.
    pub track: Color,
    pub accent: Color,
    /// Sombra suave bajo texto y trazos para leerse sobre cualquier fondo (Aire).
    pub halo: bool,
    /// El acento no tiñe texto: subraya como un rotulador fosforito (Grotesca).
    pub marker: bool,
    /// Rótulos en minúscula fina en lugar de versalitas espaciadas.
    pub lower_labels: bool,
    /// Palabras grandes en mayúsculas.
    pub caps_display: bool,
    /// Subrayado a toda altura (para fondos oscuros, donde el texto va en tinta sobre el acento).
    pub marker_full: bool,
    /// Franja de acento en el borde derecho (0 = sin franja).
    pub edge: f32,
    pub pad: f32,
    num: (&'static str, u32, f32),
    label: (&'static str, u32, f32),
    text: (&'static str, u32),
}

impl Style {
    pub fn new(id: ThemeId, accent: Color) -> Style {
        match id {
            ThemeId::Suizo => {
                let ink = Color::hex(0x111111);
                Style {
                    id,
                    panel: Some(Color::hex(0xF7F7F5)),
                    border: None,
                    radius: 10.0,
                    ink,
                    ink2: Color::hex(0x5E5E5E),
                    ink3: Color::hex(0x9C9C9C),
                    rule: Color::hex(0xD9D9D6),
                    track: Color::hex(0xE4E4E0),
                    accent: if accent.luminance() > 0.7 { Color::hex(0xE63312) } else { accent },
                    halo: false,
                    marker: false,
                    lower_labels: false,
                    caps_display: false,
                    marker_full: false,
                    edge: 0.0,
                    pad: 18.0,
                    num: (SEGOE_DISPLAY, 700, -0.035),
                    label: (DIN, 600, 0.12),
                    text: (SEGOE_TEXT, 400),
                }
            }
            ThemeId::Noche => {
                let ink = Color::hex(0xF2F2EF);
                Style {
                    id,
                    panel: Some(Color::hex(0x121214)),
                    border: Some(Color::rgba(255, 255, 255, 0.06)),
                    radius: 20.0,
                    ink,
                    ink2: Color::hex(0x8E8E93),
                    ink3: Color::hex(0x55555B),
                    rule: Color::hex(0x2A2A2F),
                    track: Color::hex(0x26262B),
                    accent: if accent.luminance() < 0.12 { accent.mix(Color::hex(0xFFFFFF), 0.4) } else { accent },
                    halo: false,
                    marker: false,
                    lower_labels: false,
                    caps_display: false,
                    marker_full: false,
                    edge: 0.0,
                    pad: 18.0,
                    num: (SEGOE_DISPLAY, 250, -0.02),
                    label: (SEGOE_TEXT, 600, 0.11),
                    text: (SEGOE_TEXT, 400),
                }
            }
            ThemeId::Instrumento => Style {
                id,
                panel: Some(Color::hex(0xE6E4DF)),
                border: None,
                radius: 26.0,
                ink: Color::hex(0x232323),
                ink2: Color::hex(0x66635D),
                ink3: Color::hex(0x9E9A92),
                rule: Color::hex(0xCAC7BF),
                track: Color::hex(0xD3D0C8),
                accent: if accent.luminance() > 0.7 { Color::hex(0xFF6A00) } else { accent },
                halo: false,
                marker: false,
                lower_labels: false,
                caps_display: false,
                    marker_full: false,
                edge: 0.0,
                pad: 18.0,
                num: (DIN, 350, -0.01),
                label: (DIN, 600, 0.1),
                text: (DIN, 400),
            },
            ThemeId::Aire => {
                let w = Color::hex(0xFFFFFF);
                Style {
                    id,
                    panel: None,
                    border: None,
                    radius: 0.0,
                    ink: w,
                    ink2: w.a(0.82),
                    ink3: w.a(0.6),
                    rule: w.a(0.38),
                    track: w.a(0.26),
                    accent,
                    halo: true,
                    marker: false,
                    lower_labels: false,
                    caps_display: false,
                    marker_full: false,
                    edge: 0.0,
                    pad: 10.0,
                    num: (SEGOE_DISPLAY, 200, -0.02),
                    label: (SEGOE_TEXT, 600, 0.14),
                    text: (SEGOE_TEXT, 400),
                }
            }
            ThemeId::Grotesca => {
                let ink = Color::hex(0x1F201D);
                Style {
                    id,
                    panel: Some(Color::hex(0xFFFFFF)),
                    border: None,
                    radius: 0.0,
                    ink,
                    ink2: ink.a(0.68),
                    ink3: ink.a(0.45),
                    rule: ink,
                    track: Color::hex(0xECECE8),
                    accent,
                    halo: false,
                    marker: true,
                    lower_labels: true,
                    caps_display: true,
                    marker_full: false,
                    edge: 7.0,
                    pad: 18.0,
                    num: (GROT, 800, -0.045),
                    label: (GROT, 400, 0.0),
                    text: (GROT, 400),
                }
            }
            ThemeId::GrotescaNegra => {
                let ink = Color::hex(0xFFFFFF);
                Style {
                    id,
                    panel: Some(Color::hex(0x1F201D)),
                    border: None,
                    radius: 0.0,
                    ink,
                    ink2: ink.a(0.72),
                    ink3: ink.a(0.46),
                    rule: ink.a(0.85),
                    track: Color::hex(0x393A35),
                    accent,
                    halo: false,
                    marker: true,
                    lower_labels: true,
                    caps_display: true,
                    marker_full: true,
                    edge: 7.0,
                    pad: 18.0,
                    num: (GROT, 800, -0.045),
                    label: (GROT, 400, 0.0),
                    text: (GROT, 400),
                }
            }
        }
    }

    /// Cifras grandes.
    pub fn num(&self, size: f32) -> Font {
        Font::new(self.num.0, size, self.num.1).tracked(self.num.2)
    }
    /// Cifras grandes con más peso (para los Mosaicos de dato principal).
    pub fn num_bold(&self, size: f32) -> Font {
        Font::new(self.num.0, size, (self.num.1 + 200).min(800)).tracked(self.num.2)
    }
    /// Rótulos: versalitas espaciadas o, en Grotesca, minúscula fina algo mayor.
    pub fn label(&self, size: f32) -> Font {
        let size = if self.lower_labels { size * 1.22 } else { size };
        Font::new(self.label.0, size, self.label.1).tracked(self.label.2)
    }
    /// Color del acento cuando tiñe texto (en Grotesca, la tinta).
    pub fn accent_ink(&self) -> Color {
        if self.marker { self.ink } else { self.accent }
    }
    pub fn text(&self, size: f32) -> Font {
        Font::new(self.text.0, size, self.text.1)
    }
    pub fn strong(&self, size: f32) -> Font {
        Font::new(self.text.0, size, 600)
    }
    /// Color legible sobre el acento.
    pub fn on_accent(&self) -> Color {
        if self.accent.luminance() > 0.45 { Color::hex(0x111111) } else { Color::hex(0xFFFFFF) }
    }
}
