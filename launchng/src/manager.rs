use winsafe::{self as w,
    AnyResult, co::{self, SW}, gui, prelude::*
};
use runtime::telegram;
use runtime::prepare::{self, ProductInfo, FSInfo, AddressInfo, ShippingInfo, ModelInfo, PaymentInfo,};
use runtime::crypt;
use runtime::product;
use runtime::upgrade;
use tokio::sync::mpsc;
use tokio::{time::{timeout, Duration}};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::io::{self, BufWriter, AsyncWriteExt};
use futures_util::StreamExt;
use chrono::{Local, DateTime, Timelike};

use crate::func_main;
use crate::login;
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS: &str = std::env::consts::OS;
const ARCH: &str = std::env::consts::ARCH;

#[derive(Clone)]
pub struct Multi {
    pub wnd2: gui::WindowModal,
    pub cek_button: gui::Button,
    pub file_combo: gui::ComboBox,
    pub url_text: gui::Edit,
    pub quantity_text: gui::Edit,
    pub push_button: gui::Button,
    pub my_list: gui::ListView,
    pub launch_button: gui::Button,
    pub fsv_checkbox: gui::CheckBox,
    pub platform_checkbox: gui::CheckBox,
    pub platform_combo: gui::ComboBox,
    pub proid_label: gui::Label,
    pub proid_text: gui::Edit,
    pub signature_label: gui::Label,
    pub signature_text: gui::Edit,
    pub code_label: gui::Label,
    pub code_text: gui::Edit,
    pub collection_label: gui::Label,
    pub collection_text: gui::Edit,
    pub link_label: gui::Label,
    pub link_text: gui::Edit,
    pub variasi_combo: gui::ComboBox,
    pub kurir_combo: gui::ComboBox,
    pub payment_combo: gui::ComboBox,
    pub shopcode_text: gui::Edit,
    pub jam_text: gui::Edit,
    pub menit_text: gui::Edit,
    pub detik_text: gui::Edit,
    pub mili_text: gui::Edit,
    pub harga_checkbox: gui::CheckBox,
    pub harga_text: gui::Edit,
    pub shared_payment_data: Arc<Mutex<Vec<PaymentInfo>>>,
    pub shared_variation_data: Arc<Mutex<Vec<ModelInfo>>>,
    pub shared_kurir_data: Arc<Mutex<Vec<ShippingInfo>>>,
}
impl Multi {
    pub fn new() -> Self {
        let shared_payment_data = Arc::new(Mutex::new(vec![]));
        let shared_variation_data = Arc::new(Mutex::new(vec![]));
        let shared_kurir_data = Arc::new(Mutex::new(vec![]));
        let dont_move = (gui::Horz::None, gui::Vert::None);
        let resized = (gui::Horz::Resize, gui::Vert::Resize);
        let _move_h = (gui::Horz::Repos, gui::Vert::None);
        let moved = (gui::Horz::Repos, gui::Vert::Repos);
        let wnd2 = gui::WindowModal::new_dlg(900);
        let cek_button = gui::Button::new_dlg(&wnd2, 901, dont_move);
        let file_combo = gui::ComboBox::new_dlg(&wnd2, 902, dont_move);
        let url_text = gui::Edit::new_dlg(&wnd2, 903, dont_move);
        let quantity_text = gui::Edit::new_dlg(&wnd2, 904, dont_move);
        let push_button = gui::Button::new_dlg(&wnd2, 905, dont_move);
        let my_list = gui::ListView::new_dlg(&wnd2, 906, resized, Some(200));
        let launch_button = gui::Button::new_dlg(&wnd2, 907, moved);
        let fsv_checkbox = gui::CheckBox::new_dlg(&wnd2, 908, dont_move);
        let platform_checkbox = gui::CheckBox::new_dlg(&wnd2, 909, dont_move);
        let platform_combo = gui::ComboBox::new_dlg(&wnd2, 910, dont_move);
        let proid_label = gui::Label::new_dlg(&wnd2, 911, dont_move);
        let proid_text = gui::Edit::new_dlg(&wnd2, 912, dont_move);
        let signature_label = gui::Label::new_dlg(&wnd2, 913, dont_move);
        let signature_text = gui::Edit::new_dlg(&wnd2, 914, dont_move);
        let code_label = gui::Label::new_dlg(&wnd2, 915, dont_move);
        let code_text = gui::Edit::new_dlg(&wnd2, 916, dont_move);
        let collection_label = gui::Label::new_dlg(&wnd2, 917, dont_move);
        let collection_text = gui::Edit::new_dlg(&wnd2, 918, dont_move);
        let link_label = gui::Label::new_dlg(&wnd2, 919, dont_move);
        let link_text = gui::Edit::new_dlg(&wnd2, 920, dont_move);
        let variasi_combo = gui::ComboBox::new_dlg(&wnd2, 921, dont_move);
        let kurir_combo = gui::ComboBox::new_dlg(&wnd2, 922, dont_move);
        let shopcode_text = gui::Edit::new_dlg(&wnd2, 923, dont_move);
        let payment_combo = gui::ComboBox::new_dlg(&wnd2, 924, dont_move);
        let jam_text = gui::Edit::new_dlg(&wnd2, 925, dont_move);
        let menit_text = gui::Edit::new_dlg(&wnd2, 926, dont_move);
        let detik_text = gui::Edit::new_dlg(&wnd2, 927, dont_move);
        let mili_text = gui::Edit::new_dlg(&wnd2, 928, dont_move);
        let harga_checkbox = gui::CheckBox::new_dlg(&wnd2, 929, dont_move);
        let harga_text = gui::Edit::new_dlg(&wnd2, 930, dont_move);
        let new_self = Self{wnd2, cek_button, file_combo, url_text, quantity_text, push_button, my_list, launch_button, fsv_checkbox, platform_checkbox, platform_combo, proid_label, proid_text, signature_label, signature_text, code_label, code_text, collection_label, collection_text, link_label, link_text, variasi_combo, kurir_combo, shopcode_text, payment_combo, jam_text, menit_text, detik_text, mili_text, harga_checkbox, harga_text, shared_payment_data, shared_variation_data, shared_kurir_data};
        new_self.events();
        new_self
    }
    pub fn run(&self, wnd: &gui::WindowMain) -> AnyResult<()> {
        let _ = self.wnd2.show_modal(wnd);
        Ok(())
    }
    pub fn events(&self) {
        let self_clone = self.clone();
        self.wnd2.on().wm_init_dialog(move |_| {
            let local: DateTime<Local> = Local::now();
            let hour = local.hour().to_string();
            let minute = match local.minute() {
                m if m <= 14 => "14", m if m <= 29 => "29", m if m <= 44 => "44", _ => "59",
            };
            let _ = self_clone.jam_text.set_text(&hour);
            let _ = self_clone.menit_text.set_text(&minute);
            let _ = self_clone.platform_combo.items().add(&[
                "Claim", "Code", "Collection id", "Link",
            ]);
            self_clone.platform_combo.items().select(Some(0));
            let _ = self_clone.my_list.cols().add("Url", 240);
            let _ = self_clone.my_list.cols().add("Quantity", 120);
            let _ = self_clone.my_list.cols().add("Variation", 120);
            let _ = self_clone.my_list.cols().add("Courier", 120);
            let _ = self_clone.my_list.cols().add("Voucher Shop", 120);
            self_clone.my_list.set_extended_style(true, w::co::LVS_EX::FULLROWSELECT);
            func_main::populate_combobox_with_files(&self_clone.file_combo, "akun");
            func_main::populate_payment_combo(&self_clone.payment_combo, self_clone.shared_payment_data.clone());
            self_clone.platform_combo.hwnd().EnableWindow(false);
            self_clone.harga_text.hwnd().EnableWindow(false);
            func_main::set_visibility(&self_clone.proid_label, &self_clone.proid_text, false);
            func_main::set_visibility(&self_clone.signature_label, &self_clone.signature_text, false);
            func_main::set_visibility(&self_clone.code_label, &self_clone.code_text, false);
            func_main::set_visibility(&self_clone.collection_label, &self_clone.collection_text, false);
            func_main::set_visibility(&self_clone.link_label, &self_clone.link_text, false);
            Ok(true)
        });
        let self_clone = self.clone();
        self.push_button.on().bn_clicked(move || {
            let url = self_clone.url_text.text().unwrap_or_else(|_| String::new());
            let quantity = self_clone.quantity_text.text().unwrap_or_else(|_| String::new());
            if url.is_empty() {
                let isi = format!("Please input the url before pushing");
                let _ = func_main::error_modal(&self_clone.wnd2, "Error push data", &isi);
            } else if quantity.is_empty() {
                let isi = format!("Please input the quantity before pushing");
                let _ = func_main::error_modal(&self_clone.wnd2, "Error push data", &isi);
            } else {
                let variasi = match self_clone.variasi_combo.items().selected_text() {
                    Ok(Some(text)) => text,
                    Ok(None) => "".to_string(),
                    Err(_) => "".to_string()
                };
                let kurir = match self_clone.kurir_combo.items().selected_text() {
                    Ok(Some(text)) => text,
                    Ok(None) => "".to_string(),
                    Err(_) => "".to_string()
                };
                let shop_code = if self_clone.shopcode_text.text().unwrap_or_else(|_| String::new()).is_empty() {
                    "".to_string()
                } else {
                    self_clone.shopcode_text.text().unwrap_or_else(|_| String::new())
                };
                let _ = self_clone.my_list.items().add(
                    &[
                        &url.to_string(), &quantity.to_string(), &variasi.to_string(), &kurir.to_string(), &shop_code.to_string(),
                    ], None, () // no icon; requires a previous set_image_list()
                );
                let _ = self_clone.url_text.set_text("");
                let _ = self_clone.quantity_text.set_text("1");
            }
            Ok(())
        });
        let self_clone = self.clone();
        self.harga_checkbox.on().bn_clicked(move || {
            println!("harga checkbox clicked!");
            if self_clone.harga_checkbox.is_checked() == true{
                self_clone.harga_text.hwnd().EnableWindow(true);
            }else{
                self_clone.harga_text.hwnd().EnableWindow(false);
            }
            Ok(())
        });
        let self_clone = self.clone();
        self.platform_checkbox.on().bn_clicked(move || {
            println!("platform checkbox clicked!");
            if self_clone.platform_checkbox.is_checked() == true{
                self_clone.fsv_checkbox.set_state(co::BST::UNCHECKED);
                self_clone.platform_combo.hwnd().EnableWindow(true);
                self_clone.platform_combo.items().select(Some(0));
                func_main::set_visibility(&self_clone.proid_label, &self_clone.proid_text, true);
                func_main::set_visibility(&self_clone.signature_label, &self_clone.signature_text, true);
                func_main::set_visibility(&self_clone.code_label, &self_clone.code_text, false);
                func_main::set_visibility(&self_clone.collection_label, &self_clone.collection_text, false);
                func_main::set_visibility(&self_clone.link_label, &self_clone.link_text, false);
            }else{
                self_clone.platform_combo.hwnd().EnableWindow(false);
                self_clone.platform_combo.items().select(Some(0));
                func_main::set_visibility(&self_clone.proid_label, &self_clone.proid_text, false);
                func_main::set_visibility(&self_clone.signature_label, &self_clone.signature_text, false);
                func_main::set_visibility(&self_clone.code_label, &self_clone.code_text, false);
                func_main::set_visibility(&self_clone.collection_label, &self_clone.collection_text, false);
                func_main::set_visibility(&self_clone.link_label, &self_clone.link_text, false);
            }
            Ok(())
        });
        let self_clone = self.clone();
        self.platform_combo.on().cbn_sel_change(move || {
            let selected_index = self_clone.platform_combo.items().selected_index();
            match selected_index {
                Some(0) => { // Claim
                    func_main::set_visibility(&self_clone.proid_label, &self_clone.proid_text, true);
                    func_main::set_visibility(&self_clone.signature_label, &self_clone.signature_text, true);
                    func_main::set_visibility(&self_clone.code_label, &self_clone.code_text, false);
                    func_main::set_visibility(&self_clone.collection_label, &self_clone.collection_text, false);
                    func_main::set_visibility(&self_clone.link_label, &self_clone.link_text, false);
                }, Some(1) => { // Code
                    func_main::set_visibility(&self_clone.proid_label, &self_clone.proid_text, false);
                    func_main::set_visibility(&self_clone.signature_label, &self_clone.signature_text, false);
                    func_main::set_visibility(&self_clone.code_label, &self_clone.code_text, true);
                    func_main::set_visibility(&self_clone.collection_label, &self_clone.collection_text, false);
                    func_main::set_visibility(&self_clone.link_label, &self_clone.link_text, false);
                }, Some(2) => { // Collection id
                    func_main::set_visibility(&self_clone.proid_label, &self_clone.proid_text, false);
                    func_main::set_visibility(&self_clone.signature_label, &self_clone.signature_text, false);
                    func_main::set_visibility(&self_clone.code_label, &self_clone.code_text, false);
                    func_main::set_visibility(&self_clone.collection_label, &self_clone.collection_text, true);
                    func_main::set_visibility(&self_clone.link_label, &self_clone.link_text, false);
                }, Some(3) => { // Link
                    func_main::set_visibility(&self_clone.proid_label, &self_clone.proid_text, false);
                    func_main::set_visibility(&self_clone.signature_label, &self_clone.signature_text, false);
                    func_main::set_visibility(&self_clone.code_label, &self_clone.code_text, false);
                    func_main::set_visibility(&self_clone.collection_label, &self_clone.collection_text, false);
                    func_main::set_visibility(&self_clone.link_label, &self_clone.link_text, true);
                }, _ => {}
            }
            Ok(())
        });
        let self2 = self.clone();
        self.cek_button.on().bn_clicked(move || {
            println!("Cek button clicked!");
            println!("{}", self2.url_text.text().unwrap_or_else(|_| String::new()));
            // Disable the button to prevent multiple async tasks from being started
            self2.cek_button.hwnd().EnableWindow(false);
            let _ = self2.cek_button.hwnd().SetWindowText("Wait");
            self2.variasi_combo.items().delete_all();
            self2.kurir_combo.items().delete_all();
            let file = match self2.file_combo.items().selected_text() {
                Ok(Some(text)) => text,
                Ok(None) => "".to_string(),
                Err(_) => "".to_string()
            };
            if self2.url_text.text().unwrap_or_else(|_| String::new()).is_empty() {
                let _ = func_main::error_modal(&self2.wnd2, "Error", "Empty URL");
                println!("Empty URL");
                self2.cek_button.hwnd().EnableWindow(true);
                //cek_button.clone().hwnd().ShowWindow(SW::HIDE);
                let _ = self2.cek_button.hwnd().SetWindowText("Cek");
            } else if file.is_empty() {
                let _ = func_main::error_modal(&self2.wnd2, "Error", "Please select a file before running the program");
                println!("Please select a file before running the program");
                self2.cek_button.hwnd().EnableWindow(true);
                self2.cek_button.hwnd().ShowWindow(SW::SHOW);
                let _ = self2.cek_button.hwnd().SetWindowText("Cek");
            }else{
                let cookie_data = prepare::create_cookie(&prepare::read_cookie_file(&file));
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
                                    let _ = func_main::error_modal(&self2.wnd2, "Error get Variation", &isi);
                                }
                            },
                            Ok(Err(e)) => {
                                println!("Error: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {:?}", e);
                                let _ = func_main::error_modal(&self2.wnd2, "Error get Variation", &isi);
                            },
                            Err(_) => {
                                eprintln!("Timeout occurred");
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nTimeout : Timeout occurred");
                                let _ = func_main::error_modal(&self2.wnd2, "Error get Variation", &isi);
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
                        chosen_model.quantity = self2.quantity_text.text().unwrap_or_else(|_| String::new()).parse::<i32>().unwrap_or(1);
                        match timeout(Duration::from_secs(10), runtime::prepare_ext::get_shipping_data(client.clone(), base_headers.clone(), shared_headers.clone(), &device_info, Some(&product_info), &address_info, &chosen_model, &chosen_payment, &chosen_shipping)).await {
                            Ok(Ok(kurirs)) => {
                                let mut shared = shared_kurir_data_clone.lock().unwrap();
                                shared.clear();
                                *shared = kurirs.clone(); 
                                let kurirs_iter: Vec<String> = kurirs.iter().map(|kurirs| kurirs.channel_name.clone()).collect();
                                for name_kurir in &kurirs_iter {
                                    println!("{}", name_kurir);
                                    let _ = self2.kurir_combo.items().add(&[name_kurir]);
                                    self2.kurir_combo.items().select(Some(0));
                                }
                            },
                            Ok(Err(e)) => {
                                println!("Error: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nError : {:?}", e);
                                let _ = func_main::error_modal(&self2.wnd2, "Error get Shipping", &isi);
                            },
                            Err(e) => {
                                println!("Timeout: {:?}", e);
                                let isi = format!("OH SNAP!!!\nSolution:\n1. Please Renew cookie!\n2. Disable Proxy\n3. Contact Administrator\n\nTimeout : {:?}", e);
                                let _ = func_main::error_modal(&self2.wnd2, "Error get Shipping", &isi);
                            }
                        };
                        self2.cek_button.clone().hwnd().EnableWindow(true);
                        let _ = self2.cek_button.hwnd().SetWindowText("Cek");
                    }else{
                        let _ = func_main::error_modal(&self2.wnd2, "Error", "Invalid URL");
                        println!("Invalid URL");
                        self2.cek_button.hwnd().EnableWindow(true);
                        let _ = self2.cek_button.hwnd().SetWindowText("Cek");
                    }
                });
            };
            Ok(())
        });
        let self_clone = self.clone();
        self.launch_button.on().bn_clicked(move || {
            let command = self_clone.generate_cmd();
            let mut new_command = Vec::new();
            if let Ok(Some(command))  = command {
                let _url = self_clone.generate_struct(command.clone());
                new_command.push("start".to_string());
                new_command.push("multi.exe".to_string());
                new_command.extend(command);
                let _ = self_clone.execute(new_command);
            }
            Ok(())
        });
    }
    fn generate_cmd(&self) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
        let self2 = self.clone();
        let list_count = self2.my_list.items().count();
        println!("Launch button clicked!");
        let url = if list_count > 0 {
            let mut collected_text = Vec::new();
            for index in 0..list_count {
                collected_text.push(self2.my_list.items().get(index).text(0));
            }
            collected_text
        } else {
            eprintln!("Variasi is not Found");
            return Ok(None);
        };
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
        let variasi = if list_count > 0 {
            let mut collected_text = Vec::new();
            for index in 0..list_count {
                collected_text.push(self2.my_list.items().get(index).text(2));
            }
            collected_text
        } else {
            eprintln!("Variasi is not Found");
            return Ok(None);
        };
        let kurir = if list_count > 0 {
            let mut collected_text = Vec::new();
            for index in 0..list_count {
                collected_text.push(self2.my_list.items().get(index).text(3));
            }
            collected_text
        } else {
            eprintln!("Kurir is not Found");
            return Ok(None);
        };
        println!("Variasi: {:?}", variasi);
        let jam = self2.jam_text.text().unwrap_or_else(|_| String::new());
        let menit = self2.menit_text.text().unwrap_or_else(|_| String::new());
        let detik = self2.detik_text.text().unwrap_or_else(|_| String::new());
        let mili = self2.mili_text.text().unwrap_or_else(|_| String::new());
        let kuan =  if list_count > 0 {
            let mut collected_text = Vec::new();
            for index in 0..list_count {
                collected_text.push(self2.my_list.items().get(index).text(1));
            }
            collected_text
        } else {
            eprintln!("Kurir is not Found");
            return Ok(None);
        };
        //let token = self2.token_text.text().unwrap_or_else(|_| String::new());
        // Menjalankan program abs.exe dengan argumen yang dibuat
        let create_command = |extra_args: Vec<String>| -> Vec<String> {
            let mut command = vec![
                "--file".to_string(), file,
                "--time".to_string(), format!("{}:{}:{}.{}", &jam, &menit, &detik, &mili),
                "--payment".to_string(), payment,
                "--harga".to_string(), harga,
                "--no-coins".to_string(),
                "--dump".to_string(),
                "--token".to_string(), "".to_string(),
            ];
            command.push("--url".to_string());
            for url1 in &url {
                command.push(url1.clone());
            }
            command.push("--kurir".to_string());
            for kurir in &kurir {
                command.push(kurir.clone());
            }
            command.push("--quantity".to_string());
            for kuan in &kuan {
                command.push(kuan.clone());
            }
            if !variasi.is_empty() {
                command.push("--product".to_string());
                for variasi in &variasi {
                    command.push(variasi.clone());
                }
            }
            if self2.harga_checkbox.state() == co::BST::CHECKED {
                command.push("--bypass".to_string());
            }
            command.extend(extra_args);
            command
        };
        let mut commands = vec![];

        if self2.fsv_checkbox.state() == co::BST::CHECKED {
            commands.push("--fsv-only".to_string());
        }
        if self2.platform_checkbox.state() == co::BST::CHECKED {
            match self2.platform_combo.items().selected_index() {
                Some(0) => {
                    commands.push("--claim-platform-vouchers".to_string());
                    commands.push("--pro-id".to_string());
                    commands.push(self2.proid_text.text().unwrap_or_else(|_| String::new()));
                    commands.push("--sign".to_string());
                    commands.push(self2.signature_text.text().unwrap_or_else(|_| String::new()));
                }
                Some(1) => {
                    commands.push("--platform-vouchers".to_string());
                    commands.push("--code-platform".to_string());
                    commands.push(self2.code_text.text().unwrap_or_else(|_| String::new()));
                }
                Some(2) => {
                    commands.push("--collection-vouchers".to_string());
                    commands.push("--collectionid".to_string());
                    commands.push(self2.collection_text.text().unwrap_or_else(|_| String::new()));
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
        let code_shop = if list_count > 0 {
            let mut collected_text = Vec::new();
            for index in 0..list_count {
                collected_text.push(self2.my_list.items().get(index).text(4));
            }
            collected_text
        } else {
            eprintln!("code_shop is not Found");
            return Ok(None);
        };
        commands.push("--shop-vouchers".to_string());
        commands.push("--code-shop".to_string());
        for code_shop in code_shop {
            commands.push(code_shop);
        }
        Ok(Some(create_command(commands)))
    }
    fn execute(&self, command: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let file = match self.file_combo.items().selected_text() {
            Ok(Some(text)) => text,
            Ok(None) => "".to_string(),
            Err(_) => "".to_string()
        };
        if !file.is_empty() {
            let _status = std::process::Command::new("cmd")
                .arg("/c")
                .args(&command)
                .spawn()
                .expect("Gagal menjalankan program");
        }else{
            let _ = func_main::error_modal(&self.wnd2, "Error", "Please select a file before running the program");
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
pub fn updater_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let taskbar = w::CoCreateInstance::<w::ITaskbarList4>(
        &co::CLSID::TaskbarList,
        None::<&w::IUnknown>,
        co::CLSCTX::INPROC_SERVER,
    ).map_err(|_| ())?;
    let check_version = Arc::new(Mutex::new(String::new()));
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowModal::new_dlg(100);
    let info_label = gui::Label::new_dlg(&wnd2, 101, dont_move);
    let progress = gui::ProgressBar::new_dlg(&wnd2, 102, dont_move);
    let download_button = gui::Button::new_dlg(&wnd2, 103, dont_move);
    let cancel_button = gui::Button::new_dlg(&wnd2, 104, dont_move);
    let rollback_button = gui::Button::new_dlg(&wnd2, 105, dont_move);

    let download_button_clone = download_button.clone();
    let info_label_clone = info_label.clone();
    let check_version_clone = check_version.clone();
    let progress_clone = progress.clone();
    let rollback_button_clone = rollback_button.clone();
    let wnd_clone = wnd.clone();
    let taskbar_clone = taskbar.clone();
    wnd2.on().wm_init_dialog(move |_| {
        progress_clone.set_marquee(true);
        rollback_button_clone.hwnd().EnableWindow(false);
        download_button_clone.hwnd().EnableWindow(false);
        let download_button_clone = download_button_clone.clone();
        let check_version = check_version_clone.clone();
        let info_label_clone = info_label_clone.clone();
        let progress_clone = progress_clone.clone();
        let wnd_clone = wnd_clone.clone();
        let taskbar = taskbar_clone.clone();
        if std::path::Path::new("update-dir-old").exists() {
            rollback_button_clone.hwnd().EnableWindow(true);
        }
        tokio::spawn(async move {
                let _ = taskbar.SetProgressState(wnd_clone.hwnd(), co::TBPF::INDETERMINATE);
                if let Some(latest_version) = upgrade::get_latest_release(&format!("https://api.github.com/repos/cari-404/abs/releases/latest")).await {
                    println!("Versi terbaru: {}", latest_version);
                    if upgrade::compare_versions(CURRENT_VERSION, &latest_version) == std::cmp::Ordering::Less {
                        let _ = info_label_clone.set_text_and_resize(&format!("Versi :{} tersedia!", latest_version));
                        download_button_clone.hwnd().EnableWindow(true);
                        let mut shared = check_version.lock().unwrap();
                        shared.clear();
                        *shared = latest_version;
                    }else {
                        let _ = info_label_clone.set_text_and_resize("Versi terbaru sudah terpasang.");
                    }
                } else {
                    let _ = info_label_clone.set_text_and_resize("Gagal mengecek versi terbaru.");
                }
                progress_clone.set_marquee(false);
                let _ = taskbar.SetProgressState(wnd_clone.hwnd(), co::TBPF::NOPROGRESS);
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
    let wnd_clone = wnd.clone();
    let taskbar_clone = taskbar.clone();
    download_button.on().bn_clicked(move || {
        let download_button_clone = download_button_clone.clone();
        let shared =  {
            let guard = check_version_clone.lock().unwrap();
            guard.clone()
        };
        let info_label_clone = info_label_clone.clone();
        let progress_clone = progress_clone.clone();
        let wnd2_clone = wnd2_clone.clone();
        let wnd_clone = wnd_clone.clone();
        let taskbar = taskbar_clone.clone();
        if download_button_clone.hwnd().GetWindowText().unwrap_or_default() == "Install" {
            let _ = std::process::Command::new("cmd")
                .arg("/C")
                .arg("updater.exe offline")
                .spawn();
            std::process::exit(0);
        }else{
            tokio::spawn(async move {
                let client = Arc::new(prepare::universal_client_skip_headers().await);
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
                let resp =  upgrade::fetch_download_response(client.clone(), &url).await;
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
                            let _ = info_label_clone.set_text_and_resize(&format!(
                                "Kecepatan: {:.2} KB/s | Diunduh: {:.2}/{:.2} MB ({:.1}%) | ETA: {}",                     speed_kb,                     downloaded_mb,                     total_mb,                     percentage,                     format_eta(eta_secs)
                            ));
                            let _ = taskbar.SetProgressValue(wnd_clone.hwnd(), downloaded as u64, total_size as u64);
                        }
                        let _ = taskbar.SetProgressState(wnd_clone.hwnd(), co::TBPF::NOPROGRESS);
                        file.flush().await.expect("failed to flush data");
                        let actual_size = tokio::fs::metadata("update.zip").await.expect("failed to calculate").len();
                        if actual_size != total_size {
                            let isi = format!("File mungkin corrupt! Ukuran seharusnya {} bytes, tetapi hanya {} bytes.", total_size, actual_size);
                            eprintln!("{}", isi);
                            let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                        }else{
                            let _ = info_label_clone.set_text_and_resize("Click to install(restart the app)");
                            let _ = download_button_clone.hwnd().SetWindowText("Install");
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
    rollback_button.on().bn_clicked(move || {
        let _ = std::process::Command::new("cmd")
            .arg("/C")
            .arg("update-dir-old\\updater.exe rollback")
            .spawn();
        std::process::exit(0);
    });
    let _ = wnd2.show_modal(wnd);
    Ok(())
}
pub fn telegram_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowModal::new_dlg(500);
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
                Ok(config) => config,     Err(e) => {
                    eprintln!("Failed to open config file: {}", e);
                    return; // Keluar dari task jika terjadi error
                }
            };
            // Mendapatkan data dari konfigurasi
            let data = match telegram::get_config(&config).await {
                Ok(data) => data,     Err(e) => {
                    eprintln!("Failed to get config data: {}", e);
                    return; // Keluar dari task jika terjadi error
                }
            };
            println!("{:?}", data);
            let _ = token_clone.set_text(&data.telegram_token);
            let _ = chat_id_clone.set_text(&data.telegram_chat_id);
            if data.telegram_notif == true{
                checkbox_clone.set_state(co::BST::CHECKED);
            }else{
                checkbox_clone.set_state(co::BST::UNCHECKED);
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
        let token_text = token_clone.text().unwrap_or_else(|_| String::new());
        let chat_id_text = chat_id_clone.text().unwrap_or_else(|_| String::new());
        if token_text.is_empty() || chat_id_text.is_empty() {
            let isi = format!("please fill token and chat id");
            let _ = func_main::error_modal(&wnd2_clone, "Error save data", &isi);
        }else{
            let wnd2_clone = wnd2_clone.clone();
            let checkbox_clone = checkbox_clone.clone();
            tokio::spawn(async move {
                let config_content = match telegram::open_config_file().await {
                    Ok(config) => config,         Err(e) => {
                        eprintln!("Failed to open config file: {}. Creating a new one.", e);
                    "{}".to_string()
                    }
                };
                let mut config: serde_json::Value = match serde_json::from_str(&config_content) {
                    Ok(json) => json,         Err(e) => {
                        eprintln!("Failed to parse config file: {}. Creating a new one.", e);
                        serde_json::json!({})
                    }
                };
                if let serde_json::Value::Object(ref mut map) = config {
                    if checkbox_clone.state() == co::BST::CHECKED {
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
            let client = Arc::new(prepare::universal_client_skip_headers().await);
            println!("test Send");
            let token_text = token_clone.text().unwrap_or_else(|_| String::new());
            let chat_id_text = chat_id_clone.text().unwrap_or_else(|_| String::new());
            if token_text.is_empty() || chat_id_text.is_empty() {
                let isi = format!("please fill token and chat id");
                let _ = func_main::error_modal(&wnd2_clone, "Error get data", &isi);
            }else{
                let data = telegram::get_data(&token_text, &chat_id_text);
                match telegram::send_msg(client.clone(), &data, "This is a test message; you can ignore it.").await {
                    Ok(_) => println!("sent"),         Err(e) => println!("error: {}", e),     };
            }
        });
        Ok(())
    });
    let _ = wnd2.show_modal(wnd);
    Ok(())
}
pub fn account_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowModal::new_dlg(1000);
    let resize_h = (gui::Horz::Resize, gui::Vert::None);
    let _resize_v = (gui::Horz::None, gui::Vert::Resize);
    let resized = (gui::Horz::Resize, gui::Vert::Resize);
    let move_h = (gui::Horz::Repos, gui::Vert::None);
    let _move_v = (gui::Horz::None, gui::Vert::Repos);
    let moved = (gui::Horz::Repos, gui::Vert::Repos);
    let _move_h_resize_v = (gui::Horz::Repos, gui::Vert::Resize);
    let _resize_h_move_v = (gui::Horz::Resize, gui::Vert::Repos);
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
    wnd2.on().wm_command_acc_menu(101 as u16, move || {
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
        let _ = my_list_clone.items().delete_all();
        let cookie_edit_text = cookie_edit_clone.text().unwrap_or_else(|_| String::new());
        let cookie_data = prepare::create_cookie(&cookie_edit_text);
        let _ = my_list_clone.items().add(
            &[
                "CSRFTOKEN",     &cookie_data.csrftoken, ], None, (),
        );
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let cookie_edit_clone = cookie_edit.clone();
    let sz_edit_clone = sz_edit.clone();
    let my_list_clone = my_list.clone();
    wnd2.on().wm_init_dialog(move |_| {
        let _ = my_list_clone.cols().add("Item", 120);
        let _ = my_list_clone.cols().add("Value", 300);
        let _ = my_list_clone.items().add(
            &[
                "Default",     "text", ], None, // no icon; requires a previous set_image_list()
            (),   // no object data; requires specifying the generic `ListView` type
        );
        let _ = my_list_clone.items().add(
            &[
                "CSRFTOKEN",     "Our CSRF token Hardwork", ], None, // no icon; requires a previous set_image_list()
            (),   // no object data; requires specifying the generic `ListView` type
        );
        let _ = my_list_clone.items().add(
            &[
                "Visible",     "True", ], None, (),
        );
        my_list_clone.set_extended_style(true, w::co::LVS_EX::FULLROWSELECT);
        func_main::populate_combobox_with_files(&file_combo_clone, "akun");
        func_main::handle_file_selection(&file_combo_clone, &cookie_edit_clone, &sz_edit_clone, &my_list_clone);
        Ok(true)
    });
    //let file_combo_clone = file_combo.clone();
    let cookie_edit_clone = cookie_edit.clone();
    let sz_edit_clone = sz_edit.clone();
    let my_list_clone = my_list.clone();
    file_combo.on().cbn_edit_change(move || {
        println!("Edit change!");
        let _ = my_list_clone.items().delete_all();
        let _ = sz_edit_clone.set_text("");
        let _ = cookie_edit_clone.set_text("");
        //println!("{}", file_combo_clone.text());
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
        let selected_index = file_combo_clone.items().selected_index();
        func_main::populate_combobox_with_files(&file_combo_clone, "akun");
        file_combo_clone.items().select(selected_index);
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
        let file = match file_combo_clone.items().selected_text() {
            Ok(Some(text)) => text,
            Ok(None) => "".to_string(),
            Err(_) => "".to_string()
        };
        let cookie = cookie_edit.text().unwrap_or_else(|_| String::new());
        let sz = sz_edit.text().unwrap_or_else(|_| String::new());
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
    let _ = wnd2.show_modal(wnd);
    //wnd2.run_main(None);
    Ok(())
}
pub fn log_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let wnd2 = gui::WindowModeless::new_dlg(wnd, 600, (0,0));
    let _txt_log = gui::Edit::new_dlg(&wnd2, 601, dont_move);
    Ok(())
}
pub fn show_fs_window(wnd: &gui::WindowMain) -> Result<(), ()> {
    let taskbar = w::CoCreateInstance::<w::ITaskbarList4>(
        &co::CLSID::TaskbarList,
        None::<&w::IUnknown>,
        co::CLSCTX::INPROC_SERVER,
    ).map_err(|_| ())?;
    let (tx_msg, rx_msg) = mpsc::unbounded_channel::<String>();
    let _ = tx_msg.send("Stopped".to_string());
    let interrupt_flag = Arc::new(AtomicBool::new(false));
    let shared_fsid = Arc::new(Mutex::new(vec![]));
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let _resize_h = (gui::Horz::Resize, gui::Vert::None);
    let _resize_v = (gui::Horz::None, gui::Vert::Resize);
    let resized = (gui::Horz::Resize, gui::Vert::Resize);
    let move_h = (gui::Horz::Repos, gui::Vert::None);
    let _move_v = (gui::Horz::None, gui::Vert::Repos);
    let moved = (gui::Horz::Repos, gui::Vert::Repos);
    let _move_h_resize_v = (gui::Horz::Repos, gui::Vert::Resize);
    let resize_h_move_v = (gui::Horz::Resize, gui::Vert::Repos);
    let wnd2 = gui::WindowModal::new_dlg(700);
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
    wnd2.on().wm_command_acc_menu(201 as u16, move || {
        if let Some(selected_item) = my_list_clone.items().focused() {
            let url = selected_item.text(5 as u32);
            let _ = func_main::open_url(&url);
        } else {
            println!("No item selected");
        }
        Ok(())
    });
    let my_list_clone = my_list.clone();
    wnd2.on().wm_command_acc_menu(202 as u16, move || {
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
    wnd2.on().wm_command_acc_menu(203 as u16, move || {
        let file = match file_combo_clone.items().selected_text() {
            Ok(Some(text)) => text,
            Ok(None) => "".to_string(),
            Err(_) => "".to_string()
        };
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
                let _ = item.set_text(6 as u32, &format!("{:?}", variation));
            });
        } else {
            println!("No item selected");
        }
        Ok(())
    });
    let file_combo_clone = file_combo.clone();
    let my_list_clone = my_list.clone();
    wnd2.on().wm_init_dialog(move |_| {
        let _ = my_list_clone.cols().add("Name", 120);
        let _ = my_list_clone.cols().add("Modelid", 120);
        let _ = my_list_clone.cols().add("Estimate", 120);
        let _ = my_list_clone.cols().add("Hidden", 120);
        let _ = my_list_clone.cols().add("Stock", 120);
        let _ = my_list_clone.cols().add("Url", 300);
        let _ = my_list_clone.cols().add("Variation", 120);
        let _ = my_list_clone.items().add(
            &[
                "Default",     "text",     "text",     "text",     "text",     "text",     "", ], None, // no icon; requires a previous set_image_list()
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
        let search_text = search_edit_clone.text().unwrap_or_else(|_| String::new());
        if search_text.is_empty() {
            let isi = format!("Please input the search text");
            let _ = func_main::error_modal(&wnd2_clone, "Error search data", &isi);
        } else {
            for i in 0..my_list_clone.items().count() {
                let item = my_list_clone.items().get(i);
                let text = item.text(0).to_lowercase();
                if text.contains(&search_text.to_lowercase()) {
                    println!("Item cocok di index {}: {}", i, text);
                    let _ = item.select(true); // pilih item
                    let _ = item.ensure_visible(); // scroll ke item
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
    let taskbar_clone = taskbar.clone();
    let wnd_clone = wnd.clone();
    cek_button.on().bn_clicked(move || {
        progress_clone.set_marquee(true);
        let _ = taskbar_clone.SetProgressState(wnd_clone.hwnd(), co::TBPF::INDETERMINATE);
        let file = match file_combo_clone.items().selected_text() {
            Ok(Some(text)) => text,
            Ok(None) => "".to_string(),
            Err(_) => "".to_string()
        };
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
            let taskbar = taskbar_clone.clone();
            let wnd_clone = wnd_clone.clone();
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
                            let _ = fs_combo_clone.items().add(&[&fsi.promotionid.to_string()]);
                        }
                        if fs_combo_clone.items().count() > Ok(0) {
                            fs_combo_clone.items().select(Some(0));
                        }
                        let mut shared = shared_fsid_clone.lock().unwrap();
                        shared.clear();
                        *shared = fsid_current.clone();
                        progress_clone.set_marquee(false);
                        let _ = taskbar.SetProgressState(wnd_clone.hwnd(), co::TBPF::NOPROGRESS);
                    }
                    Err(e) => {
                        let isi = format!("Error: {}", e);
                        let _ = func_main::error_modal(&wnd2_clone, "Error", &isi);
                        progress_clone.set_marquee(false);
                        let _ = taskbar.SetProgressState(wnd_clone.hwnd(), co::TBPF::NOPROGRESS);
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
    let wnd_clone = wnd.clone();
    single_button.on().bn_clicked(move || {
        progress_clone.set_state(w::co::PBST::NORMAL);
        let fsid = match fs_combo_clone.items().selected_text() {
            Ok(Some(text)) => text,
            Ok(None) => "".to_string(),
            Err(_) => "".to_string()
        };
        if fsid.is_empty() {
            let isi = format!("Please select a fs id before checking the fs");
            let _ = func_main::error_modal(&wnd2_clone, "Error check data", &isi);
        } else {
            let _ = mode_label_clone.set_text_and_resize("Single");
            wnd2_clone.hwnd().ShowWindow(w::co::SW::SHOWMAXIMIZED);
            let Ok(Some(selecte_fsid)) = fs_combo_clone.items().selected_text() else {
                eprintln!("Tidak ada fsid yang dipilih.");
                return Ok(());
            };
            let promotionid: i64 = match selecte_fsid.parse() {
                Ok(id) => id,     Err(_) => {
                    eprintln!("fsid yang dipilih bukan angka valid: {}", selecte_fsid);
                    return Ok(());
                }
            };
            let mut fsinfo = Vec::new();
            let shared = shared_fsid_clone.lock().unwrap();
            if let Some(matching) = shared.iter().find(|item| item.promotionid == promotionid) {
                let _ = proid_label.set_text_and_resize(&matching.promotionid.to_string());
                let _ = stime_label.set_text_and_resize(&(func_main::human_readable_time(matching.start_time)).to_string());
                let _ = etime_label.set_text_and_resize(&(func_main::human_readable_time(matching.end_time)).to_string());
                fsinfo.push(matching.clone());
            } else {
                println!("Tidak ditemukan promotionid yang cocok");
            }
            let _ = get_flashsale_products(&wnd_clone, &wnd2_clone, Arc::new(fsinfo), &file_combo_clone, &my_list_clone, &progress_clone, &interrupt_flag_clone, &tx_msg_clone, &progress_label_clone, &count_label_clone);
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
    let wnd_clone = wnd.clone();
    all_button.on().bn_clicked(move || {
        progress_clone.set_state(w::co::PBST::NORMAL);
        let shared = shared_fsid_clone.lock().unwrap();
        if shared.is_empty() {
            println!("empty shared!");
            let isi = format!("Promotionid not available\nPlease check the account selected and try again");
            let _ = func_main::error_modal(&wnd2_clone, "Error: not found", &isi);
        } else {
            let _ = mode_label_clone.set_text_and_resize("All");
            wnd2_clone.hwnd().ShowWindow(w::co::SW::SHOWMAXIMIZED);
            let _ = get_flashsale_products(&wnd_clone, &wnd2_clone, Arc::new(shared.to_vec()), &file_combo_clone, &my_list_clone, &progress_clone, &interrupt_flag_clone, &tx_msg_clone, &progress_label_clone, &count_label_clone);
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
    let taskbar_clone = taskbar.clone();
    let wnd_clone = wnd.clone();
    stop_button.on().bn_clicked(move || {
        interrupt_flag_clone.store(true, Ordering::Relaxed);
        let isi = format!("Scan was stopped by user");
        let _ = func_main::info_modal(&wnd2_clone, "Info", &isi);
        progress_clone.set_state(w::co::PBST::PAUSED);
        let _ = taskbar_clone.SetProgressState(wnd_clone.hwnd(), co::TBPF::NOPROGRESS);
        Ok(())
    });
    let interrupt_flag_clone = interrupt_flag.clone();
    let rx_msg = Arc::new(Mutex::new(rx_msg));
    let rx_msg_clone = rx_msg.clone();
    let my_list_clone = my_list.clone();
    let taskbar_clone = taskbar.clone();
    let wnd_clone = wnd.clone();
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
        let _ = my_list_clone.items().delete_all();
        println!("Window is gone, goodbye!");
        let _ = taskbar_clone.SetProgressState(wnd_clone.hwnd(), co::TBPF::NOPROGRESS);
        Ok(())
    });

    let _ = wnd2.show_modal(wnd);
    //wnd2.run_main(None);
    Ok(())
}
fn get_flashsale_products(wnd: &gui::WindowMain, wnd2: &gui::WindowModal, fsinfo: Arc<Vec<FSInfo>>, file_combo: &gui::ComboBox, my_list: &gui::ListView, progress: &gui::ProgressBar, interrupt_flag: &Arc<AtomicBool>, tx_msg: &mpsc::UnboundedSender<String>, progress_label: &gui::Label, count_label: &gui::Label) -> Result<(), ()> {
    let file = match file_combo.items().selected_text() {
        Ok(Some(text)) => text,
        Ok(None) => "".to_string(),
        Err(_) => "".to_string()
    };
    let wnd_clone = wnd.clone();
    let taskbar_clone = w::CoCreateInstance::<w::ITaskbarList4>(
        &co::CLSID::TaskbarList,
        None::<&w::IUnknown>,
        co::CLSCTX::INPROC_SERVER,
    ).map_err(|_| ())?;
    if file.is_empty() {
        let isi = format!("Please select a file before checking the fs");
        let _ = func_main::error_modal(&wnd2, "Error check data", &isi);
    } else {
        let _ = my_list.items().delete_all();
        let Ok(Some(select_cookie_file)) = file_combo.items().selected_text() else {
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
        let taskbar = taskbar_clone.clone();
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
                                        let _ = my_list_clone.items().add(&[
                                            &item.name,                                 &format!("{:?}", item.modelids.unwrap_or_default()),                                 &func_main::format_thousands(item.price_before_discount * (100 - item.raw_discount) / 100 / 100000),                                 &item.hidden_price_display.as_deref().unwrap_or("No Hide").to_string(),                                 &item.stock.to_string(),                                 &link,                             ], None, ());
                                        let _ = tx_msg.send("Running".to_string());
                                        count +=1;
                                        let _ = count_label.set_text_and_resize(&count.to_string());
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
                            let _ = progress_label_clone.set_text_and_resize(&progressinf);
                            let _ = taskbar.SetProgressValue(wnd_clone.hwnd(), potition as u64, max as u64);
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
            let _ = taskbar.SetProgressState(wnd_clone.hwnd(), co::TBPF::NOPROGRESS);
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
