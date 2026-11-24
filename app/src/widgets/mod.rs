//! Catálogo de widgets, retícula de tamaños, refresco y piezas de dibujo compartidas.
//!
//! Todo se compone sobre una retícula de tres módulos (S 170×170, M 360×170, L 360×370) con un margen
//! interior común; los textos se colocan por línea base y las cifras son tabulares.

mod info;
mod sysw;
mod timew;

pub use info::{focus_state, note_lines};
pub use timew::world_cities as timew_cities;

use std::cell::RefCell;

use crate::config::WidgetCfg;
use crate::data::{media, Data};
use crate::gfx::{Align, Canvas, Color};
use crate::theme::{Style, ThemeId};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Weather,
    Disk,
    Calendar,
    Clock,
    World,
    System,
    Net,
    Battery,
    Media,
    Focus,
    Notes,
    Progress,
    Sky,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    S,
    M,
    L,
}

impl Size {
    pub fn dims(self) -> (f32, f32) {
        match self {
            Size::S => (170.0, 170.0),
            Size::M => (360.0, 170.0),
            Size::L => (360.0, 370.0),
        }
    }
}

pub struct Meta {
    pub kind: Kind,
    pub key: &'static str,
    pub name: &'static str,
    pub layouts: &'static [(&'static str, Size)],
}

use Size::*;

pub const KINDS: [Meta; 13] = [
    Meta { kind: Kind::Weather, key: "tiempo", name: "Tiempo", layouts: &[("Pequeño", S), ("Mediano", M), ("Grande", L)] },
    Meta { kind: Kind::Disk, key: "disco", name: "Disco", layouts: &[("Pequeño", S), ("Mediano", M)] },
    Meta { kind: Kind::Calendar, key: "calendario", name: "Calendario", layouts: &[("Pequeño", S), ("Mediano", M), ("Grande", L)] },
    Meta { kind: Kind::Clock, key: "reloj", name: "Reloj", layouts: &[("Digital", S), ("Analógico", S), ("En palabras", M)] },
    Meta { kind: Kind::World, key: "husos", name: "Husos horarios", layouts: &[("Mediano", M), ("Pequeño", S)] },
    Meta { kind: Kind::System, key: "sistema", name: "Sistema (CPU y RAM)", layouts: &[("Pequeño", S), ("Mediano", M)] },
    Meta { kind: Kind::Net, key: "red", name: "Red", layouts: &[("Pequeño", S), ("Mediano", M)] },
    Meta { kind: Kind::Battery, key: "bateria", name: "Batería", layouts: &[("Pequeño", S), ("Mediano", M)] },
    Meta { kind: Kind::Media, key: "musica", name: "Música", layouts: &[("Pequeño", S), ("Mediano", M)] },
    Meta { kind: Kind::Focus, key: "foco", name: "Foco (pomodoro)", layouts: &[("Pequeño", S), ("Mediano", M)] },
    Meta { kind: Kind::Notes, key: "notas", name: "Notas", layouts: &[("Nota", S), ("Lista", M)] },
    Meta { kind: Kind::Progress, key: "progreso", name: "Progreso", layouts: &[("Año", S), ("Barras", M), ("Cuenta atrás", S)] },
    Meta { kind: Kind::Sky, key: "cielo", name: "Cielo (sol y luna)", layouts: &[("Luna", S), ("Sol", M)] },
];

pub fn meta(key: &str) -> &'static Meta {
    KINDS.iter().find(|m| m.key == key).unwrap_or(&KINDS[3])
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Media(media::Cmd),
    Focus,
    NoteLine(usize),
    EditNote,
    SetCity,
    ConnectGoogle,
    SetCountdown,
}

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub action: Action,
}

