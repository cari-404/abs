#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use ::runtime::prepare::{self, ModelInfo, ShippingInfo, PaymentInfo};
use native_windows_gui as nwg;
use native_windows_derive::NwgUi;
use native_windows_gui::NativeUi;
use serde_json::{Value};
use std::{fs, thread, io::Read};
use single_instance::SingleInstance;
use tokio::{runtime, time::{timeout, Duration}};
use std::sync::{Arc, RwLock};
use chrono::prelude::*;
use windows_version::*;

mod manager;
mod new;

#[derive(Default)]
pub struct SharedData {
    name_model: Vec<String>,
    kurirs: Vec<String>,
    rcode: String,
    logs: Vec<String>,
    ctrl_pressed: bool,
}

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_control(
        size: (500, 300),
        position: (300, 300),
        title: "Launcher for ABS",
        //flags: "WINDOW|VISIBLE|MINIMIZE_BOX|SYS_MENU",
        icon: Some(&nwg::Icon::from_bin(include_bytes!("32l.ico")).unwrap()),
        center: true,
    )]
    #[nwg_events( OnWindowClose: [App::exit], OnInit: [App::init], OnKeyRelease:[App::key(SELF, EVT_DATA)])]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 3)]
    grid: nwg::GridLayout,

    #[nwg_control(text: "URL")]
    #[nwg_layout_item(layout: grid, col: 0, row: 0)]
    url_label: nwg::Label,

    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: grid, col: 1, row: 0, col_span: 4)]
    url_text: nwg::TextInput,
    
    #[nwg_control(text: "Cek")]
    #[nwg_layout_item(layout: grid, col: 5, row: 0)]
    #[nwg_events( OnButtonClick: [App::cek], OnMouseMove: [App::on_hover])]
    cek_button: nwg::Button,

    #[nwg_control(text: "Payment")]
    #[nwg_layout_item(layout: grid, col: 0, row: 1)]
    payment_label: nwg::Label,

    #[nwg_control(v_align: nwg::VTextAlign::Bottom)]
    #[nwg_layout_item(layout: grid, col: 1, row: 1, col_span: 2)]
    payment_combo: nwg::ComboBox<String>,

    #[nwg_control(text: "Harga Max")]
    #[nwg_layout_item(layout: grid, col: 0, row: 2)]
    harga_label: nwg::Label,

    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: grid, col: 1, row: 2, col_span: 2)]
    harga_text: nwg::TextInput,
    
    #[nwg_control(text: "Kuantiti")]
    #[nwg_layout_item(layout: grid, col: 0, row: 3)]
    kuan_label: nwg::Label,

    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: grid, col: 1, row: 3, col_span: 2)]
    kuan_text: nwg::TextInput,

    #[nwg_control(text: "Pilih file")]
    #[nwg_layout_item(layout: grid, col: 3, row: 1)]
    file_label: nwg::Label,

    #[nwg_control(v_align: nwg::VTextAlign::Bottom)]
    #[nwg_layout_item(layout: grid, col: 4, row: 1, col_span: 2)]
    file_combo: nwg::ComboBox<String>,

    #[nwg_control(text: "Variasi")]
    #[nwg_layout_item(layout: grid, col: 3, row: 2)]
    variasi_label: nwg::Label,

    #[nwg_control(v_align: nwg::VTextAlign::Bottom)]
    #[nwg_layout_item(layout: grid, col: 4, row: 2, col_span: 2)]
    variasi_combo: nwg::ComboBox<String>,
    
    #[nwg_control(text: "Kurir")]
    #[nwg_layout_item(layout: grid, col: 3, row: 3)]
    kurir_label: nwg::Label,

    #[nwg_control(v_align: nwg::VTextAlign::Bottom)]
    #[nwg_layout_item(layout: grid, col: 4, row: 3, col_span: 2)]
    kurir_combo: nwg::ComboBox<String>,
    
    #[nwg_control(text: "PromotionId", font: Some(&data.font_awesome))]
    #[nwg_layout_item(layout: grid, col: 0, row: 4)]
    promotionid_label: nwg::Label,
    
    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: grid, col: 1, row: 4, col_span: 2)]
    promotionid_text: nwg::TextInput,
    
    #[nwg_control(text: "Code", h_align: nwg::HTextAlign::Center)]
    #[nwg_layout_item(layout: grid, col: 0, row: 4)]
    code_label: nwg::Label,
    
    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: grid, col: 1, row: 4, col_span: 2)]
    code_text: nwg::TextInput,
    
    #[nwg_control(text: "Shop Code")]
    #[nwg_layout_item(layout: grid, col: 3, row: 4, col_span: 2)]
    shop_checkbox: nwg::CheckBox,
    
    #[nwg_control(text: "Signature")]
    #[nwg_layout_item(layout: grid, col: 3, row: 4)]
    signature_label: nwg::Label,
    
    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: grid, col: 4, row: 4, col_span: 2)]
    signature_text: nwg::TextInput,
    
    #[nwg_control(text: "Time")]
    #[nwg_layout_item(layout: grid, col: 0, row: 5, row_span: 2)]
    time_label: nwg::Label,
    
    #[nwg_control(text: "Jam", h_align: nwg::HTextAlign::Center)]
    #[nwg_layout_item(layout: grid, col: 1, row: 5)]
    jam_label: nwg::Label,

    #[nwg_control(text: "", flags:"VISIBLE|NUMBER|AUTO_SCROLL", limit: 2)]
    #[nwg_layout_item(layout: grid, col: 1, row: 6)]
    jam_text: nwg::TextInput,
    
    #[nwg_control(text: "Menit", h_align: nwg::HTextAlign::Center)]
    #[nwg_layout_item(layout: grid, col: 2, row: 5)]
    menit_label: nwg::Label,
    
    #[nwg_control(text: "", flags:"VISIBLE|NUMBER|AUTO_SCROLL", limit: 2)]
    #[nwg_layout_item(layout: grid, col: 2, row: 6)]
    menit_text: nwg::TextInput,
    
    #[nwg_control(text: "Detik", h_align: nwg::HTextAlign::Center)]
    #[nwg_layout_item(layout: grid, col: 3, row: 5)]
    detik_label: nwg::Label,
    
    #[nwg_control(text: "59", flags:"VISIBLE|NUMBER|AUTO_SCROLL", limit: 2)]
    #[nwg_layout_item(layout: grid, col: 3, row: 6)]
    detik_text: nwg::TextInput,
    
    #[nwg_control(text: "Mili", h_align: nwg::HTextAlign::Center)]
    #[nwg_layout_item(layout: grid, col: 4, row: 5)]
    mili_label: nwg::Label,
    
    #[nwg_control(text: "900", flags:"VISIBLE|NUMBER|AUTO_SCROLL", limit: 3)]
    #[nwg_layout_item(layout: grid, col: 4, row: 6)]
    mili_text: nwg::TextInput,

    #[nwg_control(text: "Jalankan")]
    #[nwg_layout_item(layout: grid, col: 4, row: 7, col_span: 2)]
    #[nwg_events( OnButtonClick: [App::run], OnMouseMove: [App::on_hover], MousePressRightUp: [App::show_menu])]
    run_button: nwg::Button,
    
    #[nwg_control(popup: true)]
    launch: nwg::Menu,

    #[nwg_control(parent: launch, text: "Manual Run")]
    #[nwg_events(OnMenuItemSelected: [App::quick])]
    quick: nwg::MenuItem,
    
    #[nwg_control(parent: launch, text: "Generate Sruct")]
    #[nwg_events(OnMenuItemSelected: [App::quick])]
    gen_launch: nwg::MenuItem,
    
    #[nwg_control(text: "Refresh", flags: "VISIBLE|DISABLED",)]
    #[nwg_layout_item(layout: grid, col: 0, row: 7)]
    #[nwg_events( OnButtonClick: [App::refresh_file_combo], OnMouseMove: [App::on_hover])]
    refresh_button: nwg::Button,
    
    #[nwg_control(text: "Voucher")]
    #[nwg_layout_item(layout: grid, col: 1, row: 7)]
    #[nwg_events( OnButtonClick: [App::on_voucher_checkbox_change])]
    voucher_checkbox: nwg::CheckBox,
    
    #[nwg_control(text: "Code")]
    #[nwg_layout_item(layout: grid, col: 2, row: 7)]
    #[nwg_events( OnButtonClick: [App::on_code_checkbox_change])]
    code_checkbox: nwg::CheckBox,
    
    #[nwg_control(v_align: nwg::VTextAlign::Bottom, font: Some(&data.font_combo))]
    #[nwg_layout_item(layout: grid, col: 3, row: 7)]
    media_combo: nwg::ComboBox<String>,

    #[nwg_resource(family: "Segoe UI", size: 18)]
    font_combo: nwg::Font,	

    #[nwg_resource(family: "Segoe UI", size: 13)]
    font_awesome: nwg::Font,	
    
    #[nwg_resource]
    tooltip: nwg::Tooltip,
    
    #[nwg_control(text: "&File")]
    file: nwg::Menu,
    #[nwg_control(parent: file, text: "&New\tCtrl+N")]
    #[nwg_events( OnMenuItemSelected: [App::new])]
    new_home: nwg::MenuItem,
    #[nwg_control(parent: file, text: "&Refresh")]
    #[nwg_events( OnMenuItemSelected: [App::refresh_file_combo])]
    refresh_home: nwg::MenuItem,
    #[nwg_control(parent: file)]
    menu_separator: nwg::MenuSeparator,
    #[nwg_control(parent: file, text: "&Exit\tAlt+F4")]
    #[nwg_events( OnMenuItemSelected: [App::exit])]
    exit_home: nwg::MenuItem,
    
    #[nwg_control(text: "&Edit")]
    edit: nwg::Menu,
    #[nwg_control(parent: edit, text: "Set default host(dev)")]
    host_edit: nwg::MenuItem,
    #[nwg_control(parent: edit, text: "Accounts Manager(dev)")]
    #[nwg_events( OnMenuItemSelected: [App::manager])]
    manager_edit: nwg::MenuItem,
    
    #[nwg_control(text: "&View")]
    view: nwg::Menu,
    #[nwg_control(parent: view, text: "Console Logs")]
    #[nwg_events( OnMenuItemSelected: [App::log_change])]
    logs_view: nwg::MenuItem,
    #[nwg_control(parent: view, text: "Accounts Viewer")]
    #[nwg_events( OnMenuItemSelected: [App::manager])]
    manager_view: nwg::MenuItem,
    
    #[nwg_control(text: "E&xtension")]
    extension: nwg::Menu,
    #[nwg_control(parent: extension, text: "Voucher")]
    #[nwg_events( OnMenuItemSelected: [App::voucher_change])]
    voucher_extension: nwg::MenuItem,
    #[nwg_control(parent: extension, text: "Media")]
    #[nwg_events( OnMenuItemSelected: [App::media_change])]
    media_extension: nwg::MenuItem,
    
    #[nwg_control(text: "&Help")]
    help: nwg::Menu,
    #[nwg_control(parent: help, text: "Help Topics\tF1")]
    #[nwg_events( OnMenuItemSelected: [App::show_version_info])]
    topics_help: nwg::MenuItem,
    #[nwg_control(parent: help)]
    help_separator: nwg::MenuSeparator,
    #[nwg_control(parent: help, text: "About ABS")]
    #[nwg_events( OnMenuItemSelected: [App::show_version_info])]
    menu_help: nwg::MenuItem,
    
    #[nwg_control(
        size: (100, 100),
        position: (300, 300),
        title: "About",
        flags: "WINDOW",
        center: true,
        topmost: true,
    )]
    #[nwg_events( OnWindowClose: [App::close_version_info])]
    version_info_window: nwg::Window,

    #[nwg_layout(parent: version_info_window, spacing: 1)]
    grid2: nwg::GridLayout,

    #[nwg_control(parent: version_info_window, font: Some(&data.font_awesome))]
    #[nwg_layout_item(layout: grid2, col: 0, row: 0, col_span: 3)]
    version_label: nwg::Label,
    
    #[nwg_control(parent: version_info_window, font: Some(&data.font_awesome))]
    #[nwg_layout_item(layout: grid2, col: 0, row: 1, col_span: 3)]
    version_label2: nwg::Label,

    #[nwg_control(parent: version_info_window, text: "OK", focus: true, font: Some(&data.font_awesome), size: (73,23))]
    #[nwg_layout_item(layout: grid2, col: 1, row: 2, col_span: 2)]
    #[nwg_events( OnButtonClick: [App::close_version_info])]
    ok_button: nwg::Button,
    
    #[nwg_control]
    #[nwg_events( OnNotice: [App::update_ui] )]
    notice: nwg::Notice,
    
    shared_data: Arc<RwLock<SharedData>>,
    
    #[nwg_control(size: (300, 500), position: (950, 150), title: "Log Window")]
    #[nwg_events(OnWindowClose: [App::close_logs_info])]
    logs_window: nwg::Window,

    #[nwg_control(size: (290, 480), position: (10, 10), readonly: true, font: Some(&data.font_logs), flags: "VISIBLE|AUTOVSCROLL|VSCROLL",)]
    log_box: nwg::TextBox,
    
    #[nwg_resource(family: "Lucida Console", size: 11)]
    font_logs: nwg::Font,
}

