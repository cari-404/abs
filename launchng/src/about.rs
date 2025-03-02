use winsafe::{self as w,
    gui, prelude::*
};
use windows_version::*;

use crate::func_main;

pub fn about_window(wnd: &gui::WindowMain) -> Result<(), ()> {
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
    let wnd2 = gui::WindowModal::new_dlg(wnd, 2000);
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
}