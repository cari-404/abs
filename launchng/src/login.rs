use winsafe::{self as w,
    gui, prelude::*, AnyResult,
    HBITMAP,
};
use ::runtime::login::{get_qrcode, authentication_qrcode, get_cookie};
use runtime::prepare;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

use crate::func_main::png_base64_to_pixels_ptr;

pub fn login_window(wnd: &gui::WindowModal) -> Result<(), ()> {
    let (tx_msg, rx_msg) = mpsc::unbounded_channel::<String>();
    let _ = tx_msg.send("Stopped".to_string());
    let interrupt_flag = Arc::new(AtomicBool::new(false));
    let dont_move = (gui::Horz::None, gui::Vert::None);
    let loginwnd = gui::WindowModal::new_dlg(3000);
    let label = gui::Label::new_dlg(&loginwnd, 3001, dont_move);
    let button = gui::Button::new_dlg(&loginwnd, 3002, dont_move);
    let label_clone = label.clone();
    let interrupt_flag_clone = interrupt_flag.clone();
    let tx_msg_clone = tx_msg.clone();
    button.on().bn_clicked(move || {
        println!("Button clicked");
        let _ = login_internals(&label_clone, &interrupt_flag_clone, &tx_msg_clone);
        Ok(())
    });
    let label_clone = label.clone();
    let interrupt_flag_clone = interrupt_flag.clone();
    let tx_msg_clone = tx_msg.clone();
    loginwnd.on().wm_init_dialog(move |_| {
        let bitmap_result = w::HINSTANCE::GetModuleHandle(None)
        .and_then(|hinstance| {
            hinstance.LoadImageBitmap(
                w::IdObmStr::Id(1),
                w::SIZE::with(256, 256),
                w::co::LR::DEFAULTCOLOR,
            )
        });
        // Menangani kesalahan jika pemuatan bitmap gagal
        let mut bitmap = match bitmap_result {
            Ok(bitmap) => bitmap, // Jika berhasil, ambil nilai HBITMAP
            Err(err) => {
                eprintln!("Gagal memuat bitmap: {:?}", err);
                return Ok(false); // Mengembalikan false untuk menghentikan inisialisasi dialog
            }
        };
        let nbit_leak = bitmap.leak();
        unsafe {
            if let Err(err) = label_clone.hwnd().SendMessage(
                w::msg::stm::SetImage {
                    image: w::BmpIconCurMeta::Bmp(nbit_leak),
                }
            ) {
                eprintln!("Gagal mengirim pesan ke label: {}", err);
            };
            //button.hwnd().SetWindowText("Login");
        }
        let _ = login_internals(&label_clone, &interrupt_flag_clone, &tx_msg_clone);
        Ok(true)
    });
    let interrupt_flag_clone = interrupt_flag.clone();
    let rx_msg = Arc::new(Mutex::new(rx_msg));
    let rx_msg_clone = rx_msg.clone();
    loginwnd.on().wm_destroy(move || {
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
        println!("Window is gone, goodbye!");
        Ok(())
    });
    let _ = loginwnd.show_modal(wnd);
    Ok(())
}

