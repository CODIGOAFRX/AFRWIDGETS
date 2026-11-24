//! Render con Direct2D (modo software) sobre un DIB de 32 bits premultiplicado.
//! El DIB es temporal: se crea, se pinta, se entrega a la ventana con UpdateLayeredWindow
//! y se libera. Así cada widget no retiene memoria propia entre repintados.

use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::PI;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn hex(v: u32) -> Color {
        Color {
            r: ((v >> 16) & 0xff) as f32 / 255.0,
            g: ((v >> 8) & 0xff) as f32 / 255.0,
            b: (v & 0xff) as f32 / 255.0,
            a: 1.0,
        }
    }
    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
        Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a }
    }
    pub fn a(self, a: f32) -> Color {
        Color { a: self.a * a, ..self }
    }
    pub fn mix(self, o: Color, t: f32) -> Color {
        Color {
            r: self.r + (o.r - self.r) * t,
            g: self.g + (o.g - self.g) * t,
            b: self.b + (o.b - self.b) * t,
            a: self.a + (o.a - self.a) * t,
        }
    }
    /// Luminancia relativa (WCAG).
    pub fn luminance(self) -> f32 {
        fn ch(c: f32) -> f32 {
            if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        }
        0.2126 * ch(self.r) + 0.7152 * ch(self.g) + 0.0722 * ch(self.b)
    }
    fn d2d(self) -> D2D1_COLOR_F {
        D2D1_COLOR_F { r: self.r, g: self.g, b: self.b, a: self.a }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Align {
    L,
    C,
    R,
}

#[derive(Clone, Copy)]
pub struct Font {
    pub family: &'static str,
    pub size: f32,
    pub weight: u32,
    /// Interletraje en fracción del cuerpo (0,1 = 10 %). Negativo aprieta.
    pub track: f32,
}

impl Font {
    pub const fn new(family: &'static str, size: f32, weight: u32) -> Font {
        Font { family, size, weight, track: 0.0 }
    }
    pub fn tracked(self, track: f32) -> Font {
        Font { track, ..self }
    }
}

pub const UI: &str = "Segoe UI Variable Text";
pub const DISPLAY: &str = "Segoe UI Variable Display";
pub const ICONS: &str = "Segoe Fluent Icons";
/// Grotesca incrustada en el exe (Hanken Grotesk, licencia OFL): no depende de lo que tenga instalado el PC.
pub const GROTESK: &str = "Hanken Grotesk";
static GROTESK_TTF: &[u8] = include_bytes!("../assets/HankenGrotesk.ttf");

pub struct Gfx {
    pub d2d: ID2D1Factory,
    dw: IDWriteFactory,
    formats: RefCell<HashMap<(String, u32, u32), IDWriteTextFormat>>,
    round: ID2D1StrokeStyle,
    typo: IDWriteTypography,
    dcrt: ID2D1DCRenderTarget,
    icon_family: &'static str,
    private: Option<IDWriteFontCollection>,
}

impl Gfx {
    pub fn new() -> Result<Gfx> {
        unsafe {
            let d2d: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let dw: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let round = d2d.CreateStrokeStyle(
                &D2D1_STROKE_STYLE_PROPERTIES {
                    startCap: D2D1_CAP_STYLE_ROUND,
                    endCap: D2D1_CAP_STYLE_ROUND,
                    dashCap: D2D1_CAP_STYLE_ROUND,
                    lineJoin: D2D1_LINE_JOIN_ROUND,
                    miterLimit: 10.0,
                    dashStyle: D2D1_DASH_STYLE_SOLID,
                    dashOffset: 0.0,
                },
                None,
            )?;
            let typo = dw.CreateTypography()?;
            typo.AddFontFeature(DWRITE_FONT_FEATURE {
                nameTag: DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES,
                parameter: 1,
            })?;
            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let dcrt = d2d.CreateDCRenderTarget(&props)?;
            // Windows 11 trae Segoe Fluent Icons; Windows 10, Segoe MDL2 Assets (mismos códigos).
            let icon_family = if font_exists(&dw, ICONS) { ICONS } else { "Segoe MDL2 Assets" };
            let private = private_fonts(&dw);
            Ok(Gfx { d2d, dw, formats: RefCell::new(HashMap::new()), round, typo, dcrt, icon_family, private })
        }
    }

    fn format(&self, f: Font) -> IDWriteTextFormat {
        let family = if f.family == ICONS { self.icon_family } else { f.family };
        let key = (family.to_string(), f.weight, (f.size * 10.0) as u32);
        if let Some(t) = self.formats.borrow().get(&key) {
            return t.clone();
        }
        let fam = HSTRING::from(family);
        let t = unsafe {
            let coll = if family == GROTESK { self.private.as_ref() } else { None };
            self.dw
                .CreateTextFormat(
                    &fam,
                    coll,
                    DWRITE_FONT_WEIGHT(f.weight as i32),
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    f.size,
                    w!("es-es"),
                )
                .expect("formato de texto")
        };
        self.formats.borrow_mut().insert(key, t.clone());
        t
    }

    fn layout(&self, s: &str, f: Font, w: f32, h: f32, align: Align, wrap: bool) -> IDWriteTextLayout {
        let text: Vec<u16> = s.encode_utf16().collect();
        let fmt = self.format(f);
        unsafe {
            let l = self.dw.CreateTextLayout(&text, &fmt, w.max(1.0), h.max(1.0)).expect("layout");
            let _ = l.SetTextAlignment(match align {
                Align::L => DWRITE_TEXT_ALIGNMENT_LEADING,
                Align::C => DWRITE_TEXT_ALIGNMENT_CENTER,
                Align::R => DWRITE_TEXT_ALIGNMENT_TRAILING,
            });
            let _ = l.SetWordWrapping(if wrap { DWRITE_WORD_WRAPPING_WRAP } else { DWRITE_WORD_WRAPPING_NO_WRAP });
            if let Ok(sign) = self.dw.CreateEllipsisTrimmingSign(&fmt) {
                let _ = l.SetTrimming(
                    &DWRITE_TRIMMING {
                        granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER,
                        delimiter: 0,
                        delimiterCount: 0,
                    },
                    &sign,
                );
            }
            let range = DWRITE_TEXT_RANGE { startPosition: 0, length: text.len() as u32 };
            let _ = l.SetTypography(&self.typo, range);
            if f.track != 0.0 {
                if let Ok(l1) = l.cast::<IDWriteTextLayout1>() {
                    let _ = l1.SetCharacterSpacing(0.0, f.track * f.size, 0.0, range);
                }
            }
            l
        }
    }

    /// Ancho y alto del texto en una línea.
    pub fn measure(&self, s: &str, f: Font) -> (f32, f32) {
        let l = self.layout(s, f, 10000.0, 1000.0, Align::L, false);
        let mut m = DWRITE_TEXT_METRICS::default();
        unsafe {
            let _ = l.GetMetrics(&mut m);
        }
        (m.widthIncludingTrailingWhitespace, m.height)
    }

    /// Pinta en un DIB de w×h píxeles a la escala dada y devuelve el DIB listo para usar.
    pub fn paint(&self, w: i32, h: i32, scale: f32, halo: bool, draw: &mut dyn FnMut(&mut Canvas)) -> Option<Dib> {
        let dib = Dib::new(w, h)?;
        unsafe {
            let rc = RECT { left: 0, top: 0, right: w, bottom: h };
            self.dcrt.BindDC(dib.dc, &rc).ok()?;
            self.dcrt.SetDpi(96.0 * scale, 96.0 * scale);
            self.dcrt.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            self.dcrt.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            self.dcrt.BeginDraw();
            // Alfa mínimo en todo el rectángulo: así todo el widget recibe clics aunque no tenga panel.
            self.dcrt.Clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 / 255.0 }));
            let rt: ID2D1RenderTarget = self.dcrt.cast().ok()?;
            let brush = rt.CreateSolidColorBrush(&Color::hex(0xffffff).d2d(), None).ok()?;
            let mut c = Canvas { g: self, rt, brush, halo, w: w as f32 / scale, h: h as f32 / scale };
            draw(&mut c);
            self.dcrt.EndDraw(None, None).ok()?;
        }
        Some(dib)
    }
}

