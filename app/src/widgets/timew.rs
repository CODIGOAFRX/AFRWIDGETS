//! Reloj, husos horarios, calendario, progreso y cielo.

use std::f32::consts::PI;

use super::{Action, Ctx};
use crate::config::WidgetCfg;
use crate::data::{astro, tz, CalState};
use crate::gfx::{Align, Color, Font};
use crate::theme::ThemeId;
use crate::time::{self, capitalize, hhmm, Now, DIAS, DIAS_CORTOS, MESES};

// ---------- Reloj ----------

const HORAS: [&str; 12] = ["doce", "una", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez", "once"];

/// «Son las | diez | y cuarto | de la mañana», redondeado a cinco minutos.
pub fn words(h: u32, m: u32) -> (String, String, String, String) {
    let mut m5 = (m + 2) / 5 * 5;
    let mut h24 = h;
    if m5 == 60 {
        m5 = 0;
        h24 = (h24 + 1) % 24;
    }
    let (hour24, tail) = if m5 > 30 {
        let t = match m5 {
            35 => "menos veinticinco",
            40 => "menos veinte",
            45 => "menos cuarto",
            50 => "menos diez",
            _ => "menos cinco",
        };
        ((h24 + 1) % 24, t)
    } else {
        let t = match m5 {
            0 => "en punto",
            5 => "y cinco",
            10 => "y diez",
            15 => "y cuarto",
            20 => "y veinte",
            25 => "y veinticinco",
            _ => "y media",
        };
        (h24, t)
    };
    let h12 = (hour24 % 12) as usize;
    let lead = if h12 == 1 { "Es la" } else { "Son las" };
    let part = match hour24 {
        12 => "del mediodía",
        1..=5 => "de la madrugada",
        6..=11 => "de la mañana",
        13..=20 => "de la tarde",
        _ => "de la noche",
    };
    (lead.into(), HORAS[h12].into(), tail.into(), part.into())
}

/// Tamaño de letra que cabe en `avail` partiendo de `size`.
fn fit(x: &Ctx, t: &str, size: f32, avail: f32, min: f32) -> f32 {
    let w = x.c.measure(t, x.s.num(size)).0;
    if w > avail { (size * avail / w * 0.97).max(min) } else { size }
}

pub fn clock(x: &Ctx) {
    let (c, s, n) = (x.c, &x.s, &x.d.now);
    let (w, h, p) = (x.W, x.H, x.p);
    match x.layout() {
        // Pantalla de bloqueo: la fecha arriba y la hora entera en una línea, centradas, en seminegrita.
        0 => {
            let avail = w - 2.0 * p;
            let date = format!("{}, {} de {}", capitalize(DIAS[n.wd as usize]), n.d, MESES[(n.mo - 1) as usize]);
            x.strong(&date, p, p + 14.0, avail, 14.0, s.ink2, Align::C);
            let t = if x.w.b("segundos") { format!("{}:{:02}", hhmm(n.h, n.mi), n.s) } else { hhmm(n.h, n.mi) };
            let probe = if t.len() > 5 { "00:00:00" } else { "00:00" };
            // Inter Display seminegrita, la más cercana a la SF del iPhone.
            let f = |size: f32| Font::new(crate::gfx::RELOJ, size, 400).tracked(-0.02);
            let size = (100.0 * avail * 0.86 / c.measure(probe, f(100.0)).0).min(104.0);
            c.text_base(&t, p, h - p - 6.0, avail, f(size), s.ink, Align::C);
        }
        // Alargado: Big Shoulders, cifras altas y estrechas de verdad que llenan el widget, con el día encima.
        4 => {
            let avail = w - 2.0 * p;
            let dia = DIAS[n.wd as usize];
            let corto: String = dia.chars().take(3).collect();
            x.strong(&format!("{} {}", capitalize(&corto), n.d), p, p + 12.0, avail, 14.0, s.ink2, Align::C);
            let fam = if s.num(10.0).weight >= 600 { crate::gfx::ALTO_GRUESO } else { crate::gfx::ALTO_FINO };
            let f = |size: f32| Font::new(fam, size, 400);
            // Las cifras llenan el alto que queda bajo la fecha (altura de las cifras de Big Shoulders: 0,8 del cuerpo).
            let (top, bottom) = (p + 26.0, h - p + 2.0);
            let size = (100.0 * avail * 0.96 / c.measure("00:00", f(100.0)).0).min((bottom - top) / 0.8);
            c.text_base(&hhmm(n.h, n.mi), p, bottom, avail, f(size), s.ink, Align::C);
        }
        3 => {
            x.label(&format!("{} {}", DIAS[n.wd as usize], n.d), p, p + 9.0, w - 2.0 * p, s.ink2, Align::L);
            if x.w.b("segundos") {
                x.label(&format!("{:02}", n.s), p, p + 9.0, w - 2.0 * p, s.accent, Align::R);
            }
            x.num(&format!("{:02}", n.h), p - 3.0, p + 80.0, w, 64.0, s.ink, Align::L);
            x.num(&format!("{:02}", n.mi), p - 3.0, h - p, w, 64.0, s.accent, Align::L);
        }
        1 => {
            let (cx, cy) = (w / 2.0, h / 2.0);
            let r = w / 2.0 - p + 4.0;
            for i in 0..60 {
                let a = i as f32 * PI / 30.0;
                let (len, sw, col) = if i % 5 == 0 { (9.0, 2.0, s.ink) } else { (4.0, 1.0, s.ink3) };
                c.line(cx + a.sin() * (r - len), cy - a.cos() * (r - len), cx + a.sin() * r, cy - a.cos() * r, col, sw);
            }
            if s.id == ThemeId::Instrumento {
                for (i, t) in ["12", "3", "6", "9"].iter().enumerate() {
                    let a = i as f32 * PI / 2.0;
                    let (tx, ty) = (cx + a.sin() * (r - 22.0), cy - a.cos() * (r - 22.0));
                    x.c.text_base(t, tx - 20.0, ty + 4.5, 40.0, s.label(12.0).tracked(0.0), s.ink2, Align::C);
                }
            }
            let ha = ((n.h % 12) as f32 + n.mi as f32 / 60.0) * PI / 6.0;
            let ma = n.mi as f32 * PI / 30.0;
            c.line(cx, cy, cx + ha.sin() * r * 0.5, cy - ha.cos() * r * 0.5, s.ink, 4.5);
            c.line(cx, cy, cx + ma.sin() * r * 0.8, cy - ma.cos() * r * 0.8, s.ink, 2.6);
            if x.w.b("segundos") {
                let sa = n.s as f32 * PI / 30.0;
                c.line(cx - sa.sin() * 14.0, cy + sa.cos() * 14.0, cx + sa.sin() * r * 0.9, cy - sa.cos() * r * 0.9, s.accent, 1.2);
            }
            c.circle(cx, cy, 4.0, s.accent);
        }
        _ => {
            let (lead, hour, tail, part) = words(n.h, n.mi);
            x.label(&lead, p, p + 9.0, w, s.ink2, Align::L);
            let avail = w - 2.0 * p;
            let size = fit(x, &tail, 42.0, avail, 26.0).min(fit(x, &hour, 42.0, avail, 26.0));
            x.num(&hour, p - 2.0, p + 64.0, avail + 4.0, size, s.ink, Align::L);
            x.num(&tail, p - 2.0, p + 108.0, avail + 4.0, size, s.accent, Align::L);
            x.label(&part, p, h - p, w, s.ink3, Align::L);
            x.label(&hhmm(n.h, n.mi), p, h - p, avail, s.ink3, Align::R);
        }
    }
}

// ---------- Husos horarios ----------

pub fn world_cities(w: &WidgetCfg) -> Vec<&'static str> {
    let list = w.s("ciudades");
    let keys: Vec<&str> = if list.is_empty() { vec!["madrid", "nuevayork", "tokio"] } else { list.split(',').collect() };
    keys.into_iter().filter_map(|k| tz::CITIES.iter().find(|c| c.0 == k).map(|c| c.0)).take(6).collect()
}

