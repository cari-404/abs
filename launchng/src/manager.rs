use winsafe::{self as w,
    gui, prelude::*
};
use runtime::telegram;
use runtime::prepare;

use crate::func_main;
use crate::login;

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
    let wnd_clone = wnd.clone();
    tokio::spawn(async move {
        let dont_move = (gui::Horz::None, gui::Vert::None);
        let wnd2 = gui::WindowModal::new_dlg(&wnd_clone, 1000);
        //let wnd2 = gui::WindowModeless::new_dlg(&wnd_clone, 1000, w::POINT::new(0, 0));
        //let wnd2 = gui::WindowMain::new_dlg(1000, Some(101), None);
        let save_button = gui::Button::new_dlg(&wnd2, 1001, dont_move);
        let cancel_button = gui::Button::new_dlg(&wnd2, 1002, dont_move);
        let file_combo = gui::ComboBox::new_dlg(&wnd2, 1003, dont_move);
        let cookie_edit = gui::Edit::new_dlg(&wnd2, 1004, dont_move);
        let sz_edit = gui::Edit::new_dlg(&wnd2, 1005, dont_move);
        /*let my_list: gui::ListView<String> = gui::ListView::new(&wnd2, gui::ListViewOpts {
            position: (261, 36),
            size: (113, 150),
            ..Default::default()
        });*/
        let my_list = gui::ListView::new_dlg(&wnd2, 1006, dont_move, None);
        let qr_button = gui::Button::new_dlg(&wnd2, 1007, dont_move);
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
    });
    Ok(())
}
pub fn log_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowMain::new_dlg(600, None, None);
    let txt_log = gui::Edit::new_dlg(&wnd2, 601, dont_move);

    let _ = wnd2.run_main(None);
    Ok(())
}