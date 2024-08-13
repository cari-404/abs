use native_windows_gui as nwg;
use native_windows_derive::NwgUi;
use native_windows_gui::NativeUi;

use crate::SharedData;

#[derive(Default, NwgUi)]
pub struct LogWindow {
    #[nwg_control(size: (300, 500), position: (950, 150), title: "Log Window")]
    #[nwg_events(OnWindowClose: [LogWindow::close])]
    pub window: nwg::Window,

    #[nwg_control(size: (280, 490), position: (10, 10), readonly: true)]
    log_box: nwg::TextBox,
}

impl LogWindow {
    pub fn append_log(&self, log: &str) {
        let mut text = self.log_box.text();
        text.push_str(log);
        text.push_str("\n");
        self.log_box.set_text(&text);
    }

    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}
pub fn main() {
    nwg::init().expect("Failed to initialize native windows gui");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let app = LogWindow::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}