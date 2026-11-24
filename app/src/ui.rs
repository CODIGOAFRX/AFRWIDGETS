//! Diálogo de texto nativo, pequeño y modal (con los controles comunes v6 del manifiesto).

use std::cell::RefCell;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::EM_SETSEL;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::*;

thread_local! {
    static RESULT: RefCell<Option<Option<String>>> = const { RefCell::new(None) };
    static EDIT: RefCell<HWND> = RefCell::new(HWND::default());
}

const ID_OK: usize = 1;
const ID_CANCEL: usize = 2;

extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_COMMAND => {
                let id = wp.0 & 0xffff;
                if id == ID_OK {
                    let edit = EDIT.with(|e| *e.borrow());
                    let len = GetWindowTextLengthW(edit);
                    let mut buf = vec![0u16; len as usize + 1];
                    let n = GetWindowTextW(edit, &mut buf);
                    let s = String::from_utf16_lossy(&buf[..n as usize]).replace("\r\n", "\n");
                    RESULT.with(|r| *r.borrow_mut() = Some(Some(s)));
                    let _ = DestroyWindow(hwnd);
                } else if id == ID_CANCEL {
                    RESULT.with(|r| *r.borrow_mut() = Some(None));
                    let _ = DestroyWindow(hwnd);
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                RESULT.with(|r| *r.borrow_mut() = Some(None));
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_CTLCOLORSTATIC => {
                SetBkMode(HDC(wp.0 as *mut _), TRANSPARENT);
                LRESULT(GetSysColorBrush(COLOR_WINDOW).0 as isize)
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}

/// Pide un texto. Devuelve None si se cancela.
pub fn prompt(title: &str, label: &str, initial: &str, multiline: bool) -> Option<String> {
    unsafe {
        let inst = GetModuleHandleW(None).ok()?;
        let class = w!("AFRPrompt");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: inst.into(),
            lpszClassName: class,
            hbrBackground: GetSysColorBrush(COLOR_WINDOW),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassW(&wc);
        let k = GetDpiForSystem() as f32 / 96.0;
        let px = |v: f32| (v * k) as i32;
        let (w, h) = (px(440.0), px(if multiline { 360.0 } else { 196.0 }));
        let (sw, sh) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        let t = HSTRING::from(title);
        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME | WS_EX_TOPMOST,
            class,
            &t,
            WS_POPUP | WS_CAPTION | WS_SYSMENU,
            (sw - w) / 2,
            (sh - h) / 2,
            w,
            h,
            None,
            None,
            inst,
            None,
        )
        .ok()?;
        let font = CreateFontW(-px(15.0), 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, CLEARTYPE_QUALITY.0 as u32, 0, w!("Segoe UI"));
        let mut rc = RECT::default();
        let _ = GetClientRect(hwnd, &mut rc);
        let cw = rc.right;
        let ch = rc.bottom;
        let m = px(18.0);
        let child = |class: PCWSTR, text: &str, style: WINDOW_STYLE, ex: WINDOW_EX_STYLE, x: i32, y: i32, w: i32, h: i32, id: usize| {
            let txt = HSTRING::from(text);
            let c = CreateWindowExW(ex, class, &txt, WS_CHILD | WS_VISIBLE | style, x, y, w, h, hwnd, HMENU(id as *mut _), inst, None)
                .unwrap_or_default();
            SendMessageW(c, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
            c
        };
        let label_h = px(44.0);
        child(w!("STATIC"), label, WINDOW_STYLE(0), WINDOW_EX_STYLE(0), m, m, cw - 2 * m, label_h, 0);
        let bh = px(32.0);
        let eh = if multiline { ch - m * 3 - label_h - bh } else { px(30.0) };
        let es = if multiline {
            WINDOW_STYLE((ES_MULTILINE | ES_WANTRETURN | ES_AUTOVSCROLL) as u32) | WS_VSCROLL | WS_TABSTOP
        } else {
            WINDOW_STYLE(ES_AUTOHSCROLL as u32) | WS_TABSTOP
        };
        let edit = child(w!("EDIT"), &initial.replace('\n', "\r\n"), es, WS_EX_CLIENTEDGE, m, m + label_h, cw - 2 * m, eh, 100);
        EDIT.with(|e| *e.borrow_mut() = edit);
        let bw = px(104.0);
        child(w!("BUTTON"), "Cancelar", WINDOW_STYLE(BS_PUSHBUTTON as u32) | WS_TABSTOP, WINDOW_EX_STYLE(0), cw - m - bw, ch - m - bh, bw, bh, ID_CANCEL);
        child(w!("BUTTON"), "Guardar", WINDOW_STYLE(BS_DEFPUSHBUTTON as u32) | WS_TABSTOP, WINDOW_EX_STYLE(0), cw - m - 2 * bw - px(8.0), ch - m - bh, bw, bh, ID_OK);
        SendMessageW(edit, EM_SETSEL, WPARAM(0), LPARAM(-1));
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(edit);
        RESULT.with(|r| *r.borrow_mut() = None);
        let mut msg = MSG::default();
        while RESULT.with(|r| r.borrow().is_none()) {
            if !GetMessageW(&mut msg, None, 0, 0).as_bool() {
                PostQuitMessage(0);
                break;
            }
            // Intro en una línea = Guardar; Esc = Cancelar; Tab entre controles.
            if !IsDialogMessageW(hwnd, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        let _ = DeleteObject(font);
        RESULT.with(|r| r.borrow_mut().take()).flatten()
    }
}

pub fn info(title: &str, text: &str) {
    let t = HSTRING::from(title);
    let x = HSTRING::from(text);
    unsafe {
        MessageBoxW(None, &x, &t, MB_OK | MB_ICONINFORMATION | MB_TOPMOST);
    }
}

pub fn confirm(title: &str, text: &str) -> bool {
    let t = HSTRING::from(title);
    let x = HSTRING::from(text);
    unsafe { MessageBoxW(None, &x, &t, MB_OKCANCEL | MB_ICONQUESTION | MB_TOPMOST) == IDOK }
}
