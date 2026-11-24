//! Sol y luna calculados en local (sin red).

use std::f64::consts::PI;

const SYNODIC: f64 = 29.530588853;

fn rad(d: f64) -> f64 {
    d * PI / 180.0
}

/// Salida y puesta de sol en minutos locales para el día `days` (días desde 1970, fecha local).
pub fn sun(days: i64, lat: f64, lon: f64, offset_min: i32) -> Option<(u32, u32)> {
    let jd_noon = days as f64 + 2440587.5 + 0.5;
    let n = (jd_noon - 2451545.0 + 0.0008).round();
    let jstar = n - lon / 360.0;
    let m = (357.5291 + 0.98560028 * jstar).rem_euclid(360.0);
    let c = 1.9148 * rad(m).sin() + 0.02 * rad(2.0 * m).sin() + 0.0003 * rad(3.0 * m).sin();
    let l = (m + c + 180.0 + 102.9372).rem_euclid(360.0);
    let transit = 2451545.0 + jstar + 0.0053 * rad(m).sin() - 0.0069 * rad(2.0 * l).sin();
    let decl = (rad(l).sin() * rad(23.4397).sin()).asin();
    let cosw = (rad(-0.833).sin() - rad(lat).sin() * decl.sin()) / (rad(lat).cos() * decl.cos());
    if !(-1.0..=1.0).contains(&cosw) {
        return None; // sol de medianoche o noche polar
    }
    let w = cosw.acos() * 180.0 / PI;
    let to_local = |j: f64| {
        let unix = (j - 2440587.5) * 86400.0;
        let local = unix + offset_min as f64 * 60.0;
        (local.rem_euclid(86400.0) / 60.0).round() as u32
    };
    Some((to_local(transit - w / 360.0), to_local(transit + w / 360.0)))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moon {
    /// Edad en días desde la luna nueva.
    pub age: f64,
    /// Fracción iluminada 0..1.
    pub lit: f64,
    pub waxing: bool,
}

pub fn moon(unix: i64) -> Moon {
    let jd = unix as f64 / 86400.0 + 2440587.5;
    let age = (jd - 2451550.1).rem_euclid(SYNODIC);
    Moon { age, lit: (1.0 - (2.0 * PI * age / SYNODIC).cos()) / 2.0, waxing: age < SYNODIC / 2.0 }
}

impl Moon {
    pub fn name(&self) -> &'static str {
        let p = self.age / SYNODIC;
        match p {
            p if p < 0.033 || p > 0.967 => "Luna nueva",
            p if p < 0.216 => "Creciente",
            p if p < 0.284 => "Cuarto creciente",
            p if p < 0.466 => "Gibosa creciente",
            p if p < 0.534 => "Luna llena",
            p if p < 0.716 => "Gibosa menguante",
            p if p < 0.784 => "Cuarto menguante",
            _ => "Menguante",
        }
    }
    /// Días hasta la próxima luna llena y la próxima nueva.
    pub fn next(&self) -> (f64, f64) {
        let full = (SYNODIC / 2.0 - self.age).rem_euclid(SYNODIC);
        let new = (SYNODIC - self.age).rem_euclid(SYNODIC);
        (full, new)
    }
}
