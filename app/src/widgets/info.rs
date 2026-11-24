//! Tiempo, música, foco y notas.

use super::{icon_pause, icon_play, icon_skip, icon_sky, Action, Ctx};
use crate::data::media::Cmd;
use crate::data::weather::{describe, sky};
use crate::gfx::{Align, Color};
use crate::time::{self, capitalize, DIAS};

fn deg(t: f32) -> String {
    format!("{}°", t.round() as i32).replace('-', "−")
}

pub fn weather(x: &Ctx) {
    let (c, s, d) = (x.c, &x.s, x.d);
    let (w, h, p) = (x.W, x.H, x.p);
    x.hit(0.0, 0.0, w, h, Action::SetCity);
    let Some(wt) = &d.weather else {
        x.label(&d.place, p, p + 9.0, w, s.ink2, Align::L);
        x.num("—", p, p + 86.0, w, 64.0, s.ink3, Align::L);
        let msg = if d.weather_err.is_some() { "Sin conexión con Open-Meteo" } else { "Consultando el tiempo…" };
        x.text(msg, p, h - p, w - 2.0 * p, 12.0, s.ink3, Align::L);
        return;
    };
    let today = wt.days.first();
    let minmax = today.map(|t| format!("Máx. {}  ·  Mín. {}", deg(t.max), deg(t.min))).unwrap_or_default();
    match x.layout() {
        0 => {
            x.label(&d.place, p, p + 9.0, w - 2.0 * p - 40.0, s.ink2, Align::L);
            icon_sky(x, sky(wt.code), wt.is_day, w - p - 18.0, p + 18.0, 36.0);
            x.num(&deg(wt.temp), p - 3.0, p + 86.0, w, 64.0, s.ink, Align::L);
            x.text(describe(wt.code), p, h - p - 17.0, w - 2.0 * p, 12.5, s.ink, Align::L);
            x.text(&minmax, p, h - p, w - 2.0 * p, 11.5, s.ink2, Align::L);
        }
        1 => {
            x.label(&d.place, p, p + 9.0, 110.0, s.ink2, Align::L);
            icon_sky(x, sky(wt.code), wt.is_day, 138.0, p + 18.0, 32.0);
            x.num(&deg(wt.temp), p - 3.0, p + 86.0, 150.0, 64.0, s.ink, Align::L);
            x.text(describe(wt.code), p, h - p - 17.0, 136.0, 12.5, s.ink, Align::L);
            x.text(&minmax, p, h - p, 140.0, 11.5, s.ink2, Align::L);
            x.vrule(170.0, p, h - p);
            let days: Vec<_> = wt.days.iter().skip(1).take(5).collect();
            let x0 = 182.0;
            let cw = (w - p - x0) / days.len().max(1) as f32;
            for (i, dd) in days.iter().enumerate() {
                let cx = x0 + cw * i as f32 + cw / 2.0;
                let name: String = DIAS[time::weekday(dd.days) as usize].chars().take(3).collect();
                x.label(&name, cx - cw / 2.0, p + 9.0, cw, s.ink2, Align::C);
                icon_sky(x, sky(dd.code), true, cx, p + 48.0, 24.0);
                x.c.text_base(&deg(dd.max), cx - cw / 2.0, p + 92.0, cw, s.strong(13.0), s.ink, Align::C);
                x.c.text_base(&deg(dd.min), cx - cw / 2.0, p + 110.0, cw, s.text(13.0), s.ink3, Align::C);
                x.c.text_base(&format!("{} %", dd.rain), cx - cw / 2.0, h - p, cw, s.text(10.5), if dd.rain >= 50 { s.accent_ink() } else { s.ink3 }, Align::C);
            }
        }
        _ => {
            x.label(&d.place, p, p + 9.0, w * 0.6, s.ink2, Align::L);
            let dname = capitalize(DIAS[d.now.wd as usize]);
            x.label(&format!("{} {}", dname, d.now.d), p, p + 9.0, w - 2.0 * p, s.ink3, Align::R);
            x.num(&deg(wt.temp), p - 4.0, p + 108.0, 220.0, 92.0, s.ink, Align::L);
            icon_sky(x, sky(wt.code), wt.is_day, w - p - 46.0, p + 66.0, 76.0);
            x.strong(describe(wt.code), p, p + 136.0, w - 2.0 * p, 14.0, s.ink, Align::L);
            x.text(
                &format!("{}  ·  Sensación {}  ·  Humedad {} %", minmax, deg(wt.feels), wt.humidity),
                p,
                p + 156.0,
                w - 2.0 * p,
                11.5,
                s.ink2,
                Align::L,
            );
            let ry = p + 176.0;
            x.rule(p, ry, w - p);
            let days: Vec<_> = wt.days.iter().take(6).collect();
            let lo = days.iter().map(|d| d.min).fold(f32::MAX, f32::min);
            let hi = days.iter().map(|d| d.max).fold(f32::MIN, f32::max);
            let span = (hi - lo).max(1.0);
            let (bx, bw) = (196.0, w - p - 196.0 - 40.0);
            for (i, dd) in days.iter().enumerate() {
                let b = ry + 28.0 + i as f32 * 27.0;
                let name = if i == 0 { "Hoy".to_string() } else { capitalize(DIAS[time::weekday(dd.days) as usize]) };
                x.text(&name, p, b, 92.0, 13.5, s.ink, Align::L);
                icon_sky(x, sky(dd.code), true, p + 106.0, b - 5.0, 18.0);
                x.c.text_base(&deg(dd.min), p + 120.0, b, bx - p - 128.0, s.text(13.5), s.ink3, Align::R);
                c.rect(bx, b - 6.0, bw, 3.0, 1.5, s.track);
                let (a, z) = ((dd.min - lo) / span, (dd.max - lo) / span);
                c.rect(bx + bw * a, b - 6.0, (bw * (z - a)).max(3.0), 3.0, 1.5, if i == 0 { s.accent } else { s.ink2 });
                x.c.text_base(&deg(dd.max), bx + bw, b, w - p - bx - bw, s.strong(13.5), s.ink, Align::R);
            }
        }
    }
}