fn diff(t: &Now, n: &Now) -> String {
    let d = (t.offset - n.offset) as f32 / 60.0;
    let day = match t.days() - n.days() {
        1 => " · mañana",
        -1 => " · ayer",
        _ => "",
    };
    if d.abs() < 0.01 {
        format!("Aquí{day}")
    } else {
        let v = if d.fract().abs() > 0.01 { format!("{:+.1}", d).replace('.', ",") } else { format!("{:+}", d as i32) };
        format!("{} h{day}", v.replace('-', "−"))
    }
}

pub fn world(x: &Ctx) {
    let (s, n) = (&x.s, &x.d.now);
    let (w, h, p) = (x.W, x.H, x.p);
    let rows: Vec<(&str, Now)> = world_cities(x.w)
        .iter()
        .filter_map(|k| {
            let (name, zone) = tz::city(k)?;
            Some((name, Now::from_unix(n.unix, tz::offset(zone).unwrap_or(n.offset))))
        })
        .collect();
    if x.layout() == 0 {
        let rows: Vec<_> = rows.into_iter().take(4).collect();
        let k = rows.len().max(1) as f32;
        let cw = (w - 2.0 * p) / k;
        for (i, (name, t)) in rows.iter().enumerate() {
            let x0 = p + i as f32 * cw + if i > 0 { 14.0 } else { 0.0 };
            let iw = cw - if i > 0 { 14.0 } else { 0.0 } - 10.0;
            if i > 0 {
                x.vrule(p + i as f32 * cw, p, h - p);
            }
            x.label(name, x0, p + 9.0, iw, s.ink2, Align::L);
            let size = fit(x, "00:00", 34.0, iw, 18.0);
            x.num(&hhmm(t.h, t.mi), x0 - 1.0, p + 72.0, iw + 4.0, size, s.ink, Align::L);
            x.text(&diff(t, n), x0, p + 94.0, iw, 12.0, s.ink3, Align::L);
            // Línea de 24 horas con la posición de esa ciudad.
            let ly = h - p - 4.0;
            x.c.rect(x0, ly, iw, 1.0, 0.0, s.rule);
            let f = (t.h as f32 + t.mi as f32 / 60.0) / 24.0;
            x.c.circle(x0 + iw * f, ly + 0.5, 3.5, if (7..20).contains(&t.h) { s.accent } else { s.ink3 });
        }
    } else {
        for (i, (name, t)) in rows.iter().take(3).enumerate() {
            let base = p + 38.0 + i as f32 * 48.0;
            if i > 0 {
                x.rule(p, base - 32.0, w - p);
            }
            x.label(name, p, base - 14.0, w - 2.0 * p - 72.0, s.ink2, Align::L);
            x.text(&diff(t, n), p, base, w - 2.0 * p - 72.0, 11.5, s.ink3, Align::L);
            x.num(&hhmm(t.h, t.mi), p, base, w - 2.0 * p, 26.0, s.ink, Align::R);
        }
    }
}

