use winsafe::{self as w,
    gui, prelude::*, co::{self, ES, SS, SW}, AnyResult,
    HMENU, guard, HWND,
};
use ::runtime::prepare::{self, AddressInfo, ShippingInfo, ModelInfo, PaymentInfo,};
use runtime::crypt;
use tokio::{time::{timeout, Duration}};
use chrono::{Local, DateTime, Timelike};
use std::sync::{Arc, Mutex};

use crate::func_main;
use crate::about;
use crate::manager;

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
    link_label: gui::Label,
    link_text: gui::Edit,
    coins_checkbox: gui::CheckBox,
    bypass_checkbox: gui::CheckBox,
    shared_payment_data: Arc<Mutex<Vec<PaymentInfo>>>,
    shared_variation_data: Arc<Mutex<Vec<ModelInfo>>>,
    shared_kurir_data: Arc<Mutex<Vec<ShippingInfo>>>,
}

impl MyWindow {
	pub fn new() -> Self {
        let shared_payment_data = Arc::new(Mutex::new(vec![]));
        let shared_variation_data = Arc::new(Mutex::new(vec![]));
        let shared_kurir_data = Arc::new(Mutex::new(vec![]));
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
        //let _ = manager::log_window(&wnd);
        let _status_bar = gui::StatusBar::new(
            &wnd,
            &[
                gui::SbPart::Fixed(200),      // 200 pixels, never resizes
                gui::SbPart::Proportional(1), // these two will fill the remaning space
                gui::SbPart::Proportional(1),
            ],
        );
    
        // Input URL
        let _lbl_url = gui::Label::new(&wnd, gui::LabelOpts {
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
        let _lbl_payment = gui::Label::new(&wnd, gui::LabelOpts {
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
        let _lbl_file = gui::Label::new(&wnd, gui::LabelOpts {
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
        // Harga Max & Kuantiti
        let _lbl_harga = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Harga Max".to_owned(),
            position: (10, 80),
            ..Default::default()
        });
        let harga_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "1000".to_owned(),
            position: (80, 80),
            width: 150,
            control_style: ES::NUMBER,
            resize_behavior: (gui::Horz::Resize, gui::Vert::None),
            ..Default::default()
        });
        let harga_checkbox = gui::CheckBox::new(&wnd, gui::CheckBoxOpts {
            text: "Set".to_owned(),
            position: (250, 80),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
    
        let _lbl_qty = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Kuantiti".to_owned(),
            position: (10, 110),
            size: (60, 20),
            ..Default::default()
        });
        let kuan_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "1".to_owned(),
            position: (80, 110),
            width: 210,
            control_style: ES::NUMBER,
            resize_behavior: (gui::Horz::Resize, gui::Vert::None),
            ..Default::default()
        });
    
        let _lbl_kurir = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Kurir".to_owned(),
            position: (310, 110),
            size: (60, 20),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
        let kurir_combo = gui::ComboBox::new(&wnd, gui::ComboBoxOpts {
            position: (380, 110),
            width: 210,
            window_style: co::WS::CHILD | co::WS::VISIBLE | co::WS::TABSTOP | co::WS::VSCROLL | co::WS::GROUP | co::CBS::AUTOHSCROLL.into() | co::CBS::DISABLENOSCROLL.into(),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
    
        let _lbl_variasi = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Variasi".to_owned(),
            position: (310, 80),
            size: (60, 20),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
        let variasi_combo = gui::ComboBox::new(&wnd, gui::ComboBoxOpts {
            position: (380, 80),
            width: 210,
            window_style: co::WS::CHILD | co::WS::VISIBLE | co::WS::TABSTOP | co::WS::VSCROLL | co::WS::GROUP | co::CBS::AUTOHSCROLL.into() | co::CBS::DISABLENOSCROLL.into(),
            resize_behavior: (gui::Horz::Repos, gui::Vert::None),
            ..Default::default()
        });
        let coins_checkbox = gui::CheckBox::new(&wnd, gui::CheckBoxOpts {
            text: "Use Coins".to_owned(),
            position: (380, 160),
            size: (80, 20),
            check_state: co::BST::CHECKED,
            ..Default::default()
        });
        let bypass_checkbox = gui::CheckBox::new(&wnd, gui::CheckBoxOpts {
            text: "Bypass".to_owned(),
            position: (480, 160),
            size: (80, 20),
            check_state: co::BST::CHECKED,
            ..Default::default()
        });
    
        // Time
        let _time_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Time".to_owned(),
            position: (10, 160),
            size: (60, 20),
            ..Default::default()
        });
        let _jam_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Jam".to_owned(),
            position: (80, 160),
            size: (60, 20),
            control_style: SS::CENTER,
            ..Default::default()
        });
        let jam_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "23".to_owned(),
            position: (80, 190),
            width: 60,
            control_style: ES::NUMBER,
            ..Default::default()
        });
        let _menit_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Menit".to_owned(),
            position: (150, 160),
            size: (60, 20),
            control_style: SS::CENTER,
            ..Default::default()
        });
        let menit_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "59".to_owned(),
            position: (150, 190),
            width: 60,
            control_style: ES::NUMBER,
            ..Default::default()
        });
        let _detik_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Detik".to_owned(),
            position: (220, 160),
            size: (60, 20),
            control_style: SS::CENTER,
            ..Default::default()
        });
        let detik_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "59".to_owned(),
            position: (220, 190),
            width: 60,
            control_style: ES::NUMBER,
            ..Default::default()
        });
        let _mili_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Mili".to_owned(),
            position: (290, 160),
            size: (60, 20),
            control_style: SS::CENTER,
            ..Default::default()
        });
        let mili_text = gui::Edit::new(&wnd, gui::EditOpts {
            text: "900".to_owned(),
            position: (290, 190),
            width: 60,
            control_style: ES::NUMBER,
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
            items: vec!["Claim".to_owned(), "Code".to_owned(), "Collection id".to_owned(), "Link".to_owned()],
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

        let link_label = gui::Label::new(&wnd, gui::LabelOpts {
            text: "Link Voucher".to_owned(),
            position: (35, 300),
            ..Default::default()
        });

        let link_text = gui::Edit::new(&wnd, gui::EditOpts {
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
            promotionid_label, promotionid_text,
            signature_label, signature_text,
            cid_label, cid_text,
            link_label, link_text,
            coins_checkbox,
            bypass_checkbox,
            shared_payment_data, shared_variation_data, shared_kurir_data
        };
        new_self.events(); // attach our events
		new_self

    }
    pub fn run(&self) -> AnyResult<i32> {
        self.wnd.run_main(None)
	}

    fn build_menu() -> w::AnyResult<(HMENU, guard::DestroyAcceleratorTableGuard)> {
        let mut main_menu = w::HINSTANCE::GetModuleHandle(None)?
            .LoadMenu(w::IdStr::Id(1)).unwrap();
        let lmain_menu = main_menu.leak();
        let accel_table = w::HINSTANCE::GetModuleHandle(None)?
            .LoadAccelerators(w::IdStr::Str(w::WString::from_str("MENU1"))).unwrap();
		Ok((lmain_menu, accel_table))
	}

	fn events(&self) {
        let self2 = self.clone();
        self.btn_cek.on().bn_clicked(move || {
            println!("Cek button clicked!");
            println!("{}", self2.url_text.text().unwrap_or_else(|_| String::new()));
            // Disable the button to prevent multiple async tasks from being started
            self2.btn_cek.hwnd().EnableWindow(false);
            let _ = self2.btn_cek.hwnd().SetWindowText("Wait");
            self2.variasi_combo.items().delete_all();
            self2.kurir_combo.items().delete_all();
            let file = match self2.file_combo.items().selected_text() {
                    Ok(Some(text)) => text,
                    Ok(None) => "".to_string(),
                    Err(_) => "".to_string()
            };
            if self2.url_text.text().unwrap_or_else(|_| String::new()).is_empty() {
                let _ = func_main::error_cek(&self2.wnd, "Error", "Empty URL");
                println!("Empty URL");
                self2.btn_cek.hwnd().EnableWindow(true);
                //btn_cek.clone().hwnd().ShowWindow(SW::HIDE);
                let _ = self2.btn_cek.hwnd().SetWindowText("Cek");
            } else if file.is_empty() {
                let _ = func_main::error_cek(&self2.wnd, "Error", "Please select a file before running the program");
                println!("Please select a file before running the program");
                self2.btn_cek.hwnd().EnableWindow(true);
                self2.btn_cek.hwnd().ShowWindow(SW::SHOW);
                let _ = self2.btn_cek.hwnd().SetWindowText("Cek");
            }else{
                let cookie_data = prepare::CookieData::create_cookie(&prepare::read_cookie_file(&file));
                let device_info = crypt::create_devices(&func_main::get_fp_data(&file));
                let self2 = self2.clone();
                tokio::spawn(async move {
                    let client = Arc::new(prepare::universal_client_skip_headers().await);
                    let mut product_info = prepare::process_url(&self2.url_text.text().unwrap_or_else(|_| String::new()).trim());
                    if product_info.shop_id == 0 && product_info.item_id == 0 {
                        println!("Cek apakah redirect?");
                        match prepare::get_redirect_url(&self2.url_text.text().unwrap_or_else(|_| String::new()).trim()).await {
                            Ok(redirect) => {
                                product_info = prepare::process_url(&redirect);
                            }
                            Err(e) => {
                                eprintln!("Gagal mendapatkan redirect: {:?}", e);
                            }
                        }
                    }
                    println!("{}, {}", product_info.shop_id, product_info.item_id);
                    if product_info.shop_id != 0 && product_info.item_id != 0 {
                        println!("Ok URL");
                        let shared_variation_clone = self2.shared_variation_data.clone();
                        let shared_payment_data_clone = self2.shared_payment_data.clone();
                        let shared_kurir_data_clone = self2.shared_kurir_data.clone();
                        match timeout(Duration::from_secs(10), prepare::get_product(client.clone(), &product_info, &cookie_data)).await {
                            Ok(Ok((_name, model_info, _is_official_shop, fs_info, rcode))) => {
                                if rcode == "200 OK" {
                                    let fs_items = if fs_info.promotionid != 0 {
                                        println!("promotionid  : {}", fs_info.promotionid);
                                        println!("start_time   : {}", func_main::human_readable_time(fs_info.start_time));
                                        println!("end_time     : {}", func_main::human_readable_time(fs_info.end_time));
                                        match prepare::get_flash_sale_batch_get_items(client.clone(), &cookie_data, &[product_info.clone()], &fs_info).await {
                                            Ok(body) => body,
                                            Err(e) => {
                                                eprintln!("Error in get_flash_sale_batch_get_items: {:?}", e);
                                                Vec::new() // Jika error, kembalikan array kosong
                                            }
                                        }
                                    }else {
                                        Vec::new()
                                    };
                                    for (index, model) in model_info.iter().enumerate() {
                                        let flashsale = if let Some(item) = fs_items.iter().find(|item| item.modelids.as_ref().expect("").contains(&model.modelid)) {
                                            format!(
                                                "[FLASHSALE] - Est≉  {} - Hide: {} - fs-stok: {}",
                                                func_main::format_thousands(item.price_before_discount * (100 - item.raw_discount) / 100 / 100000),
                                                item.hidden_price_display.as_deref().unwrap_or("N/A"),
                                                item.stock
                                            )
                                        } else {
                                            String::new()
                                        };
                                        println!("{}. {} - Harga: {} - Stok: {} {}", index + 1, model.name, func_main::format_thousands(model.price / 100000), model.stock, flashsale);
                                    }
                                    let mut shared = shared_variation_clone.lock().unwrap();
                                    shared.clear();
                                    *shared = model_info.clone(); 
                                    let name_model_vec: Vec<String> = model_info.iter().map(|model| model.name.clone()).collect();
                                    for name_model in &name_model_vec {
                                        let _ = self2.variasi_combo.items().add(&[name_model]);
                                        self2.variasi_combo.items().select(Some(0));
                                    }
                                } else {
                                    println!("Error: {}", rcode);
                                    let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {}", rcode);
                                    let _ = func_main::error_cek(&self2.wnd, "Error get Variation", &isi);
                                }
                            },
                            Ok(Err(e)) => {
                                println!("Error: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {:?}", e);
                                let _ = func_main::error_cek(&self2.wnd, "Error get Variation", &isi);
                            },
                            Err(_) => {
                                eprintln!("Timeout occurred");
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nTimeout : Timeout occurred");
                                let _ = func_main::error_cek(&self2.wnd, "Error get Variation", &isi);
                            }
                        };
                        let base_headers = Arc::new(prepare::create_headers(&cookie_data));
                        let shared_headers = Arc::new(runtime::task::headers_checkout(&cookie_data));
                        let address_info = match prepare::address(client.clone(), base_headers.clone()).await {
                            Ok(address) => address,
                            Err(e) => {
                                // Handle the error case
                                eprintln!("Failed to get address: {}", e);
                                AddressInfo::default() // Early return or handle the error as needed
                            }
                        };
                        let mut chosen_model = {
                            let shared_v = shared_variation_clone.lock().unwrap();
                            shared_v.get(0).cloned().unwrap_or_else(|| {ModelInfo::default()})
                        };
                        let chosen_shipping = ShippingInfo::default();
                        let chosen_payment = {
                            let shared_p = shared_payment_data_clone.lock().unwrap();
                            shared_p.get(0).unwrap().clone()
                        };
                        chosen_model.quantity = self2.kuan_text.text().unwrap_or_else(|_| String::new()).parse::<i32>().unwrap_or(1);
                        match timeout(Duration::from_secs(10), runtime::prepare_ext::get_shipping_data(client.clone(), base_headers.clone(), shared_headers.clone(), &device_info, Some(&product_info), &address_info, &chosen_model, &chosen_payment, &chosen_shipping)).await {
                            Ok(Ok(kurirs)) => {
                                let mut shared = shared_kurir_data_clone.lock().unwrap();
                                shared.clear();
                                *shared = kurirs.clone(); 
                                for (index, shipping) in kurirs.iter().enumerate() {
                                    println!("{}. {} - Harga: {} - Id: {}", index + 1, shipping.channel_name, shipping.original_cost / 100000, shipping.channelid);
                                    let _ = self2.kurir_combo.items().add(&[shipping.channel_name.clone()]);
                                    self2.kurir_combo.items().select(Some(0));
                                }
                            },
                            Ok(Err(e)) => {
                                println!("Error: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {:?}", e);
                                let _ = func_main::error_cek(&self2.wnd, "Error get Shipping", &isi);
                            },
                            Err(e) => {
                                println!("Timeout: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nTimeout : {:?}", e);
                                let _ = func_main::error_cek(&self2.wnd, "Error get Shipping", &isi);
                            }
                        };
                        self2.btn_cek.clone().hwnd().EnableWindow(true);
                        let _ = self2.btn_cek.hwnd().SetWindowText("Cek");
                    }else{
                        let _ = func_main::error_cek(&self2.wnd, "Error", "Invalid URL");
                        println!("Invalid URL");
                        self2.btn_cek.hwnd().EnableWindow(true);
                        let _ = self2.btn_cek.hwnd().SetWindowText("Cek");
                    }
                });
            };
            Ok(())
        });
		let self2 = self.clone();
        self.btn_jalankan.on().bn_clicked(move || {
            let command = self2.generate_cmd();
            let mut new_command = Vec::new();
            if let Ok(Some(command))  = command {
                new_command.push("start".to_string());
                new_command.push("abs.exe".to_string());
                new_command.extend(command);
                let _ = self2.execute(new_command);
            }
            Ok(())
        });
        let self2 = self.clone();
        self.harga_checkbox.on().bn_clicked(move || {
            if self2.harga_checkbox.is_checked() == true{
                self2.harga_text.hwnd().EnableWindow(true);
                self2.bypass_checkbox.hwnd().EnableWindow(true);
            }else{
                self2.harga_text.hwnd().EnableWindow(false);
                self2.bypass_checkbox.hwnd().EnableWindow(false);
                self2.bypass_checkbox.set_state(co::BST::UNCHECKED);
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
            self2.coins_checkbox.set_state(co::BST::CHECKED);
            self2.fsv_checkbox.set_state(co::BST::CHECKED);
            let _ = self2.jam_text.set_text(&hour);
            let _ = self2.menit_text.set_text(&minute);
            self2.platform_combobox.hwnd().EnableWindow(false);
            self2.code_shop_text.hwnd().EnableWindow(false);
            self2.harga_text.hwnd().EnableWindow(false);
            self2.bypass_checkbox.hwnd().EnableWindow(false);
            func_main::set_visibility(&self2.promotionid_label, &self2.promotionid_text, false);
            func_main::set_visibility(&self2.signature_label, &self2.signature_text, false);
            func_main::set_visibility(&self2.code_label, &self2.code_platform_text, false);
            func_main::set_visibility(&self2.cid_label, &self2.cid_text, false);
            func_main::set_visibility(&self2.link_label, &self2.link_text, false);
            // Panggil fungsi untuk mengisi ComboBox dengan file di folder "akun"
            func_main::populate_combobox_with_files(&self2.file_combo, "akun");
            func_main::populate_payment_combo(&self2.payment_combo, self2.shared_payment_data.clone());
            Ok(0)
        });
        let self2 = self.clone();
        self.wnd.on().wm_context_menu(move || {
            let btn_hwnd = self2.wnd.hwnd();
            // Dapatkan posisi kursor menggunakan API Win32
            let cursor_pos = winsafe::GetCursorPos().unwrap();
            // Buat menu kontekstual
            let file_submenu = w::HMENU::CreatePopupMenu()?;

            file_submenu.append_item(&[
                w::MenuItem::Entry{cmd_id: 101, text: "Manual Run"},
                w::MenuItem::Entry{cmd_id: 102, text: "Generate Struct"},
                w::MenuItem::Entry{cmd_id: 103, text: "Save Voucher"},
                w::MenuItem::Separator,
                w::MenuItem::Entry{cmd_id: 3, text: "E&xit"},
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
            if self2.platform_combobox.items().selected_text() == Ok(Some("Claim".to_string())) || self2.platform_combobox.items().selected_text() == Ok(Some("Link".to_string())) {
                self2.bypass_checkbox.hwnd().EnableWindow(true);
            }else{
                self2.bypass_checkbox.hwnd().EnableWindow(false);
                self2.bypass_checkbox.set_state(co::BST::UNCHECKED);
            }
            Ok(())
        });
        let self2 = self.clone();
        self.fsv_checkbox.on().bn_clicked(move || {
            println!("fsv only clicked!");
            if self2.fsv_checkbox.is_checked() == true{
                self2.shop_checkbox.set_state(co::BST::UNCHECKED);
                self2.platform_checkbox.set_state(co::BST::UNCHECKED);
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
                self2.fsv_checkbox.set_state(co::BST::UNCHECKED);
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
                self2.fsv_checkbox.set_state(co::BST::UNCHECKED);
                self2.platform_combobox.hwnd().EnableWindow(true);
                func_main::set_visibility(&self2.promotionid_label, &self2.promotionid_text, true);
                func_main::set_visibility(&self2.signature_label, &self2.signature_text, true);
                self2.platform_selection();
                self2.bypass_checkbox.hwnd().EnableWindow(true);
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
                self2.bypass_checkbox.hwnd().EnableWindow(false);
                self2.bypass_checkbox.set_state(co::BST::UNCHECKED);
            }
            Ok(())
        });
        let self2 = self.clone();
        self.coins_checkbox.on().bn_clicked(move || {
            println!("Use Coins clicked!");
            if self2.coins_checkbox.is_checked() == true{
                self2.bypass_checkbox.set_state(co::BST::UNCHECKED);
            }
            Ok(())
        });
        let self2 = self.clone();
        self.bypass_checkbox.on().bn_clicked(move || {
            println!("Use Bypass clicked!");
            if self2.bypass_checkbox.is_checked() == true{
                self2.coins_checkbox.set_state(co::BST::UNCHECKED);
            }
            Ok(())
        });
		self.wnd.on().wm_command_acc_menu(101 as u16, move || {
            let command = vec!["start","abs.exe",];
            let _status = std::process::Command::new("cmd")
                .arg("/c")
                .args(&command)
                .spawn()
                .expect("Gagal menjalankan program");
            println!("{:?}", command);
			Ok(())
		});
        let self2 = self.clone();
		self.wnd.on().wm_command_acc_menu(102 as u16, move || {
            let command = self2.generate_cmd();
            if let Ok(Some(command))  = command {
                let url = self2.generate_struct(command);
                let _ = func_main::set_clipboard(&url);
            } else {
                println!("No command generated.");
            }
			Ok(())
		});
        let self2 = self.clone();
        self.wnd.on().wm_command_acc_menu(103 as u16, move || {
            let command = self2.generate_vouc();
            let mut new_command = Vec::new();
            if let Ok(Some(command))  = command {
                new_command.push("start".to_string());
                new_command.push("savevoucher.exe".to_string());
                new_command.extend(command);
                let _ = self2.generate_struct(new_command.clone());
                let _ = self2.execute(new_command);
            }
			Ok(())
		});
        
		self.wnd.on().wm_command_acc_menu(1 as u16, move || {
            println!("Menu clicked!");
			Ok(())
		});
        let self2 = self.clone();
        self.wnd.on().wm_command_acc_menu(2 as u16, move || {
            func_main::populate_combobox_with_files(&self2.file_combo, "akun");
            func_main::populate_payment_combo(&self2.payment_combo, self2.shared_payment_data.clone());
			Ok(())
		});
        let self2 = self.clone();
        self.wnd.on().wm_command_acc_menu(3 as u16, move || {
			self2.wnd.close(); // close on ESC
			Ok(())
		});
        let wnd = self.wnd.clone();
        self.wnd.on().wm_command_acc_menu(5 as u16, move || {
            let _ = manager::account_window(&wnd);
			Ok(())
		});
        let wnd = self.wnd.clone();
        self.wnd.on().wm_command_acc_menu(6 as u16, move || {
            let _ = manager::telegram_window(&wnd);
			Ok(())
		});
        let wnd = self.wnd.clone();
		self.wnd.on().wm_command_acc_menu(8 as u16, move || { 
            let _ = about::about_window(&wnd);
			Ok(())
		});
        let wnd = self.wnd.clone();
		self.wnd.on().wm_command_acc_menu(9 as u16, move || {
            println!("Menu clicked!");
            let _ = manager::show_fs_window(&wnd);
			Ok(())
		});
        let wnd = self.wnd.clone();
		self.wnd.on().wm_command_acc_menu(10 as u16, move || {
            println!("Menu clicked!");
            if let Err(e) = manager::log_window(&wnd) {
                HWND::NULL.MessageBox(
                    &format!("{:?}", e), "Uncaught error", co::MB::ICONERROR).unwrap();
            }
			Ok(())
		});
        let wnd = self.wnd.clone();
		self.wnd.on().wm_command_acc_menu(11 as u16, move || {
            let _ = manager::updater_window(&wnd);
			Ok(())
		});
        let wnd = self.wnd.clone();
		self.wnd.on().wm_command_acc_menu(12 as u16, move || {
            let _ = manager::Multi::new().run(&wnd);
			Ok(())
		});
	}
    fn generate_cmd(&self) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
        let self2 = self.clone();
        println!("Jalankan button clicked!");
        let url = self2.url_text.text().unwrap_or_else(|_| String::new());
        let Ok(Some(payment)) = self2.payment_combo.items().selected_text() else {
            eprintln!("Payment is not selected");
            return Ok(None);
        };
        let harga = if self2.harga_checkbox.state() == co::BST::CHECKED {
            self2.harga_text.text().unwrap_or_else(|_| String::new())
        }else{
            String::new()
        };
        let Ok(Some(file)) = self2.file_combo.items().selected_text() else {
            eprintln!("File is not selected");
            return Ok(None);
        };
        let variasi = if let Ok(Some(variasi)) = self2.variasi_combo.items().selected_text() {
            variasi
        } else {
            eprintln!("Variasi is not selected, using default value.");
            String::new() // Outputkan nilai kosong
        };           
        let Ok(Some(kurir)) = self2.kurir_combo.items().selected_text() else {
            eprintln!("kurir is not selected");
            return Ok(None);
        };
        println!("{}", variasi);            
        let jam = self2.jam_text.text().unwrap_or_else(|_| String::new());
        let menit = self2.menit_text.text().unwrap_or_else(|_| String::new());
        let detik = self2.detik_text.text().unwrap_or_else(|_| String::new());
        let mili = self2.mili_text.text().unwrap_or_else(|_| String::new());
        let kuan = self2.kuan_text.text().unwrap_or_else(|_| String::new());
        //let token = self2.token_text.text().unwrap_or_else(|_| String::new());
        let url_1 = url.clone();
        println!("{}", url_1);
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let url_1 = url_1.trim();
            let mut product_info = prepare::process_url(&url_1);
            if product_info.shop_id == 0 && product_info.item_id == 0 {
                if let Ok(redirect) = prepare::get_redirect_url(&url_1).await {
                    product_info = prepare::process_url(&redirect);
                }
            }
            let _ = tx.send(product_info);
        });
        let product_info = rx.recv().unwrap();
        let refe = format!("https://shopee.co.id/product/{}/{}", product_info.shop_id, product_info.item_id);
        // Menjalankan program abs.exe dengan argumen yang dibuat
        let create_command = |extra_args: Vec<String>| -> Vec<String> {
            let mut command = vec![
                "--file".to_string(), file,
                "--url".to_string(), refe,
                "--time".to_string(), format!("{}:{}:{}.{}", &jam, &menit, &detik, &mili),
                "--kurir".to_string(), kurir,
                "--payment".to_string(), payment,
                "--harga".to_string(), harga,
                "--quantity".to_string(), kuan,
                "--token".to_string(), "".to_string(),
                "--dump".to_string(), 
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
            command
        };
        let mut commands = vec![];

        if self2.fsv_checkbox.state() == co::BST::CHECKED {
            commands.push("--fsv-only".to_string());
        }
        if self2.coins_checkbox.state() == co::BST::UNCHECKED {
            commands.push("--no-coins".to_string());
        }        
        if self2.bypass_checkbox.state() == co::BST::CHECKED {
            commands.push("--bypass".to_string());
        }
        if self2.platform_checkbox.state() == co::BST::CHECKED {
            match self2.platform_combobox.items().selected_index() {
                Some(0) => {
                    commands.push("--claim-platform-vouchers".to_string());
                    commands.push("--pro-id".to_string());
                    commands.push(self2.promotionid_text.text().unwrap_or_else(|_| String::new()));
                    commands.push("--sign".to_string());
                    commands.push(self2.signature_text.text().unwrap_or_else(|_| String::new()));
                }
                Some(1) => {
                    commands.push("--platform-vouchers".to_string());
                    commands.push("--code-platform".to_string());
                    commands.push(self2.code_platform_text.text().unwrap_or_else(|_| String::new()));
                }
                Some(2) => {
                    commands.push("--collection-vouchers".to_string());
                    commands.push("--collectionid".to_string());
                    commands.push(self2.cid_text.text().unwrap_or_else(|_| String::new()));
                }
                Some(3) => {
                    let (proid, sign) = prepare::url_to_voucher_data(&self2.link_text.text().unwrap_or_else(|_| String::new()));
                    commands.push("--claim-platform-vouchers".to_string());
                    commands.push("--pro-id".to_string());
                    commands.push(proid);
                    commands.push("--sign".to_string());
                    commands.push(sign);
                }
                _ => {}
            }
        }
        if self2.shop_checkbox.state() == co::BST::CHECKED {
            commands.push("--shop-vouchers".to_string());
            commands.push("--code-shop".to_string());
            commands.push(self2.code_shop_text.text().unwrap_or_else(|_| String::new()));
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
                func_main::set_visibility(&self.link_label, &self.link_text, false);
            },
            Some(1) => {
                func_main::set_visibility(&self.code_label, &self.code_platform_text, true);
                func_main::set_visibility(&self.promotionid_label, &self.promotionid_text, false);
                func_main::set_visibility(&self.signature_label, &self.signature_text, false);
                func_main::set_visibility(&self.cid_label, &self.cid_text, false);
                func_main::set_visibility(&self.link_label, &self.link_text, false);
            },
            Some(2) => {
                func_main::set_visibility(&self.cid_label, &self.cid_text, true);
                func_main::set_visibility(&self.promotionid_label, &self.promotionid_text, false);
                func_main::set_visibility(&self.signature_label, &self.signature_text, false);
                func_main::set_visibility(&self.code_label, &self.code_platform_text, false);
                func_main::set_visibility(&self.link_label, &self.link_text, false);
            },
            Some(3) => {
                func_main::set_visibility(&self.link_label, &self.link_text, true);
                func_main::set_visibility(&self.cid_label, &self.cid_text, false);
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
    fn generate_vouc(&self) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
        let self2 = self.clone();
        let Ok(Some(file)) = self2.file_combo.items().selected_text() else {
            eprintln!("File is not selected");
            return Ok(None);
        };
        let jam = self2.jam_text.text().unwrap_or_else(|_| String::new());
        let menit = self2.menit_text.text().unwrap_or_else(|_| String::new());
        let detik = self2.detik_text.text().unwrap_or_else(|_| String::new());
        let mili = self2.mili_text.text().unwrap_or_else(|_| String::new());
        let create_command = |extra_args: Vec<String>| -> Vec<String> {
            let mut command = vec![
                "--file".to_string(), file,
                "--time".to_string(), format!("{}:{}:{}.{}", &jam, &menit, &detik, &mili),
            ];
            command.extend(extra_args);
            command
        };
        let mut commands = vec![];

        if self2.platform_checkbox.state() == co::BST::CHECKED {
            match self2.platform_combobox.items().selected_index() {
                Some(0) => {
                    commands.push("--mode".to_string());
                    commands.push("1".to_string());                    
                    commands.push("--pro-id".to_string());
                    commands.push(self2.promotionid_text.text().unwrap_or_else(|_| String::new()));
                    commands.push("--sign".to_string());
                    commands.push(self2.signature_text.text().unwrap_or_else(|_| String::new()));
                }
                //future code
                /*Some(1) => {
                    commands.push("--platform-vouchers".to_string());
                    commands.push("--code-platform".to_string());
                    commands.push(code_platform_text);
                }*/
                Some(2) => {
                    commands.push("--mode".to_string());
                    commands.push("2".to_string());
                    commands.push("--collectionid".to_string());
                    commands.push(self2.cid_text.text().unwrap_or_else(|_| String::new()));
                }
                Some(3) => {
                    let (proid, sign) = prepare::url_to_voucher_data(&self2.link_text.text().unwrap_or_else(|_| String::new()));
                    commands.push("--mode".to_string());
                    commands.push("1".to_string());                    
                    commands.push("--pro-id".to_string());
                    commands.push(proid);
                    commands.push("--sign".to_string());
                    commands.push(sign);
                }
                _ => {}
            }
        }
        Ok(Some(create_command(commands)))
    }
    fn execute(&self, command: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let file = match self.file_combo.items().selected_text() {
            Ok(Some(file)) => file,
            Ok(None) => {
                eprintln!("File is not selected");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error getting selected file: {}", e);
                return Ok(());
            }   
        };
        if !file.is_empty() {
            let _status = std::process::Command::new("cmd")
                .arg("/c")
                .args(&command)
                .spawn()
                .expect("Gagal menjalankan program");
        }else{
            let _ = func_main::error_cek(&self.wnd, "Error", "Please select a file before running the program");
        }
        Ok(())
    }
    fn generate_struct(&self, command: Vec<String>) -> String {
        let command_str = command
            .iter()
            .map(|s| {
                if s.trim().is_empty() {
                    "\" \"".to_string() // Mengganti nilai kosong dengan " "
                } else {
                    format!("\"{}\"", s) // Format nilai asli
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        println!("{}", &command_str);
        command_str
    }
}