#[allow(non_snake_case)]
pub struct Ctx<'a, 'b> {
    pub c: &'a Canvas<'b>,
    pub s: Style,
    pub d: &'a Data,
    pub w: &'a WidgetCfg,
    pub hits: RefCell<Vec<Hit>>,
    /// Ancho, alto y margen del módulo.
    pub W: f32,
    pub H: f32,
    pub p: f32,
}

impl<'a, 'b> Ctx<'a, 'b> {
    pub fn hit(&self, x: f32, y: f32, w: f32, h: f32, action: Action) {
        self.hits.borrow_mut().push(Hit { x, y, w, h, action });
    }
    pub fn layout(&self) -> u32 {
        self.w.layout.min(meta(&self.w.kind).layouts.len() as u32 - 1)
    }
    pub fn light(&self) -> bool {
        matches!(self.s.id, ThemeId::Suizo | ThemeId::Instrumento | ThemeId::Grotesca)
    }
    fn panel(&self) {
        if let Some(p) = self.s.panel {
            self.c.rect(0.0, 0.0, self.W, self.H, self.s.radius, p);
        }
        if self.s.edge > 0.0 {
            self.c.rect(self.W - self.s.edge, 0.0, self.s.edge, self.H, 0.0, self.s.accent);
        }
        if let Some(b) = self.s.border {
            self.c.rect_stroke(0.0, 0.0, self.W, self.H, self.s.radius, b, 1.0);
        }
    }

