#![windows_subsystem = "windows"]
//! AFRWIDGETS: widgets de escritorio nativos y ligeros para Windows.

mod app;
mod config;
mod data;
mod gfx;
mod theme;
mod time;
mod ui;
mod widgets;

use std::path::Path;

use serde_json::json;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};

use config::WidgetCfg;
use gfx::{Color, Dib, Gfx};
use theme::Style;

fn main() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--render-previews") {
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| "previews".into());
        if let Err(e) = previews(Path::new(&dir)) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    unsafe {
        // Una sola instancia.
        let _m = CreateMutexW(None, true, w!("Local\\AFRWIDGETS")).ok();
        if GetLastError() == ERROR_ALREADY_EXISTS {
            ui::info("AFRWIDGETS", "AFRWIDGETS ya está abierto. Lo tienes en la bandeja del sistema, junto al reloj.");
            return;
        }
        let _ = app::run();
    }
}

/// Icono de la app: azulejo oscuro con cuatro piezas, una en coral.
pub fn draw_icon(c: &gfx::Canvas) {
    let s = c.w;
    c.rect(0.0, 0.0, s, s, s * 0.23, Color::hex(0x17181B));
    let g = s * 0.12;
    let cell = (s - 3.0 * g) / 2.0;
    let r = cell * 0.26;
    let tiles = [(0, 0, 0xF4F4F1), (1, 0, 0xE4573D), (0, 1, 0xF4F4F1), (1, 1, 0xF4F4F1)];
    for (i, j, col) in tiles {
        let (x, y) = (g + i as f32 * (cell + g), g + j as f32 * (cell + g));
        if (i, j) == (1, 1) {
            c.circle(x + cell / 2.0, y + cell / 2.0, cell / 2.0, Color::hex(col));
        } else {
            c.rect(x, y, cell, cell, r, Color::hex(col));
        }
    }
}

pub fn icon_dib(g: &Gfx, px: i32) -> Option<Dib> {
    g.paint(px, px, px as f32 / 64.0, false, &mut |c| {
        // El lienzo trabaja en DIP: 64 unidades de lado.
        let _ = c;
        draw_icon(c);
    })
}

fn previews(dir: &Path) -> std::result::Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let g = Gfx::new().map_err(|e| e.to_string())?;
    let mut d = data::Data::demo();
    d.place = "Madrid".into();
    let scale = 2.0;
    let mut count = 0;
    for m in widgets::KINDS.iter() {
        for li in 0..m.layouts.len() {
            for (ti, t) in theme::THEMES.iter().enumerate() {
                let mut w = WidgetCfg {
                    id: 1,
                    kind: m.key.into(),
                    x: 0,
                    y: 0,
                    theme: t.1.into(),
                    layout: li as u32,
                    size: 1,
                    accent: "auto".into(),
                    opts: Default::default(),
                };
                demo_opts(&mut w, &d);
                let _ = ti;
                let (dw, dh) = widgets::size(&w, &d);
                let style = Style::new(t.0, theme::accent_color(&w.accent, t.0, Color::hex(0x2F5BD3)));
                let (pw, ph) = ((dw * scale).ceil() as i32, (dh * scale).ceil() as i32);
                let dib = g
                    .paint(pw, ph, scale, style.halo, &mut |c| {
                        widgets::draw(c, style, &w, &d);
                    })
                    .ok_or("no se pudo pintar")?;
                let name = format!("{}-{}-{}.png", m.key, li, t.1);
                std::fs::write(dir.join(name), gfx::png(pw, ph, &dib.pixels())).map_err(|e| e.to_string())?;
                count += 1;
            }
        }
    }
    let mut images = Vec::new();
    for px in [16, 20, 24, 32, 40, 48, 64, 128, 256] {
        let dib = icon_dib(&g, px).ok_or("icono")?;
        let png = gfx::png(px, px, &dib.pixels());
        if [32, 64, 128, 256].contains(&px) {
            std::fs::write(dir.join(format!("icon-{px}.png")), &png).map_err(|e| e.to_string())?;
        }
        images.push((px, png));
    }
    std::fs::write(dir.join("afrwidgets.ico"), gfx::ico(&images)).map_err(|e| e.to_string())?;
    println!("{count} renders y el icono en {}", dir.display());
    Ok(())
}

fn demo_opts(w: &mut WidgetCfg, d: &data::Data) {
    match w.kind.as_str() {
        "notas" if w.layout == 0 => {
            w.set("titulo", json!("Para hoy"));
            w.set("texto", json!("Llamar a Marta antes de las siete.\nComprar pilas para el mando.\nMirar el contrato del piso con calma."));
        }
        "notas" => {
            w.set("texto", json!("[x] Enviar la factura de octubre\n[ ] Preparar la clase del jueves\n[ ] Pedir cita para la ITV\n[x] Regar las plantas\n[ ] Devolver el libro a Pablo"));
        }
        "foco" => {
            w.set("end", json!(d.now.unix + 14 * 60 + 32));
            w.set("hechos", json!(3));
            w.set("dia", json!(d.now.days()));
        }
        "progreso" => {
            w.set("fecha", json!("2026-12-19"));
            w.set("etiqueta", json!("las vacaciones"));
        }
        "husos" if w.layout == 1 => w.set("ciudades", json!("madrid,nuevayork,tokio")),
        "husos" => w.set("ciudades", json!("madrid,nuevayork,mexico,tokio")),
        _ => {}
    }
}
