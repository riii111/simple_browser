use crate::cursor::Cursor;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use browser_core::constants::{CONTENT_AREA_HEIGHT, CONTENT_AREA_WIDTH, TITLE_BAR_HEIGHT};
use browser_core::display_item;
use browser_core::error::Error;
use browser_core::http::HttpResponse;
use browser_core::{
    browser::Browser,
    constants::{
        ADDRESSBAR_HEIGHT, BLACK, DARKGREY, GREY, LIGHTGREY, TOOLBAR_HEIGHT, WHITE, WINDOW_HEIGHT,
        WINDOW_INIT_X_POS, WINDOW_INIT_Y_POS, WINDOW_WIDTH,
    },
};
use core::cell::RefCell;
use noli::error::Result as OsResult;
use noli::prelude::SystemApi;
use noli::println;
use noli::rect::Rect;
use noli::sys::{api::MouseEvent, wasabi::Api};
use noli::window::{StringSize, Window};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputMode {
    Normal,  // 文字入力NG
    Editing, // 文字入力OK
}

#[derive(Debug)]
pub struct WasabiUI {
    browser: Rc<RefCell<Browser>>,
    input_url: String,
    window: Window,
    input_mode: InputMode,
    cursor: Cursor,
}

impl WasabiUI {
    pub fn new(browser: Rc<RefCell<Browser>>) -> Self {
        Self {
            browser,
            input_url: String::new(),
            input_mode: InputMode::Normal,
            window: Window::new(
                "saba".to_string(),
                WHITE,
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .unwrap(),
            cursor: Cursor::new(),
        }
    }

    fn setup_toolbar(&mut self) -> OsResult<()> {
        // ツールバーの背景の四角を描画
        self.window
            .fill_rect(LIGHTGREY, 0, 0, WINDOW_WIDTH, TOOLBAR_HEIGHT)?;

        // ツールバーのコンテンツエリアの境目の線を描画
        self.window.draw_line(
            DARKGREY,
            0,
            TOOLBAR_HEIGHT + 1,
            WINDOW_WIDTH - 1,
            TOOLBAR_HEIGHT + 1,
        )?;

        // アドレスバーの横に"Address:"という文字列を描画
        self.window
            .draw_string(BLACK, 5, 5, "Address:", StringSize::Medium, false)?;

        // アドレスバーの四角を描画
        self.window
            .fill_rect(WHITE, 70, 2, WINDOW_WIDTH - 74, 2 + ADDRESSBAR_HEIGHT)?;

        // アドレスバーの影の線を描画
        self.window.draw_line(GREY, 70, 2, WINDOW_WIDTH - 4, 2)?;
        self.window
            .draw_line(GREY, 70, 2, 70, 2 + ADDRESSBAR_HEIGHT)?;
        self.window.draw_line(BLACK, 71, 3, WINDOW_WIDTH - 5, 3)?;

        Ok(())
    }

    /// マウスの位置を制御する
    fn handle_mouse_input(&mut self) -> Result<(), Error> {
        if let Some(MouseEvent { button, position }) = Api::get_mouse_cursor_info() {
            // マウスカーソルを描画
            self.window.flush_area(self.cursor.rect());
            self.cursor.set_position(position.x, position.y);
            self.window.flush_area(self.cursor.rect());
            self.cursor.flush();

            if button.l() || button.c() || button.r() {
                // 相対位置を計算し、ツールバーの範囲内かどうかチェック
                let relative_pos = (
                    position.x - WINDOW_INIT_X_POS,
                    position.y - WINDOW_INIT_Y_POS,
                );

                // ウィンドウ外がクリックされた場合は何もしない
                if relative_pos.0 < 0
                    || relative_pos.0 > WINDOW_WIDTH
                    || relative_pos.1 < 0
                    || relative_pos.1 > WINDOW_HEIGHT
                {
                    println!("button clicked OUTSIDE window: {button:?} {position:?}");

                    return Ok(());
                }

                // アドレスバーの範囲内がクリックされた場合は、入力モードをEditingに変更
                if relative_pos.1 < TOOLBAR_HEIGHT + ADDRESSBAR_HEIGHT
                    && relative_pos.1 >= TITLE_BAR_HEIGHT
                {
                    self.clear_address_bar()?;
                    self.input_url = String::new();
                    self.input_mode = InputMode::Editing;
                    println!("button clicked in toolbar: {button:?} {position:?}");
                    return Ok(());
                }

                self.input_mode = InputMode::Normal;
            }
        }

        Ok(())
    }

    /// コンテンツエリア（HTMLが実際に描画される部分）をリセット
    fn clear_content_area(&mut self) -> Result<(), Error> {
        // コンテンツエリアを白く塗りつぶす
        if self
            .window
            .fill_rect(
                WHITE,
                0,
                TOOLBAR_HEIGHT + 2,
                CONTENT_AREA_WIDTH,
                CONTENT_AREA_HEIGHT - 2,
            )
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to clear a content area".to_string(),
            ));
        }

        self.window.flush();

        Ok(())
    }

    /// 入力されたURLに遷移開始
    fn start_navigation(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
        destination: String,
    ) -> Result<(), Error> {
        self.clear_content_area()?;

        match handle_url(destination) {
            Ok(response) => {
                let page = self.browser.borrow().current_page();
                page.borrow_mut().receive_response(response);
            }
            Err(e) => {
                return Err(e);
            }
        }

        self.update_ui()?;

        Ok(())
    }

    /// 文字を入力する
    fn handle_key_input(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        match self.input_mode {
            InputMode::Normal => {
                // InputModeがNormalのとき、キー入力を無視
                let _ = Api::read_key();
            }
            InputMode::Editing => {
                // InputModeがEditingのとき、キー入力を受け付ける
                if let Some(c) = Api::read_key() {
                    if c == 0x0A as char {
                        // Enterキーでナビゲーション開始
                        self.start_navigation(handle_url, self.input_url.clone())?;

                        self.input_url = String::new();
                        self.input_mode = InputMode::Normal;
                    } else if c == 0x7f as char || c == 0x08 as char {
                        // デリートキーorバックスペースキーが押されたので、最後の文字を削除
                        self.input_url.pop();
                        self.update_address_bar()?;
                    } else {
                        // それ以外のキーは入力文字を追加
                        self.input_url.push(c);
                        self.update_address_bar()?;
                    }
                }
            }
        }
        Ok(())
    }

    fn update_address_bar(&mut self) -> Result<(), Error> {
        // アドレスバーを白く塗りつぶす
        if self
            .window
            .fill_rect(WHITE, 72, 4, WINDOW_WIDTH - 76, ADDRESSBAR_HEIGHT - 2)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to clear an address bar".to_string(),
            ));
        }
        // input_urlをアドレスバーに描画
        if self
            .window
            .draw_string(BLACK, 74, 6, &self.input_url, StringSize::Medium, false)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to update an address bar".to_string(),
            ));
        }

        // アドレスバーの部分の画面を更新
        self.window.flush_area(
            Rect::new(
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS + TOOLBAR_HEIGHT,
                WINDOW_WIDTH,
                TOOLBAR_HEIGHT,
            )
            .expect("failed to flush an address bar"),
        );
        Ok(())
    }

    fn clear_address_bar(&mut self) -> Result<(), Error> {
        // アドレスバーを白く塗りつぶす
        if self
            .window
            .fill_rect(WHITE, 72, 4, WINDOW_WIDTH - 76, ADDRESSBAR_HEIGHT - 2)
            .is_err()
        {
            return Err(Error::InvalidUI(
                "failed to clear an address bar".to_string(),
            ));
        }

        // アドレスバー部分の画面を更新
        self.window.flush_area(
            Rect::new(
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS + TITLE_BAR_HEIGHT,
                WINDOW_WIDTH,
                TOOLBAR_HEIGHT,
            )
            .expect("failed to create a react for the address bar"),
        );
        Ok(())
    }

    fn update_ui(&mut self) -> Result<(), Error> {
        let display_items = self
            .browser
            .borrow()
            .current_page()
            .borrow()
            .display_items();
        for item in display_items {
            println!("{:?}", item);
        }
        self.window.flush();
        Ok(())
    }

    pub fn setup(&mut self) -> Result<(), Error> {
        if let Err(error) = self.setup_toolbar() {
            // OsResultとResultが持つError型は異なるため、変換が必要
            return Err(Error::InvalidUI(format!(
                "failed to initialize a toolbar with error: {:#?}",
                error
            )));
        }
        // 画面を更新する
        self.window.flush();

        Ok(())
    }

    pub fn run_app(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        loop {
            self.handle_mouse_input()?;
            self.handle_key_input(handle_url)?;
        }
    }

    pub fn start(
        &mut self,
        handle_url: fn(String) -> Result<HttpResponse, Error>,
    ) -> Result<(), Error> {
        self.setup()?;

        self.run_app(handle_url)?;

        Ok(())
    }
}
