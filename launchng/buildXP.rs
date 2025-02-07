use std::env;
extern crate winresource;

#[cfg(target_env = "gnu")]
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let res_path = env::current_dir()
            .unwrap()
            .join("resources")
            .join("resources_coff.res");

        println!("cargo:rerun-if-changed={}", res_path.display());
        println!("cargo:rustc-link-arg={}", res_path.display());
        let mut res = winresource::WindowsResource::new();
        res.set_version_info(winresource::VersionInfo::FILEVERSION, 0x0000000A000A0001);
        res.compile().unwrap();
    }
}
#[cfg(target_env = "msvc")]
fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86" || target_arch == "x86_64" {
        thunk::thunk();
    }
    let res_path = env::current_dir()
        .unwrap()
        .join("resources")
        .join("resources.res");

    println!("cargo:rerun-if-changed={}", res_path.display());
    println!("cargo:rustc-link-arg={}", res_path.display());
    let mut res = winresource::WindowsResource::new();
    res.compile().unwrap();
}