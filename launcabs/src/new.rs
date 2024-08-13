use native_windows_gui as nwg;
use native_windows_derive::NwgUi;
use native_windows_gui::NativeUi;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize)]
struct Translations {
    window_title: String,
    file_label: String,
    save_button: String,
    cancel_button: String,
    language_label: String,
}

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_control(
        size: (500, 300),
        position: (300, 300),
        title: "Create an account file",
        icon: Some(&nwg::Icon::from_bin(include_bytes!("new.ico")).unwrap()),
        center: true,
    )]
    #[nwg_events(OnWindowClose: [App::exit])]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 3)]
    grid: nwg::GridLayout,

    #[nwg_control(text: "File Name")]
    #[nwg_layout_item(layout: grid, col: 0, row: 0)]
    file_label: nwg::Label,

    #[nwg_control]
    #[nwg_layout_item(layout: grid, col: 1, row: 0, col_span: 2)]
    file_name: nwg::TextInput,

    #[nwg_control(text: "Save")]
    #[nwg_layout_item(layout: grid, col: 1, row: 1)]
    #[nwg_events(OnButtonClick: [App::saved])]
    save_button: nwg::Button,

    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: grid, col: 0, row: 1)]
    new_label: nwg::Label,

    #[nwg_control(text: "Cancel")]
    #[nwg_layout_item(layout: grid, col: 2, row: 1)]
    #[nwg_events(OnButtonClick: [App::exit])]
    cancel_button: nwg::Button,

    #[nwg_control(text: "Select Language")]
    #[nwg_layout_item(layout: grid, col: 0, row: 2)]
    language_label: nwg::Label,

    #[nwg_control(collection: vec!["English".to_string(), "Indonesian".to_string()])]
    #[nwg_layout_item(layout: grid, col: 1, row: 2)]
    #[nwg_events(OnComboxBoxSelection: [App::change_language])]
    language_combo: nwg::ComboBox<String>,

    #[nwg_resource(source_file: Some("new.ico"))]
    icon: nwg::Icon,
}

impl App {
    fn saved(&self) {
        let file_name = self.file_name.text();
        let folder_path = "akun";
        let full_path = format!("{}/{}.txt", folder_path, file_name);

        // Ensure the folder exists
        if !Path::new(folder_path).exists() {
            if let Err(e) = std::fs::create_dir(folder_path) {
                nwg::modal_error_message(
                    &self.window,
                    "Error",
                    &format!("Failed to create folder: {}", e),
                );
                return;
            }
        }

        // Create an empty file
        match File::create(&full_path) {
            Ok(_) => {
                nwg::modal_info_message(&self.window, "Success", "File saved successfully.");
                nwg::stop_thread_dispatch();
            }
            Err(e) => {
                nwg::modal_error_message(
                    &self.window,
                    "Error",
                    &format!("Failed to create file: {}", e),
                );
            }
        }
    }
    fn populate_default(&self) {
		self.new_label.set_visible(false);
		self.language_combo.set_selection(Some(0));
	}
    fn populate_new(&self) {
        self.new_label.set_text("123");
    }
    fn exit(&self) {
        if self.new_label.text() == "123" {
            nwg::stop_thread_dispatch();
        }
        nwg::stop_thread_dispatch();
    }
    fn change_language(&self) {
		if let Some(selected_language) = self.language_combo.selection_string() {
			let texts = get_texts_for_language(&selected_language);
			self.window.set_text(&texts.window_title);
			self.file_label.set_text(&texts.file_label);
			self.save_button.set_text(&texts.save_button);
			self.cancel_button.set_text(&texts.cancel_button);
			self.language_label.set_text(&texts.language_label);
		}
    }
}

pub fn main(baru: &str) {
    nwg::init().expect("Failed to initialize native windows gui");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let app = App::build_ui(Default::default()).expect("Failed to build UI");
	app.populate_default();
    if baru == "ayo" {
        app.populate_new();
    }
    nwg::dispatch_thread_events();
}

fn get_texts_for_language(language: &str) -> Translations {
    let file_name = match language {
        "Indonesian" => "lang/id.json",
        _ => "lang/en.json",
    };

    let mut file = File::open(file_name).expect("Failed to open translation file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read translation file");

    serde_json::from_str(&contents).expect("Failed to parse translation file")
}