impl App {
    fn key(&self, key: &nwg::EventData){
        let shared_data = &self.shared_data.clone();
        let mut data = shared_data.write().unwrap();
        if let nwg::EventData::OnKey(key_event) = key {
            match *key_event {
                nwg::keys::CONTROL => {
                    data.ctrl_pressed = true;
                    println!("CTRL pressed");
                }
                nwg::keys::_N => {
                    if data.ctrl_pressed {
                        println!("CTRL+N was pressed");
                        App::new(&self);
                        data.ctrl_pressed = false;
                    } else {
                        println!("N was pressed");
                    }
                }
                _ => {
                    println!("Key pressed: {}", *key_event);
                }
            }
        }
    }
    fn log_change(&self) {
        if self.logs_view.checked() == true{
            self.logs_view.set_checked(false);
            self.logs_window.set_visible(false);
        }else {
            self.logs_view.set_checked(true);
            self.logs_window.set_visible(true);
            let log_message = "Button clicked!";
            // Panggil fungsi append_log di jendela log
            self.append_log(log_message);
        }
    }
    fn append_log(&self, log: &str) {
        let mut text = self.log_box.text();
        text.push_str(log);
        text.push_str("\r\n");
        self.log_box.set_text(&text);
    }
    fn init(&self){
        self.logs_window.set_visible(true);
        let version_info = env!("CARGO_PKG_VERSION");
        let log_message1 = format!("Launcher Auto Buy Shopee Version : {} (PREVIEW)", version_info);
        self.append_log(&log_message1);
        let version_message = match (is_server(), OsVersion::current()) {
            (true, version) if version >= OsVersion::new(10, 0, 0, 20348) => "Windows Server 2022".to_string(),
            (true, version) if version >= OsVersion::new(10, 0, 0, 17763) => "Windows Server 2019".to_string(),
            (true, version) if version >= OsVersion::new(10, 0, 0, 14393) => "Windows Server 2016".to_string(),
            (true, version) if version >= OsVersion::new(6, 3, 0, 9800) => "Windows Server 2012 R2".to_string(),
            (true, version) if version >= OsVersion::new(6, 2, 0, 9200) => "Windows Server 2012".to_string(),
            (true, version) if version >= OsVersion::new(6, 1, 0, 7600) => "Windows Server 2008 R2".to_string(),
            (true, _) => format!("Windows Server Build {:?}", OsVersion::current()),
            (false, version) if version >= OsVersion::new(10, 0, 0, 22000) => format!("Windows 11 Build {:?}", OsVersion::current().build),
            (false, version) if version >= OsVersion::new(10, 0, 0, 10240) => format!("Windows 10 Build {:?}", OsVersion::current()),
            (false, version) if version >= OsVersion::new(6, 3, 0, 9800) => format!("Windows 8.1 Build {:?}", OsVersion::current()),
            (false, version) if version >= OsVersion::new(6, 2, 0, 9200) => format!("Windows 8 Build {:?}", OsVersion::current()),
            (false, version) if version >= OsVersion::new(6, 1, 1, 7601) => "Windows 7 SP1".to_string(),
            (false, version) if version >= OsVersion::new(6, 1, 0, 7600) => "Windows 7 RTM".to_string(),
            _ => format!("Running on an unsupported version {:?}", OsVersion::current()),
        };
        let log_message2 = format!("Operating System : {:?}", version_message);		
        self.append_log(&log_message2);
    }
    fn close_logs_info(&self){
        self.logs_window.set_visible(false);
    }
    fn new(&self) {
        new::main("ayo");
        manager::main();
        self.refresh_file_combo();
    }
    fn manager(&self) {
        manager::main();
        self.refresh_file_combo();
    }
    fn voucher_change(&self) {
        if self.voucher_extension.checked() == true{
            self.voucher_extension.set_checked(false);
            self.voucher_checkbox.set_visible(false);
            self.code_checkbox.set_visible(false);
        }else {
            self.voucher_extension.set_checked(true);
            self.voucher_checkbox.set_visible(true);
            self.code_checkbox.set_visible(true);
        }
    }
    fn media_change(&self) {
        if self.media_extension.checked() == true{
            self.media_extension.set_checked(false);
            self.media_combo.set_visible(false);
        }else {
            self.media_extension.set_checked(true);
            self.media_combo.set_visible(true);
        }
    }
    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.launch.popup(x, y)
    }
    fn show_version_info(&self) {
        // Show the version info window when Help ABS is selected
        self.version_info_window.set_visible(true);
    }
    fn close_version_info(&self) {
        // Close the version info window when OK button is clicked
        self.version_info_window.set_visible(false);
    }
    fn on_voucher_checkbox_change(&self) {
        if self.voucher_checkbox.check_state() == nwg::CheckBoxState::Checked{
            self.code_checkbox.set_check_state(nwg::CheckBoxState::Unchecked);
            self.on_code_checkbox_change();
            self.promotionid_label.set_visible(true);
            self.promotionid_text.set_visible(true);
            self.signature_label.set_visible(true);
            self.signature_text.set_visible(true);
        }else {
            self.promotionid_label.set_visible(false);
            self.promotionid_text.set_visible(false);
            self.signature_label.set_visible(false);
            self.signature_text.set_visible(false);
        }
    }	
    fn on_code_checkbox_change(&self) {
        if self.code_checkbox.check_state() == nwg::CheckBoxState::Checked{
            self.voucher_checkbox.set_check_state(nwg::CheckBoxState::Unchecked);
            self.on_voucher_checkbox_change();
            self.code_label.set_visible(true);
            self.code_text.set_visible(true);
            self.shop_checkbox.set_visible(true);
        }else {
            self.code_label.set_visible(false);
            self.code_text.set_visible(false);
            self.shop_checkbox.set_visible(false);
        }
    }
    fn on_hover(&self) {
        self.tooltip.set_enabled(true);
        self.tooltip.register(&self.run_button.handle, "Tekan tombol untuk menjalankan aplikasi!");
        self.tooltip.register(&self.cek_button.handle, "Cek Variasi dan Kurir");
        self.tooltip.register(&self.refresh_button.handle, "Regenerate account files\n(DEBUG)");
    }
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
    fn cek(&self) {
        // Disable the button to prevent multiple async tasks from being started
        self.cek_button.set_enabled(false);
        self.cek_button.set_text("Wait");
        clear_combo_box(&self.variasi_combo);
        clear_combo_box(&self.kurir_combo);
        let start = self.url_text.text();
        let file = self.file_combo.selection_string().unwrap_or_default();
        let cookie_content = prepare::read_cookie_file(&file);
        let url_1 = start.trim();
        println!("{}", url_1);
        // Memproses URL
        let mut shop_id = String::new();
        let mut item_id = String::new();
        if !url_1.is_empty() {
            if !url_1.contains("/product/") {
                let split: Vec<&str> = url_1.split('.').collect();
                shop_id = split[split.len() - 2].to_string();
                item_id = split[split.len() - 1].split('?').next().unwrap_or("").to_string();
            } else {
                let split2: Vec<&str> = url_1.split('/').collect();
                shop_id = split2[split2.len() - 2].to_string();
                item_id = split2[split2.len() - 1].split('?').next().unwrap_or("").to_string();
            }
        }
        // Clone the notice sender and runtime to move into the new thread
        let notice_sender = self.notice.sender();
        let shared_data = self.shared_data.clone();

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            rt.block_on(async {
                App::cek_async(&shop_id, &item_id, &cookie_content, shared_data).await;

                // Send a notice to update the UI
                notice_sender.notice();
            });
        });
    }
    async fn cek_async(shop_id: &str, item_id: &str, cookie_content: &str, shared_data: Arc<RwLock<SharedData>>) {
        // Memanggil get_product dengan timeout
        match timeout(Duration::from_secs(10), prepare::get_product(shop_id, item_id, cookie_content)).await {
            Ok(Ok((name, model_info, is_official_shop, rcode))) => {
                let mut data = shared_data.write().unwrap();
                data.name_model = model_info.iter().map(|model| model.name.clone()).collect();
                data.rcode = rcode;
            },
            Ok(Err(e)) => {
                let mut data = shared_data.write().unwrap();
                data.rcode = format!("Error: {:?}", e);
            },
            Err(e) => {
                let mut data = shared_data.write().unwrap();
                data.rcode = format!("Timeout: {:?}", e);
            }
        };

        // Memanggil get_kurir dengan timeout
        let (state, city, district, addressid) = match prepare::address(&cookie_content).await {
            Ok(address) => address,
            Err(e) => {
                // Handle the error case
                eprintln!("Failed to get address: {}", e);
                return; // Early return or handle the error as needed
            }
        };
        match timeout(Duration::from_secs(10), prepare::kurir(&cookie_content, &shop_id, &item_id, &state, &city, &district)).await {
            Ok(Ok(kurirs)) => {
                let mut data = shared_data.write().unwrap();
                data.kurirs = kurirs.iter().map(|kurirs| kurirs.channel_name.clone()).collect();
            },
            Ok(Err(e)) => {
                let mut data = shared_data.write().unwrap();
                data.kurirs = vec![format!("Error: {:?}", e)];
            },
            Err(e) => {
                let mut data = shared_data.write().unwrap();
                data.kurirs = vec![format!("Timeout: {:?}", e)];
            }
        };
    }
    fn update_ui(&self) {
        // Update the ComboBox with the new data
        let data = self.shared_data.read().unwrap();
            
        if data.rcode == "200 OK" {
            self.variasi_combo.set_collection(data.name_model.clone());
            self.variasi_combo.set_selection(Some(0));
            for log in &data.name_model {
                self.append_log(log);
            }
        } else if data.rcode.contains("CronetError"){
            let isi = format!("OH SNAP!!!\nSolution:\nCHECK INTERNET CONNECTION\n\nError : {}", data.rcode);
            let p = nwg::MessageParams {
                title: "Error get Variation",
                content: &isi,
                buttons: nwg::MessageButtons::Ok,
                icons: nwg::MessageIcons::Error
            };
            nwg::modal_message(&self.variasi_combo, &p);
            println!("Error: {}", data.rcode);
        } else {
            let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {}", data.rcode);
            let p = nwg::MessageParams {
                title: "Error get Variation",
                content: &isi,
                buttons: nwg::MessageButtons::Ok,
                icons: nwg::MessageIcons::Error
            };
            nwg::modal_message(&self.variasi_combo, &p);
            println!("Error: {}", data.rcode);
        }
        
        self.kurir_combo.set_collection(data.kurirs.clone());
        if !data.kurirs.is_empty() {
            let kurie = data.kurirs.join(", ");
            if kurie.contains("CronetError"){
                let isi = format!("OH SNAP!!!\nSolution:\nCHECK INTERNET CONNECTION\n\nError : {}", kurie);
                let p = nwg::MessageParams {
                    title: "Error get Shipping",
                    content: &isi,
                    buttons: nwg::MessageButtons::Ok,
                    icons: nwg::MessageIcons::Error
                };
                nwg::modal_message(&self.kurir_combo, &p);
                println!("Error: {}", kurie);
                clear_combo_box(&self.kurir_combo);
            }else{
                self.kurir_combo.set_selection(Some(0));
            }
        }

        // Update the Button text and enable it
        self.cek_button.set_enabled(true);
        self.cek_button.set_text("Cek");
    }
    fn quick(&self){
        let command = vec!["start","abs.exe",];
        let _status = std::process::Command::new("cmd")
            .arg("/c")
            .args(&command)
            .spawn()
            .expect("Gagal menjalankan program");
        println!("{:?}", command);
    }
    fn run(&self) {
        let url = self.url_text.text();
        let payment = self.payment_combo.selection_string().unwrap_or_default();
        let harga = self.harga_text.text();
        let file = self.file_combo.selection_string().unwrap_or_default();
        let variasi = self.variasi_combo.selection_string().unwrap_or_default();
        let kurir = self.kurir_combo.selection_string().unwrap_or_default();
        let jam = self.jam_text.text();
        let menit = self.menit_text.text();
        let detik = self.detik_text.text();
        let mili = self.mili_text.text();
        let kuan = self.kuan_text.text();
        let time_arg = format!("{}:{}:{}.{}", &jam, &menit, &detik, &mili);
        // Menjalankan program abs.exe dengan argumen yang dibuat
        let command = match (
            self.code_checkbox.check_state(),
            self.voucher_checkbox.check_state(),
        ) {
            (nwg::CheckBoxState::Checked, _) => {
                if self.shop_checkbox.check_state() == nwg::CheckBoxState::Checked {
                    let p1 = nwg::MessageParams {
                        title: "Newer feature detected 1",
                        content: "Tes new features code shop",
                        buttons: nwg::MessageButtons::Ok,
                        icons: nwg::MessageIcons::Info,
                    };
                    assert!(nwg::modal_message(&self.window, &p1) == nwg::MessageChoice::Ok);
                    None
                    /*Some(vec![
                        "start",
                        "abs.exe",
                    ])*/
                } else {
                    let p1 = nwg::MessageParams {
                        title: "Newer feature detected 2",
                        content: "Tes new features code",
                        buttons: nwg::MessageButtons::Ok,
                        icons: nwg::MessageIcons::Info,
                    };
                    assert!(nwg::modal_message(&self.window, &p1) == nwg::MessageChoice::Ok);
                    None
                    /*Some(vec![
                        "start",
                        "abs.exe",
                    ])*/
                }
            }
            (_, nwg::CheckBoxState::Checked) => {
                let p1 = nwg::MessageParams {
                    title: "Newer feature detected 3",
                    content: "Tes new features voucher",
                    buttons: nwg::MessageButtons::Ok,
                    icons: nwg::MessageIcons::Info,
                };
                assert!(nwg::modal_message(&self.window, &p1) == nwg::MessageChoice::Ok);
                None
                /*Some(vec![
                    "start",
                    "abs.exe",
                ])*/
            }
            _ => {
                // Default case
                Some(vec![
                    "start",
                    "abs.exe",
                    "--file", &file,
                    "--url", &url,
                    "--time", &time_arg,
                    "--product", &variasi,
                    "--kurir", &kurir,
                    "--payment", &payment,
                    "--harga", &harga,
                    "--quantity", &kuan,
                ])
            }
        };
        
        if let Some(command) = command {
            if !file.is_empty() {
                let _status = std::process::Command::new("cmd")
                    .arg("/c")
                    .args(&command)
                    .spawn()
                    .expect("Gagal menjalankan program");
                println!("{:?}", command);
            }else{
                let p = nwg::MessageParams {
                title: "Error",
                content: "Please select a file before running the program",
                buttons: nwg::MessageButtons::Ok,
                icons: nwg::MessageIcons::Error
                };
                assert!(nwg::modal_message(&self.file_combo, &p) == nwg::MessageChoice::Ok);
                return;
            }
        }
        
    }
    fn populate_file_combo(&self) {
        let folder_path = "akun";
        let files = manager::get_file_names(folder_path);

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
                icons: nwg::MessageIcons::Warning,
            };
            assert!(nwg::modal_message(&self.file_combo, &p) == nwg::MessageChoice::Ok);
            println!("Failed to read the folder contents or no files found");
        }
    }
    fn populate_payment_combo(&self) {
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
                                    self.payment_combo.push(name.to_string());
                                    if !name.is_empty() {
                                        self.payment_combo.set_selection(Some(0));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let p = nwg::MessageParams {
                title: "File not found",
                content: "File tidak ada.\n \nHarap Download ulang",
                buttons: nwg::MessageButtons::Ok,
                icons: nwg::MessageIcons::Warning
            };
            assert!(nwg::modal_message(&self.payment_combo, &p) == nwg::MessageChoice::Ok);
            println!("Failed to read the folder contents");
        }
    }
    fn populate_default(&self) {
        let local: DateTime<Local> = Local::now();
        let hour = local.hour().to_string();
        let minute = match local.minute() {
            m if m <= 14 => "14",
            m if m <= 29 => "29",
            m if m <= 44 => "44",
            _ => "59",
        };
        println!("Current local hour: {}", hour);
        println!("Current local minute set: {}", minute);
        let media_model = vec!["No Media","Live","Video",];
        let version_info = env!("CARGO_PKG_VERSION");
        let ver_label_info1 = "Launcher Auto Buy Shopee";
        let ver_label_info2 = format!("Version : {} (PREVIEW)", version_info);
        self.version_label.set_text(ver_label_info1);
        self.version_label2.set_text(&ver_label_info2);
        self.harga_text.set_text("1000");
        self.kuan_text.set_text("1");
        self.jam_text.set_text(&hour);
        self.menit_text.set_text(&minute);
        self.refresh_button.set_visible(false);
        self.promotionid_label.set_visible(false);
        self.promotionid_text.set_visible(false);
        self.signature_label.set_visible(false);
        self.signature_text.set_visible(false);
        self.code_label.set_visible(false);
        self.code_text.set_visible(false);
        self.shop_checkbox.set_visible(false);
        self.voucher_extension.set_checked(true);
        self.media_extension.set_checked(true);
        self.logs_view.set_checked(true);
        self.host_edit.set_enabled(false);
        for name in &media_model {
            self.media_combo.push(name.to_string());
            self.media_combo.set_selection(Some(0));
        }
    }
    fn refresh_file_combo(&self) {
        clear_combo_box(&self.file_combo);
        clear_combo_box(&self.payment_combo);
        self.populate_file_combo();
        self.populate_payment_combo()
    }
}