fn mmss(sec: u32) -> String {
    format!("{}:{:02}", sec / 60, sec % 60)
}

pub fn media(x: &Ctx) {
    let (c, s, d) = (x.c, &x.s, x.d);
    let (w, h, p) = (x.W, x.H, x.p);
    let Some(m) = &d.media else {
        x.label("Música", p, p + 9.0, w, s.ink2, Align::L);
        x.strong("No suena nada", p, p + 56.0, w - 2.0 * p, 15.0, s.ink, Align::L);
        c.text_wrap(
            "Pon algo en Spotify, el navegador o cualquier reproductor.",
            p,
            p + 66.0,
            if x.layout() == 0 { w - 2.0 * p } else { 260.0 },
            60.0,
            s.text(12.0),
            s.ink3,
            Align::L,
        );
        return;
    };
    let ink_on = s.panel.filter(|c| c.a > 0.5).unwrap_or(Color::hex(0x111111));
    let play = |cx: f32, cy: f32, r: f32| {
        c.circle(cx, cy, r, s.ink);
        if m.playing { icon_pause(c, cx, cy, r * 0.7, ink_on) } else { icon_play(c, cx + r * 0.06, cy, r * 0.7, ink_on) }
        x.hit(cx - r - 4.0, cy - r - 4.0, 2.0 * r + 8.0, 2.0 * r + 8.0, Action::Media(Cmd::Toggle));
    };
    let skip = |cx: f32, cy: f32, next: bool| {
        icon_skip(c, cx, cy, 10.0, s.ink2, next);
        x.hit(cx - 16.0, cy - 16.0, 32.0, 32.0, Action::Media(if next { Cmd::Next } else { Cmd::Prev }));
    };
    let eq = |rx: f32, base: f32| {
        if m.playing {
            for (i, hh) in [7.0, 11.0, 5.0].iter().enumerate() {
                c.rect(rx - 14.0 + i as f32 * 5.0, base - hh, 3.0, *hh, 1.0, s.accent);
            }
        }
    };
    let app = if m.app.is_empty() { "Música".to_string() } else { m.app.clone() };
    if x.layout() == 0 {
        x.label(&app, p, p + 9.0, w - 2.0 * p - 20.0, s.ink2, Align::L);
        eq(w - p + 1.0, p + 10.0);
        c.text_wrap(&m.title, p, p + 22.0, w - 2.0 * p, 44.0, s.strong(15.0), s.ink, Align::L);
        let tl = c.measure(&m.title, s.strong(15.0)).0 > w - 2.0 * p;
        x.text(&m.artist, p, p + if tl { 80.0 } else { 60.0 }, w - 2.0 * p, 12.5, s.ink2, Align::L);
        play(p + 16.0, h - p - 16.0, 16.0);
        skip(p + 54.0, h - p - 16.0, true);
    } else {
        let side = h - 2.0 * p;
        // Portada gráfica: bloque de acento con la inicial del título.
        c.rect(p, p, side, side, if x.light() { 4.0 } else { 10.0 }, s.accent);
        let ini: String = m.title.chars().find(|ch| ch.is_alphanumeric()).map(|ch| ch.to_uppercase().collect()).unwrap_or_default();
        x.c.text_base(&ini, p + 12.0, p + side - 14.0, side, s.num_bold(72.0), s.on_accent(), Align::L);
        let x0 = p + side + 18.0;
        let iw = w - p - x0;
        x.label(&app, x0, p + 9.0, iw - 20.0, s.ink2, Align::L);
        eq(w - p + 1.0, p + 10.0);
        c.text_wrap(&m.title, x0, p + 22.0, iw, 44.0, s.strong(16.0), s.ink, Align::L);
        x.text(&m.artist, x0, p + 86.0, iw, 12.5, s.ink2, Align::L);
        let cy = h - p - 15.0;
        skip(w - p - 86.0, cy, false);
        play(w - p - 48.0, cy, 15.0);
        skip(w - p - 10.0, cy, true);
        if m.len > 0 {
            let bw = iw - 112.0;
            x.bar(x0, cy - 1.0, bw, 2.0, m.pos as f32 / m.len as f32, s.ink);
            x.label(&format!("{} / {}", mmss(m.pos), mmss(m.len)), x0, p + 104.0, iw, s.ink3, Align::L);
        }
    }
}

