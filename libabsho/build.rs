use chrono::{Datelike, Utc};
extern crate winresource;
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let version_str = std::env::var("CARGO_PKG_VERSION").unwrap();
		let parts: Vec<&str> = version_str.split('.').collect();
		let major = parts.get(0).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
		let minor = parts.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
		let patch = parts.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
		let now = Utc::now();
		let year = now.year() as u16 % 100;
		let month = now.month() as u16;
		let build = year * 100 + month; // contoh: 2025/06 => 2506
		let version_u64 =
			((major as u64) << 48) |
			((minor as u64) << 32) |
			((patch as u64) << 16) |
			(build as u64);
		let mut res = winresource::WindowsResource::new();
		res.set_version_info(winresource::VersionInfo::FILEVERSION, version_u64);
		res.set_version_info(winresource::VersionInfo::PRODUCTVERSION, version_u64);
        res.compile().unwrap();
    }
}