fn login_internals(label: &gui::Label, interrupt_flag: &Arc<AtomicBool>, tx_msg: &mpsc::UnboundedSender<String>) -> AnyResult<()> {
    println!("start login_internals");
    let label_clone2 = label.clone();
    interrupt_flag.store(false, Ordering::Relaxed);
    let interrupt_flag_clone = interrupt_flag.clone();
    let tx_msg = tx_msg.clone();
    tokio::spawn(async move {
        let client = Arc::new(prepare::universal_client_skip_headers().await);
        println!("start login_internals async");
        loop{
            let qrcode = match get_qrcode(client.clone()).await{
                Ok(qrcode) => qrcode,
                Err(err) => {
                    eprintln!("Error mendapatkan QR code: {}", err);
                    return;
                }
            };
            let qrcode_base64 = qrcode.qrcode_base64.clone();
            /*
            let decode_qrcode = decode(qrcode_base64).unwrap();
            let cursor = Cursor::new(decode_qrcode);
            let png_image = load_from_memory(&cursor.get_ref())
                .expect("Gagal memuat gambar dari data base64");
            let bitmap = match png_image {
                ImageRgba8(rgb_image) => rgb_image,
                _ => png_image.to_rgba8(), // Konversi ke RGB8 jika format awal berbeda
            };
            */
            let (mut pixels, width, height, _stride) = unsafe { png_base64_to_pixels_ptr(&qrcode_base64).unwrap() };
            println!("Bitmap berhasil dibuat. Ukuran: {:?} x {:?}", width, height);
            //let mut pixels = bitmap.into_raw();
            //pixels.chunks_exact_mut(4).for_each(|chunk| chunk[0..3].reverse());
            let mut hbitmap = match HBITMAP::CreateBitmap(
                w::SIZE::with(width.try_into().unwrap(), height.try_into().unwrap()), // Ukuran bitmap
                1,
                32,
                pixels.as_mut_ptr().cast(), // Pointer ke data piksel
            ) {
                Ok(bitmap) => bitmap,
                Err(err) => {
                    eprintln!("Gagal membuat HBITMAP: {}", err);
                    return;
                }
            };
            let hbitmap_leak = hbitmap.leak();
            let hbitmap2 = hbitmap.leak();
            // Update GUI dengan gambar
            unsafe {
                if let Err(err) = label_clone2.hwnd().SendMessage(
                    w::msg::stm::SetImage {
                        image: w::BmpIconCurMeta::Bmp(hbitmap_leak),
                    }
                ) {
                    let _ = label_clone2.hwnd().SendMessage(
                        w::msg::stm::SetImage {
                            image: w::BmpIconCurMeta::Bmp(hbitmap2),
                        }
                    );
                    eprintln!("Gagal mengirim pesan ke label: {}", err);
                };
            }
            let mut qrcode_token = String::new();
            loop {
                if interrupt_flag_clone.load(Ordering::Relaxed) {
                    println!("Proses dibatalkan oleh user.");
                    let _ = tx_msg.send("Stopped".to_string());
                    break;
                }
                sleep(Duration::from_secs(5)).await;
                // Menangani hasil dari authentication_qrcode
                let (status, qrct) = match authentication_qrcode(client.clone(), &qrcode).await {
                    Ok(result) => result,
                    Err(err) => {
                        eprintln!("Error mendapatkan status QR code: {}", err);
                        break; // Menghentikan loop jika terjadi error
                    }
                };
            
                match status.as_str() {
                    "CONFIRMED" => {
                        println!("QR code berhasil di konfirmasi");
                        let _ = tx_msg.send("Running".to_string());
                        qrcode_token = qrct;
                        break; // Keluar dari loop jika QR code berhasil dikonfirmasi
                    },
                    "SCANNED" => {
                        println!("QR code telah discan");
                        println!("Klik konfirmasi pada perangkat Anda");
                        let _ = tx_msg.send("Running".to_string());
                        continue;
                    },
                    "EXPIRED" => {
                        println!("QR code telah kadaluarsa");
                        println!("Mengulangi proses mendapatkan QR code");
                        let _ = tx_msg.send("Running".to_string());
                        break;
                    },
                    _ => {
                        println!("Status QR code: {}", status);
                        println!("Mengulangi proses mendapatkan QR code");
                        let _ = tx_msg.send("Running".to_string());
                        continue;
                    },
                }
            }
            if interrupt_flag_clone.load(Ordering::SeqCst) {
                println!("Task was interrupted, exiting...");
                let _ = tx_msg.send("Stopped".to_string());
                return;
            }
            if qrcode_token.is_empty() {
                continue;
            }else{
                let cookie = match get_cookie(client.clone(), &qrcode_token).await{
                    Ok(cookie) => cookie,
                    Err(err) => {
                        eprintln!("Error mendapatkan cookie: {}", err);
                        return;
                    }
                };
            }
        }
    });
    Ok(())
}
//Archive
/*loginwnd.on().wm_init_dialog(move |_| {
    let mut file_in = w::FileMapped::open(
        "G:\\BOT\\abs2\\target\\x86_64-pc-windows-gnu\\release\\nbit.bmp",
        w::FileAccess::ExistingReadOnly,
    )?;
    let wstr = w::WString::parse(file_in.as_slice())?;
    let raw_bytes = file_in.as_mut_slice();
    let text = w::WString::parse(raw_bytes)?.to_string();
    println!("wstr: {}", wstr);
    println!("text: {}", text);
    let img_src = ImageReader::open("G:\\BOT\\abs2\\target\\x86_64-pc-windows-gnu\\release\\nbit.bmp")
        .unwrap()
        .decode()
        .unwrap();

    println!("Gambar dimensi: {:?}", img_src.dimensions());
    let img = img_src.to_rgba8();
    let mut pixels = img.into_raw();
    pixels.chunks_exact_mut(4).for_each(|chunk| chunk[0..3].reverse());
    /*
    let bitmap_result = w::HINSTANCE::GetModuleHandle(None)
    .and_then(|hinstance| {
        hinstance.LoadImageBitmap(
            w::IdObmStr::Id(1),
            w::SIZE::new(164, 164),
            w::co::LR::DEFAULTCOLOR,
        )
    });
    // Menangani kesalahan jika pemuatan bitmap gagal
    let mut bitmap = match bitmap_result {
        Ok(bitmap) => bitmap, // Jika berhasil, ambil nilai HBITMAP
        Err(err) => {
            eprintln!("Gagal memuat bitmap: {:?}", err);
            return Ok(false); // Mengembalikan false untuk menghentikan inisialisasi dialog
        }
    };
    */
    println!("raw_bytes 1: {:?}", raw_bytes);
    println!("raw_bytes 2: {:?}", raw_bytes.as_mut_ptr());
    let nbit = <HBITMAP as gdi_Hbitmap>::CreateBitmap(
        w::SIZE::new(164, 164),
        1,
        32,
        pixels.as_mut_ptr().cast(),
    );
    //let bitmap_leak = bitmap.leak();
    //println!("bitmap_leak: {:?}", bitmap_leak);
    let nbit_leak = nbit?.leak();
    println!("nbit: {:?}", nbit_leak);
    unsafe {
        label_clone.hwnd().SendMessage(
            w::msg::stm::SetImage {
                image: w::BmpIconCurMeta::Bmp(nbit_leak),
            }
        );
        button.hwnd().SetWindowText("Login");
    }
    Ok(true)
});*/