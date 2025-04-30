use runtime::prepare::{ShippingInfo, ModelInfo, PaymentInfo, FSItems};

use crate::get_user_input;
use crate::Opt;
use crate::format_thousands;

pub fn choose_shipping(shippings: &[ShippingInfo], opt: &Opt) -> Option<ShippingInfo> {
    println!("shipping yang tersedia:");

    for (index, shipping) in shippings.iter().enumerate() {
        println!("{}. {} - Harga: {} - Id: {}", index + 1, shipping.channel_name, shipping.original_cost / 100000, shipping.channelid);
    }

    if let Some(kurir) = &opt.kurir {
        // If opt.kurir is present, find the shipping with a matching channel_name
        if let Some(selected_shipping) = shippings.iter().find(|shipping| shipping.channel_name == *kurir) {
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
pub fn choose_payment(payments: &[PaymentInfo], opt: &Opt) -> Option<PaymentInfo> {
	println!("payment yang tersedia:");

    for (index, payment) in payments.iter().enumerate() {
        println!("{}. {} - Services: {} - Id: {}", index + 1, payment.name, payment.txn_fee / 100000, payment.channel_id);
    }

    if let Some(bayar) = &opt.payment {
        // If opt.payment is present, find the payment with a matching name
        if let Some(selected_payment) = payments.iter().find(|payment| payment.name == *bayar) {
            println!("{:?}", selected_payment);
            return Some(selected_payment.clone());
        } else {
            println!("Tidak ada payment dengan nama '{}'", bayar);
            return None;
        }
    }

    // Convert user input to a number
    if let Ok(choice_index) = get_user_input("Pilih payment yang disediakan: ").trim().parse::<usize>() {
        // Return the selected payment based on the index
        println!("{:?}", payments.get(choice_index - 1).cloned());
        return payments.get(choice_index - 1).cloned();
    }

    None
}
pub fn choose_model(models: &[ModelInfo], opt: &Opt, fs_items: &[FSItems]) -> Option<ModelInfo> {
    println!("Variasi yang tersedia:");

    for (index, model) in models.iter().enumerate() {
        let flashsale = if let Some(item) = fs_items.iter().find(|item| item.modelids.as_ref().expect("").contains(&model.modelid)) {
            format!(
                "[FLASHSALE] - Est≉ {} - Hide: {} - fs-stok: {}",
                format_thousands(item.price_before_discount * (100 - item.raw_discount) / 100 / 100000),
                item.hidden_price_display.as_deref().unwrap_or("N/A"),
                item.stock
            )
        } else {
            String::new()
        };
        println!("{}. {} - Harga: {} - Stok: {} {}", index + 1, model.name, format_thousands(model.price / 100000), model.stock, flashsale);
    }
    // Check if there is only one model
    if models.len() == 1 {
        println!("Hanya satu variasi tersedia. Memilih secara otomatis.");
        return Some(models[0].clone());
    }

    if let Some(product) = &opt.product {
        // If opt.product is present, find the model with a matching name
        if let Some(selected_model) = models.iter().find(|model| model.name == *product) {
            println!("{:?}", selected_model);
            return Some(selected_model.clone());
        } else {
            println!("Tidak ada model dengan nama '{}'", product);
            return None;
        }
    }

    // Mengubah input pengguna ke dalam bentuk angka
    if let Ok(choice_index) = get_user_input("Pilih Variasi yang disediakan: ").trim().parse::<usize>() {
        // If opt.product is not present, proceed with user input logic
        if let Some(selected_model) = models.get(choice_index - 1) {
            println!("{:?}", selected_model);
            return Some(selected_model.clone());
        }
    }

    None
}