    /// Rótulo en versalitas espaciadas, por línea base.
    pub fn label(&self, t: &str, x: f32, base: f32, w: f32, col: Color, a: Align) -> f32 {
        let col = if self.s.marker && col == self.s.accent { self.s.ink } else { col };
        let t = if self.s.lower_labels { t.to_lowercase() } else { t.to_uppercase() };
        self.c.text_base(&t, x, base, w, self.s.label(10.5), col, a)
    }
    /// Cifras y palabras grandes. Con `marker`, el acento se pinta como subrayado fosforito detrás.
    pub fn num(&self, t: &str, x: f32, base: f32, w: f32, size: f32, col: Color, a: Align) -> f32 {
        let t = if self.s.caps_display { t.to_uppercase() } else { t.to_string() };
        let f = self.s.num(size);
        if self.s.marker && col == self.s.accent {
            let tw = self.c.measure(&t, f).0.min(w);
            let x0 = match a {
                Align::L => x,
                Align::C => x + (w - tw) / 2.0,
                Align::R => x + w - tw,
            };
            if self.s.marker_full {
                self.c.rect(x0 - size * 0.08, base - size * 0.84, tw + size * 0.16, size * 0.98, 0.0, self.s.accent);
                return self.c.text_base(&t, x, base, w, f, self.s.on_accent(), a);
            }
            self.c.rect(x0 - size * 0.04, base - size * 0.5, tw + size * 0.08, size * 0.42, 0.0, self.s.accent);
            return self.c.text_base(&t, x, base, w, f, self.s.ink, a);
        }
        self.c.text_base(&t, x, base, w, f, col, a)
    }
    pub fn text(&self, t: &str, x: f32, base: f32, w: f32, size: f32, col: Color, a: Align) -> f32 {
        self.c.text_base(t, x, base, w, self.s.text(size), col, a)
    }
    pub fn strong(&self, t: &str, x: f32, base: f32, w: f32, size: f32, col: Color, a: Align) -> f32 {
        self.c.text_base(t, x, base, w, self.s.strong(size), col, a)
    }
    /// Cifra grande con su unidad pequeña detrás (p. ej. «76» + «%»).
    pub fn figure(&self, n: &str, unit: &str, x: f32, base: f32, size: f32, col: Color) -> f32 {
        let w = self.num(n, x, base, 400.0, size, col, Align::L);
        if unit.is_empty() {
            return w;
        }
        let gap = size * 0.08;
        let u = self.c.text_base(unit, x + w + gap, base, 120.0, self.s.text((size * 0.28).max(12.0)), self.s.ink2, Align::L);
        w + gap + u
    }
    /// Filete horizontal de 1 px.
    pub fn rule(&self, x1: f32, y: f32, x2: f32) {
        self.c.rect(x1, y, x2 - x1, 1.0, 0.0, self.s.rule);
    }
    pub fn vrule(&self, x: f32, y1: f32, y2: f32) {
        self.c.rect(x, y1, 1.0, y2 - y1, 0.0, self.s.rule);
    }
    /// Barra fina con pista.
    pub fn bar(&self, x: f32, y: f32, w: f32, h: f32, frac: f32, col: Color) {
        self.c.rect(x, y, w, h, h / 2.0, self.s.track);
        let f = frac.clamp(0.0, 1.0);
        if f > 0.0 {
            self.c.rect(x, y, (w * f).max(h), h, h / 2.0, col);
        }
    }
    /// Barra segmentada: `n` piezas, llenas según `frac`.
    pub fn segs(&self, x: f32, y: f32, w: f32, h: f32, n: usize, frac: f32, col: Color) {
        let gap = (w / n as f32 * 0.22).clamp(2.0, 4.0);
        let sw = (w - gap * (n as f32 - 1.0)) / n as f32;
        let on = (frac.clamp(0.0, 1.0) * n as f32).round() as usize;
        let r = (h.min(sw) * 0.3).min(2.0);
        for i in 0..n {
            let c = if i < on { col } else { self.s.track };
            self.c.rect(x + i as f32 * (sw + gap), y, sw, h, r, c);
        }
    }
    /// Anillo de progreso fino.
    pub fn ring(&self, cx: f32, cy: f32, r: f32, frac: f32, col: Color, sw: f32) {
        self.c.ring(cx, cy, r, self.s.track, sw);
        let f = frac.clamp(0.0, 1.0);
        if f > 0.004 {
            self.c.arc(cx, cy, r, 0.0, 360.0 * f, col, sw);
        }
    }
    /// Línea de historial normalizada a `max`.
    pub fn spark(&self, x: f32, y: f32, w: f32, h: f32, vals: &[f32], max: f32, col: Color, sw: f32, fill: bool) {
        if vals.len() < 2 || max <= 0.0 {
            return;
        }
        let n = crate::data::HIST.max(vals.len());
        let step = w / (n - 1) as f32;
        let x0 = x + w - step * (vals.len() - 1) as f32;
        let pts: Vec<(f32, f32)> =
            vals.iter().enumerate().map(|(i, v)| (x0 + step * i as f32, y + h - h * (v / max).clamp(0.0, 1.0))).collect();
        if fill {
            let mut area = pts.clone();
            area.push((x + w, y + h));
            area.push((x0, y + h));
            self.c.path(&area, col.a(0.14), None, true);
        }
        self.c.path(&pts, col, Some(sw), false);
    }
    pub fn warn(&self) -> Color {
        if self.light() { Color::hex(0xD12B0F) } else { Color::hex(0xFF6B4A) }
    }
    /// Luna: lado iluminado y en sombra según la estética.
    pub fn moon(&self, cx: f32, cy: f32, r: f32, lit: f32, waxing: bool) {
        if self.light() {
            self.c.moon(cx, cy, r, lit, waxing, Color::hex(0xFFFFFF), self.s.ink);
            self.c.ring(cx, cy, r, self.s.ink, 1.2);
        } else {
            self.c.moon(cx, cy, r, lit, waxing, self.s.ink, self.s.track);
        }
    }
}

/// Tamaño en DIP del módulo según el diseño elegido.
pub fn size(w: &WidgetCfg, _d: &Data) -> (f32, f32) {
    let m = meta(&w.kind);
    let l = (w.layout as usize).min(m.layouts.len() - 1);
    m.layouts[l].1.dims()
}