// ---------- Calendario ----------

fn cal_status(x: &Ctx) -> Option<&'static str> {
    match &x.d.cal {
        CalState::NoCreds => Some("Configura Google para ver tus eventos"),
        CalState::Disconnected => Some("Conectar Google Calendar"),
        CalState::Connecting => Some("Esperando al navegador…"),
        CalState::Error(e) if e == "REAUTH" => Some("Vuelve a conectar Google"),
        CalState::Error(_) => Some("Sin conexión con Google"),
        CalState::Ready => None,
    }
}

fn when_label(e: &crate::data::google::Event, today: i64, n: &Now) -> String {
    let day = (e.start + n.offset as i64 * 60).div_euclid(86400);
    let st = Now::from_unix(e.start, n.offset);
    let t = if e.all_day { "todo el día".to_string() } else { hhmm(st.h, st.mi) };
    match day - today {
        d if d <= 0 => format!("Hoy · {t}"),
        1 => format!("Mañana · {t}"),
        _ => {
            let (_, _, dd) = time::civil_from_days(day);
            let wd: String = DIAS[time::weekday(day) as usize].chars().take(3).collect();
            format!("{wd} {dd} · {t}")
        }
    }
}

pub fn calendar(x: &Ctx) {
    let (c, s, d) = (x.c, &x.s, x.d);
    let n = &d.now;
    let (w, h, p) = (x.W, x.H, x.p);
    let today = n.days();
    let ev_day = |e: &crate::data::google::Event| (e.start + n.offset as i64 * 60).div_euclid(86400);
    let status = cal_status(x);
    let upcoming: Vec<_> = d.events.iter().filter(|e| e.end > n.unix).collect();
    let date_block = |x0: f32, bw: f32| {
        x.label(DIAS[n.wd as usize], x0, p + 9.0, bw, s.accent, Align::L);
        x.num(&format!("{}", n.d), x0 - 4.0, p + 88.0, bw + 6.0, 76.0, s.ink, Align::L);
        x.label(&format!("{} {}", MESES[n.mo as usize - 1], n.y), x0, p + 108.0, bw, s.ink3, Align::L);
    };
    match x.layout() {
        0 => {
            date_block(p, w - 2.0 * p);
            match (upcoming.first(), status) {
                (_, Some(st)) => {
                    x.text(st, p, h - p, w - 2.0 * p, 11.5, s.ink3, Align::L);
                    x.hit(0.0, h - p - 20.0, w, p + 20.0, Action::ConnectGoogle);
                }
                (Some(e), None) => {
                    c.circle(p + 3.0, h - p - 4.0, 3.0, s.accent);
                    let st = Now::from_unix(e.start, n.offset);
                    let tw = if e.all_day { 0.0 } else { x.strong(&hhmm(st.h, st.mi), p + 12.0, h - p, 60.0, 12.0, s.ink, Align::L) + 6.0 };
                    x.text(&e.title, p + 12.0 + tw, h - p, w - 2.0 * p - 12.0 - tw, 12.0, s.ink2, Align::L);
                }
                (None, None) => {
                    x.text("Sin eventos próximos", p, h - p, w - 2.0 * p, 12.0, s.ink3, Align::L);
                }
            }
        }
        1 => {
            date_block(p, 136.0);
            let x0 = 176.0;
            x.vrule(160.0, p, h - p);
            let iw = w - p - x0;
            if let Some(st) = status {
                c.text_wrap(st, x0, h / 2.0 - 16.0, iw, 40.0, s.text(13.0), s.ink2, Align::L);
                x.hit(150.0, 0.0, w - 150.0, h, Action::ConnectGoogle);
            } else if upcoming.is_empty() {
                x.text("Nada en el calendario", x0, h / 2.0 + 5.0, iw, 13.0, s.ink3, Align::L);
            }
            if status.is_none() {
                for (i, e) in upcoming.iter().take(3).enumerate() {
                    let b = p + 22.0 + i as f32 * 46.0;
                    x.label(&when_label(e, today, n), x0, b, iw, if ev_day(e) <= today { s.accent } else { s.ink3 }, Align::L);
                    x.strong(&e.title, x0, b + 19.0, iw, 14.0, s.ink, Align::L);
                }
            }
        }
        _ => {
            x.label(MESES[n.mo as usize - 1], p, p + 9.0, w, s.ink, Align::L);
            x.label(&format!("{}", n.y), p, p + 9.0, w - 2.0 * p, s.ink3, Align::R);
            let cw = (w - 2.0 * p) / 7.0;
            for (i, l) in DIAS_CORTOS.iter().enumerate() {
                x.label(l, p + cw * i as f32, p + 40.0, cw, s.ink3, Align::C);
            }
            let first = time::days_from_civil(n.y, n.mo, 1);
            let lead = time::weekday(first) as i64;
            let dim = time::days_in_month(n.y, n.mo) as i64;
            for k in 0..dim {
                let cell = lead + k;
                let cx = p + cw * (cell % 7) as f32 + cw / 2.0;
                let base = p + 72.0 + 30.0 * (cell / 7) as f32;
                let day = first + k;
                let label = format!("{}", k + 1);
                if day == today {
                    c.circle(cx, base - 5.0, 13.0, s.accent);
                    x.c.text_base(&label, cx - cw / 2.0, base, cw, s.strong(14.0), s.on_accent(), Align::C);
                } else {
                    let col = if day < today { s.ink3 } else { s.ink };
                    x.c.text_base(&label, cx - cw / 2.0, base, cw, s.text(14.0), col, Align::C);
                }
                if d.events.iter().any(|e| ev_day(e) == day) {
                    c.circle(cx, base + 10.0, 1.8, if day < today { s.ink3 } else { s.ink2 });
                }
            }
            let ry = p + 250.0;
            x.rule(p, ry, w - p);
            if let Some(st) = status {
                x.text(st, p, ry + 30.0, w - 2.0 * p, 13.0, s.ink2, Align::L);
                x.hit(0.0, ry, w, h - ry, Action::ConnectGoogle);
            } else if upcoming.is_empty() {
                x.text("Nada en el calendario", p, ry + 30.0, w - 2.0 * p, 13.0, s.ink3, Align::L);
            }
            if status.is_none() {
                for (i, e) in upcoming.iter().take(3).enumerate() {
                    let b = ry + 26.0 + i as f32 * 28.0;
                    x.label(&when_label(e, today, n), p, b, 120.0, if ev_day(e) <= today { s.accent } else { s.ink3 }, Align::L);
                    x.text(&e.title, p + 128.0, b, w - 2.0 * p - 128.0, 13.5, s.ink, Align::L);
                }
            }
        }
    }
}

