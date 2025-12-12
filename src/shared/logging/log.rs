use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;

lazy_static! {
    static ref PRINT_INFO: AtomicBool = AtomicBool::new(true);
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BlackBold,
    RedBold,
    GreenBold,
    YellowBold,
    BlueBold,
    MagentaBold,
    CyanBold,
    WhiteBold,
    BlackUnderlined,
    RedUnderlined,
    GreenUnderlined,
    YellowUnderlined,
    BlueUnderlined,
    MagentaUnderlined,
    CyanUnderlined,
    WhiteUnderlined,
    BlackBackground,
    RedBackground,
    GreenBackground,
    YellowBackground,
    BlueBackground,
    MagentaBackground,
    CyanBackground,
    WhiteBackground,
    BlackBright,
    RedBright,
    GreenBright,
    YellowBright,
    BlueBright,
    MagentaBright,
    CyanBright,
    WhiteBright,
    BlackBoldBright,
    RedBoldBright,
    GreenBoldBright,
    YellowBoldBright,
    BlueBoldBright,
    MagentaBoldBright,
    CyanBoldBright,
    WhiteBoldBright,
    BlackBackgroundBright,
    RedBackgroundBright,
    GreenBackgroundBright,
    YellowBackgroundBright,
    BlueBackgroundBright,
    MagentaBackgroundBright,
    CyanBackgroundBright,
    WhiteBackgroundBright,
}