/// Estado del pomodoro guardado en las opciones del widget.
pub struct Focus {
    pub phase_break: bool,
    pub total: u32,
    pub left: u32,
    pub running: bool,
    pub done: i64,
}

pub fn focus_state(w: &crate::config::WidgetCfg, now: &time::Now) -> Focus {
    let phase_break = w.s("fase") == "descanso";
    let total = (if phase_break { w.i("descanso", 5) } else { w.i("minutos", 25) }).max(1) as u32 * 60;
    let end = w.i("end", 0);
    let paused = w.i("left", 0);
    let (running, left) = if end > 0 { (true, (end - now.unix).max(0) as u32) } else if paused > 0 { (false, paused as u32) } else { (false, total) };
    let done = if w.i("dia", 0) == now.days() { w.i("hechos", 0) } else { 0 };
    Focus { phase_break, total, left, running, done }
}

pub fn focus(x: &Ctx) {
    let (s, d) = (&x.s, x.d);
    let (w, h, p) = (x.W, x.H, x.p);
    let f = focus_state(x.w, &d.now);
    x.hit(0.0, 0.0, w, h, Action::Focus);
    let remaining = f.left as f32 / f.total as f32;
    let col = if f.phase_break { s.ink2 } else { s.accent };
    let status = match (f.phase_break, f.running, f.left == f.total) {
        (false, true, _) => "En marcha · clic para pausar",
        (true, true, _) => "Descansando",
        (_, false, false) => "En pausa · clic para seguir",
        (false, false, true) => "Clic para empezar",
        (true, false, true) => "Clic para descansar",
    };
    let phase = if f.phase_break { "Descanso" } else { "Foco" };
    let done = format!("{} hoy", f.done);
    if x.layout() == 0 {
        x.label(phase, p, p + 9.0, w, s.ink2, Align::L);
        x.label(&done, p, p + 9.0, w - 2.0 * p, s.ink3, Align::R);
        let (cx, cy, r) = (w / 2.0, p + 70.0, 40.0);
        x.ring(cx, cy, r, remaining, col, 3.0);
        x.num(&mmss(f.left), cx - r, cy + 10.0, 2.0 * r, 26.0, s.ink, Align::C);
        x.text(status.split(" · ").next().unwrap_or(status), p, h - p, w - 2.0 * p, 11.5, s.ink2, Align::C);
    } else {
        x.label(phase, p, p + 9.0, w, s.ink2, Align::L);
        x.label(&done, p, p + 9.0, w - 2.0 * p, s.ink3, Align::R);
        x.num(&mmss(f.left), p - 4.0, p + 88.0, w, 76.0, s.ink, Align::L);
        x.text(status, p, p + 112.0, w - 2.0 * p, 12.5, s.ink2, Align::L);
        let n = (f.total / 60).clamp(5, 50) as usize;
        x.segs(p, h - p - 10.0, w - 2.0 * p, 10.0, n, 1.0 - remaining, col);
    }
}

