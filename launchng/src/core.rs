use winsafe::{self as w,
    gui, prelude::*, co::{self, ES, SS, SW}, AnyResult,
    HMENU, guard,
};
use ::runtime::prepare::{self, AddressInfo};
use ::runtime::telegram;
use tokio::{time::{timeout, Duration}};
use chrono::prelude::*;
use windows_version::*;

use crate::func_main;
use crate::login;

#[derive(Clone)]
pub struct MyWindow {
	wnd:       gui::WindowMain, // responsible for managing the window
    url_text: gui::Edit,
    file_combo: gui::ComboBox,
    payment_combo: gui::ComboBox,
    variasi_combo: gui::ComboBox,
    kurir_combo: gui::ComboBox,
    harga_text: gui::Edit,
    harga_checkbox: gui::CheckBox,
    kuan_text: gui::Edit,
    jam_text: gui::Edit,
    menit_text: gui::Edit,
    detik_text: gui::Edit,
    mili_text: gui::Edit,
    btn_cek: gui::Button,
    btn_jalankan: gui::Button,
    fsv_checkbox: gui::CheckBox,
    platform_checkbox: gui::CheckBox,
    platform_combobox: gui::ComboBox,
    shop_checkbox: gui::CheckBox,
    code_label: gui::Label,
    code_platform_text: gui::Edit,
    code_shop_text: gui::Edit,
    promotionid_label: gui::Label,
    promotionid_text: gui::Edit,
    signature_label: gui::Label,
    signature_text: gui::Edit,
    cid_label: gui::Label,
    cid_text: gui::Edit,
}