/// Colección privada con la grotesca incrustada (Windows 10 1703 o posterior).
fn private_fonts(dw: &IDWriteFactory) -> Option<IDWriteFontCollection> {
    unsafe {
        let f5: IDWriteFactory5 = dw.cast().ok()?;
        let loader = f5.CreateInMemoryFontFileLoader().ok()?;
        f5.RegisterFontFileLoader(&loader).ok()?;
        let file = loader
            .CreateInMemoryFontFileReference(&f5, GROTESK_TTF.as_ptr() as *const _, GROTESK_TTF.len() as u32, None)
            .ok()?;
        let builder = f5.CreateFontSetBuilder().ok()?;
        builder.AddFontFile(&file).ok()?;
        let set = builder.CreateFontSet().ok()?;
        let coll = f5.CreateFontCollectionFromFontSet(&set).ok()?;
        coll.cast().ok()
    }
}

fn font_exists(dw: &IDWriteFactory, name: &str) -> bool {
    unsafe {
        let mut coll: Option<IDWriteFontCollection> = None;
        if dw.GetSystemFontCollection(&mut coll, false).is_err() {
            return false;
        }
        let Some(coll) = coll else { return false };
        let mut idx = 0u32;
        let mut exists = BOOL(0);
        let n = HSTRING::from(name);
        coll.FindFamilyName(&n, &mut idx, &mut exists).is_ok() && exists.as_bool()
    }
}

