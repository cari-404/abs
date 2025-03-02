//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod func_main;
mod login;
mod about;
mod manager;

use winsafe::{prelude::*, co, AnyResult, HWND};
use core::MyWindow;

#[tokio::main]
async fn main() {
	if let Err(e) = run_app() {
		HWND::NULL.MessageBox(
			&e.to_string(), "Uncaught error", co::MB::ICONERROR).unwrap();
	}
}

fn run_app() -> AnyResult<i32> {
	MyWindow::new() // create our main window...
		.run()       // ...and run it
		.map_err(|err| err.into())
}