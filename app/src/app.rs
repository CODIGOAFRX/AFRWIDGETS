//! Anfitrión: ventanas de los widgets, bandeja, menús, arrastre, temporizador y trabajo en segundo plano.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};

use serde_json::{json, Value};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Registry::*;
use windows::Win32::System::SystemInformation::GetSystemTime;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::config::{Config, WidgetCfg};
use crate::data::{self, google, media, secure, sys, weather, CalState, Data, Msg, Waker, WM_APP_DATA};
use crate::gfx::{Color, Gfx};
use crate::theme::{self, Style, ThemeId, ACCENTS, THEMES};
use crate::time;
use crate::ui;
use crate::widgets::{self, Action, Hit, Kind, KINDS};

const WM_TRAY: u32 = WM_APP + 1;
const TIMER: usize = 1;
const SITE: &str = "https://pedrogomez.dev/proyecto/afrwidgets";

static DESKTOP_FRONT: AtomicBool = AtomicBool::new(false);

struct Win {
    hwnd: HWND,
    hits: Vec<Hit>,
    stamp: String,
    scale: f32,
}

struct Drag {
    id: u32,
    cursor: POINT,
    origin: (i32, i32),
    moved: bool,
}

pub struct App {
    pub cfg: Config,
    pub data: Data,
    gfx: Gfx,
    main: HWND,
    wins: HashMap<u32, Win>,
    rx: Receiver<Msg>,
    waker: Waker,
    cpu: sys::CpuSampler,
    net: sys::NetSampler,
    media_tx: Option<Sender<media::In>>,
    weather_at: i64,
    weather_busy: bool,
    cal_at: i64,
    cal_busy: bool,
    sys_at: i64,
    disk_at: i64,
    drag: Option<Drag>,
    accent: Color,
    icon: HICON,
    taskbar_msg: u32,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

/// Acceso al estado. Si ya está prestado (un menú o diálogo modal en curso), no hace nada.
fn with<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|a| a.try_borrow_mut().ok().and_then(|mut g| g.as_mut().map(f)))
}

fn wstr<const N: usize>(s: &str) -> [u16; N] {
    let mut out = [0u16; N];
    for (i, c) in s.encode_utf16().take(N - 1).enumerate() {
        out[i] = c;
    }
    out
}

pub fn system_accent() -> Color {
    let mut v = 0u32;
    let mut size = 4u32;
    let ok = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\DWM"),
            w!("AccentColor"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut v as *mut u32 as *mut _),
            Some(&mut size),
        )
        .is_ok()
    };
    if ok {
        Color::rgba((v & 0xff) as u8, (v >> 8 & 0xff) as u8, (v >> 16 & 0xff) as u8, 1.0)
    } else {
        Color::hex(0x2F5BD3)
    }
}

pub fn style_for(w: &WidgetCfg, system: Color) -> Style {
    let id = ThemeId::from_key(&w.theme);
    Style::new(id, theme::accent_color(&w.accent, id, system))
}

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");

fn autostart_enabled() -> bool {
    let mut size = 0u32;
    unsafe { RegGetValueW(HKEY_CURRENT_USER, RUN_KEY, w!("AFRWIDGETS"), RRF_RT_REG_SZ, None, None, Some(&mut size)).is_ok() }
}

fn set_autostart(on: bool) {
    unsafe {
        if on {
            let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
            let v: Vec<u16> = format!("\"{exe}\"").encode_utf16().chain([0]).collect();
            let _ = RegSetKeyValueW(HKEY_CURRENT_USER, RUN_KEY, w!("AFRWIDGETS"), REG_SZ.0, Some(v.as_ptr() as *const _), (v.len() * 2) as u32);
        } else {
            let _ = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, w!("AFRWIDGETS"));
        }
    }
}

// ---------- Arranque ----------

pub fn run() -> Result<()> {
    unsafe {
        let inst = GetModuleHandleW(None)?;
        let main_class = w!("AFRMain");
        RegisterClassW(&WNDCLASSW { lpfnWndProc: Some(main_proc), hInstance: inst.into(), lpszClassName: main_class, ..Default::default() });
        let widget_class = w!("AFRWidget");
        RegisterClassW(&WNDCLASSW {
            style: CS_DBLCLKS,
            lpfnWndProc: Some(widget_proc),
            hInstance: inst.into(),
            lpszClassName: widget_class,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        });
        let main = CreateWindowExW(WINDOW_EX_STYLE(0), main_class, w!("AFRWIDGETS"), WS_POPUP, 0, 0, 0, 0, None, None, inst, None)?;
        let (tx, rx) = channel();
        let gfx = Gfx::new()?;
        let icon = crate::icon_dib(&gfx, 32).map(|d| d.to_icon()).unwrap_or_default();
        let mut cfg = Config::load();
        let mut data = Data::new();
        data.place = cfg.place.clone();
        data.lat = cfg.lat;
        data.lon = cfg.lon;
        data.cal = if google::creds(&cfg.google_client_id, &cfg.google_client_secret).is_none() {
            CalState::NoCreds
        } else if cfg.google_token.is_empty() {
            CalState::Disconnected
        } else {
            CalState::Ready
        };
        data.disks = sys::disks();
        data.battery = sys::battery();
        let first_run = cfg.widgets.is_empty() && !cfg.welcomed;
        if first_run {
            default_layout(&mut cfg);
        }
        let app = App {
            cfg,
            data,
            gfx,
            main,
            wins: HashMap::new(),
            rx,
            waker: Waker { tx, hwnd: main.0 as usize },
            cpu: Default::default(),
            net: Default::default(),
            media_tx: None,
            weather_at: 0,
            weather_busy: false,
            cal_at: 0,
            cal_busy: false,
            sys_at: 0,
            disk_at: 0,
            drag: None,
            accent: system_accent(),
            icon,
            taskbar_msg: RegisterWindowMessageW(w!("TaskbarCreated")),
        };
        APP.with(|a| *a.borrow_mut() = Some(app));
        with(|a| {
            a.tray(NIM_ADD);
            let ids: Vec<u32> = a.cfg.widgets.iter().map(|w| w.id).collect();
            for id in ids {
                a.create_window(id);
            }
            a.tick();
            if first_run {
                a.cfg.welcomed = true;
                a.cfg.save();
                a.balloon("AFRWIDGETS ya está en tu escritorio", "Clic derecho sobre un widget para cambiar su estética. Desde este icono añades más.");
            }
        });
        SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, None, Some(foreground_hook), 0, 0, WINEVENT_OUTOFCONTEXT);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        with(|a| {
            a.tray(NIM_DELETE);
            a.cfg.save();
        });
    }
    Ok(())
}

