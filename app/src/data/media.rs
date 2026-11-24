//! Lo que suena ahora, a través de los controles multimedia de Windows (Spotify, navegador, etc.).

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

use super::{Msg, Waker};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Media {
    pub title: String,
    pub artist: String,
    pub app: String,
    pub playing: bool,
    /// Posición y duración en segundos (0 si la app no lo publica).
    pub pos: u32,
    pub len: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd {
    Toggle,
    Next,
    Prev,
}

fn app_name(id: &str) -> String {
    let base = id.rsplit(['\\', '!']).next().unwrap_or(id).trim_end_matches(".exe");
    let lower = base.to_lowercase();
    for (k, v) in [
        ("spotify", "Spotify"),
        ("chrome", "Chrome"),
        ("msedge", "Edge"),
        ("firefox", "Firefox"),
        ("zunemusic", "Reproductor multimedia"),
        ("vlc", "VLC"),
        ("applemusic", "Apple Music"),
        ("tidal", "TIDAL"),
    ] {
        if lower.contains(k) {
            return v.into();
        }
    }
    base.to_string()
}

fn read(mgr: &Manager) -> Option<Media> {
    let s = mgr.GetCurrentSession().ok()?;
    let props = s.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let playing = s.GetPlaybackInfo().ok().and_then(|i| i.PlaybackStatus().ok()) == Some(Status::Playing);
    let (pos, len) = s
        .GetTimelineProperties()
        .ok()
        .and_then(|t| Some(((t.Position().ok()?.Duration / 10_000_000) as u32, (t.EndTime().ok()?.Duration / 10_000_000) as u32)))
        .unwrap_or((0, 0));
    Some(Media {
        title: props.Title().ok()?.to_string_lossy(),
        artist: props.Artist().map(|a| a.to_string_lossy()).unwrap_or_default(),
        app: s.SourceAppUserModelId().map(|a| app_name(&a.to_string_lossy())).unwrap_or_default(),
        playing,
        pos,
        len,
    })
}

pub enum In {
    Cmd(Cmd),
    Wake,
    Stop,
}

/// Hilo que vigila la sesión multimedia. Se despierta con los eventos de Windows (cambio de canción,
/// de reproducción o de app) y, mientras suena algo, cada 5 s para mover la barra de progreso.
/// Termina cuando se cierra el canal.
pub fn run(waker: Waker, rx: Receiver<In>, tx: Sender<In>) {
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
    let Ok(op) = Manager::RequestAsync() else { return };
    let Ok(mgr) = op.get() else { return };
    let t = tx.clone();
    let _ = mgr.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
        let _ = t.send(In::Wake);
        Ok(())
    }));
    let mut subscribed: Option<(String, Session, EventRegistrationToken, EventRegistrationToken)> = None;
    let mut last: Option<Option<Media>> = None;
    loop {
        let cur = mgr.GetCurrentSession().ok();
        let cur_id = cur.as_ref().and_then(|s| s.SourceAppUserModelId().ok()).map(|h| h.to_string_lossy());
        if subscribed.as_ref().map(|x| &x.0) != cur_id.as_ref() {
            if let Some((_, s, a, b)) = subscribed.take() {
                let _ = s.RemoveMediaPropertiesChanged(a);
                let _ = s.RemovePlaybackInfoChanged(b);
            }
            if let (Some(s), Some(id)) = (cur.clone(), cur_id.clone()) {
                let (t1, t2) = (tx.clone(), tx.clone());
                let a = s.MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
                    let _ = t1.send(In::Wake);
                    Ok(())
                }));
                let b = s.PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
                    let _ = t2.send(In::Wake);
                    Ok(())
                }));
                if let (Ok(a), Ok(b)) = (a, b) {
                    subscribed = Some((id, s, a, b));
                }
            }
        }
        let m = read(&mgr);
        let playing = m.as_ref().map(|m| m.playing).unwrap_or(false);
        if last.as_ref() != Some(&m) {
            waker.send(Msg::Media(m.clone()));
            last = Some(m);
        }
        let wait = Duration::from_secs(if playing { 5 } else { 60 });
        match rx.recv_timeout(wait) {
            Ok(In::Cmd(cmd)) => {
                if let Ok(s) = mgr.GetCurrentSession() {
                    let r = match cmd {
                        Cmd::Toggle => s.TryTogglePlayPauseAsync(),
                        Cmd::Next => s.TrySkipNextAsync(),
                        Cmd::Prev => s.TrySkipPreviousAsync(),
                    };
                    if let Ok(op) = r {
                        let _ = op.get();
                    }
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Ok(In::Wake) => {
                // Las apps disparan varios eventos seguidos: se agrupan.
                std::thread::sleep(Duration::from_millis(200));
                while let Ok(i) = rx.try_recv() {
                    if !matches!(i, In::Wake) {
                        let _ = tx.send(i);
                        break;
                    }
                }
            }
            Ok(In::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}
