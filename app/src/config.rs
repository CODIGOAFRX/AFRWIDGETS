//! Configuración en %APPDATA%\AFRWIDGETS\config.json.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WidgetCfg {
    pub id: u32,
    pub kind: String,
    pub x: i32,
    pub y: i32,
    #[serde(default = "noche")]
    pub theme: String,
    #[serde(default)]
    pub layout: u32,
    /// 0 pequeño, 1 mediano, 2 grande.
    #[serde(default = "one")]
    pub size: u32,
    #[serde(default = "auto")]
    pub accent: String,
    #[serde(default)]
    pub opts: Map<String, Value>,
}

fn noche() -> String {
    "noche".into()
}
fn one() -> u32 {
    1
}
fn auto() -> String {
    "auto".into()
}

impl WidgetCfg {
    pub fn s(&self, k: &str) -> String {
        self.opts.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
    }
    pub fn i(&self, k: &str, def: i64) -> i64 {
        self.opts.get(k).and_then(|v| v.as_i64()).unwrap_or(def)
    }
    pub fn b(&self, k: &str) -> bool {
        self.opts.get(k).and_then(|v| v.as_bool()).unwrap_or(false)
    }
    pub fn set(&mut self, k: &str, v: Value) {
        self.opts.insert(k.into(), v);
    }
    pub fn scale(&self) -> f32 {
        [0.82, 1.0, 1.24][self.size.min(2) as usize]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default)]
    pub widgets: Vec<WidgetCfg>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default = "madrid")]
    pub place: String,
    #[serde(default = "lat")]
    pub lat: f64,
    #[serde(default = "lon")]
    pub lon: f64,
    #[serde(default)]
    pub google_client_id: String,
    #[serde(default)]
    pub google_client_secret: String,
    /// Token de refresco de Google cifrado con DPAPI.
    #[serde(default)]
    pub google_token: String,
    #[serde(default)]
    pub welcomed: bool,
    #[serde(default)]
    pub next_id: u32,
}

fn madrid() -> String {
    "Madrid".into()
}
fn lat() -> f64 {
    40.4168
}
fn lon() -> f64 {
    -3.7038
}

pub fn dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("AFRWIDGETS")
}

impl Config {
    pub fn load() -> Config {
        let p = dir().join("config.json");
        std::fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::from_str("{}").unwrap())
    }

    pub fn save(&self) {
        let d = dir();
        let _ = std::fs::create_dir_all(&d);
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let tmp = d.join("config.json.tmp");
            if std::fs::write(&tmp, s).is_ok() {
                let _ = std::fs::rename(&tmp, d.join("config.json"));
            }
        }
    }

    pub fn new_id(&mut self) -> u32 {
        self.next_id = self.next_id.max(self.widgets.iter().map(|w| w.id).max().unwrap_or(0)) + 1;
        self.next_id
    }
}
