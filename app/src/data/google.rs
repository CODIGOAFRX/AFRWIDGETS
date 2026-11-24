//! Google Calendar con OAuth 2.0 para apps de escritorio: redirección a 127.0.0.1 y PKCE.
//! Ámbito de sólo lectura. El token de refresco se guarda cifrado con DPAPI (ver secure.rs).

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use serde_json::Value;
use windows::core::*;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use super::{http, secure};
use crate::time;

const SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub title: String,
    pub start: i64,
    pub end: i64,
    pub all_day: bool,
    pub color: u32,
}

#[derive(Clone)]
pub struct Creds {
    pub id: String,
    pub secret: String,
}

/// Credenciales del cliente OAuth: las del archivo de configuración o, si no, las del momento de compilar.
pub fn creds(cfg_id: &str, cfg_secret: &str) -> Option<Creds> {
    if !cfg_id.is_empty() {
        return Some(Creds { id: cfg_id.into(), secret: cfg_secret.into() });
    }
    match (option_env!("AFR_GOOGLE_CLIENT_ID"), option_env!("AFR_GOOGLE_CLIENT_SECRET")) {
        (Some(id), Some(s)) if !id.is_empty() => Some(Creds { id: id.into(), secret: s.into() }),
        _ => None,
    }
}

pub fn open_url(url: &str) {
    let u = HSTRING::from(url);
    unsafe {
        ShellExecuteW(None, w!("open"), &u, None, None, SW_SHOWNORMAL);
    }
}

const PAGE: &str = "<!doctype html><meta charset=utf-8><title>AFRWIDGETS</title>\
<body style=\"font:16px/1.5 'Segoe UI',sans-serif;background:#111215;color:#f4f4f1;display:grid;place-items:center;height:100vh;margin:0\">\
<main style=\"max-width:28rem;padding:2rem\"><h1 style=\"font-weight:400;font-size:1.6rem\">{T}</h1><p style=\"color:#a9a9a5\">{P}</p></main>";

/// Abre el navegador, espera el código y lo cambia por un token de refresco. Bloquea hasta 3 minutos.
pub fn connect(c: &Creds) -> std::result::Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let redirect = format!("http://127.0.0.1:{port}");
    let verifier = secure::b64(&secure::random(48), true);
    let challenge = secure::b64(&secure::sha256(verifier.as_bytes()), true);
    let state = secure::b64(&secure::random(16), true);
    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}\
&code_challenge={}&code_challenge_method=S256&state={}&access_type=offline&prompt=consent",
        http::encode(&c.id),
        http::encode(&redirect),
        http::encode(SCOPE),
        challenge,
        state
    );
    open_url(&url);

    let deadline = Instant::now() + Duration::from_secs(180);
    let code = loop {
        if Instant::now() > deadline {
            return Err("Se acabó el tiempo para iniciar sesión".into());
        }
        let (mut sock, _) = match listener.accept() {
            Ok(s) => s,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(150));
                continue;
            }
        };
        let _ = sock.set_nonblocking(false);
        let _ = sock.set_read_timeout(Some(Duration::from_secs(5)));
        let mut buf = [0u8; 8192];
        let n = sock.read(&mut buf).unwrap_or(0);
        let req = String::from_utf8_lossy(&buf[..n]);
        let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
        let query = path.split_once('?').map(|x| x.1).unwrap_or("");
        let get = |k: &str| {
            query.split('&').find_map(|kv| kv.split_once('=').filter(|(a, _)| *a == k).map(|(_, b)| http::decode(b)))
        };
        let (title, para, result) = if let Some(err) = get("error") {
            ("No se ha conectado", "Google ha cancelado el permiso. Puedes cerrar esta pestaña.", Some(Err(err)))
        } else if let (Some(code), Some(st)) = (get("code"), get("state")) {
            if st == state {
                ("Calendario conectado", "Ya puedes cerrar esta pestaña y volver al escritorio.", Some(Ok(code)))
            } else {
                ("No se ha conectado", "La respuesta no coincide con la petición.", Some(Err("estado".into())))
            }
        } else {
            ("", "", None)
        };
        let Some(result) = result else {
            let _ = sock.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            continue;
        };
        let body = PAGE.replace("{T}", title).replace("{P}", para);
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = sock.write_all(resp.as_bytes());
        break result?;
    };

    let r = http::post_form(
        "https://oauth2.googleapis.com/token",
        &[
            ("code", &code),
            ("client_id", &c.id),
            ("client_secret", &c.secret),
            ("redirect_uri", &redirect),
            ("grant_type", "authorization_code"),
            ("code_verifier", &verifier),
        ],
    )?;
    let v: Value = serde_json::from_str(&r.body).map_err(|e| e.to_string())?;
    v["refresh_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| v["error_description"].as_str().unwrap_or("Google no ha devuelto el token").to_string())
}