pub struct Dib {
    pub dc: HDC,
    bmp: HBITMAP,
    old: HGDIOBJ,
    pub bits: *mut u8,
    pub w: i32,
    pub h: i32,
}

impl Dib {
    fn new(w: i32, h: i32) -> Option<Dib> {
        unsafe {
            let dc = CreateCompatibleDC(None);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let bmp = CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
            let old = SelectObject(dc, bmp);
            Some(Dib { dc, bmp, old, bits: bits as *mut u8, w, h })
        }
    }

    /// Entrega la imagen a una ventana en capas en (x, y).
    pub fn present(&self, hwnd: HWND, x: i32, y: i32) {
        unsafe {
            let pos = POINT { x, y };
            let size = SIZE { cx: self.w, cy: self.h };
            let src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let _ = UpdateLayeredWindow(
                hwnd,
                None,
                Some(&pos),
                Some(&size),
                self.dc,
                Some(&src),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
        }
    }

    /// Copia de los píxeles BGRA premultiplicados.
    pub fn pixels(&self) -> Vec<u8> {
        let n = (self.w * self.h * 4) as usize;
        unsafe { std::slice::from_raw_parts(self.bits, n).to_vec() }
    }

    /// Crea un HICON con estos píxeles.
    pub fn to_icon(&self) -> HICON {
        unsafe {
            let mask = CreateBitmap(self.w, self.h, 1, 1, None);
            let info = ICONINFO { fIcon: TRUE, xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: self.bmp };
            let icon = CreateIconIndirect(&info).unwrap_or_default();
            let _ = DeleteObject(mask);
            icon
        }
    }
}

impl Drop for Dib {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(self.bmp);
            let _ = DeleteDC(self.dc);
        }
    }
}

pub struct Canvas<'a> {
    g: &'a Gfx,
    rt: ID2D1RenderTarget,
    brush: ID2D1SolidColorBrush,
    pub halo: bool,
    pub w: f32,
    pub h: f32,
}

fn pt(x: f32, y: f32) -> D2D_POINT_2F {
    D2D_POINT_2F { x, y }
}

impl<'a> Canvas<'a> {
    fn set(&self, c: Color) {
        unsafe { self.brush.SetColor(&c.d2d()) }
    }

    pub fn gfx(&self) -> &Gfx {
        self.g
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        self.set(c);
        let rect = D2D_RECT_F { left: x, top: y, right: x + w, bottom: y + h };
        unsafe {
            if r > 0.0 {
                let r = r.min(w / 2.0).min(h / 2.0);
                self.rt.FillRoundedRectangle(&D2D1_ROUNDED_RECT { rect, radiusX: r, radiusY: r }, &self.brush);
            } else {
                self.rt.FillRectangle(&rect, &self.brush);
            }
        }
    }

