use std::fs::{File, create_dir_all};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Telemetry {
    pub enabled: bool,
    pub name: String,
}

impl Default for Telemetry {
    fn default() -> Self {
        Telemetry {
            enabled: false,
            name: String::new(),
        }
    }
}
impl Telemetry {
    pub fn new(app_name: &str) -> Self {
        let log_dir = "logs";
        create_dir_all(log_dir).unwrap();
        let name = format!(
            "{}/{}-{}-{}.log",
            log_dir,
            app_name,
            env!("CARGO_PKG_VERSION"),
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        );
        writeln!(File::create(&name).unwrap(), "runtime version : {}", env!("CARGO_PKG_VERSION")).expect("Gagal menulis ke file log");
        Self {
            enabled: true,
            name,
        }
    }
    pub fn write(&self, message: &str) {
        if self.enabled {
            let mut file = File::options()
                .append(true)
                .open(&self.name)
                .unwrap();
            writeln!(file, "[Telemetry] {}", message).unwrap();
        }
    }
}