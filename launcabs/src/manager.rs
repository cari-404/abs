use ::runtime::prepare;
use native_windows_gui as nwg;
use native_windows_derive::NwgUi;
use native_windows_gui::NativeUi;
use std::fs;
use std::fs::File;
use std::io::{Write, Read};
use std::time::Duration;

use crate::new;

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_control(
        size: (500, 300),
        position: (300, 300),
        title: "Accounts Manager",
        //flags: "WINDOW|VISIBLE|MINIMIZE_BOX|SYS_MENU",
        icon: Some(&nwg::Icon::from_bin(include_bytes!("32l.ico")).unwrap()),
        center: true,
    )]
    #[nwg_events( OnWindowClose: [App::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 3)]
    grid: nwg::GridLayout,
    
    #[nwg_control(text: "File")]
    #[nwg_layout_item(layout: grid, col: 0, row: 0)]
    file_label: nwg::Label,

    #[nwg_control(v_align: nwg::VTextAlign::Bottom)]
    #[nwg_events( OnComboxBoxSelection: [App::update])]
    #[nwg_layout_item(layout: grid, col: 1, row: 0, col_span: 4)]
    file_combo: nwg::ComboBox<String>,
    
    #[nwg_control(text: "new?")]
    #[nwg_layout_item(layout: grid, col: 5, row: 0)]
	#[nwg_events(OnButtonClick: [App::open_custom_dialog])]
    cek_button: nwg::Button,

    #[nwg_control(text: "Cookie")]
    #[nwg_layout_item(layout: grid, col: 0, row: 2)]
    payment_label: nwg::Label,

    #[nwg_control(
        text: "",
        flags: "VISIBLE|AUTOVSCROLL|VSCROLL",
        limit: 32767,
    )]
    #[nwg_layout_item(layout: grid, col: 1, row: 2, col_span: 5, row_span: 6)]
    payment_text: nwg::TextBox,
    
    #[nwg_control(text: "FZ")]
    #[nwg_layout_item(layout: grid, col: 0, row: 8)]
    harga_label: nwg::Label,

    #[nwg_control(
        text: "",
        flags: "VISIBLE|AUTOVSCROLL|VSCROLL",
        limit: 32767,
    )]
    #[nwg_layout_item(layout: grid, col: 1, row: 8, col_span: 5, row_span: 2)]
    harga_text: nwg::TextBox,
	
	#[nwg_control(text: "Refresh", flags: "VISIBLE|DISABLED",)]
    #[nwg_layout_item(layout: grid, col: 0, row: 10)]
    #[nwg_events( OnButtonClick: [App::refresh_file_combo])]
    refresh_button: nwg::Button,
	
    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: grid, col: 1, row: 10, col_span: 2)]
    state_label: nwg::Label,
    #[nwg_control(text: "Save")]
    #[nwg_layout_item(layout: grid, col: 4, row: 10)]
	#[nwg_events( OnButtonClick: [App::saved])]
    save_button: nwg::Button,
	#[nwg_control(text: "Cancel")]
    #[nwg_layout_item(layout: grid, col: 5, row: 10)]
	#[nwg_events( OnButtonClick: [App::exit])]
	cancel_button: nwg::Button,

    #[nwg_control(interval: Duration::from_secs(3))]
    #[nwg_events(OnTimerTick: [App::hide_label])]
    timer: nwg::AnimationTimer,
}