fn work_area() -> RECT {
    let mut rc = RECT::default();
    unsafe {
        let _ = SystemParametersInfoW(SPI_GETWORKAREA, 0, Some(&mut rc as *mut _ as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));
    }
    rc
}

/// Colocación inicial: una columna a la derecha y otra a su lado.
fn default_layout(cfg: &mut Config) {
    let wa = work_area();
    let k = unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem() } as f32 / 96.0;
    let picks: [(&str, u32, &str); 5] = [("reloj", 2, "aire"), ("tiempo", 1, "noche"), ("calendario", 1, "noche"), ("sistema", 0, "noche"), ("disco", 0, "noche")];
    let mut y = wa.top + (32.0 * k) as i32;
    let x = wa.right - (360.0 * k) as i32;
    for (key, layout, theme) in picks {
        let id = cfg.new_id();
        let mut w = WidgetCfg { id, kind: key.into(), x, y, theme: theme.into(), layout, size: 1, accent: "auto".into(), opts: Default::default() };
        let (_, h) = widgets::size(&w, &Data::new());
        if y + (h * k) as i32 > wa.bottom {
            w.x = x - (350.0 * k) as i32;
            w.y = wa.top + (32.0 * k) as i32;
        }
        y += ((h + 16.0) * k) as i32;
        cfg.widgets.push(w);
    }
}

impl App {
    fn wcfg(&self, id: u32) -> Option<&WidgetCfg> {
        self.cfg.widgets.iter().find(|w| w.id == id)
    }
    fn wcfg_mut(&mut self, id: u32) -> Option<&mut WidgetCfg> {
        self.cfg.widgets.iter_mut().find(|w| w.id == id)
    }
    fn id_of(&self, hwnd: HWND) -> Option<u32> {
        self.wins.iter().find(|(_, w)| w.hwnd == hwnd).map(|(id, _)| *id)
    }
    fn has(&self, k: Kind) -> bool {
        self.cfg.widgets.iter().any(|w| widgets::meta(&w.kind).kind == k)
    }

    fn create_window(&mut self, id: u32) {
        let Some(w) = self.wcfg(id).cloned() else { return };
        unsafe {
            let inst = GetModuleHandleW(None).unwrap_or_default();
            let title = HSTRING::from(format!("AFRWIDGETS · {}", widgets::meta(&w.kind).name));
            let Ok(hwnd) = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                w!("AFRWidget"),
                &title,
                WS_POPUP,
                w.x,
                w.y,
                10,
                10,
                None,
                None,
                inst,
                None,
            ) else {
                return;
            };
            self.wins.insert(id, Win { hwnd, hits: Vec::new(), stamp: String::new(), scale: 1.0 });
            self.render(id, true);
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            let _ = SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
        self.ensure_visible(id);
    }

    /// Si un widget quedó fuera de todas las pantallas (otro monitor desconectado), lo trae de vuelta.
    fn ensure_visible(&mut self, id: u32) {
        let Some(win) = self.wins.get(&id) else { return };
        let mut rc = RECT::default();
        unsafe {
            let _ = GetWindowRect(win.hwnd, &mut rc);
            let center = POINT { x: (rc.left + rc.right) / 2, y: (rc.top + rc.bottom) / 2 };
            if MonitorFromPoint(center, MONITOR_DEFAULTTONULL).is_invalid() {
                let wa = work_area();
                if let Some(w) = self.wcfg_mut(id) {
                    w.x = wa.left + 40;
                    w.y = wa.top + 40;
                }
                self.render(id, true);
            }
        }
    }

