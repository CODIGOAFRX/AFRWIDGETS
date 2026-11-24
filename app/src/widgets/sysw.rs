//! Disco, sistema (CPU y RAM), red y batería.

use super::Ctx;
use crate::data::sys::fmt_bytes;
use crate::gfx::Align;

fn gb(b: u64) -> String {
    fmt_bytes(b as f64)
}

/// «2,34 MB/s» → («2,34», «MB/s»).
fn rate(b: f64) -> (String, String) {
    let t = fmt_bytes(b);
    match t.split_once(' ') {
        Some((n, u)) => (n.to_string(), format!("{u}/s")),
        None => (t, "/s".into()),
    }
}

pub fn disk(x: &Ctx) {
    let (s, d) = (&x.s, x.d);
    let (w, h, p) = (x.W, x.H, x.p);
    if d.disks.is_empty() {
        x.label("Disco", p, p + 9.0, w, s.ink2, Align::L);
        x.text("Leyendo unidades…", p, h - p, w, 12.0, s.ink3, Align::L);
        return;
    }
    if x.layout() == 0 {
        let k = d.disks.iter().find(|k| k.letter == 'C').unwrap_or(&d.disks[0]);
        let used = 1.0 - k.free as f32 / k.total as f32;
        let name = if k.label.is_empty() { format!("{}:", k.letter) } else { format!("{}: {}", k.letter, k.label) };
        x.label(&name, p, p + 9.0, w - 2.0 * p, s.ink2, Align::L);
        x.figure(&format!("{}", (used * 100.0).round() as i32), "% usado", p - 3.0, p + 86.0, 64.0, s.ink);
        x.segs(p, h - p - 32.0, w - 2.0 * p, 10.0, 16, used, if used > 0.9 { x.warn() } else { s.accent });
        x.text(&format!("{} libres de {}", gb(k.free), gb(k.total)), p, h - p, w - 2.0 * p, 12.0, s.ink2, Align::L);
    } else {
        x.label("Almacenamiento", p, p + 9.0, w, s.ink2, Align::L);
        let total: u64 = d.disks.iter().map(|k| k.free).sum();
        x.label(&format!("{} libres", gb(total)), p, p + 9.0, w - 2.0 * p, s.ink3, Align::R);
        for (i, k) in d.disks.iter().take(3).enumerate() {
            let b = p + 48.0 + i as f32 * 42.0;
            let used = 1.0 - k.free as f32 / k.total as f32;
            let name = if k.label.is_empty() { format!("{}:", k.letter) } else { format!("{}:  {}", k.letter, k.label) };
            x.strong(&name, p, b, w * 0.5, 13.5, s.ink, Align::L);
            x.text(&format!("{} % · {} libres", (used * 100.0).round() as i32, gb(k.free)), p, b, w - 2.0 * p, 12.0, s.ink2, Align::R);
            x.bar(p, b + 9.0, w - 2.0 * p, 3.0, used, if used > 0.9 { x.warn() } else { s.accent });
        }
    }
}

pub fn system(x: &Ctx) {
    let (s, d) = (&x.s, x.d);
    let (w, h, p) = (x.W, x.H, x.p);
    let mem = d.mem.0 as f32 / d.mem.1 as f32;
    let cpu = d.cpu;
    let pct = |f: f32| format!("{}", (f * 100.0).round() as i32);
    if x.layout() == 0 {
        for (i, (label, f)) in [("CPU", cpu), ("Memoria", mem)].iter().enumerate() {
            let top = p + i as f32 * 70.0;
            x.label(label, p, top + 9.0, w, s.ink2, Align::L);
            x.figure(&pct(*f), "%", p - 2.0, top + 46.0, 36.0, s.ink);
            x.segs(p, top + 54.0, w - 2.0 * p, 6.0, 20, *f, if *f > 0.9 { x.warn() } else if i == 0 { s.accent } else { s.ink });
        }
    } else {
        x.label("CPU", p, p + 9.0, 140.0, s.ink2, Align::L);
        x.figure(&pct(cpu), "%", p - 3.0, p + 74.0, 58.0, s.ink);
        x.label("Memoria", p, h - p - 22.0, 140.0, s.ink2, Align::L);
        x.text(&format!("{} % · {} de {}", pct(mem), gb(d.mem.0), gb(d.mem.1)), p, h - p, 150.0, 12.0, s.ink, Align::L);
        let gx = 176.0;
        let (gy, gw, gh) = (p + 4.0, w - p - gx, h - 2.0 * p - 4.0);
        x.rule(gx, gy + gh, gx + gw);
        x.c.rect(gx, gy + gh / 2.0, gw, 1.0, 0.0, s.rule.a(0.5));
        let cpu_h: Vec<f32> = d.cpu_hist.iter().copied().collect();
        let mem_h: Vec<f32> = d.mem_hist.iter().copied().collect();
        x.spark(gx, gy, gw, gh, &mem_h, 1.0, s.ink3, 1.2, false);
        x.spark(gx, gy, gw, gh, &cpu_h, 1.0, s.accent, 1.8, true);
    }
}

