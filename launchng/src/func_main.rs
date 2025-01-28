use std::{fs, io::{Write, Read}};
use winsafe::{self as w,
    gui, path, prelude::*, co::{self, SW},
};
use ::runtime::prepare::{self};
use serde_json::{Value};
use std::ffi::CStr;
pub fn error_modal(wnd: &gui::WindowModal, title: &str, message: &str) -> Result<(), ()> {
    wnd.hwnd().MessageBox(message, title, co::MB::OK | co::MB::ICONSTOP).ok();
    Ok(())
}

pub fn error_cek(wnd: &gui::WindowMain, title: &str, message: &str) -> Result<(), ()> {
    wnd.hwnd().MessageBox(message, title, co::MB::OK | co::MB::ICONSTOP).ok();
    Ok(())
}
pub fn handle_file_selection(file_combo: &gui::ComboBox, cookie_edit: &gui::Edit, sz_edit: &gui::Edit, my_list: &gui::ListView) {
    if let Some(file) = file_combo.items().selected_text() {
        println!("selected file: {}", file);
        if !file.is_empty() {
            my_list.items().delete_all();
            let cookie_content = prepare::read_cookie_file(&file);
            let cookie_data = prepare::create_cookie(&cookie_content);
            let header_dir = format!("./header/{}", file);
            let fp_folder = format!("{}/af-ac-enc-sz-token.txt", &header_dir);
            if path::exists(&header_dir) == false {
                if let Err(e) = fs::create_dir_all(&header_dir) {
                    eprintln!("Failed to create directory: {}", e);
                    return;
                }
            }
            if fs::File::open(&fp_folder).is_err() {
                if let Err(e) = fs::File::create(&fp_folder) {
                    eprintln!("Failed to create file: {}", e);
                    return;
                }
            }
            let mut sz_token_content = String::new();
            if let Ok(mut file) = fs::File::open(&fp_folder) {
                if let Err(e) = file.read_to_string(&mut sz_token_content) {
                    eprintln!("Failed to read file: {}", e);
                    return;
                }
            } else {
                eprintln!("Failed to open file: {}", fp_folder);
                return;
            }
            println!("sz-token:{}", sz_token_content);
            sz_edit.set_text(&sz_token_content);
            cookie_edit.set_text(&cookie_content);
            my_list.items().add(
                &[
                    "CSRFTOKEN",
                    &cookie_data.csrftoken,
                ],
                None, // no icon; requires a previous set_image_list()
                (),   // no object data; requires specifying the generic `ListView` type
            );
        }
    } else {
        println!("No file selected");
    }
}

pub fn populate_combobox_with_files(combo: &gui::ComboBox, folder_path: &str) {
    combo.items().delete_all();
    let files = get_file_names(folder_path);
    if path::exists(folder_path) == true {
        match files {
            Ok(files) => {
                if !files.is_empty() {
                    println!("Reading folder: {}", folder_path);
                    println!("Available files:");
                    for file_name in files {
                        println!("{}", file_name);
                        combo.items().add(&[&file_name]); // Add file to combobox.
                    }
                    if combo.items().count() > 0 {
                        combo.items().select(Some(0));
                    }
                } else {
                    println!("No .txt files found in the folder.");
                }
            }
            Err(err) => {
                println!("Failed to read the folder contents: {:?}", err);
            }
        }
    } else {
        println!("Folder not found.");
    }
}
pub fn get_file_names(dir: &str) -> Result<Vec<String>, co::ERROR> {
    let mut files: Vec<String> = Vec::new();
    let filter = Some("*.txt"); // Filter untuk mencari file dengan ekstensi .txt

    // Iterasi file dalam direktori
    for file_path in path::dir_list(dir, filter) {
        let file_path = file_path?; // Tangani setiap hasil
        if let Some(file_name) = path::get_file_name(&file_path) {
            files.push(file_name.to_string());
        }
    }

    Ok(files)
}
pub fn populate_payment_combo(combo: &gui::ComboBox) {
    combo.items().delete_all();
    // Buka file "payment.txt"
    if let Ok(mut file) = fs::File::open("payment.txt") {
        let mut json_data = String::new();
        let _ = file.read_to_string(&mut json_data);
        let hasil: Value = serde_json::from_str(&json_data).expect("REASON");
        if let Some(data) = hasil.get("data").and_then(|data| data.as_array()) {
            for data_value in data {
                if let Some(payment_array) = data_value.get("payment").and_then(|payment| payment.as_array()) {
                    for payment_value in payment_array {
                        if let Some(payment_obj) = payment_value.as_object() {
                            if let Some(name) = payment_obj.get("name").and_then(|v| v.as_str()) {
                                combo.items().add(&[name.to_string()]);
                                if combo.items().count() > 0 {
                                    combo.items().select(Some(0));
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("Failed to read the folder contents");
    }
}
pub fn detect_wine() -> Result<String, Box<dyn std::error::Error>> {
    let hntdll = w::HINSTANCE::GetModuleHandle(Some("ntdll.dll"))?;
    let run_win = match hntdll.GetProcAddress("wine_get_version") {
        Ok(pwine_get_version) => {
            let wine_get_version: fn() -> *const i8 = unsafe { 
                std::mem::transmute(pwine_get_version) 
            };
            let wine_version_ptr = wine_get_version();
            let c_str = unsafe { CStr::from_ptr(wine_version_ptr) };
            let a = format!("Wine {}", c_str.to_string_lossy());
            a
        },
        Err(_) => {
            "Windows NT".to_string()
        },
    };
    Ok(run_win)
}
pub fn save_cookie_fp_file(file: &str, content_cookie: &str, content_fp: &str) -> Result<(), Box<dyn std::error::Error>> {
    /*let mut file = fs::File::create(file_name)?;
    file.write_all(data.as_bytes())?;*/
    let file_path = format!("./akun/{}", file);
    let fp_folder = format!("./header/{}/af-ac-enc-sz-token.txt", file);

    // Create the directory if it does not exist
    if let Err(e) = fs::create_dir_all(format!("./header/{}", file)) {
        eprintln!("Failed to create directory: {}", e);
        return Ok(());
    }

    // Create the file
    let mut file = match fs::File::create(&file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create file: {}", e);
            return Ok(());
        }
    };
    // Create the file
    let mut file_fp = match fs::File::create(&fp_folder) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create file: {}", e);
            return Ok(());
        }
    };

    // Write content to the file
    if let Err(e) = file.write_all(content_cookie.as_bytes()) {
        eprintln!("Failed to write content_cookie to file: {}", e);
        return Ok(());
    }

    if let Err(e) = file_fp.write_all(content_fp.as_bytes()) {
        eprintln!("Failed to write content_fp to file: {}", e);
        return Ok(());
    }
    Ok(())
}
pub fn set_visibility(label: &gui::Label, text: &gui::Edit, visible: bool) {
    label.hwnd().ShowWindow(if visible { SW::SHOW } else { SW::HIDE });
    text.hwnd().ShowWindow(if visible { SW::SHOW } else { SW::HIDE });
}