    fn render(&mut self, id: u32, force: bool) {
        let Some(w) = self.wcfg(id).cloned() else { return };
        let Some(win) = self.wins.get(&id) else { return };
        let stamp = widgets::stamp(&w, &self.data)
            + &format!("{}{}{}{}{}", w.theme, w.layout, w.size, w.accent, serde_json::to_string(&w.opts).unwrap_or_default());
        if !force && stamp == win.stamp {
            return;
        }
        let hwnd = win.hwnd;
        let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as f32 / 96.0;
        let scale = w.scale() * dpi;
        let (dw, dh) = widgets::size(&w, &self.data);
        let (pw, ph) = ((dw * scale).ceil() as i32, (dh * scale).ceil() as i32);
        let style = style_for(&w, self.accent);
        let data = &self.data;
        let mut hits = Vec::new();
        let dib = self.gfx.paint(pw, ph, scale, style.halo, &mut |c| hits = widgets::draw(c, style, &w, data));
        if let Some(dib) = dib {
            dib.present(hwnd, w.x, w.y);
        }
        if let Some(win) = self.wins.get_mut(&id) {
            win.hits = hits;
            win.stamp = stamp;
            win.scale = scale;
        }
    }

    fn render_all(&mut self, force: bool) {
        let ids: Vec<u32> = self.wins.keys().copied().collect();
        for id in ids {
            self.render(id, force);
        }
    }

    fn remove(&mut self, id: u32) {
        if let Some(win) = self.wins.remove(&id) {
            unsafe {
                let _ = DestroyWindow(win.hwnd);
            }
        }
        self.cfg.widgets.retain(|w| w.id != id);
        self.cfg.save();
        self.sync_media();
    }

    fn add(&mut self, key: &str) {
        let wa = work_area();
        let id = self.cfg.new_id();
        let n = self.cfg.widgets.len() as i32;
        let w = WidgetCfg {
            id,
            kind: key.into(),
            x: (wa.left + wa.right) / 2 - 160 + (n % 5) * 24,
            y: (wa.top + wa.bottom) / 2 - 120 + (n % 5) * 24,
            theme: "noche".into(),
            layout: 0,
            size: 1,
            accent: "auto".into(),
            opts: Default::default(),
        };
        self.cfg.widgets.push(w);
        self.cfg.save();
        self.create_window(id);
        self.sync_media();
        self.tick();
    }

    // ---------- Datos ----------

    fn sync_media(&mut self) {
        let want = self.has(Kind::Media);
        if want && self.media_tx.is_none() {
            let (tx, rx) = channel();
            let waker = self.waker.clone();
            let tx2 = tx.clone();
            std::thread::spawn(move || media::run(waker, rx, tx2));
            self.media_tx = Some(tx);
        } else if !want {
            if let Some(tx) = self.media_tx.take() {
                let _ = tx.send(media::In::Stop);
            }
            self.data.media = None;
        }
    }

    fn fetch_weather(&mut self) {
        if self.weather_busy {
            return;
        }
        self.weather_busy = true;
        self.weather_at = self.data.now.unix;
        let (lat, lon, waker) = (self.cfg.lat, self.cfg.lon, self.waker.clone());
        std::thread::spawn(move || waker.send(Msg::Weather(weather::fetch(lat, lon))));
    }

    fn fetch_calendar(&mut self) {
        if self.cal_busy || self.cfg.google_token.is_empty() {
            return;
        }
        let Some(creds) = google::creds(&self.cfg.google_client_id, &self.cfg.google_client_secret) else { return };
        let Some(token) = secure::unprotect(&self.cfg.google_token) else {
            self.data.cal = CalState::Error("REAUTH".into());
            return;
        };
        self.cal_busy = true;
        self.cal_at = self.data.now.unix;
        let waker = self.waker.clone();
        std::thread::spawn(move || waker.send(Msg::Calendar(google::fetch(&creds, &token))));
    }

