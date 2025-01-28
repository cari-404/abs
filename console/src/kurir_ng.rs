use runtime::prepare::{self, ShippingInfo};
use runtime::task::{self};

use crate::get_user_input;
use crate::Opt;

pub fn choose_shipping(shippings: &[ShippingInfo], opt: &Opt) -> Option<ShippingInfo> {
    println!("shipping yang tersedia:");

    for (index, shipping) in shippings.iter().enumerate() {
        println!("{}. {} - Harga: {} - Id: {}", index + 1, shipping.channel_name, shipping.original_cost / 100000, shipping.channelid);
    }

    if let Some(kurir) = opt.kurir.clone() {
        // If opt.kurir is present, find the shipping with a matching channel_name
        if let Some(selected_shipping) = shippings.iter().find(|shipping| shipping.channel_name == kurir) {
            println!("{:?}", selected_shipping);
            return Some(selected_shipping.clone());
        } else {
            println!("Tidak ada shipping dengan nama '{}'", kurir);
            return None;
        }
    }

	let user_input = get_user_input("Pilih Shipping yang disediakan: ");

    // Convert user input to a number
    if let Ok(choice_index) = user_input.trim().parse::<usize>() {
        // Return the selected shipping based on the index
        println!("{:?}", shippings.get(choice_index - 1).cloned());
        return shippings.get(choice_index - 1).cloned();
    } else if user_input.trim().to_uppercase() == "N" {
        println!("Menampilkan lebih banyak pilihan...");
    }

    None
}