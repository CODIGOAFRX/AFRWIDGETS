//! Proveedores de datos y la foto de datos que leen los widgets al pintarse.

pub mod astro;
pub mod google;
pub mod http;
pub mod media;
pub mod secure;
pub mod sys;
pub mod tz;
pub mod weather;

use std::collections::VecDeque;
use std::sync::mpsc::Sender;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

use crate::time::{self, Now};

pub const WM_APP_DATA: u32 = WM_APP + 7;

pub enum Msg {
    Weather(Result<weather::Weather, String>),
    Place(Result<weather::Place, String>),
    Calendar(Result<Vec<google::Event>, String>),
    Google(Result<String, String>),
    Media(Option<media::Media>),
}

/// Canal de vuelta desde los hilos de trabajo: envía el dato y despierta el bucle de mensajes.
#[derive(Clone)]
pub struct Waker {
    pub tx: Sender<Msg>,
    pub hwnd: usize,
}

impl Waker {
    pub fn send(&self, m: Msg) {
        let _ = self.tx.send(m);
        unsafe {
            let _ = PostMessageW(HWND(self.hwnd as *mut _), WM_APP_DATA, WPARAM(0), LPARAM(0));
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CalState {
    /// Falta el cliente OAuth (credenciales de Google Cloud).
    NoCreds,
    Disconnected,
    Connecting,
    Ready,
    Error(String),
}

pub struct Data {
    pub now: Now,
    pub cpu: f32,
    pub cpu_hist: VecDeque<f32>,
    pub mem: (u64, u64),
    pub mem_hist: VecDeque<f32>,
    pub net: (f64, f64),
    pub net_hist: VecDeque<(f64, f64)>,
    pub disks: Vec<sys::Disk>,
    pub battery: sys::Battery,
    pub weather: Option<weather::Weather>,
    pub weather_err: Option<String>,
    pub place: String,
    pub lat: f64,
    pub lon: f64,
    pub events: Vec<google::Event>,
    pub cal: CalState,
    pub media: Option<media::Media>,
}

pub const HIST: usize = 48;

impl Data {
    pub fn new() -> Data {
        Data {
            now: time::now(),
            cpu: 0.0,
            cpu_hist: VecDeque::new(),
            mem: sys::memory(),
            mem_hist: VecDeque::new(),
            net: (0.0, 0.0),
            net_hist: VecDeque::new(),
            disks: Vec::new(),
            battery: sys::Battery::default(),
            weather: None,
            weather_err: None,
            place: "Madrid".into(),
            lat: 40.4168,
            lon: -3.7038,
            events: Vec::new(),
            cal: CalState::Disconnected,
            media: None,
        }
    }

    pub fn push<T>(q: &mut VecDeque<T>, v: T) {
        q.push_back(v);
        while q.len() > HIST {
            q.pop_front();
        }
    }

    /// Datos de muestra para los renders de la landing (etiquetados como ejemplo allí).
    pub fn demo() -> Data {
        let mut d = Data::new();
        d.now = Now::from_unix(time::local_midnight_unix(2026, 11, 17, 60) + 10 * 3600 + 42 * 60 + 17, 60);
        d.cpu = 0.23;
        // Ruido determinista para que las gráficas de muestra parezcan reales.
        let mut seed = 0x2545_F491u32;
        let mut rnd = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed % 10_000) as f32 / 10_000.0
        };
        let mut v = 0.18f32;
        d.cpu_hist = (0..HIST)
            .map(|_| {
                v = (v * 0.6 + 0.4 * (0.08 + 0.3 * rnd() * rnd() + if rnd() > 0.93 { 0.35 } else { 0.0 })).clamp(0.03, 0.95);
                v
            })
            .collect();
        d.cpu = *d.cpu_hist.back().unwrap();
        d.mem = (9_800_000_000, 16_900_000_000);
        d.mem_hist = (0..HIST).map(|i| 0.56 + 0.02 * (i as f32 * 0.13).sin() + 0.01 * rnd()).collect();
        let mut r = 1_400_000.0f64;
        d.net_hist = (0..HIST)
            .map(|_| {
                r = (r * 0.55 + 0.45 * (300_000.0 + 2_600_000.0 * (rnd() as f64).powi(2))).max(40_000.0);
                (r, 60_000.0 + 140_000.0 * rnd() as f64)
            })
            .collect();
        d.net = *d.net_hist.back().unwrap();
        d.disks = vec![
            sys::Disk { letter: 'C', label: "Windows".into(), total: 511_000_000_000, free: 187_000_000_000 },
            sys::Disk { letter: 'D', label: "Proyectos".into(), total: 1_000_000_000_000, free: 612_000_000_000 },
        ];
        d.battery = sys::Battery { present: true, percent: 76, charging: false, plugged: false, remaining: Some(4 * 3600 + 35 * 60), saver: false };
        let today = d.now.days();
        d.weather = Some(weather::Weather {
            temp: 14.0,
            feels: 12.6,
            code: 2,
            is_day: true,
            humidity: 58,
            wind: 11.0,
            sunrise: Some(8 * 60 + 5),
            sunset: Some(17 * 60 + 52),
            days: [(2, 9.0, 16.0, 10), (61, 8.0, 13.0, 70), (3, 7.0, 12.0, 30), (0, 5.0, 14.0, 0), (1, 6.0, 15.0, 5), (80, 9.0, 14.0, 60)]
                .iter()
                .enumerate()
                .map(|(i, &(code, min, max, rain))| weather::Day { days: today + i as i64, code, min, max, rain })
                .collect(),
        });
        let at = |day: i64, h: i64, m: i64| (day * 86400 + h * 3600 + m * 60) - 60 * 60;
        let ev = |title: &str, day: i64, h: i64, m: i64, dur: i64, color: u32| google::Event {
            title: title.into(),
            start: at(day, h, m),
            end: at(day, h, m) + dur * 60,
            all_day: false,
            color,
        };
        d.events = vec![
            ev("Revisión de diseño", today, 12, 0, 45, 0x4285F4),
            ev("Clase de Rust", today, 17, 30, 90, 0x33B679),
            ev("Cena con Lucía", today + 1, 21, 0, 120, 0xE67C73),
            ev("Entrega AFRWIDGETS", today + 3, 10, 0, 60, 0x4285F4),
            ev("Dentista", today + 6, 9, 15, 30, 0xF6BF26),
            ev("Concierto", today + 10, 20, 30, 150, 0x8E24AA),
            ev("Mudanza de Pablo", today - 5, 10, 0, 240, 0x33B679),
            ev("Retro del sprint", today - 2, 16, 0, 60, 0x4285F4),
        ];
        d.events.sort_by_key(|e| e.start);
        d.cal = CalState::Ready;
        d.media = Some(media::Media {
            title: "Mar de fondo".into(),
            artist: "Las Lindes".into(),
            app: "Spotify".into(),
            playing: true,
            pos: 94,
            len: 221,
        });
        d
    }
}