    fn drain(&mut self) {
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Msg::Weather(r) => {
                    self.weather_busy = false;
                    match r {
                        Ok(w) => {
                            self.data.weather = Some(w);
                            self.data.weather_err = None;
                        }
                        Err(e) => {
                            self.data.weather_err = Some(e);
                            // Reintento en 5 minutos.
                            self.weather_at = self.data.now.unix - 25 * 60;
                        }
                    }
                }
                Msg::Place(r) => match r {
                    Ok(p) => {
                        self.cfg.place = p.name.clone();
                        self.cfg.lat = p.lat;
                        self.cfg.lon = p.lon;
                        self.data.place = p.name;
                        self.data.lat = p.lat;
                        self.data.lon = p.lon;
                        self.data.weather = None;
                        self.cfg.save();
                        self.weather_busy = false;
                        self.fetch_weather();
                    }
                    Err(e) => ui::info("AFRWIDGETS", &format!("No he podido encontrar esa ciudad: {e}")),
                },
                Msg::Calendar(r) => {
                    self.cal_busy = false;
                    match r {
                        Ok(ev) => {
                            self.data.events = ev;
                            self.data.cal = CalState::Ready;
                        }
                        Err(e) => {
                            self.data.cal = CalState::Error(e);
                            self.cal_at = self.data.now.unix - 8 * 60;
                        }
                    }
                }
                Msg::Google(r) => match r {
                    Ok(refresh) => {
                        self.cfg.google_token = secure::protect(&refresh).unwrap_or_default();
                        self.cfg.save();
                        self.data.cal = CalState::Ready;
                        self.cal_busy = false;
                        self.fetch_calendar();
                    }
                    Err(e) => {
                        self.data.cal = CalState::Disconnected;
                        ui::info("AFRWIDGETS", &format!("No se ha podido conectar con Google: {e}"));
                    }
                },
                Msg::Media(m) => self.data.media = m,
            }
        }
        self.render_all(false);
    }

    fn focus_tick(&mut self) {
        let now = self.data.now.unix;
        let today = self.data.now.days();
        let mut done = Vec::new();
        for w in self.cfg.widgets.iter_mut().filter(|w| w.kind == "foco") {
            let end = w.i("end", 0);
            if end > 0 && now >= end {
                let was_break = w.s("fase") == "descanso";
                if !was_break {
                    let n = if w.i("dia", 0) == today { w.i("hechos", 0) } else { 0 };
                    w.set("hechos", json!(n + 1));
                    w.set("dia", json!(today));
                }
                w.set("fase", json!(if was_break { "foco" } else { "descanso" }));
                w.set("end", json!(0));
                w.set("left", json!(0));
                done.push(was_break);
            }
        }
        for was_break in done {
            unsafe {
                let _ = windows::Win32::System::Diagnostics::Debug::MessageBeep(MB_ICONASTERISK);
            }
            if was_break {
                self.balloon("Descanso terminado", "Clic en el widget de foco para empezar otra vuelta.");
            } else {
                self.balloon("¡Bloque de foco completado!", "Toca descansar. Clic en el widget cuando quieras empezar el descanso.");
            }
            self.cfg.save();
        }
    }

    /// Latido: actualiza sólo los datos que algún widget necesita y repinta lo que haya cambiado.
    pub fn tick(&mut self) {
        self.data.now = time::now();
        let now = self.data.now.unix;
        if (self.has(Kind::System) || self.has(Kind::Net)) && now - self.sys_at >= 2 {
            self.sys_at = now;
            if self.has(Kind::System) {
                if let Some(c) = self.cpu.sample() {
                    self.data.cpu = c;
                    Data::push(&mut self.data.cpu_hist, c);
                }
                self.data.mem = sys::memory();
                let m = self.data.mem.0 as f32 / self.data.mem.1 as f32;
                Data::push(&mut self.data.mem_hist, m);
            }
            if self.has(Kind::Net) {
                if let Some(v) = self.net.sample() {
                    self.data.net = v;
                    Data::push(&mut self.data.net_hist, v);
                }
            }
        }
        if (self.has(Kind::Disk) || self.has(Kind::Battery)) && now - self.disk_at >= 30 {
            self.disk_at = now;
            self.data.disks = sys::disks();
            self.data.battery = sys::battery();
        }
        if (self.has(Kind::Weather) || self.has(Kind::Sky)) && now - self.weather_at >= 30 * 60 {
            self.fetch_weather();
        }
        if self.has(Kind::Calendar) && now - self.cal_at >= 10 * 60 {
            self.fetch_calendar();
        }
        if self.has(Kind::Focus) {
            self.focus_tick();
        }
        self.render_all(false);
        self.schedule();
    }

    /// Programa el siguiente latido justo en la frontera que haga falta (cada segundo sólo si algo lo pide).
    fn schedule(&self) {
        let now = self.data.now.unix;
        let mut wait = 60 - (now % 60);
        for w in &self.cfg.widgets {
            let p = widgets::period(w) as i64;
            wait = wait.min(p - now % p);
        }
        let ms = unsafe { GetSystemTime().wMilliseconds } as i64;
        let delay = (wait * 1000 - ms + 15).max(50) as u32;
        unsafe {
            SetTimer(self.main, TIMER, delay, None);
        }
    }

    // ---------- Bandeja ----------

    fn tray(&self, op: NOTIFY_ICON_MESSAGE) {
        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.main,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAY,
            hIcon: self.icon,
            szTip: wstr("AFRWIDGETS"),
            ..Default::default()
        };
        unsafe {
            let _ = Shell_NotifyIconW(op, &nid);
        }
    }

    fn balloon(&self, title: &str, text: &str) {
        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.main,
            uID: 1,
            uFlags: NIF_INFO,
            szInfo: wstr(text),
            szInfoTitle: wstr(title),
            dwInfoFlags: NIIF_INFO,
            ..Default::default()
        };
        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    fn tray_menu(&self) -> HMENU {
        unsafe {
            let m = CreatePopupMenu().unwrap_or_default();
            let add = CreatePopupMenu().unwrap_or_default();
            for (i, k) in KINDS.iter().enumerate() {
                append(add, MF_STRING, 2000 + i, k.name);
            }
            append(m, MF_POPUP, add.0 as usize, "Añadir widget");
            append(m, MF_SEPARATOR, 0, "");
            append(m, MF_STRING | checked(self.cfg.locked), 2100, "Bloquear posiciones");
            append(m, MF_STRING | checked(autostart_enabled()), 2101, "Iniciar con Windows");
            append(m, MF_STRING, 2105, "Recolocar todos los widgets");
            append(m, MF_SEPARATOR, 0, "");
            append(m, MF_STRING, 2102, "Abrir carpeta de configuración");
            append(m, MF_STRING, 2103, "Web de AFRWIDGETS");
            append(m, MF_SEPARATOR, 0, "");
            append(m, MF_STRING, 2104, "Salir");
            m
        }
    }

    fn tray_cmd(&mut self, cmd: usize) {
        match cmd {
            2000..=2099 => {
                if let Some(k) = KINDS.get(cmd - 2000) {
                    self.add(k.key);
                }
            }
            2100 => {
                self.cfg.locked = !self.cfg.locked;
                self.cfg.save();
            }
            2101 => set_autostart(!autostart_enabled()),
            2102 => {
                let _ = std::fs::create_dir_all(crate::config::dir());
                google::open_url(&crate::config::dir().display().to_string());
            }
            2103 => google::open_url(SITE),
            2104 => unsafe {
                PostQuitMessage(0);
            },
            2105 => {
                self.cfg.widgets.clear();
                for (_, w) in self.wins.drain() {
                    unsafe {
                        let _ = DestroyWindow(w.hwnd);
                    }
                }
                let old = std::mem::take(&mut self.cfg.widgets);
                let _ = old;
                default_layout(&mut self.cfg);
                self.cfg.save();
                let ids: Vec<u32> = self.cfg.widgets.iter().map(|w| w.id).collect();
                for id in ids {
                    self.create_window(id);
                }
                self.sync_media();
                self.tick();
            }
            _ => {}
        }
    }

    // ---------- Menú del widget ----------

    fn widget_menu(&self, id: u32) -> Option<HMENU> {
        let w = self.wcfg(id)?;
        let meta = widgets::meta(&w.kind);
        unsafe {
            let m = CreatePopupMenu().ok()?;
            let th = CreatePopupMenu().ok()?;
            for (i, t) in THEMES.iter().enumerate() {
                append(th, MF_STRING | checked(w.theme == t.1), 1000 + i, t.2);
            }
            append(m, MF_POPUP, th.0 as usize, "Estética");
            if meta.layouts.len() > 1 {
                let lm = CreatePopupMenu().ok()?;
                for (i, l) in meta.layouts.iter().enumerate() {
                    append(lm, MF_STRING | checked(w.layout as usize == i), 1100 + i, &format!("{} · {}", l.0, match l.1 { widgets::Size::S => "S", widgets::Size::M => "M", widgets::Size::L => "L" }));
                }
                append(m, MF_POPUP, lm.0 as usize, "Diseño");
            }
            let ac = CreatePopupMenu().ok()?;
            for (i, a) in ACCENTS.iter().enumerate() {
                append(ac, MF_STRING | checked(w.accent == a.0), 1300 + i, a.1);
            }
            append(m, MF_POPUP, ac.0 as usize, "Color de acento");
            let sz = CreatePopupMenu().ok()?;
            for (i, l) in ["Pequeño", "Mediano", "Grande"].iter().enumerate() {
                append(sz, MF_STRING | checked(w.size as usize == i), 1200 + i, l);
            }
            append(m, MF_POPUP, sz.0 as usize, "Tamaño");
            append(m, MF_SEPARATOR, 0, "");
            match meta.kind {
                Kind::Clock => append(m, MF_STRING | checked(w.b("segundos")), 1400, "Mostrar segundos"),
                Kind::Weather | Kind::Sky => append(m, MF_STRING, 1401, &format!("Cambiar ciudad ({})…", self.cfg.place)),
                Kind::Calendar => {
                    if self.cfg.google_token.is_empty() {
                        append(m, MF_STRING, 1402, "Conectar Google Calendar…");
                    } else {
                        append(m, MF_STRING, 1404, "Actualizar ahora");
                        append(m, MF_STRING, 1403, "Desconectar Google");
                    }
                }
                Kind::Notes => {
                    append(m, MF_STRING, 1405, "Editar texto…");
                    append(m, MF_STRING, 1406, "Cambiar título…");
                }
                Kind::Focus => {
                    append(m, MF_STRING, 1410, "Reiniciar");
                    append(m, MF_STRING, 1415, "Saltar a la siguiente fase");
                    let d = CreatePopupMenu().ok()?;
                    for (i, v) in [15, 25, 45, 50].iter().enumerate() {
                        append(d, MF_STRING | checked(w.i("minutos", 25) == *v), 1420 + i, &format!("Foco de {v} min"));
                    }
                    append(d, MF_SEPARATOR, 0, "");
                    for (i, v) in [5, 10, 15].iter().enumerate() {
                        append(d, MF_STRING | checked(w.i("descanso", 5) == *v), 1430 + i, &format!("Descanso de {v} min"));
                    }
                    append(m, MF_POPUP, d.0 as usize, "Duración");
                }
                Kind::Progress => append(m, MF_STRING, 1407, "Fijar cuenta atrás…"),
                Kind::World => {
                    let cm = CreatePopupMenu().ok()?;
                    let sel = widgets::timew_cities(w);
                    for (i, c) in data::tz::CITIES.iter().enumerate() {
                        append(cm, MF_STRING | checked(sel.contains(&c.0)), 1500 + i, c.1);
                    }
                    append(m, MF_POPUP, cm.0 as usize, "Ciudades");
                }
                _ => {}
            }
            if !matches!(meta.kind, Kind::Disk | Kind::System | Kind::Net | Kind::Battery | Kind::Media) {
                append(m, MF_SEPARATOR, 0, "");
            }
            append(m, MF_STRING, 1601, "Duplicar");
            append(m, MF_STRING | checked(self.cfg.locked), 1602, "Bloquear posiciones");
            append(m, MF_STRING, 1600, "Quitar widget");
            Some(m)
        }
    }
}

