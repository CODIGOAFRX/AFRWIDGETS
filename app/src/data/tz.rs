//! Husos horarios con la base de datos de Windows (sin tablas propias).

use std::cell::RefCell;
use std::collections::HashMap;

use windows::Win32::Foundation::SYSTEMTIME;
use windows::Win32::System::SystemInformation::GetSystemTime;
use windows::Win32::System::Time::{
    EnumDynamicTimeZoneInformation, SystemTimeToTzSpecificLocalTimeEx, DYNAMIC_TIME_ZONE_INFORMATION,
};

/// (clave, nombre, zona de Windows)
pub const CITIES: [(&str, &str, &str); 20] = [
    ("madrid", "Madrid", "Romance Standard Time"),
    ("canarias", "Canarias", "GMT Standard Time"),
    ("londres", "Londres", "GMT Standard Time"),
    ("berlin", "Berlín", "W. Europe Standard Time"),
    ("atenas", "Atenas", "GTB Standard Time"),
    ("nuevayork", "Nueva York", "Eastern Standard Time"),
    ("chicago", "Chicago", "Central Standard Time"),
    ("losangeles", "Los Ángeles", "Pacific Standard Time"),
    ("mexico", "Ciudad de México", "Central Standard Time (Mexico)"),
    ("bogota", "Bogotá", "SA Pacific Standard Time"),
    ("lima", "Lima", "SA Pacific Standard Time"),
    ("santiago", "Santiago", "Pacific SA Standard Time"),
    ("buenosaires", "Buenos Aires", "Argentina Standard Time"),
    ("saopaulo", "São Paulo", "E. South America Standard Time"),
    ("dubai", "Dubái", "Arabian Standard Time"),
    ("bombay", "Bombay", "India Standard Time"),
    ("pekin", "Pekín", "China Standard Time"),
    ("seul", "Seúl", "Korea Standard Time"),
    ("tokio", "Tokio", "Tokyo Standard Time"),
    ("sidney", "Sídney", "AUS Eastern Standard Time"),
];

pub fn city(key: &str) -> Option<(&'static str, &'static str)> {
    CITIES.iter().find(|c| c.0 == key).map(|c| (c.1, c.2))
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Option<DYNAMIC_TIME_ZONE_INFORMATION>>> = RefCell::new(HashMap::new());
}

fn find(zone: &str) -> Option<DYNAMIC_TIME_ZONE_INFORMATION> {
    CACHE.with(|c| {
        if let Some(v) = c.borrow().get(zone) {
            return *v;
        }
        let mut found = None;
        let mut i = 0;
        loop {
            let mut d = DYNAMIC_TIME_ZONE_INFORMATION::default();
            if unsafe { EnumDynamicTimeZoneInformation(i, &mut d) } != 0 {
                break;
            }
            let len = d.TimeZoneKeyName.iter().position(|&x| x == 0).unwrap_or(0);
            if String::from_utf16_lossy(&d.TimeZoneKeyName[..len]) == zone {
                found = Some(d);
                break;
            }
            i += 1;
        }
        c.borrow_mut().insert(zone.to_string(), found);
        found
    })
}

fn secs(s: &SYSTEMTIME) -> i64 {
    crate::time::days_from_civil(s.wYear as i32, s.wMonth as u32, s.wDay as u32) * 86400
        + s.wHour as i64 * 3600
        + s.wMinute as i64 * 60
        + s.wSecond as i64
}

/// Minutos de diferencia con UTC en esa zona ahora mismo.
pub fn offset(zone: &str) -> Option<i32> {
    let d = find(zone)?;
    unsafe {
        let utc = GetSystemTime();
        let mut local = SYSTEMTIME::default();
        SystemTimeToTzSpecificLocalTimeEx(Some(&d), &utc, &mut local).ok()?;
        Some(((secs(&local) - secs(&utc)) as f64 / 60.0).round() as i32)
    }
}