impl Color {
    pub fn code(&self) -> &'static str {
        match self {
            Color::Reset => "\x1b[0m",
            Color::Black => "\x1b[0;30m",
            Color::Red => "\x1b[0;31m",
            Color::Green => "\x1b[0;32m",
            Color::Yellow => "\x1b[0;33m",
            Color::Blue => "\x1b[0;34m",
            Color::Magenta => "\x1b[0;35m",
            Color::Cyan => "\x1b[0;36m",
            Color::White => "\x1b[0;37m",
            Color::BlackBold => "\x1b[1;30m",
            Color::RedBold => "\x1b[1;31m",
            Color::GreenBold => "\x1b[1;32m",
            Color::YellowBold => "\x1b[1;33m",
            Color::BlueBold => "\x1b[1;34m",
            Color::MagentaBold => "\x1b[1;35m",
            Color::CyanBold => "\x1b[1;36m",
            Color::WhiteBold => "\x1b[1;37m",
            Color::BlackUnderlined => "\x1b[4;30m",
            Color::RedUnderlined => "\x1b[4;31m",
            Color::GreenUnderlined => "\x1b[4;32m",
            Color::YellowUnderlined => "\x1b[4;33m",
            Color::BlueUnderlined => "\x1b[4;34m",
            Color::MagentaUnderlined => "\x1b[4;35m",
            Color::CyanUnderlined => "\x1b[4;36m",
            Color::WhiteUnderlined => "\x1b[4;37m",
            Color::BlackBackground => "\x1b[40m",
            Color::RedBackground => "\x1b[41m",
            Color::GreenBackground => "\x1b[42m",
            Color::YellowBackground => "\x1b[43m",
            Color::BlueBackground => "\x1b[44m",
            Color::MagentaBackground => "\x1b[45m",
            Color::CyanBackground => "\x1b[46m",
            Color::WhiteBackground => "\x1b[47m",
            Color::BlackBright => "\x1b[0;90m",
            Color::RedBright => "\x1b[0;91m",
            Color::GreenBright => "\x1b[0;92m",
            Color::YellowBright => "\x1b[0;93m",
            Color::BlueBright => "\x1b[0;94m",
            Color::MagentaBright => "\x1b[0;95m",
            Color::CyanBright => "\x1b[0;96m",
            Color::WhiteBright => "\x1b[0;97m",
            Color::BlackBoldBright => "\x1b[1;90m",
            Color::RedBoldBright => "\x1b[1;91m",
            Color::GreenBoldBright => "\x1b[1;92m",
            Color::YellowBoldBright => "\x1b[1;93m",
            Color::BlueBoldBright => "\x1b[1;94m",
            Color::MagentaBoldBright => "\x1b[1;95m",
            Color::CyanBoldBright => "\x1b[1;96m",
            Color::WhiteBoldBright => "\x1b[1;97m",
            Color::BlackBackgroundBright => "\x1b[0;100m",
            Color::RedBackgroundBright => "\x1b[0;101m",
            Color::GreenBackgroundBright => "\x1b[0;102m",
            Color::YellowBackgroundBright => "\x1b[0;103m",
            Color::BlueBackgroundBright => "\x1b[0;104m",
            Color::MagentaBackgroundBright => "\x1b[0;105m",
            Color::CyanBackgroundBright => "\x1b[0;106m",
            Color::WhiteBackgroundBright => "\x1b[0;107m",
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

pub struct TimePrinter {
    start_time: DateTime<Utc>,
    message: String,
}

impl TimePrinter {
    pub fn new() -> Self {
        Self {
            start_time: Utc::now(),
            message: String::new(),
        }
    }

    pub fn with_message(message: &str) -> Self {
        info2(message);
        Self {
            start_time: Utc::now(),
            message: message.to_string(),
        }
    }


    fn print_internal(&self, color: Color, custom_message: Option<&str>) {
        if PRINT_INFO.load(Ordering::Relaxed) {
            let elapsed = Utc::now().signed_duration_since(self.start_time);
            let elapsed_millis = elapsed.num_milliseconds();

            let message = match custom_message {
                Some(msg) => msg,
                None => &self.message,
            };

            println!(
                "{}{} {} millis: {}{}",
                color,
                Utc::now(),
                message,
                elapsed_millis,
                Color::Reset
            );
        }
    }

    pub fn print(&self) {
        self.print_internal(Color::WhiteBold, None);
    }

    pub fn log(&self) {
        self.print_internal(Color::GreenBold, None);
    }

    pub fn info(&self) {
        self.print_internal(Color::YellowBold, None);
    }

    pub fn warning(&self) {
        self.print_internal(Color::MagentaBold, None);
    }

    pub fn error(&self) {
        self.print_internal(Color::RedBold, None);
    }

    pub fn print_with_message(&self, message: &str) {
        self.print_internal(Color::WhiteBold, Some(&*(self.message.clone() + " " + message)));
    }

    pub fn log_with_message(&self, message: &str) {
        self.print_internal(Color::GreenBold, Some(&*(self.message.clone() + " " + message)));
    }

    pub fn info_with_message(&self, message: &str) {
        self.print_internal(Color::YellowBold, Some(&*(self.message.clone() + " " + message)));
    }

    pub fn warning_with_message(&self, message: &str) {
        self.print_internal(Color::MagentaBold, Some(&*(self.message.clone() + " " + message)));
    }

    pub fn error_with_message(&self, message: &str) {
        self.print_internal(Color::RedBold, Some(&*(self.message.clone() + " " + message)));
    }
}

// Static logging functions
pub fn format_print(color: Color, message: &str) {
    if PRINT_INFO.load(Ordering::Relaxed) {
        println!("{}{} {}{}", color, Utc::now(), message, Color::Reset);
    }
}


pub fn info(message: &str) {
    format_print(Color::WhiteBold, message);
}

pub fn info2(message: &str) {
    format_print(Color::YellowBold, message);
}


pub fn warning(message: &str) {
    format_print(Color::MagentaBold, message);
}


pub fn error(message: &str) {
    format_print(Color::RedBold, message);
}

pub fn success(message: &str) {
    format_print(Color::GreenBold, message);
}


pub fn debug(message: &str) {
    format_print(Color::CyanBold, message);
}

// Configuration function to enable/disable logging
pub fn set_print_info(enabled: bool) {
    PRINT_INFO.store(enabled, Ordering::Relaxed);
}

// Initialize logging based on configuration
pub fn init_from_config(is_prod: bool) {
    set_print_info(!is_prod);
}