fn checked(b: bool) -> MENU_ITEM_FLAGS {
    if b { MF_CHECKED } else { MF_UNCHECKED }
}

unsafe fn append(m: HMENU, flags: MENU_ITEM_FLAGS, id: usize, text: &str) {
    let t = HSTRING::from(text);
    let _ = AppendMenuW(m, flags, id, &t);
}

fn popup(menu: HMENU) -> usize {
    let main = with(|a| a.main).unwrap_or_default();
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(main);
        let cmd = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY, pt.x, pt.y, 0, main, None);
        let _ = DestroyMenu(menu);
        let _ = PostMessageW(main, WM_NULL, WPARAM(0), LPARAM(0));
        cmd.0 as usize
    }
}

/// Órdenes del menú de un widget. Las que abren diálogos lo hacen sin tener prestado el estado.
fn widget_cmd(id: u32, cmd: usize) {
    if cmd == 0 {
        return;
    }
    match cmd {
        1401 => return action(id, Action::SetCity),
        1402 => return action(id, Action::ConnectGoogle),
        1405 => return action(id, Action::EditNote),
        1407 => return action(id, Action::SetCountdown),
        1406 => {
            let cur = with(|a| a.wcfg(id).map(|w| w.s("titulo"))).flatten().unwrap_or_default();
            if let Some(t) = ui::prompt("Título de la nota", "Escribe el título que quieres ver arriba de la nota.", &cur, false) {
                with(|a| {
                    if let Some(w) = a.wcfg_mut(id) {
                        w.set("titulo", json!(t.trim()));
                    }
                });
            }
        }
        1600 => {
            with(|a| a.remove(id));
            return;
        }
        _ => {}
    }
    with(|a| {
        let now = a.data.now.unix;
        let mut recreate = false;
        if let Some(w) = a.wcfg_mut(id) {
            match cmd {
                1000..=1099 => w.theme = THEMES.get(cmd - 1000).map(|t| t.1).unwrap_or("noche").into(),
                1100..=1199 => w.layout = (cmd - 1100) as u32,
                1200..=1299 => w.size = (cmd - 1200) as u32,
                1300..=1399 => w.accent = ACCENTS.get(cmd - 1300).map(|t| t.0).unwrap_or("auto").into(),
                1400 => {
                    let v = !w.b("segundos");
                    w.set("segundos", json!(v));
                }
                1410 => {
                    w.set("end", json!(0));
                    w.set("left", json!(0));
                    w.set("fase", json!("foco"));
                }
                1415 => {
                    let brk = w.s("fase") == "descanso";
                    w.set("fase", json!(if brk { "foco" } else { "descanso" }));
                    w.set("end", json!(0));
                    w.set("left", json!(0));
                }
                1420..=1423 => w.set("minutos", json!([15, 25, 45, 50][cmd - 1420])),
                1430..=1432 => w.set("descanso", json!([5, 10, 15][cmd - 1430])),
                1500..=1599 => {
                    if let Some(c) = data::tz::CITIES.get(cmd - 1500) {
                        let mut sel: Vec<&str> = widgets::timew_cities(w);
                        if let Some(p) = sel.iter().position(|k| *k == c.0) {
                            if sel.len() > 1 {
                                sel.remove(p);
                            }
                        } else if sel.len() < 6 {
                            sel.push(c.0);
                        }
                        let v = sel.join(",");
                        w.set("ciudades", json!(v));
                    }
                }
                1601 => recreate = true,
                1602 => {}
                _ => {}
            }
        }
        let _ = now;
        match cmd {
            1403 => {
                a.cfg.google_token.clear();
                a.data.events.clear();
                a.data.cal = if google::creds(&a.cfg.google_client_id, &a.cfg.google_client_secret).is_some() { CalState::Disconnected } else { CalState::NoCreds };
            }
            1404 => {
                a.cal_busy = false;
                a.fetch_calendar();
            }
            1602 => a.cfg.locked = !a.cfg.locked,
            _ => {}
        }
        if recreate {
            if let Some(mut copy) = a.wcfg(id).cloned() {
                copy.id = a.cfg.new_id();
                copy.x += 32;
                copy.y += 32;
                let nid = copy.id;
                a.cfg.widgets.push(copy);
                a.create_window(nid);
                a.sync_media();
            }
        }
        a.cfg.save();
        a.render(id, true);
        a.schedule();
    });
}

