pub static WINDOW_INIT_X_POS: i64 = 30;
pub static WINDOW_INIT_Y_POS: i64 = 50;

pub static WINDOW_WIDTH: i64 = 600;
pub static WINDOW_HEIGHT: i64 = 400;
pub static WINDOW_PADDING: i64 = 5;

// noliライブラリに定義されている定数
pub static TITLE_BAR_HEIGHT: i64 = 24;

pub static TOOLBAR_HEIGHT: i64 = 26;

pub static CONTENT_AREA_WIDTH: i64 = WINDOW_WIDTH - WINDOW_PADDING * 2;
pub static CONTENT_AREA_HEIGHT: i64 =
    WINDOW_HEIGHT - TITLE_BAR_HEIGHT - TOOLBAR_HEIGHT - WINDOW_PADDING * 2;

pub static CHAR_WIDTH: i64 = 8;
pub static CHAR_HEIGHT: i64 = 16;
pub static CHAR_HEIGHT_WITH_PADDING: i64 = CHAR_HEIGHT + 4;

pub static WHITE: u32 = 0xffffff;
pub static LIGHTGREY: u32 = 0xd3d3d3;
pub static GREY: u32 = 0x808080;
pub static DARKGREY: u32 = 0x5a5a5a;
pub static BLACK: u32 = 0x000000;

pub static ADDRESSBAR_HEIGHT: i64 = 20;
