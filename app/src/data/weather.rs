//! Tiempo con Open-Meteo (gratis, sin clave).

use serde_json::Value;

use super::http;

#[derive(Clone, Debug, PartialEq)]
pub struct Day {
    /// Días desde 1970 (fecha local del sitio).
    pub days: i64,
    pub code: u32,
    pub min: f32,
    pub max: f32,
    pub rain: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Weather {
    pub temp: f32,
    pub feels: f32,
    pub code: u32,
    pub is_day: bool,
    pub humidity: u32,
    pub wind: f32,
    pub days: Vec<Day>,
    /// Minutos desde medianoche, hora local del sitio.
    pub sunrise: Option<u32>,
    pub sunset: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sky {
    Clear,
    Partly,
    Cloud,
    Fog,
    Drizzle,
    Rain,
    Snow,
    Storm,
}

pub fn sky(code: u32) -> Sky {
    match code {
        0 => Sky::Clear,
        1 | 2 => Sky::Partly,
        3 => Sky::Cloud,
        45 | 48 => Sky::Fog,
        51..=57 => Sky::Drizzle,
        61..=67 | 80..=82 => Sky::Rain,
        71..=77 | 85 | 86 => Sky::Snow,
        95..=99 => Sky::Storm,
        _ => Sky::Cloud,
    }
}

pub fn describe(code: u32) -> &'static str {
    match code {
        0 => "Despejado",
        1 => "Casi despejado",
        2 => "Parcialmente nublado",
        3 => "Cubierto",
        45 | 48 => "Niebla",
        51 | 53 | 55 => "Llovizna",
        56 | 57 => "Llovizna helada",
        61 => "Lluvia débil",
        63 => "Lluvia",
        65 => "Lluvia fuerte",
        66 | 67 => "Lluvia helada",
        71 | 73 | 75 | 77 => "Nieve",
        80 | 81 => "Chubascos",
        82 => "Chubascos fuertes",
        85 | 86 => "Chubascos de nieve",
        95 => "Tormenta",
        96 | 99 => "Tormenta con granizo",
        _ => "Variable",
    }
}

pub struct Place {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

pub fn geocode(name: &str) -> Result<Place, String> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=es&format=json",
        http::encode(name.trim())
    );
    let r = http::get(&url, &[])?;
    let v: Value = serde_json::from_str(&r.body).map_err(|e| e.to_string())?;
    let p = v["results"].get(0).ok_or("No encuentro esa ciudad")?;
    Ok(Place {
        name: p["name"].as_str().unwrap_or(name).to_string(),
        lat: p["latitude"].as_f64().ok_or("sin latitud")?,
        lon: p["longitude"].as_f64().ok_or("sin longitud")?,
    })
}

fn hm(s: &str) -> Option<u32> {
    let h: u32 = s.get(11..13)?.parse().ok()?;
    let m: u32 = s.get(14..16)?.parse().ok()?;
    Some(h * 60 + m)
}

pub fn fetch(lat: f64, lon: f64) -> Result<Weather, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat:.4}&longitude={lon:.4}\
&current=temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m,is_day\
&daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,precipitation_probability_max\
&timezone=auto&forecast_days=6"
    );
    let r = http::get(&url, &[])?;
    if r.status != 200 {
        return Err(format!("Open-Meteo respondió {}", r.status));
    }
    let v: Value = serde_json::from_str(&r.body).map_err(|e| e.to_string())?;
    let c = &v["current"];
    let d = &v["daily"];
    let f = |x: &Value| x.as_f64().unwrap_or(0.0) as f32;
    let mut days = Vec::new();
    if let Some(times) = d["time"].as_array() {
        for (i, t) in times.iter().enumerate() {
            let s = t.as_str().unwrap_or("");
            let (y, m, dd) = (
                s.get(0..4).and_then(|x| x.parse().ok()).unwrap_or(1970),
                s.get(5..7).and_then(|x| x.parse().ok()).unwrap_or(1),
                s.get(8..10).and_then(|x| x.parse().ok()).unwrap_or(1),
            );
            days.push(Day {
                days: crate::time::days_from_civil(y, m, dd),
                code: d["weather_code"][i].as_u64().unwrap_or(3) as u32,
                min: f(&d["temperature_2m_min"][i]),
                max: f(&d["temperature_2m_max"][i]),
                rain: d["precipitation_probability_max"][i].as_u64().unwrap_or(0) as u32,
            });
        }
    }
    Ok(Weather {
        temp: f(&c["temperature_2m"]),
        feels: f(&c["apparent_temperature"]),
        code: c["weather_code"].as_u64().unwrap_or(3) as u32,
        is_day: c["is_day"].as_u64().unwrap_or(1) == 1,
        humidity: c["relative_humidity_2m"].as_u64().unwrap_or(0) as u32,
        wind: f(&c["wind_speed_10m"]),
        sunrise: d["sunrise"][0].as_str().and_then(hm),
        sunset: d["sunset"][0].as_str().and_then(hm),
        days,
    })
}
