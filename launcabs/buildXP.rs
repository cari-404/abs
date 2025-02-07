extern crate winresource;
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
		let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
		if target_arch == "x86" || target_arch == "x86_64" {
			thunk::thunk();
		}
        let mut res = winresource::WindowsResource::new();
        res.set_icon("launcher.ico");
		res.set_manifest(r#"
		<?xml version="1.0" encoding="UTF-8" standalone="yes"?> 
		<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
			<description>native-windows-gui comctl32 manifest</description> 
			<dependency>
				<dependentAssembly>
					<assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" /> 
				</dependentAssembly>
			</dependency>
		</assembly>
		"#);
		res.set_version_info(winresource::VersionInfo::FILEVERSION, 0x0000000900090001);
        res.compile().unwrap();
    }
}