/// Cada cuántos segundos puede cambiar lo que enseña (alineado al reloj).
pub fn period(w: &WidgetCfg) -> u32 {
    match meta(&w.kind).kind {
        Kind::Clock if w.b("segundos") => 1,
        Kind::Focus if w.i("end", 0) > 0 => 1,
        Kind::System | Kind::Net => 2,
        Kind::Media => 5,
        Kind::Disk | Kind::Battery => 30,
        _ => 60,
    }
}

pub fn draw(c: &Canvas, s: Style, w: &WidgetCfg, d: &Data) -> Vec<Hit> {
    let p = s.pad;
    let x = Ctx { c, s, d, w, hits: RefCell::new(Vec::new()), W: c.w, H: c.h, p };
    x.panel();
    match meta(&w.kind).kind {
        Kind::Clock => timew::clock(&x),
        Kind::World => timew::world(&x),
        Kind::Calendar => timew::calendar(&x),
        Kind::Progress => timew::progress(&x),
        Kind::Sky => timew::sky(&x),
        Kind::Disk => sysw::disk(&x),
        Kind::System => sysw::system(&x),
        Kind::Net => sysw::net(&x),
        Kind::Battery => sysw::battery(&x),
        Kind::Weather => info::weather(&x),
        Kind::Media => info::media(&x),
        Kind::Focus => info::focus(&x),
        Kind::Notes => info::notes(&x),
    }
    x.hits.into_inner()
}

/// Huella de lo que enseña el widget: si no cambia, no se repinta.
pub fn stamp(w: &WidgetCfg, d: &Data) -> String {
    let n = &d.now;
    match meta(&w.kind).kind {
        Kind::Clock if w.b("segundos") => format!("{}", n.unix),
        Kind::Clock | Kind::World | Kind::Progress => format!("{}{}{}{}", n.d, n.h, n.mi, n.offset),
        Kind::Calendar => format!("{}{}{:?}{:?}", n.d, n.h * 60 + n.mi, d.cal, d.events),
        Kind::Sky => format!("{}{}{}{}", n.d, n.h, n.mi / 5, d.place),
        Kind::Disk => format!("{:?}", d.disks.iter().map(|k| (k.letter, k.free / 200_000_000)).collect::<Vec<_>>()),
        Kind::System => format!("{:.0}{:.0}{}", d.cpu * 100.0, d.mem.0 as f64 / 1e8, d.cpu_hist.len() % 2),
        Kind::Net => format!("{:.0}{:.0}{}", d.net.0 / 1e4, d.net.1 / 1e4, d.net_hist.len()),
        Kind::Battery => format!("{:?}{}", d.battery, d.battery.remaining.unwrap_or(0) / 60),
        Kind::Weather => format!("{:?}{:?}{}{}", d.weather, d.weather_err, d.place, n.d),
        Kind::Media => match &d.media {
            Some(m) if m.playing && m.len > 0 => format!("{:?}{}", d.media, n.unix / 5),
            _ => format!("{:?}", d.media),
        },
        Kind::Focus if w.i("end", 0) > 0 => format!("{}", n.unix),
        Kind::Focus => format!("{}", n.days()),
        Kind::Notes => String::new(),
    }
}

// ---------- Iconos geométricos (sin fuentes de iconos) ----------

pub fn icon_play(c: &Canvas, cx: f32, cy: f32, r: f32, col: Color) {
    c.path(&[(cx - r * 0.38, cy - r * 0.55), (cx + r * 0.58, cy), (cx - r * 0.38, cy + r * 0.55)], col, None, true);
}

pub fn icon_pause(c: &Canvas, cx: f32, cy: f32, r: f32, col: Color) {
    c.rect(cx - r * 0.42, cy - r * 0.5, r * 0.28, r, r * 0.06, col);
    c.rect(cx + r * 0.14, cy - r * 0.5, r * 0.28, r, r * 0.06, col);
}