fn access_token(c: &Creds, refresh: &str) -> std::result::Result<String, String> {
    let r = http::post_form(
        "https://oauth2.googleapis.com/token",
        &[("client_id", &c.id), ("client_secret", &c.secret), ("refresh_token", refresh), ("grant_type", "refresh_token")],
    )?;
    let v: Value = serde_json::from_str(&r.body).map_err(|e| e.to_string())?;
    if let Some(t) = v["access_token"].as_str() {
        return Ok(t.to_string());
    }
    if v["error"].as_str() == Some("invalid_grant") {
        return Err("REAUTH".into());
    }
    Err(v["error_description"].as_str().unwrap_or("No se pudo renovar el acceso").to_string())
}

fn hex_color(s: &str) -> u32 {
    u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap_or(0x4285F4)
}

/// Eventos desde el día 1 del mes actual hasta dos meses después, de todos los calendarios visibles.
pub fn fetch(c: &Creds, refresh: &str) -> std::result::Result<Vec<Event>, String> {
    let token = access_token(c, refresh)?;
    let auth = format!("Bearer {token}");
    let hdr = [("Authorization", auth.as_str())];
    let list = http::get(
        "https://www.googleapis.com/calendar/v3/users/me/calendarList?minAccessRole=reader&fields=items(id,backgroundColor,selected,primary)",
        &hdr,
    )?;
    if list.status == 401 {
        return Err("REAUTH".into());
    }
    let lv: Value = serde_json::from_str(&list.body).map_err(|e| e.to_string())?;
    let now = time::now();
    let t0 = time::local_midnight_unix(now.y, now.mo, 1, now.offset);
    let t1 = t0 + 62 * 86400;
    let mut out = Vec::new();
    let items = lv["items"].as_array().cloned().unwrap_or_default();
    for cal in items.iter().filter(|c| c["selected"].as_bool().unwrap_or(false) || c["primary"].as_bool().unwrap_or(false)).take(8) {
        let id = cal["id"].as_str().unwrap_or("primary");
        let color = hex_color(cal["backgroundColor"].as_str().unwrap_or("#4285F4"));
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events?singleEvents=true&orderBy=startTime&maxResults=120\
&timeMin={}&timeMax={}&fields=items(summary,start,end,status)",
            http::encode(id),
            http::encode(&time::rfc3339_utc(t0)),
            http::encode(&time::rfc3339_utc(t1))
        );
        let Ok(r) = http::get(&url, &hdr) else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&r.body) else { continue };
        for e in v["items"].as_array().cloned().unwrap_or_default() {
            if e["status"].as_str() == Some("cancelled") {
                continue;
            }
            let when = |k: &str| {
                e[k]["dateTime"].as_str().or(e[k]["date"].as_str()).and_then(|s| time::parse_rfc3339(s, now.offset))
            };
            let (Some((start, all_day)), Some((end, _))) = (when("start"), when("end")) else { continue };
            out.push(Event {
                title: e["summary"].as_str().unwrap_or("(Sin título)").to_string(),
                start,
                end,
                all_day,
                color,
            });
        }
    }
    out.sort_by_key(|e| (e.start, !e.all_day));
    Ok(out)
}