async fn initialize_gui() {
    nwg::init().expect("Failed to initialize native windows gui");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
}
async fn main_loop() {
    let app = App::build_ui(Default::default()).expect("Failed to build UI");
    app.populate_default();
    app.populate_file_combo();
    app.populate_payment_combo();
    app.on_hover();
    nwg::dispatch_thread_events();
}
fn show_already_running_error() {
    let p = nwg::MessageParams {
        title: "Error",
        content: "Another instance is already running. Exiting...",
        buttons: nwg::MessageButtons::Ok,
        icons: nwg::MessageIcons::Error,
    };
    assert!(nwg::message(&p) == nwg::MessageChoice::Ok);
}
fn main() {
    nwg::enable_visual_styles();
    let instance_id = "your_unique_instance_id";
    let single_instance = SingleInstance::new(instance_id).expect("Failed to create single instance");

    if !single_instance.is_single() {
        eprintln!("Another instance is already running.");
        show_already_running_error();
        std::process::exit(1);
    }

    // Create a runtime for async operations
    let rt = runtime::Runtime::new().unwrap();

    // Run the async main function
    rt.block_on(async {
        initialize_gui().await;
        main_loop().await;
    });
}
fn clear_combo_box(combo_box: &nwg::ComboBox<String>) {
    while combo_box.len() > 0 {
        combo_box.remove(0);
    }
}