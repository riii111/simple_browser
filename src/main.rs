#![no_std]
#![no_main]

extern crate alloc;

use core::cell::RefCell;

use crate::alloc::string::ToString;
use alloc::rc::Rc;
use browser_core::browser::Browser;
use browser_core::http::HttpResponse;
use noli::*;
use ui_wasabi::app::WasabiUI;

fn main() -> u64 {
    // ブラウザ構造体を初期化
    let browser = Browser::new();

    // WasabiUI構造体を初期化
    let ui = Rc::new(RefCell::new(WasabiUI::new(browser)));

    // アプリの実行を開始
    match ui.borrow_mut().start() {
        Ok(_) => {}
        Err(e) => {
            println!("browser fails to start: {:#?}", e);
            return 1;
        }
    };

    0
}

entry_point!(main);
