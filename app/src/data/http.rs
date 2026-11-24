//! Cliente HTTPS mínimo sobre WinHTTP (viene con Windows, no añade peso).

use windows::core::*;
use windows::Win32::Networking::WinHttp::*;

pub struct Response {
    pub status: u32,
    pub body: String,
}

struct H(*mut std::ffi::c_void);
impl Drop for H {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = WinHttpCloseHandle(self.0);
            }
        }
    }
}

pub fn get(url: &str, headers: &[(&str, &str)]) -> std::result::Result<Response, String> {
    request("GET", url, headers, None)
}

pub fn post_form(url: &str, form: &[(&str, &str)]) -> std::result::Result<Response, String> {
    let body = form.iter().map(|(k, v)| format!("{}={}", k, encode(v))).collect::<Vec<_>>().join("&");
    request("POST", url, &[("Content-Type", "application/x-www-form-urlencoded")], Some(body.as_bytes()))
}

pub fn request(
    method: &str,
    url: &str,
    headers: &[(&str, &str)],
    body: Option<&[u8]>,
) -> std::result::Result<Response, String> {
    let rest = url.strip_prefix("https://").ok_or("sólo https")?;
    let (host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    unsafe {
        let session = H(WinHttpOpen(
            w!("AFRWIDGETS/1.0"),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        ));
        if session.0.is_null() {
            return Err("WinHttpOpen".into());
        }
        let _ = WinHttpSetTimeouts(session.0, 8000, 8000, 10000, 15000);
        let hhost = HSTRING::from(host);
        let conn = H(WinHttpConnect(session.0, &hhost, INTERNET_DEFAULT_HTTPS_PORT, 0));
        if conn.0.is_null() {
            return Err("sin conexión".into());
        }
        let hm = HSTRING::from(method);
        let hp = HSTRING::from(path);
        let req = H(WinHttpOpenRequest(conn.0, &hm, &hp, PCWSTR::null(), PCWSTR::null(), std::ptr::null(), WINHTTP_FLAG_SECURE));
        if req.0.is_null() {
            return Err("WinHttpOpenRequest".into());
        }
        let mut hdr = String::new();
        for (k, v) in headers {
            hdr.push_str(&format!("{}: {}\r\n", k, v));
        }
        let hw: Vec<u16> = hdr.encode_utf16().collect();
        let (ptr, len) = match body {
            Some(b) => (b.as_ptr() as *const std::ffi::c_void, b.len() as u32),
            None => (std::ptr::null(), 0),
        };
        WinHttpSendRequest(req.0, if hw.is_empty() { None } else { Some(&hw) }, Some(ptr), len, len, 0)
            .map_err(|e| format!("envío: {e}"))?;
        WinHttpReceiveResponse(req.0, std::ptr::null_mut()).map_err(|e| format!("respuesta: {e}"))?;
        let mut status = 0u32;
        let mut size = 4u32;
        let _ = WinHttpQueryHeaders(
            req.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some(&mut status as *mut u32 as *mut _),
            &mut size,
            std::ptr::null_mut(),
        );
        let mut out = Vec::new();
        loop {
            let mut avail = 0u32;
            if WinHttpQueryDataAvailable(req.0, &mut avail).is_err() || avail == 0 {
                break;
            }
            let mut buf = vec![0u8; avail as usize];
            let mut read = 0u32;
            if WinHttpReadData(req.0, buf.as_mut_ptr() as *mut _, avail, &mut read).is_err() || read == 0 {
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
            if out.len() > 4 * 1024 * 1024 {
                break;
            }
        }
        Ok(Response { status, body: String::from_utf8_lossy(&out).into_owned() })
    }
}

/// Codificación de URL (RFC 3986).
pub fn encode(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => o.push(b as char),
            _ => o.push_str(&format!("%{:02X}", b)),
        }
    }
    o
}

pub fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
                out.push(b'%');
            }
            b'+' => out.push(b' '),
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