impl MyWindow {
	pub fn new() -> Self {
        let (menu, accel_table) = Self::build_menu().unwrap();
        let wnd = gui::WindowMain::new(gui::WindowMainOpts {
            title: "Launcher *NG* for ABS".to_owned(),
            style: gui::WindowMainOpts::default().style
                | co::WS::MINIMIZEBOX | co::WS::MAXIMIZEBOX | co::WS::SIZEBOX,
            class_icon: gui::Icon::Id(101),
            size: (600, 410), // Lebar dan tinggi jendela
            menu,
            accel_table: Some(accel_table),
            ..Default::default()
        });
    
        let status_bar = gui::StatusBar::new(
            &wnd,
            &[
                gui::SbPart::Fixed(200),      // 200 pixels, never resizes
                gui::SbPart::Proportional(1), // these two will fill the remaning space
                gui::SbPart::Proportional(1),
            ],
        );
    
        // Input URL
        let lbl_url = gui::Label::new(&wnd, gui::LabelOpts {
            text: "URL".to_owned(),
            position: (10, 10),
            ..Default::default()
        });
        let url_text = gui::Edit::new(&wnd, gui::EditOpts {
            position: (80, 10),
            width: 400,
            resize_behavior: (gui::Horz::Resize, gui::Vert::None),
            ..Default::default()
        });
        let btn_cek = gui::Button::new(&wnd, gui::ButtonOpts {
            text: "Cek".to_owned(),
            position: (500, 10),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
    
        // Payment ComboBox
        let lbl_payment = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Payment".to_owned(),
            position: (10, 50),
            ..Default::default()
        });
        let payment_combo = gui::ComboBox::new(&wnd, gui::ComboBoxOpts {
            position: (80, 50),
            width: 210,
            items: vec!["ShopeePay".to_owned(), "GoPay".to_owned(), "OVO".to_owned()],
            selected_item: Some(0),
            resize_behavior: (gui::Horz::Resize, gui::Vert::None),
            ..Default::default()
        });
    
        // File Picker
        let lbl_file = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Pilih file".to_owned(),
            position: (310, 50),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
        let file_combo = gui::ComboBox::new(&wnd, gui::ComboBoxOpts {
            position: (380, 50),
            width: 210,
            selected_item: Some(0),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
        let file_combo_clone = file_combo.clone();
        // Harga Max & Kuantiti
        let lbl_harga = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Harga Max".to_owned(),
            position: (10, 80),
            ..Default::default()
        });
        let harga_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "1000".to_owned(),
            position: (80, 80),
            width: 150,
            edit_style: ES::NUMBER,
            resize_behavior: (gui::Horz::Resize, gui::Vert::None),
            ..Default::default()
        });
        let harga_checkbox = gui::CheckBox::new(&wnd, gui::CheckBoxOpts {
            text: "Set".to_owned(),
            position: (250, 80),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
    
        let lbl_qty = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Kuantiti".to_owned(),
            position: (10, 110),
            size: (60, 20),
            ..Default::default()
        });
        let kuan_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "1".to_owned(),
            position: (80, 110),
            width: 210,
            edit_style: ES::NUMBER,
            resize_behavior: (gui::Horz::Resize, gui::Vert::None),
            ..Default::default()
        });
    
        let lbl_kurir = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Kurir".to_owned(),
            position: (310, 110),
            size: (60, 20),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
        let kurir_combo = gui::ComboBox::new(&wnd, gui::ComboBoxOpts {
            position: (380, 110),
            width: 210,
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
    
        let lbl_variasi = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Variasi".to_owned(),
            position: (310, 80),
            size: (60, 20),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
        let variasi_combo = gui::ComboBox::new(&wnd, gui::ComboBoxOpts {
            position: (380, 80),
            width: 210,
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
    
        // Time
        let time_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Time".to_owned(),
            position: (10, 160),
            size: (60, 20),
            ..Default::default()
        });
        let jam_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Jam".to_owned(),
            position: (80, 160),
            size: (60, 20),
            label_style: SS::CENTER,
            ..Default::default()
        });
        let jam_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "23".to_owned(),
            position: (80, 190),
            width: 60,
            edit_style: ES::NUMBER,
            ..Default::default()
        });
        let menit_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Menit".to_owned(),
            position: (150, 160),
            size: (60, 20),
            label_style: SS::CENTER,
            ..Default::default()
        });
        let menit_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "59".to_owned(),
            position: (150, 190),
            width: 60,
            edit_style: ES::NUMBER,
            ..Default::default()
        });
        let detik_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Detik".to_owned(),
            position: (220, 160),
            size: (60, 20),
            label_style: SS::CENTER,
            ..Default::default()
        });
        let detik_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "59".to_owned(),
            position: (220, 190),
            width: 60,
            edit_style: ES::NUMBER,
            ..Default::default()
        });
        let mili_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Mili".to_owned(),
            position: (290, 160),
            size: (60, 20),
            label_style: SS::CENTER,
            ..Default::default()
        });
        let mili_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "900".to_owned(),
            position: (290, 190),
            width: 60,
            edit_style: ES::NUMBER,
            ..Default::default()
        });
    
        // Tombol Jalankan
        let btn_jalankan = gui::Button::new(&wnd, gui::ButtonOpts {
            text: "Jalankan".to_owned(),
            position: (500, 300),
            resize_behavior: (gui::Horz::Resize, gui::Vert::Repos),
            ..Default::default()
        });

        /*let voucher_groupbox = gui::Button::new(&wnd, gui::ButtonOpts {
            text: "Voucher".to_owned(),
            position: (10, 220),
            width: 400,
            height: 150,
            button_style: BS::GROUPBOX,
            window_style: WS::CHILD | WS::VISIBLE,
            ..Default::default()
        });*/
    
        // Checkbox sebagai pengganti Radio Buttons
        let fsv_checkbox = gui::CheckBox::new(&wnd, gui::CheckBoxOpts {
            text: "fsv only".to_owned(),
            position: (18, 240),
            size: (80, 20),
            ..Default::default()
        });
    
        let platform_checkbox = gui::CheckBox::new(&wnd, gui::CheckBoxOpts {
            text: "Platform".to_owned(),
            position: (18, 270),
            size: (80, 20),
            ..Default::default()
        });

        let platform_combobox = gui::ComboBox::new(&wnd, gui::ComboBoxOpts {
            position: (130, 270),
            width: 210,
            items: vec!["Claim".to_owned(), "Code".to_owned(), "Collection id".to_owned()],
            selected_item: Some(0),
            ..Default::default()
        });
    
        let shop_checkbox = gui::CheckBox::new(&wnd, gui::CheckBoxOpts {
            text: "Shop Code".to_owned(),
            position: (18, 360),
            size: (80, 20),
            ..Default::default()
        });

        let code_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Code".to_owned(),
            position: (35, 300),
            ..Default::default()
        });

        let code_platform_text = gui::Edit::new(&wnd, gui::EditOpts {
            position: (130, 300),
            width: 210,
            ..Default::default()
        });

        let code_shop_text = gui::Edit::new(&wnd, gui::EditOpts {
            position: (130, 360),
            width: 210,
            ..Default::default()
        });

        let cid_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "CollectionId".to_owned(),
            position: (35, 300),
            ..Default::default()
        });

        let cid_text = gui::Edit::new(&wnd, gui::EditOpts {
            position: (130, 300),
            width: 210,
            ..Default::default()
        });

        let promotionid_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "PromotionId".to_owned(),
            position: (35, 300),
            ..Default::default()
        });

        let promotionid_text = gui::Edit::new(&wnd, gui::EditOpts {
            position: (130, 300),
            width: 210,
            ..Default::default()
        });

        let signature_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Signature".to_owned(),
            position: (35, 330),
            ..Default::default()
        });

        let signature_text = gui::Edit::new(&wnd, gui::EditOpts {
            position: (130, 330),
            width: 210,
            ..Default::default()
        });
    
        // CheckBox Official
        /*let chk_official = gui::CheckBox::new(&wnd, gui::CheckBoxOpts {
            text: "Official".to_owned(),
            position: (10, 330),
            size: (80, 20),
            ..Default::default()
        });*/

        let new_self = Self{
            wnd, url_text, file_combo, payment_combo,
            variasi_combo, kurir_combo,
            harga_text, harga_checkbox, kuan_text,
            jam_text, menit_text, detik_text, mili_text,
            btn_cek, btn_jalankan, fsv_checkbox, 
            shop_checkbox, 
            platform_checkbox, platform_combobox,
            code_label, code_platform_text, code_shop_text,
            promotionid_label,
            promotionid_text,
            signature_label,
            signature_text,
            cid_label,
            cid_text,
        };
        new_self.events(); // attach our events
		new_self

    }
    pub fn run(&self) -> AnyResult<i32> {
		self.wnd.run_main(None) // simply let the window manager do the hard work
	}

    fn build_menu() -> w::AnyResult<(HMENU, guard::DestroyAcceleratorTableGuard)> {
        let main_menu = w::HINSTANCE::GetModuleHandle(None)?
            .LoadMenu(w::IdStr::Id(1)).unwrap();
        let accel_table = w::HINSTANCE::GetModuleHandle(None)?
            .LoadAccelerators(w::IdStr::Str(w::WString::from_str("MENU1"))).unwrap();
		Ok((main_menu, accel_table))
	}

	fn events(&self) {
        let self2 = self.clone();
        self.btn_cek.on().bn_clicked(move || {
            println!("Cek button clicked!");
            println!("{}", self2.url_text.text());
            // Disable the button to prevent multiple async tasks from being started
            self2.btn_cek.hwnd().EnableWindow(false);
            self2.btn_cek.set_text("Wait");
            self2.variasi_combo.items().delete_all();
            self2.kurir_combo.items().delete_all();
            let file = self2.file_combo.text();
            if self2.url_text.text().is_empty() {
                let _ = func_main::error_cek(&self2.wnd, "Error", "Empty URL");
                println!("Empty URL");
                self2.btn_cek.hwnd().EnableWindow(true);
                //btn_cek.clone().hwnd().ShowWindow(SW::HIDE);
                self2.btn_cek.set_text("Cek");
            } else if file.is_empty() {
                let _ = func_main::error_cek(&self2.wnd, "Error", "Please select a file before running the program");
                println!("Please select a file before running the program");
                self2.btn_cek.hwnd().EnableWindow(true);
                self2.btn_cek.hwnd().ShowWindow(SW::SHOW);
                self2.btn_cek.set_text("Cek");
            }else{
                let start = self2.url_text.text();
                let cookie_content = prepare::read_cookie_file(&file);
                let cookie_data = prepare::create_cookie(&cookie_content);
                let url_1 = start.trim();
                println!("{}", url_1);
                let product_info = prepare::process_url(url_1);
                println!("{}, {}", product_info.shop_id, product_info.item_id);
                if product_info.shop_id != 0 && product_info.item_id != 0 {
                    println!("Ok URL");
                    let variasi_combo_clone = self2.variasi_combo.clone();
                    let kurir_combo_clone = self2.kurir_combo.clone();
                    let btn_cek_cek = self2.btn_cek.clone();
                    let wnd_clone_cek = self2.wnd.clone();
                    tokio::spawn(async move {
                        // Memanggil get_product dengan timeout
                        match timeout(Duration::from_secs(10), prepare::get_product(&product_info, &cookie_data)).await {
                            Ok(Ok((name, model_info, is_official_shop, rcode))) => {
                                if rcode == "200 OK" {
                                    let name_model_vec: Vec<String> = model_info.iter().map(|model| model.name.clone()).collect();
                                    for name_model in &name_model_vec {
                                        println!("{}", name_model);
                                        variasi_combo_clone.items().add(&[name_model]);
                                        variasi_combo_clone.items().select(Some(0));
                                    }
                                } else {
                                    println!("Error: {}", rcode);
                                    let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {}", rcode);
                                    let _ = func_main::error_cek(&wnd_clone_cek, "Error get Variation", &isi);
                                }
                            },
                            Ok(Err(e)) => {
                                println!("Error: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {:?}", e);
                                let _ = func_main::error_cek(&wnd_clone_cek, "Error get Variation", &isi);
                            },
                            Err(e) => {
                                println!("Timeout: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nTimeout : {:?}", e);
                                let _ = func_main::error_cek(&wnd_clone_cek, "Error get Variation", &isi);
                            }
                        };
    
                        // Memanggil get_kurir dengan timeout
                        let address_info = match prepare::address(&cookie_data).await {
                            Ok(address) => address,
                            Err(e) => {
                                // Handle the error case
                                eprintln!("Failed to get address: {}", e);
                                AddressInfo::default() // Early return or handle the error as needed
                            }
                        };
                        match timeout(Duration::from_secs(10), prepare::kurir(&cookie_data, &product_info, &address_info)).await {
                            Ok(Ok(kurirs)) => {
                                let kurirs_iter: Vec<String> = kurirs.iter().map(|kurirs| kurirs.channel_name.clone()).collect();
                                for name_kurir in &kurirs_iter {
                                    println!("{}", name_kurir);
                                    kurir_combo_clone.items().add(&[name_kurir]);
                                    kurir_combo_clone.items().select(Some(0));
                                }
                            },
                            Ok(Err(e)) => {
                                println!("Error: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {:?}", e);
                                let _ = func_main::error_cek(&wnd_clone_cek, "Error get Shipping", &isi);
                            },
                            Err(e) => {
                                println!("Timeout: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nTimeout : {:?}", e);
                                let _ = func_main::error_cek(&wnd_clone_cek, "Error get Shipping", &isi);
                            }
                        };
                        btn_cek_cek.clone().hwnd().EnableWindow(true);
                        btn_cek_cek.set_text("Cek");
                        Ok::<(), ()>(())
                    });
                }else{
                    let _ = func_main::error_cek(&self2.wnd, "Error", "Invalid URL");
                    println!("Invalid URL");
                    self2.btn_cek.hwnd().EnableWindow(true);
                    self2.btn_cek.set_text("Cek");
                }
            }
            Ok(())
        });
		let self2 = self.clone();
        self.btn_jalankan.on().bn_clicked(move || {
            let command = self2.generate_cmd();
            let file = self2.file_combo.text();
            if let Ok(Some(command))  = command {
                if !file.is_empty() {
                    let _status = std::process::Command::new("cmd")
                        .arg("/c")
                        .args(&command)
                        .spawn()
                        .expect("Gagal menjalankan program");
                    println!("{:?}", command);
                }else{
                    let _ = func_main::error_cek(&self2.wnd, "Error", "Please select a file before running the program");
                }
            }
            Ok(())
        });
        let self2 = self.clone();
        self.wnd.on().wm_create(move |_| {
            let version_info = env!("CARGO_PKG_VERSION");
            let log_message1 = format!("Launcher \x1b[3mNG\x1b[0m Auto Buy Shopee Version : {}", version_info);
            println!("{}", log_message1);
            let local: DateTime<Local> = Local::now();
            let hour = local.hour().to_string();
            let minute = match local.minute() {
                m if m <= 14 => "14",
                m if m <= 29 => "29",
                m if m <= 44 => "44",
                _ => "59",
            };
            self2.jam_text.set_text(&hour);
            self2.menit_text.set_text(&minute);
            self2.platform_combobox.hwnd().EnableWindow(false);
            self2.code_shop_text.hwnd().EnableWindow(false);
            func_main::set_visibility(&self2.promotionid_label, &self2.promotionid_text, false);
            func_main::set_visibility(&self2.signature_label, &self2.signature_text, false);
            func_main::set_visibility(&self2.code_label, &self2.code_platform_text, false);
            func_main::set_visibility(&self2.cid_label, &self2.cid_text, false);
            // Panggil fungsi untuk mengisi ComboBox dengan file di folder "akun"
            func_main::populate_combobox_with_files(&self2.file_combo, "akun");
            func_main::populate_payment_combo(&self2.payment_combo);
            Ok(0)
        });
        let self2 = self.clone();
        self.wnd.on().wm_context_menu(move || {
            let btn_hwnd = self2.btn_jalankan.hwnd();
            // Dapatkan posisi kursor menggunakan API Win32
            let cursor_pos = winsafe::GetCursorPos().unwrap();
            // Buat menu kontekstual
            let file_submenu = w::HMENU::CreatePopupMenu()?;

            file_submenu.append_item(&[
                w::MenuItem::Entry(101, "&Open video...\tCtrl+O"),
                w::MenuItem::Separator,
                w::MenuItem::Entry(co::DLGID::CANCEL.into(), "E&xit"),
            ])?;

            // Tampilkan menu di posisi kursor
            file_submenu.TrackPopupMenu(
                w::co::TPM::RIGHTBUTTON,
                cursor_pos,
                &btn_hwnd,
            ).unwrap();

            Ok(())
        });
        let self2 = self.clone();
        self.platform_combobox.on().cbn_sel_change(move || {
            println!("Platform Combobox clicked!");
            self2.platform_selection();
            Ok(())
        });
        let self2 = self.clone();
        self.fsv_checkbox.on().bn_clicked(move || {
            println!("fsv only clicked!");
            if self2.fsv_checkbox.is_checked() == true{
                self2.shop_checkbox.set_check_state(gui::CheckState::Unchecked);
                self2.platform_checkbox.set_check_state(gui::CheckState::Unchecked);
                self2.platform_combobox.hwnd().EnableWindow(false);
                func_main::set_visibility(&self2.promotionid_label, &self2.promotionid_text, false);
                func_main::set_visibility(&self2.signature_label, &self2.signature_text, false);
                func_main::set_visibility(&self2.code_label, &self2.code_platform_text, false);
                self2.code_shop_text.hwnd().EnableWindow(false);
            }
            Ok(())
        });
        let self2 = self.clone();
        self.shop_checkbox.on().bn_clicked(move || {
            println!("Code clicked!");
            if self2.shop_checkbox.is_checked() == true{
                self2.fsv_checkbox.set_check_state(gui::CheckState::Unchecked);
                self2.code_shop_text.hwnd().EnableWindow(true);
                self2.code_shop_text.hwnd().ShowWindow(SW::SHOW);
            }else{
                self2.code_shop_text.hwnd().EnableWindow(false);
            }
            Ok(())
        });
        let self2 = self.clone();
        self.platform_checkbox.on().bn_clicked(move || {
            println!("Voucher clicked!");
            if self2.platform_checkbox.is_checked() == true{
                self2.fsv_checkbox.set_check_state(gui::CheckState::Unchecked);
                self2.platform_combobox.hwnd().EnableWindow(true);
                func_main::set_visibility(&self2.promotionid_label, &self2.promotionid_text, true);
                func_main::set_visibility(&self2.signature_label, &self2.signature_text, true);
                self2.platform_selection();
            }else{
                self2.platform_combobox.hwnd().EnableWindow(false);
                self2.platform_combobox.items().select(Some(0));
                self2.platform_selection();
                func_main::set_visibility(&self2.promotionid_label, &self2.promotionid_text, false);
                func_main::set_visibility(&self2.signature_label, &self2.signature_text, false);
                self2.promotionid_label.hwnd().EnableWindow(false);
                self2.promotionid_text.hwnd().EnableWindow(false);
                self2.signature_label.hwnd().EnableWindow(false);
                self2.signature_text.hwnd().EnableWindow(false);
            }
            Ok(())
        });
        let self2 = self.clone();
		self.wnd.on().wm_command_accel_menu(1 as u16, move || {
            println!("Menu clicked!");
			Ok(())
		});
        let self2 = self.clone();
        self.wnd.on().wm_command_accel_menu(2 as u16, move || {
            func_main::populate_combobox_with_files(&self2.file_combo, "akun");
            func_main::populate_payment_combo(&self2.payment_combo);
			Ok(())
		});
        let self2 = self.clone();
        self.wnd.on().wm_command_accel_menu(3 as u16, move || {
			self2.wnd.close(); // close on ESC
			Ok(())
		});
        let wnd = self.wnd.clone();
		self.wnd.on().wm_command_accel_menu(8 as u16, move || { 
            let dont_move = (gui::Horz::None, gui::Vert::None);
            println!("About APL");
            let version_info = env!("CARGO_PKG_VERSION");
            let version_message = match (is_server(), OsVersion::current()) {
                (true, version) if version >= OsVersion::new(10, 0, 0, 20348) => "Windows Server 2022".to_string(),
                (true, version) if version >= OsVersion::new(10, 0, 0, 17763) => "Windows Server 2019".to_string(),
                (true, version) if version >= OsVersion::new(10, 0, 0, 14393) => "Windows Server 2016".to_string(),
                (true, version) if version >= OsVersion::new(6, 3, 0, 9800) => "Windows Server 2012 R2".to_string(),
                (true, version) if version >= OsVersion::new(6, 2, 0, 9200) => "Windows Server 2012".to_string(),
                (true, version) if version >= OsVersion::new(6, 1, 0, 7600) => "Windows Server 2008 R2".to_string(),
                (true, _) => format!("Windows Server Build {:?}", OsVersion::current()),
                (false, version) if version >= OsVersion::new(10, 0, 0, 22000) => format!("Windows 11 Build {:?}", OsVersion::current().build),
                (false, version) if version >= OsVersion::new(10, 0, 0, 10240) => format!("Windows 10 Build {:?}", OsVersion::current().build),
                (false, version) if version >= OsVersion::new(6, 3, 0, 9800) => format!("Windows 8.1 Build {:?}", OsVersion::current().build),
                (false, version) if version >= OsVersion::new(6, 2, 0, 9200) => format!("Windows 8 Build {:?}", OsVersion::current().build),
                (false, version) if version >= OsVersion::new(6, 1, 1, 7601) => "Windows 7 SP1".to_string(),
                (false, version) if version >= OsVersion::new(6, 1, 0, 7600) => "Windows 7 RTM".to_string(),
                _ => format!("Running on an unsupported version {:?}", OsVersion::current()),
            };
            let run_win = match func_main::detect_wine() {
                Ok(result) => {
                    println!("{}", result);
                    result
                },
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    "Error".to_string()
                }
            };
            println!("{}", run_win);
            let wnd2 = gui::WindowModal::new_dlg(&wnd, 2000);
            let ok_button = gui::Button::new_dlg(&wnd2, 2001, dont_move);
            let ver_label = gui::Label::new_dlg(&wnd2, 2002, dont_move);
            let os_label = gui::Label::new_dlg(&wnd2, 2003, dont_move);
            let run_label = gui::Label::new_dlg(&wnd2, 2004, dont_move);
            //ver_label.set_text_and_resize(&version_info);
            let wnd2_clone = wnd2.clone();
            ok_button.on().bn_clicked(move || {
                println!("Close About APL");
                wnd2_clone.close();
                Ok(())
            });
            wnd2.on().wm_init_dialog(move |_| {
                println!("Start About APL");
                ver_label.set_text_and_resize(&version_info);
                os_label.set_text_and_resize(&version_message);
                run_label.set_text_and_resize(&run_win);
                Ok(true)
            });
            let _ = wnd2.show_modal();
			Ok(())
		});
        let wnd = self.wnd.clone();
        self.wnd.on().wm_command_accel_menu(5 as u16, move || {
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
                save_button.on().bn_clicked(move || {
                    let file = file_combo_clone.text();
                    let cookie = cookie_edit.text();
                    let sz = sz_edit.text();
                    if file.is_empty() {
                        println!("Please select a file before saving the cookie");
                    } else if cookie.is_empty() {
                        println!("Please input the cookie before saving");
                    } else {
                        if file.contains(".txt") {
                            println!("File contains .txt");
                            let _ = func_main::save_cookie_fp_file(&file, &cookie, &sz);
                        } else {
                            println!("File does not contain .txt");
                            let file_fix = format!("{}.txt", file);
                            let _ = func_main::save_cookie_fp_file(&file_fix, &cookie, &sz);
                        }
                        println!("Cookie saved successfully");
                    }
                    Ok(())
                });
                let _ = wnd2.show_modal();
                //wnd2.run_main(None);
            });
			Ok(())
		});
        let wnd = self.wnd.clone();
        self.wnd.on().wm_command_accel_menu(6 as u16, move || {
            let dont_move = (gui::Horz::None, gui::Vert::None);
            let wnd2 = gui::WindowModal::new_dlg(&wnd, 500);
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
            wnd2.on().wm_init_dialog(move |_| {
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
                                println!("saved success");
                                //let isi = format!("Token and chat ID saved successfully");
                                //let _ = func_main::info_modal(&wnd2_clone, "Success", &isi);
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
		});
	}
    fn generate_cmd(&self) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
        let self2 = self.clone();
        println!("Jalankan button clicked!");
        let url = self2.url_text.text();
        let Some(payment) = self2.payment_combo.items().selected_text() else {
            eprintln!("Payment is not selected");
            return Ok(None);
        };
        let harga = if self2.harga_checkbox.check_state() == gui::CheckState::Checked{
            self2.harga_text.text()
        }else{
            String::new()
        };
        let Some(file) = self2.file_combo.items().selected_text() else {
            eprintln!("File is not selected");
            return Ok(None);
        };
        let variasi = if let Some(variasi) = self2.variasi_combo.items().selected_text() {
            variasi
        } else {
            eprintln!("Variasi is not selected, using default value.");
            String::new() // Outputkan nilai kosong
        };           
        let Some(kurir) = self2.kurir_combo.items().selected_text() else {
            eprintln!("kurir is not selected");
            return Ok(None);
        };
        println!("{}", variasi);            
        let jam = self2.jam_text.text();
        let menit = self2.menit_text.text();
        let detik = self2.detik_text.text();
        let mili = self2.mili_text.text();
        let kuan = self2.kuan_text.text();
        let time_arg = format!("{}:{}:{}.{}", &jam, &menit, &detik, &mili);
        //let token = self2.token_text.text();
        let code_platform_text = self2.code_platform_text.text();
        let code_shop_text = self2.code_shop_text.text();
        let promotionid_text = self2.promotionid_text.text();
        let signature_text = self2.signature_text.text();
        let collectionid = self2.cid_text.text();
        let url_1 = url.trim();
        println!("{}", url_1);
        let product_info = prepare::process_url(url_1);
        let refe = format!("https://shopee.co.id/product/{}/{}", product_info.shop_id, product_info.item_id);
        // Menjalankan program abs.exe dengan argumen yang dibuat
        let create_command = |extra_args: Vec<String>| -> Vec<String> {
            let mut command = vec![
                "start".to_string(),
                "abs.exe".to_string(),
                "--file".to_string(), file,
                "--url".to_string(), refe,
                "--time".to_string(), time_arg,
                "--kurir".to_string(), kurir,
                "--payment".to_string(), payment,
                "--harga".to_string(), harga,
                "--quantity".to_string(), kuan,
                "--token".to_string(), "".to_string(),
            ];
            // Tambahkan --product hanya jika variasi_combo memiliki lebih dari 1 item
            let count = match unsafe {
                self2.variasi_combo.hwnd().SendMessage(w::msg::cb::GetCount {})
            } {
                Ok(count) => count,
                Err(e) => {
                    eprintln!("Failed to get ComboBox count: {:?}", e);
                    0 // Fallback ke nilai default
                }
            };
            if count > 1 {
                command.push("--product".to_string());
                command.push(variasi);
            }
            command.extend(extra_args);
            //
            println!("{:?}", command);
            command
        };
        let mut commands = vec![];

        if self2.fsv_checkbox.check_state() == gui::CheckState::Checked {
            commands.push("--fsv-only".to_string());
        }
        if self2.platform_checkbox.check_state() == gui::CheckState::Checked {
            match self2.platform_combobox.items().selected_index() {
                Some(0) => {
                    commands.push("--claim-platform-vouchers".to_string());
                    commands.push("--pro-id".to_string());
                    commands.push(promotionid_text);
                    commands.push("--sign".to_string());
                    commands.push(signature_text);
                }
                Some(1) => {
                    commands.push("--platform-vouchers".to_string());
                    commands.push("--code-platform".to_string());
                    commands.push(code_platform_text);
                }
                Some(2) => {
                    commands.push("--collection-vouchers".to_string());
                    commands.push("--collectionid".to_string());
                    commands.push(collectionid);
                }
                _ => {}
            }
        }
        if self2.shop_checkbox.check_state() == gui::CheckState::Checked {
            commands.push("--shop-vouchers".to_string());
            commands.push("--code-shop".to_string());
            commands.push(code_shop_text);
        }
        Ok(Some(create_command(commands)))
    }
    fn platform_selection(&self) {
        let index = self.platform_combobox.items().selected_index();
        match index {
            Some(0) => {
                self.promotionid_label.hwnd().EnableWindow(true);
                self.promotionid_text.hwnd().EnableWindow(true);
                self.signature_label.hwnd().EnableWindow(true);
                self.signature_text.hwnd().EnableWindow(true);
                func_main::set_visibility(&self.promotionid_label, &self.promotionid_text, true);
                func_main::set_visibility(&self.signature_label, &self.signature_text, true);
                func_main::set_visibility(&self.code_label, &self.code_platform_text, false);
                func_main::set_visibility(&self.cid_label, &self.cid_text, false);
            },
            Some(1) => {
                func_main::set_visibility(&self.code_label, &self.code_platform_text, true);
                func_main::set_visibility(&self.promotionid_label, &self.promotionid_text, false);
                func_main::set_visibility(&self.signature_label, &self.signature_text, false);
                func_main::set_visibility(&self.cid_label, &self.cid_text, false);
            },
            Some(2) => {
                func_main::set_visibility(&self.cid_label, &self.cid_text, true);
                func_main::set_visibility(&self.promotionid_label, &self.promotionid_text, false);
                func_main::set_visibility(&self.signature_label, &self.signature_text, false);
                func_main::set_visibility(&self.code_label, &self.code_platform_text, false);
            },
            _ => {
                self.promotionid_label.hwnd().EnableWindow(false);
                self.promotionid_text.hwnd().EnableWindow(false);
                self.signature_label.hwnd().EnableWindow(false);
                self.signature_text.hwnd().EnableWindow(false);
            }
        }
    }
}