// ---------- Progreso ----------

pub fn progress(x: &Ctx) {
    let (c, s, n) = (x.c, &x.s, &x.d.now);
    let (w, h, p) = (x.W, x.H, x.p);
    let secs = (n.h * 3600 + n.mi * 60 + n.s) as f32;
    let doy = (n.days() - time::days_from_civil(n.y, 1, 1)) as f32;
    let ylen = (time::days_from_civil(n.y + 1, 1, 1) - time::days_from_civil(n.y, 1, 1)) as f32;
    let year = (doy + secs / 86400.0) / ylen;
    let month = ((n.d - 1) as f32 + secs / 86400.0) / time::days_in_month(n.y, n.mo) as f32;
    let week = (n.wd as f32 + secs / 86400.0) / 7.0;
    let day = secs / 86400.0;
    let pct = |f: f32| format!("{}", (f * 100.0).floor() as i32);
    match x.layout() {
        0 => {
            x.label(&format!("{}", n.y), p, p + 9.0, w, s.ink2, Align::L);
            x.figure(&pct(year), "%", p - 3.0, p + 86.0, 64.0, s.ink);
            // Doce meses: los pasados en tinta, el actual en acento.
            let (y0, bh) = (h - p - 34.0, 12.0);
            let gap = 3.0;
            let sw = (w - 2.0 * p - gap * 11.0) / 12.0;
            for m in 0..12u32 {
                let xx = p + m as f32 * (sw + gap);
                let col = if m + 1 < n.mo { s.ink } else if m + 1 == n.mo { s.accent } else { s.track };
                c.rect(xx, y0, sw, bh, 2.0, col);
            }
            x.text(&format!("Quedan {} días", (ylen - doy - 1.0) as i32), p, h - p, w - 2.0 * p, 12.0, s.ink2, Align::L);
        }
        1 => {
            let mes = capitalize(MESES[n.mo as usize - 1]);
            let rows = [("Hoy", day), ("Semana", week), (mes.as_str(), month), ("Año", year)];
            for (i, (label, f)) in rows.iter().enumerate() {
                let b = p + 9.0 + i as f32 * 36.0;
                x.label(label, p, b, w, s.ink2, Align::L);
                x.c.text_base(&format!("{} %", pct(*f)), p, b + 1.0, w - 2.0 * p, s.strong(13.0), s.ink, Align::R);
                x.bar(p, b + 9.0, w - 2.0 * p, 3.0, *f, if i == 3 { s.accent } else { s.ink });
            }
        }
        _ => {
            let label = x.w.s("etiqueta");
            let parsed = time::parse_rfc3339(&x.w.s("fecha"), n.offset).map(|(t, _)| (t + n.offset as i64 * 60).div_euclid(86400));
            match parsed {
                Some(td) => {
                    let left = td - n.days();
                    x.label("Cuenta atrás", p, p + 9.0, w, s.ink2, Align::L);
                    let (num, unit) = match left {
                        0 => ("Hoy".to_string(), String::new()),
                        1 => ("1".into(), "día".into()),
                        l if l < 0 => (format!("{}", -l), "días desde".into()),
                        l => (format!("{l}"), "días".into()),
                    };
                    x.figure(&num, &unit, p - 4.0, p + 92.0, 76.0, s.accent);
                    let lbl = if label.is_empty() { "la fecha".to_string() } else { label.clone() };
                    x.text(&if left > 0 { format!("para {lbl}") } else { lbl }, p, h - p - 18.0, w - 2.0 * p, 13.0, s.ink, Align::L);
                    let (y, m, dd) = time::civil_from_days(td);
                    x.label(&format!("{} {} {}", dd, &MESES[m as usize - 1][..3], y), p, h - p, w, s.ink3, Align::L);
                }
                None => {
                    x.label("Cuenta atrás", p, p + 9.0, w, s.ink2, Align::L);
                    x.num("—", p, p + 92.0, w, 64.0, s.ink3, Align::L);
                    c.text_wrap("Clic para elegir una fecha", p, h - p - 34.0, w - 2.0 * p, 40.0, s.text(13.0), s.ink2, Align::L);
                    x.hit(0.0, 0.0, w, h, Action::SetCountdown);
                }
            }
        }
    }
}