pub fn net(x: &Ctx) {
    let (c, s, d) = (x.c, &x.s, x.d);
    let (w, h, p) = (x.W, x.H, x.p);
    let (dn, du) = rate(d.net.0);
    let (un, uu) = rate(d.net.1);
    let arrow = |cx: f32, cy: f32, down: bool, col| {
        let k = if down { 1.0 } else { -1.0 };
        c.line(cx, cy - 5.0 * k, cx, cy + 5.0 * k, col, 1.6);
        c.path(&[(cx - 4.0, cy + 1.0 * k), (cx, cy + 5.5 * k), (cx + 4.0, cy + 1.0 * k)], col, Some(1.6), false);
    };
    if x.layout() == 0 {
        for (i, (label, n, u, down)) in [("Bajada", &dn, &du, true), ("Subida", &un, &uu, false)].iter().enumerate() {
            let top = p + i as f32 * 70.0;
            arrow(p + 4.0, top + 5.0, *down, if *down { s.accent } else { s.ink2 });
            x.label(label, p + 14.0, top + 9.0, w, s.ink2, Align::L);
            x.figure(n, u, p - 2.0, top + 50.0, 38.0, s.ink);
        }
    } else {
        arrow(p + 4.0, p + 5.0, true, s.accent);
        x.label("Bajada", p + 14.0, p + 9.0, 120.0, s.ink2, Align::L);
        x.figure(&dn, &du, p - 2.0, p + 52.0, 38.0, s.ink);
        arrow(p + 4.0, p + 85.0, false, s.ink2);
        x.label("Subida", p + 14.0, p + 89.0, 120.0, s.ink2, Align::L);
        x.figure(&un, &uu, p - 2.0, h - p, 26.0, s.ink);
        let rx: Vec<f32> = d.net_hist.iter().map(|v| v.0 as f32).collect();
        let tx: Vec<f32> = d.net_hist.iter().map(|v| v.1 as f32).collect();
        let max = rx.iter().chain(tx.iter()).fold(64_000.0f32, |a, b| a.max(*b)) * 1.15;
        let gx = 176.0;
        let (gy, gw, gh) = (p + 4.0, w - p - gx, h - 2.0 * p - 4.0);
        x.rule(gx, gy + gh, gx + gw);
        x.spark(gx, gy, gw, gh, &tx, max, s.ink3, 1.2, false);
        x.spark(gx, gy, gw, gh, &rx, max, s.accent, 1.8, true);
    }
}

fn time_left(sec: u32) -> String {
    let m = sec / 60;
    if m >= 60 { format!("{} h {:02} min", m / 60, m % 60) } else { format!("{m} min") }
}

pub fn battery(x: &Ctx) {
    let (c, s, d) = (x.c, &x.s, x.d);
    let (w, h, p) = (x.W, x.H, x.p);
    let b = d.battery;
    let state = if !b.present {
        "A la corriente".to_string()
    } else if b.charging {
        "Cargando".to_string()
    } else if b.plugged {
        "Conectado".to_string()
    } else if let Some(r) = b.remaining {
        format!("Quedan {}", time_left(r))
    } else {
        "Con batería".to_string()
    };
    let f = if b.present { b.percent as f32 / 100.0 } else { 1.0 };
    let col = if b.present && !b.plugged && b.percent <= 15 { x.warn() } else { s.accent };
    let n = if b.present { format!("{}", b.percent) } else { "—".into() };
    if x.layout() == 0 {
        x.label("Batería", p, p + 9.0, w, s.ink2, Align::L);
        if b.charging || b.saver {
            x.label(if b.charging { "Cargando" } else { "Ahorro" }, p, p + 9.0, w - 2.0 * p, s.accent, Align::R);
        }
        x.figure(&n, if b.present { "%" } else { "" }, p - 3.0, p + 86.0, 64.0, s.ink);
        x.segs(p, h - p - 34.0, w - 2.0 * p, 14.0, 10, f, col);
        x.text(&state, p, h - p, w - 2.0 * p, 12.0, s.ink2, Align::L);
    } else {
        x.label("Batería", p, p + 9.0, 150.0, s.ink2, Align::L);
        x.figure(&n, if b.present { "%" } else { "" }, p - 4.0, p + 96.0, 80.0, s.ink);
        x.text(&state, p, h - p, 160.0, 12.5, s.ink2, Align::L);
        // Pila de diez celdas verticales con su borne.
        let (bx, by, bw, bh) = (180.0, p + 30.0, w - p - 180.0 - 8.0, h - 2.0 * p - 46.0);
        let gap = 4.0;
        let cw = (bw - gap * 9.0) / 10.0;
        let on = (f * 10.0).round() as usize;
        for i in 0..10 {
            c.rect(bx + i as f32 * (cw + gap), by, cw, bh, 3.0, if i < on { col } else { s.track });
        }
        c.rect(bx + bw + 3.0, by + bh * 0.3, 4.0, bh * 0.4, 2.0, s.track);
        if b.charging {
            x.label("Cargando", bx, h - p, bw, s.accent, Align::R);
        }
    }
}