/// Acciones de clic dentro de un widget.
fn action(id: u32, act: Action) {
    match act {
        Action::Media(cmd) => {
            with(|a| {
                if let Some(tx) = &a.media_tx {
                    let _ = tx.send(media::In::Cmd(cmd));
                }
            });
        }
        Action::Focus => {
            with(|a| {
                let now = a.data.now.unix;
                if let Some(w) = a.wcfg_mut(id) {
                    let st = crate::widgets::focus_state(w, &time::now());
                    if st.running {
                        w.set("left", json!(st.left));
                        w.set("end", json!(0));
                    } else {
                        w.set("end", json!(now + st.left as i64));
                        w.set("left", json!(0));
                    }
                }
                a.cfg.save();
                a.render(id, true);
                a.schedule();
            });
        }
        Action::NoteLine(i) => {
            with(|a| {
                if let Some(w) = a.wcfg_mut(id) {
                    let mut lines = crate::widgets::note_lines(w);
                    if let Some(l) = lines.get_mut(i) {
                        let body = l.trim_start_matches("[x]").trim_start_matches("[X]").trim_start_matches("[ ]").trim_start().to_string();
                        let done = l.starts_with("[x]") || l.starts_with("[X]");
                        *l = format!("{} {}", if done { "[ ]" } else { "[x]" }, body);
                    }
                    w.set("texto", json!(lines.join("\n")));
                }
                a.cfg.save();
                a.render(id, true);
            });
        }
        Action::EditNote => {
            let Some((cur, list)) = with(|a| a.wcfg(id).map(|w| (w.s("texto"), w.layout == 1))).flatten() else { return };
            let label = if list { "Una tarea por línea. Clic en una tarea del widget para tacharla." } else { "Escribe lo que quieras tener a la vista." };
            if let Some(t) = ui::prompt("Nota", label, &cur, true) {
                with(|a| {
                    if let Some(w) = a.wcfg_mut(id) {
                        w.set("texto", Value::String(t));
                    }
                    a.cfg.save();
                    a.render(id, true);
                });
            }
        }
        Action::SetCity => {
            let cur = with(|a| a.cfg.place.clone()).unwrap_or_default();
            if let Some(name) = ui::prompt("Ciudad", "¿De qué ciudad quieres el tiempo? (Se busca en Open-Meteo.)", &cur, false) {
                if !name.trim().is_empty() {
                    with(|a| {
                        let waker = a.waker.clone();
                        std::thread::spawn(move || waker.send(Msg::Place(weather::geocode(&name))));
                    });
                }
            }
        }
        Action::SetCountdown => {
            let (fecha, etiqueta) = with(|a| a.wcfg(id).map(|w| (w.s("fecha"), w.s("etiqueta")))).flatten().unwrap_or_default();
            let Some(f) = ui::prompt("Cuenta atrás", "Fecha en formato AAAA-MM-DD (por ejemplo 2026-12-24).", &fecha, false) else { return };
            let f = f.trim().to_string();
            if time::parse_rfc3339(&f, 0).is_none() {
                ui::info("AFRWIDGETS", "Esa fecha no se entiende. Usa AAAA-MM-DD, por ejemplo 2026-12-24.");
                return;
            }
            let label = ui::prompt("Cuenta atrás", "¿Qué esperas? (por ejemplo «las vacaciones»)", &etiqueta, false).unwrap_or(etiqueta);
            with(|a| {
                if let Some(w) = a.wcfg_mut(id) {
                    w.set("fecha", json!(f));
                    w.set("etiqueta", json!(label.trim()));
                }
                a.cfg.save();
                a.render(id, true);
            });
        }
        Action::ConnectGoogle => connect_google(),
    }
}