    /// Rectángulo con degradado vertical (de arriba abajo).
    pub fn rect_grad(&self, x: f32, y: f32, w: f32, h: f32, r: f32, top: Color, bottom: Color) {
        unsafe {
            let stops = [
                D2D1_GRADIENT_STOP { position: 0.0, color: top.d2d() },
                D2D1_GRADIENT_STOP { position: 1.0, color: bottom.d2d() },
            ];
            let Ok(coll) = self.rt.CreateGradientStopCollection(&stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP) else {
                return self.rect(x, y, w, h, r, top);
            };
            let props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES { startPoint: pt(x, y), endPoint: pt(x, y + h) };
            let Ok(brush) = self.rt.CreateLinearGradientBrush(&props, None, &coll) else {
                return self.rect(x, y, w, h, r, top);
            };
            let rect = D2D_RECT_F { left: x, top: y, right: x + w, bottom: y + h };
            self.rt.FillRoundedRectangle(&D2D1_ROUNDED_RECT { rect, radiusX: r, radiusY: r }, &brush);
        }
    }

    pub fn rect_stroke(&self, x: f32, y: f32, w: f32, h: f32, r: f32, c: Color, sw: f32) {
        self.set(c);
        let i = sw / 2.0;
        let rect = D2D_RECT_F { left: x + i, top: y + i, right: x + w - i, bottom: y + h - i };
        unsafe {
            self.rt.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT { rect, radiusX: (r - i).max(0.0), radiusY: (r - i).max(0.0) },
                &self.brush,
                sw,
                None,
            );
        }
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, c: Color, sw: f32) {
        if self.halo {
            self.set(Color::rgba(0, 0, 0, 0.28 * c.a));
            unsafe { self.rt.DrawLine(pt(x1, y1 + 1.0), pt(x2, y2 + 1.0), &self.brush, sw + 1.0, &self.g.round) }
        }
        self.set(c);
        unsafe { self.rt.DrawLine(pt(x1, y1), pt(x2, y2), &self.brush, sw, &self.g.round) }
    }

    pub fn circle(&self, cx: f32, cy: f32, r: f32, c: Color) {
        if self.halo && c.a > 0.05 {
            self.set(Color::rgba(0, 0, 0, 0.22 * c.a));
            let e = D2D1_ELLIPSE { point: pt(cx, cy + 1.0), radiusX: r + 0.6, radiusY: r + 0.6 };
            unsafe { self.rt.FillEllipse(&e, &self.brush) }
        }
        self.set(c);
        let e = D2D1_ELLIPSE { point: pt(cx, cy), radiusX: r, radiusY: r };
        unsafe { self.rt.FillEllipse(&e, &self.brush) }
    }

    pub fn ring(&self, cx: f32, cy: f32, r: f32, c: Color, sw: f32) {
        self.set(c);
        let e = D2D1_ELLIPSE { point: pt(cx, cy), radiusX: r, radiusY: r };
        unsafe { self.rt.DrawEllipse(&e, &self.brush, sw, None) }
    }

    /// Arco de circunferencia. Ángulos en grados: 0 arriba, sentido horario.
    pub fn arc(&self, cx: f32, cy: f32, r: f32, start: f32, sweep: f32, c: Color, sw: f32) {
        if sweep.abs() < 0.1 {
            return;
        }
        if sweep.abs() >= 359.9 {
            self.ring(cx, cy, r, c, sw);
            return;
        }
        let p = |deg: f32| {
            let a = (deg - 90.0) * PI / 180.0;
            pt(cx + r * a.cos(), cy + r * a.sin())
        };
        unsafe {
            let Ok(geo) = self.g.d2d.CreatePathGeometry() else { return };
            let Ok(sink) = geo.Open() else { return };
            sink.BeginFigure(p(start), D2D1_FIGURE_BEGIN_HOLLOW);
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: p(start + sweep),
                size: D2D_SIZE_F { width: r, height: r },
                rotationAngle: 0.0,
                sweepDirection: if sweep > 0.0 { D2D1_SWEEP_DIRECTION_CLOCKWISE } else { D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE },
                arcSize: if sweep.abs() > 180.0 { D2D1_ARC_SIZE_LARGE } else { D2D1_ARC_SIZE_SMALL },
            });
            sink.EndFigure(D2D1_FIGURE_END_OPEN);
            let _ = sink.Close();
            self.set(c);
            self.rt.DrawGeometry(&geo, &self.brush, sw, &self.g.round);
        }
    }

    /// Polígono relleno o polilínea.
    pub fn path(&self, pts: &[(f32, f32)], c: Color, stroke: Option<f32>, closed: bool) {
        if pts.len() < 2 {
            return;
        }
        unsafe {
            let Ok(geo) = self.g.d2d.CreatePathGeometry() else { return };
            let Ok(sink) = geo.Open() else { return };
            sink.BeginFigure(
                pt(pts[0].0, pts[0].1),
                if stroke.is_some() { D2D1_FIGURE_BEGIN_HOLLOW } else { D2D1_FIGURE_BEGIN_FILLED },
            );
            for p in &pts[1..] {
                sink.AddLine(pt(p.0, p.1));
            }
            sink.EndFigure(if closed { D2D1_FIGURE_END_CLOSED } else { D2D1_FIGURE_END_OPEN });
            let _ = sink.Close();
            if self.halo && stroke.is_none() && c.a > 0.05 {
                self.set(Color::rgba(0, 0, 0, 0.22 * c.a));
                let m = windows::Foundation::Numerics::Matrix3x2 { M11: 1.0, M12: 0.0, M21: 0.0, M22: 1.0, M31: 0.0, M32: 1.0 };
                self.rt.SetTransform(&m);
                self.rt.FillGeometry(&geo, &self.brush, None);
                self.rt.SetTransform(&windows::Foundation::Numerics::Matrix3x2 { M11: 1.0, M12: 0.0, M21: 0.0, M22: 1.0, M31: 0.0, M32: 0.0 });
            }
            self.set(c);
            match stroke {
                Some(sw) => self.rt.DrawGeometry(&geo, &self.brush, sw, &self.g.round),
                None => self.rt.FillGeometry(&geo, &self.brush, None),
            }
        }
    }

    /// Disco iluminado de la luna: `lit` de 0 a 1 y `waxing` (creciente: luz a la derecha).
    pub fn moon(&self, cx: f32, cy: f32, r: f32, lit: f32, waxing: bool, light: Color, dark: Color) {
        self.circle(cx, cy, r, dark);
        // Terminador: elipse de semieje horizontal |1-2·lit|·r.
        let k = 1.0 - 2.0 * lit; // 1 = nueva, -1 = llena
        let n = 48;
        let mut pts = Vec::with_capacity(n * 2 + 2);
        let side = if waxing { 1.0 } else { -1.0 };
        for i in 0..=n {
            let t = -PI / 2.0 + PI * i as f32 / n as f32;
            pts.push((cx + side * r * t.cos(), cy + r * t.sin()));
        }
        for i in 0..=n {
            let t = PI / 2.0 - PI * i as f32 / n as f32;
            pts.push((cx + side * k * r * t.cos(), cy + r * t.sin()));
        }
        self.path(&pts, light, None, true);
    }

    /// Texto en una caja. Devuelve el ancho pintado.
    pub fn text(&self, s: &str, x: f32, y: f32, w: f32, h: f32, f: Font, c: Color, align: Align) -> f32 {
        self.text_ex(s, x, y, w, h, f, c, align, false)
    }

    pub fn text_wrap(&self, s: &str, x: f32, y: f32, w: f32, h: f32, f: Font, c: Color, align: Align) -> f32 {
        self.text_ex(s, x, y, w, h, f, c, align, true)
    }

    fn text_ex(&self, s: &str, x: f32, y: f32, w: f32, h: f32, f: Font, c: Color, align: Align, wrap: bool) -> f32 {
        if s.is_empty() {
            return 0.0;
        }
        let l = self.g.layout(s, f, w, h, align, wrap);
        unsafe {
            if self.halo {
                self.set(Color::rgba(0, 0, 0, 0.32 * c.a));
                for (dx, dy) in [(0.0, 1.0), (0.0, 1.6), (0.8, 0.8), (-0.8, 0.8)] {
                    self.rt.DrawTextLayout(pt(x + dx, y + dy), &l, &self.brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
                }
            }
            self.set(c);
            self.rt.DrawTextLayout(pt(x, y), &l, &self.brush, D2D1_DRAW_TEXT_OPTIONS_CLIP);
            let mut m = DWRITE_TEXT_METRICS::default();
            let _ = l.GetMetrics(&mut m);
            m.width
        }
    }

    /// Texto de una línea colocado por su línea base (como en una retícula tipográfica). Devuelve el ancho.
    pub fn text_base(&self, s: &str, x: f32, base: f32, w: f32, f: Font, c: Color, align: Align) -> f32 {
        if s.is_empty() {
            return 0.0;
        }
        let l = self.g.layout(s, f, w.max(1.0), f.size * 2.0, align, false);
        unsafe {
            let mut lm = [DWRITE_LINE_METRICS::default(); 1];
            let mut n = 0u32;
            let _ = l.GetLineMetrics(Some(&mut lm), &mut n);
            let y = base - lm[0].baseline;
            if self.halo {
                self.set(Color::rgba(0, 0, 0, 0.32 * c.a));
                for (dx, dy) in [(0.0, 1.0), (0.0, 1.6), (0.8, 0.8), (-0.8, 0.8)] {
                    self.rt.DrawTextLayout(pt(x + dx, y + dy), &l, &self.brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
                }
            }
            self.set(c);
            self.rt.DrawTextLayout(pt(x, y), &l, &self.brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
            let mut m = DWRITE_TEXT_METRICS::default();
            let _ = l.GetMetrics(&mut m);
            m.width.min(w)
        }
    }

    pub fn measure(&self, s: &str, f: Font) -> (f32, f32) {
        self.g.measure(s, f)
    }
}

// ---------- PNG e ICO mínimos (sin dependencias) ----------

fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xffff_ffffu32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &d in data {
        a = (a + d as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// PNG RGBA sin comprimir a partir de BGRA premultiplicado.
pub fn png(w: i32, h: i32, bgra: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((w * h * 4 + h) as usize);
    for y in 0..h as usize {
        raw.push(0);
        for x in 0..w as usize {
            let i = (y * w as usize + x) * 4;
            let a = bgra[i + 3];
            let un = |v: u8| if a == 0 { 0 } else { ((v as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8 };
            // El alfa 1/255 de relleno no se exporta.
            let a2 = if a <= 1 { 0 } else { a };
            raw.extend_from_slice(&[un(bgra[i + 2]), un(bgra[i + 1]), un(bgra[i]), a2]);
        }
    }
    let mut z = vec![0x78, 0x01];
    for (i, chunk) in raw.chunks(65535).enumerate() {
        let last = (i + 1) * 65535 >= raw.len();
        z.push(if last { 1 } else { 0 });
        let n = chunk.len() as u16;
        z.extend_from_slice(&n.to_le_bytes());
        z.extend_from_slice(&(!n).to_le_bytes());
        z.extend_from_slice(chunk);
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut chunk = |kind: &[u8], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut c = kind.to_vec();
        c.extend_from_slice(data);
        out.extend_from_slice(&c);
        out.extend_from_slice(&crc32(&c).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(b"IHDR", &ihdr);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    out
}

/// ICO con varias imágenes PNG dentro.
pub fn ico(images: &[(i32, Vec<u8>)]) -> Vec<u8> {
    let mut out = vec![0, 0, 1, 0];
    out.extend_from_slice(&(images.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * images.len() as u32;
    for (size, data) in images {
        let s = if *size >= 256 { 0 } else { *size as u8 };
        out.extend_from_slice(&[s, s, 0, 0, 1, 0, 32, 0]);
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += data.len() as u32;
    }
    for (_, data) in images {
        out.extend_from_slice(data);
    }
    out
}