impl App {
    fn open_custom_dialog(&self) {
		self.window.set_visible(false);
		new::main("");
		self.window.set_visible(true);
		self.refresh_file_combo();
    }
    fn saved(&self) {
        self.state_label.set_visible(true);
        if let Some(file) = self.file_combo.selection_string() {
            if !file.is_empty() {
                let file_path = format!("./akun/{}", file);
                let fp_folder = format!("./header/{}/af-ac-enc-sz-token.txt", file);
    
                // Create the directory if it does not exist
                if let Err(e) = fs::create_dir_all(format!("./header/{}", file)) {
                    eprintln!("Failed to create directory: {}", e);
                    self.state_label.set_text("Failed to create directory");
                    return;
                }
    
                // Create the file
                let mut file = match File::create(&file_path) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("Failed to create file: {}", e);
                        self.state_label.set_text("Failed to create file");
                        return;
                    }
                };
                // Create the file
                let mut file_fp = match File::create(&fp_folder) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("Failed to create file: {}", e);
                        self.state_label.set_text("Failed to create file");
                        return;
                    }
                };
    
                // Write content to the file
                let content_cookie = self.payment_text.text();
                let content_fp = self.harga_text.text();
                if let Err(e) = file.write_all(content_cookie.as_bytes()) {
                    eprintln!("Failed to write content_cookie to file: {}", e);
                    self.state_label.set_text("Failed to write content_cookie to file");
                    return;
                }
    
                if let Err(e) = file_fp.write_all(content_fp.as_bytes()) {
                    eprintln!("Failed to write content_fp to file: {}", e);
                    self.state_label.set_text("Failed to write content_fp to file");
                    return;
                }
    
                self.state_label.set_text("Save file Success");
            } else {
                self.state_label.set_text("No file selected");
            }
        } else {
            self.state_label.set_text("No file selected");
        }
        self.state_label.set_visible(true);
        self.timer.start();
    }
    fn hide_label(&self) {
        self.state_label.set_visible(false);
        self.timer.stop();
    }
	fn update(&self) {
		if let Some(file) = self.file_combo.selection_string() {
			if !file.is_empty() {
                let cookie_content = prepare::read_cookie_file(&file);
                let fp_folder = format!("./header/{}/af-ac-enc-sz-token.txt", &file);
                // Create the header folder if it doesn't exist
                if let Err(e) = fs::create_dir_all(&format!("./header/{}", &file)) {
                    eprintln!("Failed to create directory: {}", e);
                    return;
                }

                // Check if the file exists, and create it if not
                if File::open(&fp_folder).is_err() {
                    if let Err(e) = File::create(&fp_folder) {
                        eprintln!("Failed to create file: {}", e);
                        return;
                    }
                }

                // Read the contents of the sz-token file
                let mut sz_token_content = String::new();
                if let Ok(mut file) = File::open(&fp_folder) {
                    if let Err(e) = file.read_to_string(&mut sz_token_content) {
                        eprintln!("Failed to read file: {}", e);
                        return;
                    }
                } else {
                    eprintln!("Failed to open file: {}", fp_folder);
                    return;
                }
                println!("sz-token:{}", sz_token_content);
				self.payment_text.set_text(&cookie_content);
				self.harga_text.set_text(&sz_token_content);
			}
		}
	}
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
	fn refresh_file_combo(&self) {
		let empty_collection: Vec<String> = Vec::new();
		self.file_combo.set_collection(empty_collection);
		self.populate_file_combo();
	}
	fn populate_file_combo(&self) {
		let folder_path = "akun";
		let files = get_file_names(folder_path);

		if !files.is_empty() {
			println!("Reading folder akun");
			println!("Available file");
			for file_name in &files {
				println!("{}", file_name);
			}
			self.file_combo.set_collection(files.clone());
			self.file_combo.set_selection(Some(0));
        } else {
            let p = nwg::MessageParams {
				title: "Folder not found",
				content: "Folder akun tidak ada.\n \nHarap buat folder bernama akun",
				buttons: nwg::MessageButtons::Ok,
				icons: nwg::MessageIcons::Warning
			};
			assert!(nwg::modal_message(&self.file_combo, &p) == nwg::MessageChoice::Ok);
			println!("Failed to read the folder contents");
        }
    }
}

pub fn main() {
    nwg::init().expect("Failed to initialize native windows gui");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let app = App::build_ui(Default::default()).expect("Failed to build UI");
	app.populate_file_combo();
	app.update();
    nwg::dispatch_thread_events();
}

pub fn get_file_names(folder_path: &str) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.metadata().map(|m| m.is_file()).unwrap_or(false) {
                    if let Some(file_name) = entry.file_name().to_str() {
                        files.push(file_name.to_string());
                    }
                }
            }
        }
    }
    files
}