// ---------- Cielo ----------

fn dur(m: u32) -> String {
    format!("{} h {:02} min", m / 60, m % 60)
}

pub fn sky(x: &Ctx) {
    let (c, s, d) = (x.c, &x.s, x.d);
    let n = &d.now;
    let (w, h, p) = (x.W, x.H, x.p);
    let mins = n.h * 60 + n.mi;
    let moon = astro::moon(n.unix);
    if x.layout() == 0 {
        x.label("Luna", p, p + 9.0, w, s.ink2, Align::L);
        x.label(&format!("{} %", (moon.lit * 100.0).round() as i32), p, p + 9.0, w - 2.0 * p, s.ink3, Align::R);
        x.moon(w / 2.0, p + 60.0, 34.0, moon.lit as f32, moon.waxing);
        x.strong(moon.name(), p, h - p - 18.0, w - 2.0 * p, 13.0, s.ink, Align::C);
        let (full, new) = moon.next();
        let next = if full < new && full >= 1.0 { format!("Llena en {} días", full.round() as i32) } else { format!("Nueva en {} días", new.round() as i32) };
        x.text(&next, p, h - p, w - 2.0 * p, 11.5, s.ink3, Align::C);
        return;
    }
    x.label(&d.place, p, p + 9.0, w * 0.5, s.ink2, Align::L);
    x.hit(0.0, 0.0, w, 36.0, Action::SetCity);
    let sun = d.weather.as_ref().and_then(|wt| Some((wt.sunrise?, wt.sunset?))).or_else(|| astro::sun(n.days(), d.lat, d.lon, n.offset));
    let Some((rise, set)) = sun else {
        x.text("Hoy el sol ni sale ni se pone aquí", p, h / 2.0, w - 2.0 * p, 13.0, s.ink2, Align::C);
        return;
    };
    let horizon = h - p - 26.0;
    let (rx, ry) = (w / 2.0 - p - 8.0, horizon - p - 26.0);
    let cx = w / 2.0;
    let at = |f: f32| {
        let a = (-90.0 + 180.0 * f) * PI / 180.0;
        (cx + a.sin() * rx, horizon - a.cos() * ry)
    };
    x.rule(p, horizon, w - p);
    for i in 0..44 {
        let (a, b) = (at(i as f32 / 44.0), at((i as f32 + 0.4) / 44.0));
        c.line(a.0, a.1, b.0, b.1, s.ink3, 1.4);
    }
    let day = mins >= rise && mins <= set;
    if day {
        let f = (mins - rise) as f32 / (set - rise).max(1) as f32;
        let pts: Vec<(f32, f32)> = (0..=48).map(|i| at(f * i as f32 / 48.0)).collect();
        c.path(&pts, s.accent, Some(2.2), false);
        let (sx, sy) = at(f);
        c.circle(sx, sy, 6.0, s.accent);
        x.label(&format!("Quedan {} de luz", dur(set - mins)), p, p + 9.0, w - 2.0 * p, s.ink, Align::R);
    } else {
        let till = if mins < rise { rise - mins } else { 24 * 60 - mins + rise };
        x.moon(cx, horizon - ry * 0.55, 11.0, moon.lit as f32, moon.waxing);
        x.label(&format!("Amanece en {}", dur(till)), p, p + 9.0, w - 2.0 * p, s.ink, Align::R);
    }
    x.text(&hhmm(rise / 60, rise % 60), p, h - p, 80.0, 12.5, s.ink2, Align::L);
    x.text(&format!("{} de sol", dur(set - rise)), p, h - p, w - 2.0 * p, 11.5, s.ink3, Align::C);
    x.text(&hhmm(set / 60, set % 60), p, h - p, w - 2.0 * p, 12.5, s.ink2, Align::R);
    let _ = Color::hex(0);
}

#[cfg(test)]
mod tests {
    use super::words;
    #[test]
    fn spanish_words() {
        assert_eq!(words(10, 15).1, "diez");
        assert_eq!(words(10, 15).2, "y cuarto");
        assert_eq!(words(12, 58).1, "una");
        assert_eq!(words(12, 58).0, "Es la");
        assert_eq!(words(0, 40).2, "menos veinte");
        assert_eq!(words(13, 0).3, "de la tarde");
    }
}
