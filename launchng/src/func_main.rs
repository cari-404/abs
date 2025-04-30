use std::{fs, io::{Write, Read}};
use winsafe::{self as w,
    gui, path, prelude::*, co::{self, SW, SEE_MASK},
};
use ::runtime::prepare::{self};
use std::{ffi::CStr, ptr, io::{self, Error}};
use std::sync::{Arc, Mutex};
use chrono::{Local, DateTime, Utc};
use windows_sys::Win32::System::DataExchange::*;
use windows_sys::Win32::System::Memory::*;

pub fn error_modal(wnd: &gui::WindowModal, title: &str, message: &str) -> Result<(), ()> {
    wnd.hwnd().MessageBox(message, title, co::MB::OK | co::MB::ICONSTOP).ok();
    Ok(())
}
pub fn info_modal(wnd: &gui::WindowModal, title: &str, message: &str) -> Result<(), ()> {
    wnd.hwnd().MessageBox(message, title, co::MB::OK | co::MB::ICONINFORMATION).ok();
    Ok(())
}

pub fn error_cek(wnd: &gui::WindowMain, title: &str, message: &str) -> Result<(), ()> {
    wnd.hwnd().MessageBox(message, title, co::MB::OK | co::MB::ICONSTOP).ok();
    Ok(())
}
pub fn get_fp_data(file: &str) -> String {
    let mut sz_token_content = String::new();
    let header_dir = format!("./header/{}", file);
    let fp_folder = format!("{}/af-ac-enc-sz-token.txt", &header_dir);
    if path::exists(&header_dir) == false {
        if let Err(e) = fs::create_dir_all(&header_dir) {
            eprintln!("Failed to create directory: {}", e);
            return Default::default();
        }
    }
    if fs::File::open(&fp_folder).is_err() {
        if let Err(e) = fs::File::create(&fp_folder) {
            eprintln!("Failed to create file: {}", e);
            return Default::default();
        }
    }
    if let Ok(mut file) = fs::File::open(&fp_folder) {
        if let Err(e) = file.read_to_string(&mut sz_token_content) {
            eprintln!("Failed to read file: {}", e);
            return Default::default();
        }
    } else {
        eprintln!("Failed to open file: {}", fp_folder);
        return Default::default();
    }
    println!("sz-token:{}", sz_token_content);
    sz_token_content
}
pub fn handle_file_selection(file_combo: &gui::ComboBox, cookie_edit: &gui::Edit, sz_edit: &gui::Edit, my_list: &gui::ListView) {
    if let Some(file) = file_combo.items().selected_text() {
        println!("selected file: {}", file);
        if !file.is_empty() {
            my_list.items().delete_all();
            let cookie_content = prepare::read_cookie_file(&file);
            let cookie_data = prepare::create_cookie(&cookie_content);
            let sz_token_content = get_fp_data(&file);
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
pub fn populate_payment_combo(combo: &gui::ComboBox, shared_variation_clone: Arc<Mutex<Vec<prepare::PaymentInfo>>>) {
    combo.items().delete_all();
    // Buka file "payment.txt"
    if let Ok(mut file) = fs::File::open("payment.txt") {
        let mut json_data = String::new();
        let _ = file.read_to_string(&mut json_data);
        match prepare::get_payment(&json_data) {
            Ok(payment_info_list) => {
                let mut shared = shared_variation_clone.lock().unwrap();
                shared.clear();
                *shared = payment_info_list.clone(); 
                for payment_info in payment_info_list {
                    combo.items().add(&[payment_info.name.to_string()]);
                    if combo.items().count() > 0 {
                        combo.items().select(Some(0));
                    }
                }
            }
            Err(e) => {
                println!("Error parsing payment info: {}", e);
            }
        }
    } else {
        println!("Failed to read the file contents");
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
pub fn format_thousands(num: i64) -> String {
    let num_str = num.to_string();
    let mut formatted = String::new();
    let len = num_str.len();
    for (i, c) in num_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted.push('.');
        }
        formatted.push(c);
    }
    formatted
}
pub fn human_readable_time(epoch: i64) -> DateTime<Local> {
    let utc = DateTime::<Utc>::from_timestamp(epoch, 0).expect("Invalid timestamp");
    utc.with_timezone(&Local)
}
const CF_UNICODETEXT: u32 = 13;
pub fn set_clipboard(text: &str) -> io::Result<()> {
    let utf16: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let bytes = unsafe {
        std::slice::from_raw_parts(
            utf16.as_ptr() as *const u8,
            utf16.len() * std::mem::size_of::<u16>(),
        )
    };

    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err(Error::last_os_error());
        }
        EmptyClipboard();

        let hglobal = GlobalAlloc(GMEM_MOVEABLE, bytes.len());
        if hglobal.is_null() {
            CloseClipboard();
            return Err(Error::last_os_error());
        }

        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            CloseClipboard();
            return Err(Error::last_os_error());
        }

        ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());

        SetClipboardData(CF_UNICODETEXT, hglobal);
        CloseClipboard();
    }
    Ok(())
}
pub fn open_url(url: &str) -> winsafe::AnyResult<()> {
    let mut exec_info = w::SHELLEXECUTEINFO {
        mask: SEE_MASK::default(), // bisa juga pakai SEE_MASK::NOASYNC | lainnya
        hwnd: None,
        verb: Some("open"),
        file: url,
        parameters: None,
        directory: None,
        show: SW::SHOWNORMAL,
        id_list: None,
        class: None,
        hkey_class: None,
        hot_key: None,
        hicon_hmonitor: Default::default(), // penting! karena ini enum, harus di-set
    };
    w::ShellExecuteEx(&mut exec_info)?;
    Ok(())
}