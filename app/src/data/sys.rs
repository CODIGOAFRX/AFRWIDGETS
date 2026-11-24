//! Datos del propio Windows: CPU, memoria, discos, red y batería.

use windows::core::*;
use windows::Win32::Foundation::FILETIME;
use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIfEntry2, GetIfTable2, MIB_IF_ROW2, MIB_IF_TABLE2};
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Threading::GetSystemTimes;

fn ft(f: FILETIME) -> u64 {
    ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64
}

#[derive(Default)]
pub struct CpuSampler {
    last: Option<(u64, u64)>,
}

impl CpuSampler {
    /// Uso de CPU (0..1) desde la última muestra.
    pub fn sample(&mut self) -> Option<f32> {
        let (mut idle, mut kernel, mut user) = (FILETIME::default(), FILETIME::default(), FILETIME::default());
        unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).ok()? };
        let total = ft(kernel) + ft(user);
        let idle = ft(idle);
        let r = self.last.and_then(|(t0, i0)| {
            let dt = total.saturating_sub(t0);
            if dt == 0 {
                return None;
            }
            Some(1.0 - (idle.saturating_sub(i0)) as f32 / dt as f32)
        });
        self.last = Some((total, idle));
        r.map(|v| v.clamp(0.0, 1.0))
    }
}

/// (usada, total) en bytes.
pub fn memory() -> (u64, u64) {
    let mut m = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
    unsafe {
        if GlobalMemoryStatusEx(&mut m).is_err() {
            return (0, 1);
        }
    }
    (m.ullTotalPhys - m.ullAvailPhys, m.ullTotalPhys.max(1))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Disk {
    pub letter: char,
    pub label: String,
    pub total: u64,
    pub free: u64,
}

pub fn disks() -> Vec<Disk> {
    let mut out = Vec::new();
    let mask = unsafe { GetLogicalDrives() };
    for i in 0..26u32 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let letter = (b'A' + i as u8) as char;
        let root = HSTRING::from(format!("{}:\\", letter));
        unsafe {
            if GetDriveTypeW(&root) != 3 {
                // Sólo discos fijos (DRIVE_FIXED).
                continue;
            }
            let (mut avail, mut total, mut free) = (0u64, 0u64, 0u64);
            if GetDiskFreeSpaceExW(&root, Some(&mut avail), Some(&mut total), Some(&mut free)).is_err() || total == 0 {
                continue;
            }
            let mut name = [0u16; 64];
            let _ = GetVolumeInformationW(&root, Some(&mut name), None, None, None, None);
            let len = name.iter().position(|&c| c == 0).unwrap_or(0);
            let label = String::from_utf16_lossy(&name[..len]);
            out.push(Disk { letter, label, total, free: avail });
        }
    }
    out
}

#[derive(Default)]
pub struct NetSampler {
    last: Option<(u64, u64, std::time::Instant)>,
    /// Interfaces físicas conectadas; la tabla completa sólo se relee cada minuto.
    ifaces: Vec<u32>,
    listed: Option<std::time::Instant>,
}

impl NetSampler {
    /// Bytes por segundo (bajada, subida) desde la última muestra.
    pub fn sample(&mut self) -> Option<(f64, f64)> {
        let now = std::time::Instant::now();
        if self.listed.map_or(true, |t| now.duration_since(t).as_secs() >= 60) {
            self.ifaces = physical_ifaces();
            self.listed = Some(now);
            self.last = None;
        }
        let (mut rx, mut tx) = (0u64, 0u64);
        for &idx in &self.ifaces {
            let mut row = MIB_IF_ROW2 { InterfaceIndex: idx, ..Default::default() };
            unsafe {
                if GetIfEntry2(&mut row).is_ok() {
                    rx += row.InOctets;
                    tx += row.OutOctets;
                }
            }
        }
        let r = self.last.map(|(r0, t0, i0)| {
            let dt = now.duration_since(i0).as_secs_f64().max(0.001);
            (rx.saturating_sub(r0) as f64 / dt, tx.saturating_sub(t0) as f64 / dt)
        });
        self.last = Some((rx, tx, now));
        r
    }
}

fn physical_ifaces() -> Vec<u32> {
    let mut out = Vec::new();
    unsafe {
        let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
        if GetIfTable2(&mut table).is_err() || table.is_null() {
            return out;
        }
        let n = (*table).NumEntries as usize;
        let rows = std::slice::from_raw_parts((*table).Table.as_ptr(), n);
        for r in rows {
            let hardware = r.InterfaceAndOperStatusFlags._bitfield & 1 != 0;
            // 24 = loopback. Sólo interfaces físicas y conectadas, para no contar dos veces los virtuales.
            if r.Type != 24 && hardware && r.OperStatus == IfOperStatusUp {
                out.push(r.InterfaceIndex);
            }
        }
        FreeMibTable(table as *const _);
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Battery {
    pub present: bool,
    pub percent: u8,
    pub charging: bool,
    pub plugged: bool,
    /// Segundos restantes, si Windows lo sabe.
    pub remaining: Option<u32>,
    pub saver: bool,
}

pub fn battery() -> Battery {
    let mut s = SYSTEM_POWER_STATUS::default();
    unsafe {
        if GetSystemPowerStatus(&mut s).is_err() {
            return Battery::default();
        }
    }
    Battery {
        present: s.BatteryFlag != 128 && s.BatteryFlag != 255,
        percent: if s.BatteryLifePercent == 255 { 0 } else { s.BatteryLifePercent },
        charging: s.BatteryFlag & 8 != 0,
        plugged: s.ACLineStatus == 1,
        remaining: if s.BatteryLifeTime == u32::MAX { None } else { Some(s.BatteryLifeTime) },
        saver: s.SystemStatusFlag == 1,
    }
}

pub fn fmt_bytes(b: f64) -> String {
    const U: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = b;
    let mut i = 0;
    while v >= 1000.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if v >= 100.0 || i == 0 {
        format!("{:.0} {}", v, U[i])
    } else if v >= 10.0 {
        format!("{:.1} {}", v, U[i]).replace('.', ",")
    } else {
        format!("{:.2} {}", v, U[i]).replace('.', ",")
    }
}

pub fn fmt_rate(b: f64) -> String {
    format!("{}/s", fmt_bytes(b))
}
