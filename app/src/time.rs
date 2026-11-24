//! Fechas sin dependencias: calendario civil, hora local y RFC 3339.

use windows::Win32::Foundation::SYSTEMTIME;
use windows::Win32::System::SystemInformation::{GetLocalTime, GetSystemTime};

pub const MESES: [&str; 12] = [
    "enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre",
    "noviembre", "diciembre",
];
pub const DIAS: [&str; 7] = ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"];
pub const DIAS_CORTOS: [&str; 7] = ["L", "M", "X", "J", "V", "S", "D"];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Now {
    pub y: i32,
    pub mo: u32,
    pub d: u32,
    pub h: u32,
    pub mi: u32,
    pub s: u32,
    /// 0 = lunes.
    pub wd: u32,
    /// Minutos de diferencia con UTC (hora local - UTC).
    pub offset: i32,
    /// Segundos Unix (UTC).
    pub unix: i64,
}

pub fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y } as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = m as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((if m <= 2 { y + 1 } else { y }) as i32, m, d)
}

/// 0 = lunes.
pub fn weekday(days: i64) -> u32 {
    (days + 3).rem_euclid(7) as u32
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    (days_from_civil(ny, nm, 1) - days_from_civil(y, m, 1)) as u32
}

fn st_secs(s: &SYSTEMTIME) -> i64 {
    days_from_civil(s.wYear as i32, s.wMonth as u32, s.wDay as u32) * 86400
        + s.wHour as i64 * 3600
        + s.wMinute as i64 * 60
        + s.wSecond as i64
}

pub fn now() -> Now {
    unsafe {
        let l = GetLocalTime();
        let u = GetSystemTime();
        let offset = ((st_secs(&l) - st_secs(&u)) as f64 / 60.0).round() as i32;
        let days = days_from_civil(l.wYear as i32, l.wMonth as u32, l.wDay as u32);
        Now {
            y: l.wYear as i32,
            mo: l.wMonth as u32,
            d: l.wDay as u32,
            h: l.wHour as u32,
            mi: l.wMinute as u32,
            s: l.wSecond as u32,
            wd: weekday(days),
            offset,
            unix: st_secs(&u),
        }
    }
}

impl Now {
    pub fn days(&self) -> i64 {
        days_from_civil(self.y, self.mo, self.d)
    }
    /// Construye un instante local a partir de segundos Unix y el desfase dado.
    pub fn from_unix(unix: i64, offset: i32) -> Now {
        let local = unix + offset as i64 * 60;
        let days = local.div_euclid(86400);
        let sec = local.rem_euclid(86400) as u32;
        let (y, mo, d) = civil_from_days(days);
        Now { y, mo, d, h: sec / 3600, mi: sec / 60 % 60, s: sec % 60, wd: weekday(days), offset, unix }
    }
}

/// Segundos Unix de una fecha local (medianoche) con el desfase dado.
pub fn local_midnight_unix(y: i32, m: u32, d: u32, offset: i32) -> i64 {
    days_from_civil(y, m, d) * 86400 - offset as i64 * 60
}

/// RFC 3339 a UTC: devuelve (unix, ¿es de día completo?). Las fechas sin hora se toman a medianoche local.
pub fn parse_rfc3339(s: &str, local_offset: i32) -> Option<(i64, bool)> {
    let b = s.as_bytes();
    let num = |a: usize, n: usize| -> Option<i64> { s.get(a..a + n)?.parse().ok() };
    let (y, m, d) = (num(0, 4)? as i32, num(5, 2)? as u32, num(8, 2)? as u32);
    if b.len() <= 10 {
        return Some((local_midnight_unix(y, m, d, local_offset), true));
    }
    let (h, mi, se) = (num(11, 2)?, num(14, 2)?, num(17, 2)?);
    let mut t = days_from_civil(y, m, d) * 86400 + h * 3600 + mi * 60 + se;
    // Zona: Z o ±hh:mm después de los segundos (y fracciones opcionales).
    let mut i = 19;
    if b.get(i) == Some(&b'.') {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    match b.get(i) {
        Some(b'Z') | Some(b'z') | None => {}
        Some(&sign) if sign == b'+' || sign == b'-' => {
            let oh = num(i + 1, 2)?;
            let om = num(i + 4, 2)?;
            let off = oh * 3600 + om * 60;
            t -= if sign == b'+' { off } else { -off };
        }
        _ => {}
    }
    Some((t, false))
}

/// "2026-11-24T00:00:00Z" a partir de segundos Unix.
pub fn rfc3339_utc(unix: i64) -> String {
    let n = Now::from_unix(unix, 0);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", n.y, n.mo, n.d, n.h, n.mi, n.s)
}

pub fn hhmm(h: u32, m: u32) -> String {
    format!("{:02}:{:02}", h, m)
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
