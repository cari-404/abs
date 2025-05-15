use winsafe::{self as w,
    gui, prelude::*
};
use runtime::telegram;
use runtime::prepare;
use runtime::product;
use runtime::prepare::{ProductInfo, FSInfo};
use runtime::upgrade;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::io::{self, BufWriter, AsyncWriteExt};
use futures_util::StreamExt;

use crate::func_main;
use crate::login;
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS: &str = std::env::consts::OS;
const ARCH: &str = std::env::consts::ARCH;

pub fn updater_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let check_version = Arc::new(Mutex::new(String::new()));
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowModal::new_dlg(wnd, 100);
    let info_label = gui::Label::new_dlg(&wnd2, 101, dont_move);
    let progress = gui::ProgressBar::new_dlg(&wnd2, 102, dont_move);
    let download_button = gui::Button::new_dlg(&wnd2, 103, dont_move);
    let cancel_button = gui::Button::new_dlg(&wnd2, 104, dont_move);

    let download_button_clone = download_button.clone();
    let info_label_clone = info_label.clone();
    let check_version_clone = check_version.clone();
    let progress_clone = progress.clone();
    wnd2.on().wm_init_dialog(move |_| {
        progress_clone.set_marquee(true);
        download_button_clone.hwnd().EnableWindow(false);
        let download_button_clone = download_button_clone.clone();
        let check_version = check_version_clone.clone();
        let info_label_clone = info_label_clone.clone();
        let progress_clone = progress_clone.clone();
        tokio::spawn(async move {
            if let Some(latest_version) = upgrade::get_latest_release(&format!("https://api.github.com/repos/cari-404/abs/releases/latest")).await {
                println!("Versi terbaru: {}", latest_version);
                if upgrade::compare_versions(CURRENT_VERSION, &latest_version) == std::cmp::Ordering::Less {
                    info_label_clone.set_text(&format!("Versi :{} tersedia!", latest_version));
                    download_button_clone.hwnd().EnableWindow(true);
                    let mut shared = check_version.lock().unwrap();
                    shared.clear();
                    *shared = latest_version;
                }else {
                    info_label_clone.set_text("Versi terbaru sudah terpasang.");
                }
            } else {
                info_label_clone.set_text("Gagal mengecek versi terbaru.");
            }
            progress_clone.set_marquee(false);
        });
        Ok(true)
    });
    let wnd2_clone = wnd2.clone();
    cancel_button.on().bn_clicked(move || {
        wnd2_clone.close();
        Ok(())
    });
    let wnd2_clone = wnd2.clone();
    let download_button_clone = download_button.clone();
    let info_label_clone = info_label.clone();
    let check_version_clone = check_version.clone();
    let progress_clone = progress.clone();
    download_button.on().bn_clicked(move || {
        let download_button_clone = download_button_clone.clone();
        let shared =  {
            let guard = check_version_clone.lock().unwrap();
            guard.clone()
        };
        let info_label_clone = info_label_clone.clone();
        let progress_clone = progress_clone.clone();
        let wnd2_clone = wnd2_clone.clone();
        if download_button_clone.text() == "Install" {
            let _ = std::process::Command::new("cmd")
                .arg("/C")
                .arg("updater.exe offline")
                .spawn();
            std::process::exit(0);
        }else{
            tokio::spawn(async move {
                let arch = if ARCH == "x86"{
                    "i686"
                }else{
                    ARCH
                };
                use windows_version::OsVersion;
                let version = OsVersion::current();
                let os = if version >= OsVersion::new(10, 0, 0, 10240) {
                    OS
                } else if version >= OsVersion::new(6, 1, 0, 7600) {
                    "windows7"
                } else if version >= OsVersion::new(5, 1, 0, 2600) {
                    "windowsxp"
                } else {
                    OS
                };
                use tokio::fs::OpenOptions;
                let url = format!("https://github.com/cari-404/abs/releases/download/v{}/ABS_{}-{}-v{}.zip", shared, os, arch, shared);
                println!("URL unduhan: {}", url);
                let resp =  upgrade::fetch_download_response(&url).await;
                match resp {
                    Ok((response, total_size)) => {
                        let file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true) // Hindari data lama tertinggal
                            .open("update.zip")
                            .await
                            .expect("Failed to create file");
                        let mut file = BufWriter::new(file);
                        let mut stream = response.bytes_stream();
                        let mut downloaded: u64 = 0;
                        progress_clone.set_range(0, total_size as u32);
                        progress_clone.set_position(0);
                        let start = std::time::Instant::now();
                        while let Some(chunk) = stream.next().await {
                            let chunk = chunk.map_err(|e| {
                                eprintln!("Error saat mengunduh chunk: {}", e);
                                io::Error::new(io::ErrorKind::Other, "Error saat menerima data")
                            }).expect("Langkah Kocak");
                            file.write_all(&chunk).await.expect("failed to write");
                            downloaded += chunk.len() as u64;
                            progress_clone.set_position(downloaded as u32);
                            let elapsed = start.elapsed().as_secs_f64();
                            let speed_kb = (downloaded as f64 / elapsed) / 1024.0;
                            let downloaded_mb = downloaded as f64 / 1024.0 / 1024.0;
                            let total_mb = total_size as f64 / 1024.0 / 1024.0;
                            let percentage = (downloaded as f64 / total_size as f64) * 100.0;
                        
                            let remaining = total_size.saturating_sub(downloaded);
                            let eta_secs = if speed_kb > 0.0 {
                                (remaining as f64 / 1024.0) / speed_kb
                            } else {
                                0.0
                            };
                        
                            info_label_clone.set_text(&format!(
                                "Kecepatan: {:.2} KB/s | Diunduh: {:.2}/{:.2} MB ({:.1}%) | ETA: {}",
                                speed_kb,
                                downloaded_mb,
                                total_mb,
                                percentage,
                                format_eta(eta_secs)
                            ));
                        }
                        file.flush().await.expect("failed to flush data");
                        let actual_size = tokio::fs::metadata("update.zip").await.expect("failed to calculate").len();
                        if actual_size != total_size {
                            let isi = format!("File mungkin corrupt! Ukuran seharusnya {} bytes, tetapi hanya {} bytes.", total_size, actual_size);
                            eprintln!("{}", isi);
                            let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                        }else{
                            info_label_clone.set_text("Click to install(restart the app)");
                            download_button_clone.set_text("Install");
                        }
                    }
                    Err(e) => {
                        let isi = format!("Error: {}", e);
                        let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                    }
                }
            });
        }
        Ok(())
    });
    let _ = wnd2.show_modal();
    Ok(())
}
pub fn telegram_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowModal::new_dlg(wnd, 500);
    let save_button = gui::Button::new_dlg(&wnd2, 501, dont_move);
    let cancel_button = gui::Button::new_dlg(&wnd2, 502, dont_move);
    let test_button = gui::Button::new_dlg(&wnd2, 503, dont_move);
    let token = gui::Edit::new_dlg(&wnd2, 504, dont_move);
    let chat_id = gui::Edit::new_dlg(&wnd2, 505, dont_move);
    let checkbox = gui::CheckBox::new_dlg(&wnd2, 506, dont_move);
    let wnd2_clone = wnd2.clone();
    let token_clone = token.clone();
    let chat_id_clone = chat_id.clone();
    let checkbox_clone = checkbox.clone();
    wnd2_clone.on().wm_init_dialog(move |_| {
        let token_clone = token_clone.clone();
        let chat_id_clone = chat_id_clone.clone();
        let checkbox_clone = checkbox_clone.clone();
        tokio::spawn(async move {
            // Membuka file konfigurasi
            let config = match telegram::open_config_file().await {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Failed to open config file: {}", e);
                    return; // Keluar dari task jika terjadi error
                }
            };
            // Mendapatkan data dari konfigurasi
            let data = match telegram::get_config(&config).await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to get config data: {}", e);
                    return; // Keluar dari task jika terjadi error
                }
            };
            println!("{:?}", data);
            token_clone.set_text(&data.telegram_token);
            chat_id_clone.set_text(&data.telegram_chat_id);
            if data.telegram_notif == true{
                checkbox_clone.set_check_state(gui::CheckState::Checked);
            }else{
                checkbox_clone.set_check_state(gui::CheckState::Unchecked);
            }
        });
        Ok(true)
    });
    let wnd2_clone = wnd2.clone();
    cancel_button.on().bn_clicked(move || {
        wnd2_clone.close();
        Ok(())
    });
    let wnd2_clone = wnd2.clone();
    let token_clone = token.clone();
    let chat_id_clone = chat_id.clone();
    let checkbox_clone = checkbox.clone();
    save_button.on().bn_clicked(move || {
        let token_text = token_clone.text();
        let chat_id_text = chat_id_clone.text();
        if token_text.is_empty() || chat_id_text.is_empty() {
            let isi = format!("please fill token and chat id");
            let _ = func_main::error_modal(&wnd2_clone, "Error save data", &isi);
        }else{
            let wnd2_clone = wnd2_clone.clone();
            let checkbox_clone = checkbox_clone.clone();
            tokio::spawn(async move {
                let config_content = match telegram::open_config_file().await {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Failed to open config file: {}. Creating a new one.", e);
                    "{}".to_string()
                    }
                };
                let mut config: serde_json::Value = match serde_json::from_str(&config_content) {
                    Ok(json) => json,
                    Err(e) => {
                        eprintln!("Failed to parse config file: {}. Creating a new one.", e);
                        serde_json::json!({})
                    }
                };
                if let serde_json::Value::Object(ref mut map) = config {
                    if checkbox_clone.check_state() == gui::CheckState::Checked {
                        map.insert("telegram_notif".to_string(), serde_json::Value::Bool(true));
                    }else{
                        map.insert("telegram_notif".to_string(), serde_json::Value::Bool(false));
                    }
                    map.insert("telegram_token".to_string(), serde_json::Value::String(token_text));
                    map.insert("telegram_chat_id".to_string(), serde_json::Value::String(chat_id_text));
                } else {
                    let isi = format!("Invalid config format. Unable to save data.");
                    let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                    return;
                }
                match telegram::save_config_file(serde_json::to_string_pretty(&config).unwrap()).await {
                    Ok(_) => {
                        let isi = format!("Token and chat ID saved successfully");
                        let _ = func_main::info_modal(&wnd2_clone, "Success saving data", &isi);
                        wnd2_clone.close();
                    }
                    Err(e) => {
                        let isi = format!("Failed to save config file: {}", e);
                        let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                    }
                };
            });
        }
        Ok(())
    });
    let wnd2_clone = wnd2.clone();
    test_button.on().bn_clicked(move || {
        let token_clone = token.clone();
        let chat_id_clone = chat_id.clone();
        let wnd2_clone = wnd2_clone.clone();
        tokio::spawn(async move {
            println!("test Send");
            let token_text = token_clone.text();
            let chat_id_text = chat_id_clone.text();
            if token_text.is_empty() || chat_id_text.is_empty() {
                let isi = format!("please fill token and chat id");
                let _ = func_main::error_modal(&wnd2_clone, "Error get data", &isi);
            }else{
                let data = telegram::get_data(&token_text, &chat_id_text);
                match telegram::send_msg(&data, "This is a test message; you can ignore it.").await {
                    Ok(_) => println!("sent"),
                    Err(e) => println!("error: {}", e),
                };
            }
        });
        Ok(())
    });
    let _ = wnd2.show_modal();
    Ok(())
}
pub fn account_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowModal::new_dlg(wnd, 1000);
    let resize_h = (gui::Horz::Resize, gui::Vert::None);
    let resize_v = (gui::Horz::None, gui::Vert::Resize);
    let resized = (gui::Horz::Resize, gui::Vert::Resize);
    let move_h = (gui::Horz::Repos, gui::Vert::None);
    let move_v = (gui::Horz::None, gui::Vert::Repos);
    let moved = (gui::Horz::Repos, gui::Vert::Repos);
    let move_h_resize_v = (gui::Horz::Repos, gui::Vert::Resize);
    let resize_h_move_v = (gui::Horz::Resize, gui::Vert::Repos);
    //let wnd2 = gui::WindowModeless::new_dlg(&wnd_clone, 1000, w::POINT::new(0, 0));
    //let wnd2 = gui::WindowMain::new_dlg(1000, Some(101), None);
    let save_button = gui::Button::new_dlg(&wnd2, 1001, moved);
    let cancel_button = gui::Button::new_dlg(&wnd2, 1002, moved);
    let file_combo = gui::ComboBox::new_dlg(&wnd2, 1003, resize_h);
    let cookie_edit = gui::Edit::new_dlg(&wnd2, 1004, dont_move);
    let sz_edit = gui::Edit::new_dlg(&wnd2, 1005, dont_move);
    let my_list = gui::ListView::new_dlg(&wnd2, 1006, resized, Some(100));
    let qr_button = gui::Button::new_dlg(&wnd2, 1007, move_h);
    let my_list_clone = my_list.clone();
    wnd2.on().wm_command_accel_menu(101 as u16, move || {
        if let Some(selected_item) = my_list_clone.items().focused() {
            let _ = func_main::set_clipboard(&selected_item.text(1));
        } else {
            println!("No item selected");
        }
        Ok(())
    });
    let wnd2_clone = wnd2.clone();
    qr_button.on().bn_clicked(move || {
        println!("QR Button clicked!");
        let _ = login::login_window(&wnd2_clone);
        Ok(())
    });
    let cookie_edit_clone = cookie_edit.clone();
    let my_list_clone = my_list.clone();
    cookie_edit.on().en_change(move || {
        my_list_clone.items().delete_all();
        let cookie_data = prepare::create_cookie(&cookie_edit_clone.text());
        my_list_clone.items().add(
            &[
                "CSRFTOKEN",
                &cookie_data.csrftoken,
            ],
            None, (),
        );
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let cookie_edit_clone = cookie_edit.clone();
    let sz_edit_clone = sz_edit.clone();
    let my_list_clone = my_list.clone();
    wnd2.on().wm_init_dialog(move |_| {
        my_list_clone.columns().add(&[
            ("Item", 120),
            ("Value", 300),
        ]);
        my_list_clone.items().add(
            &[
                "Default",
                "text",
            ],
            None, // no icon; requires a previous set_image_list()
            (),   // no object data; requires specifying the generic `ListView` type
        );
        my_list_clone.items().add(
            &[
                "CSRFTOKEN",
                "Our CSRF token Hardwork",
            ],
            None, // no icon; requires a previous set_image_list()
            (),   // no object data; requires specifying the generic `ListView` type
        );
        my_list_clone.items().add(
            &[
                "Visible",
                "True",
            ], None, (),
        );
        my_list_clone.set_extended_style(true, w::co::LVS_EX::FULLROWSELECT);
        func_main::populate_combobox_with_files(&file_combo_clone, "akun");
        func_main::handle_file_selection(&file_combo_clone, &cookie_edit_clone, &sz_edit_clone, &my_list_clone);
        Ok(true)
    });
    let file_combo_clone = file_combo.clone();
    let cookie_edit_clone = cookie_edit.clone();
    let sz_edit_clone = sz_edit.clone();
    let my_list_clone = my_list.clone();
    file_combo.on().cbn_edit_change(move || {
        println!("Edit change!");
        my_list_clone.items().delete_all();
        sz_edit_clone.set_text("");
        cookie_edit_clone.set_text("");
        println!("{}", file_combo_clone.text());
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let cookie_edit_clone = cookie_edit.clone();
    let sz_edit_clone = sz_edit.clone();
    let my_list_clone = my_list.clone();
    file_combo.on().cbn_sel_change(move || {
        func_main::handle_file_selection(&file_combo_clone, &cookie_edit_clone, &sz_edit_clone, &my_list_clone);
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    file_combo.on().cbn_drop_down(move || {
        let selected_text = file_combo_clone.text();
        func_main::populate_combobox_with_files(&file_combo_clone, "akun");
        file_combo_clone.set_text(&selected_text);
        Ok(())
    });
    wnd2.on().wm_destroy(move || {
            println!("Window is gone, goodbye!");
            Ok(())
        },
    );
    let wnd2_clone = wnd2.clone();
    cancel_button.on().bn_clicked(move || {
        wnd2_clone.close();
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let wnd2_clone = wnd2.clone();
    save_button.on().bn_clicked(move || {
        let file = file_combo_clone.text();
        let cookie = cookie_edit.text();
        let sz = sz_edit.text();
        if file.is_empty() {
            let isi = format!("Please select a file before saving the cookie");
            let _ = func_main::error_modal(&wnd2_clone, "Error save data", &isi);
        } else if cookie.is_empty() {
            let isi = format!("Please input the cookie before saving");
            let _ = func_main::error_modal(&wnd2_clone, "Error save data", &isi);
        } else {
            if file.contains(".txt") {
                println!("File contains .txt");
                let _ = func_main::save_cookie_fp_file(&file, &cookie, &sz);
            } else {
                println!("File does not contain .txt");
                let file_fix = format!("{}.txt", file);
                let _ = func_main::save_cookie_fp_file(&file_fix, &cookie, &sz);
            }
            let isi = format!("Cookie saved successfully");
            let _ = func_main::info_modal(&wnd2_clone, "Success saving data", &isi);
            wnd2_clone.close();
        }
        Ok(())
    });
    let _ = wnd2.show_modal();
    //wnd2.run_main(None);
    Ok(())
}
pub fn log_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowModeless::new_dlg(wnd, 600, w::POINT{x:0, y:0});
    let txt_log = gui::Edit::new_dlg(&wnd2, 601, dont_move);
    Ok(())
}
pub fn show_fs_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let (tx_msg, rx_msg) = mpsc::unbounded_channel::<String>();
    let _ = tx_msg.send("Stopped".to_string());
    let interrupt_flag = Arc::new(AtomicBool::new(false));
    let shared_fsid = Arc::new(Mutex::new(vec![]));
    let wnd_clone = wnd.clone();
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let resize_h = (gui::Horz::Resize, gui::Vert::None);
    let resize_v = (gui::Horz::None, gui::Vert::Resize);
    let resized = (gui::Horz::Resize, gui::Vert::Resize);
    let move_h = (gui::Horz::Repos, gui::Vert::None);
    let move_v = (gui::Horz::None, gui::Vert::Repos);
    let moved = (gui::Horz::Repos, gui::Vert::Repos);
    let move_h_resize_v = (gui::Horz::Repos, gui::Vert::Resize);
    let resize_h_move_v = (gui::Horz::Resize, gui::Vert::Repos);
    let wnd2 = gui::WindowModal::new_dlg(&wnd_clone, 700);
    //let wnd2 = gui::WindowModeless::new_dlg(&wnd_clone, 1000, w::POINT::new(0, 0));
    //let wnd2 = gui::WindowMain::new_dlg(1000, Some(101), None);
    let cek_button = gui::Button::new_dlg(&wnd2, 701, move_h);
    let fs_combo = gui::ComboBox::new_dlg(&wnd2, 702, dont_move);
    let single_button = gui::Button::new_dlg(&wnd2, 703, dont_move);
    let all_button = gui::Button::new_dlg(&wnd2, 704, dont_move);
    let my_list = gui::ListView::new_dlg(&wnd2, 705, resized, Some(200));
    let proid_label = gui::Label::new_dlg(&wnd2, 706, dont_move);
    let stime_label = gui::Label::new_dlg(&wnd2, 707, dont_move);
    let etime_label = gui::Label::new_dlg(&wnd2, 708, dont_move);
    let progress = gui::ProgressBar::new_dlg(&wnd2, 709, resize_h_move_v);
    let file_combo = gui::ComboBox::new_dlg(&wnd2, 710, dont_move);
    let stop_button = gui::Button::new_dlg(&wnd2, 711, moved);
    let progress_label = gui::Label::new_dlg(&wnd2, 712, moved);
    let search_edit = gui::Edit::new_dlg(&wnd2, 713, dont_move);
    let search_button = gui::Button::new_dlg(&wnd2, 714, dont_move);
    let mode_label = gui::Label::new_dlg(&wnd2, 715, dont_move);
    let count_label = gui::Label::new_dlg(&wnd2, 716, dont_move);

    let my_list_clone = my_list.clone();
    wnd2.on().wm_command_accel_menu(201 as u16, move || {
        if let Some(selected_item) = my_list_clone.items().focused() {
            let url = selected_item.text(5 as u32);
            let _ = func_main::open_url(&url);
        } else {
            println!("No item selected");
        }
        Ok(())
    });
    let my_list_clone = my_list.clone();
    wnd2.on().wm_command_accel_menu(202 as u16, move || {
        if let Some(selected_item) = my_list_clone.items().focused() {
            println!("Selected item: {:?}", selected_item.index());
            let url = selected_item.text(5 as u32);
            println!("Selected item text: {}", url);
            let _ = func_main::set_clipboard(&url);
        } else {
            println!("No item selected");
        }
        Ok(())
    });
    let my_list_clone = my_list.clone();
    let file_combo_clone = file_combo.clone();
    wnd2.on().wm_command_accel_menu(203 as u16, move || {
        let file = file_combo_clone.text();
        if let Some(selected_item) = my_list_clone.items().focused() {
            let product_info = prepare::process_url(&selected_item.text(5 as u32));
            let cookie_data = prepare::create_cookie(&prepare::read_cookie_file(&file));
            let v: Vec<i64> = serde_json::from_str(&selected_item.text(1 as u32))?;
            let selected_item_index = selected_item.index();
            let my_list_clone = my_list_clone.clone();
            tokio::spawn(async move {
                let client_clone = Arc::new(prepare::universal_client_skip_headers().await);
                let mut variation = Vec::new();
                match prepare::get_product(client_clone, &product_info, &cookie_data).await {
                    Ok((_, model_info, _, _, _)) => {
                        for model in model_info.iter() {
                            if v.contains(&model.modelid) {
                                println!("[FLASHSALE]{}", model.name); 
                                variation.push(model.name.clone());
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                let item = my_list_clone.items().get(selected_item_index);
                item.set_text(6 as u32, &format!("{:?}", variation));
            });
        } else {
            println!("No item selected");
        }
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let my_list_clone = my_list.clone();
    wnd2.on().wm_init_dialog(move |_| {
        my_list_clone.columns().add(&[
            ("Name", 120),
            ("Modelid", 120),
            ("estimate", 120),
            ("hidden", 120),
            ("stock", 120),
            ("Url", 300),
            ("Variation",120),
        ]);
        my_list_clone.items().add(
            &[
                "Default",
                "text",
                "text",
                "text",
                "text",
                "text",
                "",
            ],
            None, // no icon; requires a previous set_image_list()
            (),   // no object data; requires specifying the generic `ListView` type
        );
        my_list_clone.set_extended_style(true, w::co::LVS_EX::FULLROWSELECT);
        func_main::populate_combobox_with_files(&file_combo_clone, "akun");
        Ok(true)
    });
    let wnd2_clone = wnd2.clone();
    let search_edit_clone = search_edit.clone();
    let my_list_clone = my_list.clone();
    search_button.on().bn_clicked(move || {
        let search_text = search_edit_clone.text();
        if search_text.is_empty() {
            let isi = format!("Please input the search text");
            let _ = func_main::error_modal(&wnd2_clone, "Error search data", &isi);
        } else {
            for i in 0..my_list_clone.items().count() {
                let item = my_list_clone.items().get(i);
                let text = item.text(0).to_lowercase();
                if text.contains(&search_text.to_lowercase()) {
                    println!("Item cocok di index {}: {}", i, text);
                    item.select(true); // pilih item
                    item.ensure_visible(); // scroll ke item
                }
            }
        };
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let wnd2_clone = wnd2.clone();
    let fs_combo_clone = fs_combo.clone();
    let shared_fsid_clone = shared_fsid.clone();
    let progress_clone = progress.clone();
    cek_button.on().bn_clicked(move || {
        progress_clone.set_marquee(true);
        let file = file_combo_clone.text();
        if file.is_empty() {
            let isi = format!("Please select a file before checking the fs");
            let _ = func_main::error_modal(&wnd2_clone, "Error check data", &isi);
            progress_clone.set_marquee(false);
        } else {
            let cookie_data = prepare::create_cookie(&prepare::read_cookie_file(&file));
            let fs_combo_clone = fs_combo_clone.clone();
            let wnd2_clone = wnd2_clone.clone();
            let shared_fsid_clone = shared_fsid_clone.clone();
            let progress_clone = progress_clone.clone();
            tokio::spawn(async move {
                let client_clone = Arc::new(prepare::universal_client_skip_headers().await);
                fs_combo_clone.items().delete_all();
                let fsid_current = product::get_current_fsid(client_clone, &cookie_data).await;
                match fsid_current {
                    Ok(fsid_current) => {
                        if fsid_current.is_empty() {
                            let _ = func_main::error_modal(&wnd2_clone, "Info", "Tidak ada fsid yang tersedia.\nPeriksa akun yang dipilih");
                            return;
                        }
                        for fsi in &fsid_current {
                            println!("{}", fsi.promotionid);
                            fs_combo_clone.items().add(&[&fsi.promotionid.to_string()]);
                        }
                        if fs_combo_clone.items().count() > 0 {
                            fs_combo_clone.items().select(Some(0));
                        }
                        let mut shared = shared_fsid_clone.lock().unwrap();
                        shared.clear();
                        *shared = fsid_current.clone();
                        progress_clone.set_marquee(false);
                    }
                    Err(e) => {
                        let isi = format!("Error: {}", e);
                        let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                        progress_clone.set_marquee(false);
                    }
                };
            });
        };
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let fs_combo_clone = fs_combo.clone();
    let my_list_clone = my_list.clone();
    let wnd2_clone = wnd2.clone();
    let progress_clone = progress.clone();
    let interrupt_flag_clone = interrupt_flag.clone();
    let shared_fsid_clone = shared_fsid.clone();
    let tx_msg_clone = tx_msg.clone();
    let progress_label_clone = progress_label.clone();
    let mode_label_clone = mode_label.clone();
    let count_label_clone = count_label.clone();
    single_button.on().bn_clicked(move || {
        progress_clone.set_state(w::co::PBST::NORMAL);
        let fsid = fs_combo_clone.text();
        if fsid.is_empty() {
            let isi = format!("Please select a fs id before checking the fs");
            let _ = func_main::error_modal(&wnd2_clone, "Error check data", &isi);
        } else {
            mode_label_clone.set_text("Single");
            wnd2_clone.hwnd().ShowWindow(w::co::SW::SHOWMAXIMIZED);
            let Some(selecte_fsid) = fs_combo_clone.items().selected_text() else {
                eprintln!("Tidak ada fsid yang dipilih.");
                return Ok(());
            };
            let promotionid: i64 = match selecte_fsid.parse() {
                Ok(id) => id,
                Err(_) => {
                    eprintln!("fsid yang dipilih bukan angka valid: {}", selecte_fsid);
                    return Ok(());
                }
            };
            let mut fsinfo = Vec::new();
            let shared = shared_fsid_clone.lock().unwrap();
            if let Some(matching) = shared.iter().find(|item| item.promotionid == promotionid) {
                proid_label.set_text_and_resize(&matching.promotionid.to_string());
                stime_label.set_text(&(func_main::human_readable_time(matching.start_time)).to_string());
                etime_label.set_text(&(func_main::human_readable_time(matching.end_time)).to_string());
                fsinfo.push(matching.clone());
            } else {
                println!("Tidak ditemukan promotionid yang cocok");
            }
            let _ = get_flashsale_products(&wnd2_clone, Arc::new(fsinfo), &file_combo_clone, &my_list_clone, &progress_clone, &interrupt_flag_clone, &tx_msg_clone, &progress_label_clone, &count_label_clone);
        };
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let my_list_clone = my_list.clone();
    let wnd2_clone = wnd2.clone();
    let progress_clone = progress.clone();
    let interrupt_flag_clone = interrupt_flag.clone();
    let shared_fsid_clone = shared_fsid.clone();
    let tx_msg_clone = tx_msg.clone();
    let progress_label_clone = progress_label.clone();
    let mode_label_clone = mode_label.clone();
    let count_label_clone = count_label.clone();
    all_button.on().bn_clicked(move || {
        progress_clone.set_state(w::co::PBST::NORMAL);
        let shared = shared_fsid_clone.lock().unwrap();
        if shared.is_empty() {
            println!("empty shared!");
            let isi = format!("Promotionid not available\nPlease check the account selected and try again");
            let _ = func_main::error_modal(&wnd2_clone, "Error: not found", &isi);
        } else {
            mode_label_clone.set_text("All");
            wnd2_clone.hwnd().ShowWindow(w::co::SW::SHOWMAXIMIZED);
            let _ = get_flashsale_products(&wnd2_clone, Arc::new(shared.to_vec()), &file_combo_clone, &my_list_clone, &progress_clone, &interrupt_flag_clone, &tx_msg_clone, &progress_label_clone, &count_label_clone);
        }
        Ok(())
    });
    my_list.on().lvn_column_click(move |_| {
        println!("Column clicked!");
        Ok(())
    });
    let wnd2_clone = wnd2.clone();
    let interrupt_flag_clone = interrupt_flag.clone();
    let progress_clone = progress.clone();
    stop_button.on().bn_clicked(move || {
        interrupt_flag_clone.store(true, Ordering::Relaxed);
        let isi = format!("Scan was stopped by user");
        let _ = func_main::info_modal(&wnd2_clone, "Info", &isi);
        progress_clone.set_state(w::co::PBST::PAUSED);
        Ok(())
    });
    let interrupt_flag_clone = interrupt_flag.clone();
    let rx_msg = Arc::new(Mutex::new(rx_msg));
    let rx_msg_clone = rx_msg.clone();
    let my_list_clone = my_list.clone();
    wnd2.on().wm_destroy(move || {
        interrupt_flag_clone.store(true, Ordering::SeqCst);
        interrupt_flag_clone.store(true, Ordering::Relaxed);
        loop{
            if let Ok(mut rx) = rx_msg_clone.lock() {
                if let Ok(msg) = rx.try_recv() {
                    println!("Got from task: {}", msg);
                    if msg == "Stopped" {
                        break;
                    }
                }
            }
        }
        my_list_clone.items().delete_all();
        println!("Window is gone, goodbye!");
        Ok(())
    });

    let _ = wnd2.show_modal();
    //wnd2.run_main(None);
    Ok(())
}
fn get_flashsale_products(wnd2: &gui::WindowModal, fsinfo: Arc<Vec<FSInfo>>, file_combo: &gui::ComboBox, my_list: &gui::ListView, progress: &gui::ProgressBar, interrupt_flag: &Arc<AtomicBool>, tx_msg: &mpsc::UnboundedSender<String>, progress_label: &gui::Label, count_label: &gui::Label) -> Result<(), ()> {
    let file = file_combo.text();
    if file.is_empty() {
        let isi = format!("Please select a file before checking the fs");
        let _ = func_main::error_modal(&wnd2, "Error check data", &isi);
    } else {
        my_list.items().delete_all();
        let Some(select_cookie_file) = file_combo.items().selected_text() else {
            eprintln!("Tidak ada file cookie yang dipilih.");
            return Ok(());
        };
        let cookie_content = prepare::read_cookie_file(&select_cookie_file);
        let cookie_data = prepare::create_cookie(&cookie_content);
        let wnd2_clone = wnd2.clone();
        let my_list_clone = my_list.clone();
        let progress_clone = progress.clone();
        interrupt_flag.store(false, Ordering::Relaxed);
        let interrupt_flag_clone = interrupt_flag.clone();
        let tx_msg = tx_msg.clone();
        let progress_label_clone = progress_label.clone();
        let fsinfo_cloned = fsinfo.clone();
        let count_label = count_label.clone();
        tokio::spawn(async move {
            let client_clone = Arc::new(prepare::universal_client_skip_headers().await);
            let mut count = 0;
            let mut max = 0;
            let mut potition = 0;
            for fsinfoiter in fsinfo_cloned.iter() {
                let fsid_current = product::get_itemids_from_fsid(client_clone.clone(), &fsinfoiter, &cookie_data).await;
                match fsid_current {
                    Ok(fsid_current) => {
                        max += (fsid_current.len() + 15) / 16;
                        progress_clone.set_range(0, max as u32);
                        progress_clone.set_position(0);
                        for fsi in fsid_current.chunks(16) {
                            if interrupt_flag_clone.load(Ordering::Relaxed) {
                                println!("Proses dibatalkan oleh user.");
                                let _ = tx_msg.send("Stopped".to_string());
                                break;
                            }
                            let batch: Vec<ProductInfo> = fsi.to_vec();
                            let fs_items = prepare::get_flash_sale_batch_get_items(client_clone.clone(), &cookie_data, &batch, &fsinfoiter).await;
                            match fs_items {
                                Ok(fs_items) => {
                                    for item in fs_items {
                                        if interrupt_flag_clone.load(Ordering::SeqCst) {
                                            println!("Task was interrupted, exiting...");
                                            let _ = tx_msg.send("Stopped".to_string());
                                            return;
                                        }
                                        let link = format!("https://shopee.co.id/product/{}/{}", item.shopid, item.itemid);
                                        my_list_clone.items().add(&[
                                            &item.name,
                                            &format!("{:?}", item.modelids.unwrap_or_default()),
                                            &func_main::format_thousands(item.price_before_discount * (100 - item.raw_discount) / 100 / 100000),
                                            &item.hidden_price_display.as_deref().unwrap_or("No Hide").to_string(),
                                            &item.stock.to_string(),
                                            &link,
                                        ], None, ());
                                        let _ = tx_msg.send("Running".to_string());
                                        count +=1;
                                        count_label.set_text(&count.to_string());
                                    }
                                }
                                Err(e) => {
                                    let isi = format!("Error: {}", e);
                                    let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                                }
                            }
                            potition += 1;
                            progress_clone.set_position(potition as u32);
                            let progressinf = format!("{}/{}", potition, max);
                            println!("Progress: {}", progressinf);
                            progress_label_clone.set_text(&progressinf);  
                        }
                    }
                    Err(e) => {
                        let isi = format!("Error: {}", e);
                        let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                    }
                };
                let _ = tx_msg.send("Stopped".to_string());
            }
            let _ = tx_msg.send("Stopped".to_string());
        });
    };
    Ok(())
}
fn format_eta(seconds: f64) -> String {
    let total_secs = seconds.round() as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    match (hours, minutes, secs) {
        (0, 0, s) => format!("{}s", s),
        (0, m, s) => format!("{}m {}s", m, s),
        (h, m, s) => format!("{}h {}m {}s", h, m, s),
    }
}