fn connect_google() {
    let Some((cid, csec, state)) = with(|a| (a.cfg.google_client_id.clone(), a.cfg.google_client_secret.clone(), a.data.cal.clone())) else { return };
    if state == CalState::Connecting {
        return;
    }
    if state == CalState::Ready {
        with(|a| {
            a.cal_busy = false;
            a.fetch_calendar();
        });
        return;
    }
    let creds = match google::creds(&cid, &csec) {
        Some(c) => c,
        None => {
            ui::info(
                "Conectar Google Calendar",
                "Esta copia no trae cliente OAuth de Google. Crea uno de tipo «App de escritorio» en console.cloud.google.com \
(API de Google Calendar activada) y pega aquí su ID y su secreto. Sólo se piden permisos de lectura.",
            );
            let Some(id) = ui::prompt("Google · ID de cliente", "ID de cliente (termina en .apps.googleusercontent.com)", "", false) else { return };
            let Some(secret) = ui::prompt("Google · Secreto de cliente", "Secreto de cliente", "", false) else { return };
            with(|a| {
                a.cfg.google_client_id = id.trim().into();
                a.cfg.google_client_secret = secret.trim().into();
                a.cfg.save();
            });
            google::Creds { id: id.trim().into(), secret: secret.trim().into() }
        }
    };
    with(|a| {
        a.data.cal = CalState::Connecting;
        a.render_all(true);
        let waker = a.waker.clone();
        std::thread::spawn(move || waker.send(Msg::Google(google::connect(&creds))));
    });
}

// ---------- Procedimientos de ventana ----------

