#![no_std]
#![no_main]

extern crate alloc;

use crate::alloc::string::ToString;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use browser_core::browser::Browser;
use browser_core::error::Error;
use browser_core::http::HttpResponse;
use browser_core::url::Url;
use core::cell::RefCell;
use net_wasabi::http::HttpClient;
use noli::*;
use ui_wasabi::app::WasabiUI;

/// ネットワークコンポーネントとUIコンポーネントをつなぐロジック
/// 異なるプラットフォームにも移植しやすくするため、コンポーネント間の依存性を最小限に抑える
fn handle_url(url: String) -> Result<HttpResponse, Error> {
    // URLパース
    let parsed_url = match Url::new(url.to_string()).parse() {
        Ok(url) => url,
        Err(e) => {
            return Err(Error::UnexpectedInput(format!(
                "input html is not supported: {:#?}",
                e
            )))
        }
    };

    let client = HttpClient::new();
    let response = match client.get(
        parsed_url.host(),
        parsed_url.port().parse::<u16>().expect(&format!(
            "port number should be u16 but got {}",
            parsed_url.port()
        )),
        parsed_url.path(),
    ) {
        Ok(res) => {
            // HTTPレスポンスのステータスコードが302(リダイレクト)のとき、転送する
            if res.status_code() == 302 {
                let location = match res.header_value("Location") {
                    Ok(value) => value,
                    Err(_) => return Ok(res),
                };
                let redirect_parsed_url = Url::new(location);

                let redirect_res = match client.get(
                    redirect_parsed_url.host(),
                    redirect_parsed_url.port().parse::<u16>().expect(&format!(
                        "port number should be u16 but got {}",
                        redirect_parsed_url.port()
                    )),
                    redirect_parsed_url.path(),
                ) {
                    Ok(res) => res,
                    Err(e) => return Err(Error::Network(format!("{:?}", e))),
                };

                redirect_res
            } else {
                res
            }
        }
        Err(e) => {
            return Err(Error::Network(format!(
                "failed to get http response: {:?}",
                e
            )))
        }
    };
    Ok(response)
}

fn main() -> u64 {
    // ブラウザ構造体を初期化
    let browser = Browser::new();

    // WasabiUI構造体を初期化
    let ui = Rc::new(RefCell::new(WasabiUI::new(browser)));

    // アプリの実行を開始
    match ui.borrow_mut().start(handle_url) {
        Ok(_) => {}
        Err(e) => {
            println!("browser fails to start: {:#?}", e);
            return 1;
        }
    };

    0
}

entry_point!(main);
