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
        res.set_version_info(winresource::VersionInfo::FILEVERSION, 0x0000000A000A0000);
        res.compile().unwrap();
    }
}
#[cfg(target_env = "msvc")]
fn main() {
    println!("cargo:rustc-link-arg=resources/resources.res");
}