pub fn icon_skip(c: &Canvas, cx: f32, cy: f32, r: f32, col: Color, next: bool) {
    let d = if next { 1.0 } else { -1.0 };
    c.path(&[(cx - d * r * 0.45, cy - r * 0.45), (cx + d * r * 0.28, cy), (cx - d * r * 0.45, cy + r * 0.45)], col, None, true);
    c.rect(cx + d * r * 0.34 - r * 0.07, cy - r * 0.45, r * 0.14, r * 0.9, r * 0.04, col);
}

/// Icono del tiempo con formas puras: disco de sol en acento, nube maciza, lluvia en trazos finos.
pub fn icon_sky(x: &Ctx, sky: crate::data::weather::Sky, day: bool, cx: f32, cy: f32, size: f32) {
    use crate::data::weather::Sky::*;
    let (c, s) = (x.c, &x.s);
    let sun_col = if s.id == ThemeId::Aire && s.accent.luminance() > 0.8 { Color::hex(0xFFC94D) } else { s.accent };
    let cloud_col = match s.id {
        ThemeId::Suizo => s.ink,
        ThemeId::Instrumento => Color::hex(0x8F8B83),
        ThemeId::Noche => Color::hex(0xC9C9CE),
        ThemeId::Aire => Color::hex(0xFFFFFF),
        ThemeId::Grotesca | ThemeId::GrotescaNegra => s.ink,
    };
    let sw = (size * 0.045).max(1.3);
    let sun = |cx: f32, cy: f32, r: f32| {
        if day {
            c.circle(cx, cy, r, sun_col);
        } else {
            c.moon(cx, cy, r, 0.3, false, sun_col, Color::rgba(0, 0, 0, 0.0));
        }
    };
    let cloud = |cx: f32, cy: f32, k: f32| {
        let u = size * k;
        c.rect(cx - 0.44 * u, cy, 0.88 * u, 0.24 * u, 0.12 * u, cloud_col);
        c.circle(cx - 0.17 * u, cy + 0.02 * u, 0.2 * u, cloud_col);
        c.circle(cx + 0.09 * u, cy - 0.04 * u, 0.27 * u, cloud_col);
    };
    match sky {
        Clear => sun(cx, cy, size * 0.3),
        Partly => {
            sun(cx - size * 0.13, cy - size * 0.12, size * 0.22);
            cloud(cx + size * 0.08, cy + size * 0.06, 0.8);
        }
        Cloud => cloud(cx, cy, 0.95),
        Fog => {
            for (i, wf) in [0.8, 0.95, 0.6].iter().enumerate() {
                let y = cy - size * 0.18 + i as f32 * size * 0.18;
                c.rect(cx - size * wf / 2.0, y, size * wf, sw * 1.2, sw, if i == 1 { cloud_col } else { cloud_col.a(0.55) });
            }
        }
        Drizzle | Rain | Snow | Storm => {
            cloud(cx, cy - size * 0.16, 0.9);
            let base = cy + size * 0.2;
            for i in 0..3 {
                let px = cx - size * 0.2 + i as f32 * size * 0.2;
                match sky {
                    Snow => c.circle(px, base + (i % 2) as f32 * size * 0.07, size * 0.04, cloud_col),
                    Drizzle => c.circle(px, base + (i % 2) as f32 * size * 0.06, size * 0.028, cloud_col),
                    Storm if i == 1 => {
                        let u = size * 0.2;
                        c.path(
                            &[(px + u * 0.2, base - u * 0.4), (px - u * 0.3, base + u * 0.45), (px, base + u * 0.45), (px - u * 0.2, base + u * 1.1), (px + u * 0.4, base + u * 0.15), (px + u * 0.05, base + u * 0.15)],
                            sun_col,
                            None,
                            true,
                        );
                    }
                    _ => c.line(px + size * 0.04, base - size * 0.05, px - size * 0.04, base + size * 0.12, cloud_col, sw),
                }
            }
        }
    }
}