pub fn note_lines(w: &crate::config::WidgetCfg) -> Vec<String> {
    w.s("texto").lines().map(|l| l.to_string()).collect()
}

pub fn notes(x: &Ctx) {
    let (c, s) = (x.c, &x.s);
    let (w, h, p) = (x.W, x.H, x.p);
    let lines = note_lines(x.w);
    let title = x.w.s("titulo");
    let title = if title.is_empty() { if x.layout() == 0 { "Nota".to_string() } else { "Pendientes".to_string() } } else { title };
    x.label(&title, p, p + 9.0, w - 2.0 * p - 40.0, s.accent, Align::L);
    if lines.iter().all(|l| l.trim().is_empty()) {
        c.text_wrap("Doble clic para escribir", p, p + 24.0, w - 2.0 * p, 40.0, s.text(13.5), s.ink3, Align::L);
        return;
    }
    if x.layout() == 0 {
        c.text_wrap(&lines.join("\n"), p, p + 22.0, w - 2.0 * p, h - 2.0 * p - 22.0, s.text(13.5), s.ink, Align::L);
        return;
    }
    let items: Vec<(usize, &String)> = lines.iter().enumerate().filter(|(_, l)| !l.trim().is_empty()).collect();
    let done_n = items.iter().filter(|(_, l)| l.starts_with("[x]") || l.starts_with("[X]")).count();
    x.label(&format!("{}/{}", done_n, items.len()), p, p + 9.0, w - 2.0 * p, s.ink3, Align::R);
    for (row, (i, l)) in items.iter().enumerate().take(4) {
        let b = p + 42.0 + row as f32 * 28.0;
        let done = l.starts_with("[x]") || l.starts_with("[X]");
        let label = l.trim_start_matches("[x]").trim_start_matches("[X]").trim_start_matches("[ ]").trim_start_matches("- ").trim();
        let (cx, cy) = (p + 7.0, b - 4.5);
        if done {
            c.circle(cx, cy, 7.5, s.accent);
            c.path(&[(cx - 3.2, cy + 0.2), (cx - 0.8, cy + 2.6), (cx + 3.4, cy - 2.4)], s.on_accent(), Some(1.6), false);
        } else {
            c.ring(cx, cy, 7.0, s.ink3, 1.3);
        }
        let tw = x.text(label, p + 24.0, b, w - 2.0 * p - 24.0, 13.5, if done { s.ink3 } else { s.ink }, Align::L);
        if done {
            c.rect(p + 24.0, b - 4.5, tw, 1.0, 0.0, s.ink3);
        }
        x.hit(0.0, b - 20.0, w, 28.0, Action::NoteLine(*i));
    }
    if items.len() > 4 {
        x.label(&format!("+{} más", items.len() - 4), p, h - p + 4.0, w - 2.0 * p, s.ink3, Align::R);
    }
}