extern "system" fn foreground_hook(_: HWINEVENTHOOK, _: u32, hwnd: HWND, _: i32, _: i32, _: u32, _: u32) {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            return;
        }
        let mut buf = [0u16; 64];
        let n = GetClassNameW(hwnd, &mut buf);
        let class = String::from_utf16_lossy(&buf[..n.max(0) as usize]);
        let desktop = class == "WorkerW" || class == "Progman";
        if desktop == DESKTOP_FRONT.load(Ordering::Relaxed) {
            return;
        }
        DESKTOP_FRONT.store(desktop, Ordering::Relaxed);
        // Con el escritorio delante (Win+D), los widgets suben; al volver a una app, bajan otra vez al fondo.
        let hwnds: Vec<HWND> = with(|a| a.wins.values().map(|w| w.hwnd).collect()).unwrap_or_default();
        for h in hwnds {
            if desktop {
                let _ = SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            } else {
                let _ = SetWindowPos(h, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                let _ = SetWindowPos(h, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
    }
}

extern "system" fn main_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER => {
                let _ = KillTimer(hwnd, TIMER);
                if with(|a| a.tick()).is_none() {
                    // Estado ocupado por un menú: reintenta enseguida.
                    SetTimer(hwnd, TIMER, 500, None);
                }
                LRESULT(0)
            }
            WM_APP_DATA => {
                with(|a| a.drain());
                LRESULT(0)
            }
            WM_TRAY => {
                let ev = (lp.0 & 0xffff) as u32;
                if ev == WM_RBUTTONUP || ev == WM_LBUTTONUP {
                    if let Some(m) = with(|a| a.tray_menu()) {
                        let cmd = popup(m);
                        if cmd != 0 {
                            with(|a| a.tray_cmd(cmd));
                        }
                    }
                }
                LRESULT(0)
            }
            WM_SETTINGCHANGE | WM_DWMCOLORIZATIONCOLORCHANGED => {
                with(|a| {
                    a.accent = system_accent();
                    a.render_all(true);
                });
                DefWindowProcW(hwnd, msg, wp, lp)
            }
            WM_DISPLAYCHANGE => {
                with(|a| {
                    let ids: Vec<u32> = a.wins.keys().copied().collect();
                    for id in ids {
                        a.ensure_visible(id);
                    }
                });
                LRESULT(0)
            }
            WM_POWERBROADCAST => {
                with(|a| {
                    a.data.battery = sys::battery();
                    a.render_all(false);
                });
                LRESULT(1)
            }
            WM_ENDSESSION | WM_QUERYENDSESSION => {
                with(|a| a.cfg.save());
                LRESULT(1)
            }
            _ => {
                let tb = with(|a| a.taskbar_msg).unwrap_or(0);
                if tb != 0 && msg == tb {
                    with(|a| a.tray(NIM_ADD));
                    return LRESULT(0);
                }
                DefWindowProcW(hwnd, msg, wp, lp)
            }
        }
    }
}

fn hit_at(a: &App, id: u32, lp: LPARAM) -> Option<Action> {
    let win = a.wins.get(&id)?;
    let x = (lp.0 & 0xffff) as i16 as f32 / win.scale;
    let y = ((lp.0 >> 16) & 0xffff) as i16 as f32 / win.scale;
    win.hits.iter().rev().find(|h| x >= h.x && x <= h.x + h.w && y >= h.y && y <= h.y + h.h).map(|h| h.action)
}

extern "system" fn widget_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
            WM_WINDOWPOSCHANGING => {
                if !DESKTOP_FRONT.load(Ordering::Relaxed) {
                    let pos = &mut *(lp.0 as *mut WINDOWPOS);
                    pos.hwndInsertAfter = HWND_BOTTOM;
                    pos.flags &= !SWP_NOZORDER;
                }
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                with(|a| {
                    if let Some(id) = a.id_of(hwnd) {
                        let mut cur = POINT::default();
                        let _ = GetCursorPos(&mut cur);
                        let w = a.wcfg(id).cloned();
                        if let Some(w) = w {
                            a.drag = Some(Drag { id, cursor: cur, origin: (w.x, w.y), moved: false });
                        }
                    }
                });
                SetCapture(hwnd);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                with(|a| {
                    if a.cfg.locked {
                        return;
                    }
                    let Some(d) = a.drag.as_mut() else { return };
                    let mut cur = POINT::default();
                    let _ = GetCursorPos(&mut cur);
                    let (dx, dy) = (cur.x - d.cursor.x, cur.y - d.cursor.y);
                    if !d.moved && dx.abs() + dy.abs() < 5 {
                        return;
                    }
                    d.moved = true;
                    let (id, ox, oy) = (d.id, d.origin.0, d.origin.1);
                    // Imán a una rejilla de 8 px para alinear widgets con facilidad.
                    let (nx, ny) = ((ox + dx + 4) / 8 * 8, (oy + dy + 4) / 8 * 8);
                    if let Some(w) = a.wcfg_mut(id) {
                        w.x = nx;
                        w.y = ny;
                    }
                    let _ = SetWindowPos(hwnd, None, nx, ny, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
                });
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let _ = ReleaseCapture();
                let act = with(|a| {
                    let d = a.drag.take()?;
                    if d.moved {
                        a.cfg.save();
                        return None;
                    }
                    hit_at(a, d.id, lp).map(|act| (d.id, act))
                })
                .flatten();
                if let Some((id, act)) = act {
                    if act != Action::ConnectGoogle || true {
                        action(id, act);
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONDBLCLK => {
                let target = with(|a| {
                    let id = a.id_of(hwnd)?;
                    let k = widgets::meta(&a.wcfg(id)?.kind).kind;
                    Some((id, k))
                })
                .flatten();
                if let Some((id, Kind::Notes)) = target {
                    action(id, Action::EditNote);
                }
                LRESULT(0)
            }
            WM_RBUTTONUP => {
                if let Some((id, Some(menu))) = with(|a| a.id_of(hwnd).map(|id| (id, a.widget_menu(id)))).flatten() {
                    let cmd = popup(menu);
                    widget_cmd(id, cmd);
                }
                LRESULT(0)
            }
            WM_DPICHANGED => {
                with(|a| {
                    if let Some(id) = a.id_of(hwnd) {
                        a.render(id, true);
                    }
                });
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}

// Reexportes que usan los menús.
impl App {
    #[allow(dead_code)]
    pub fn gfx(&self) -> &Gfx {
        &self.gfx
    }
}
