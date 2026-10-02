//! Terminal console formatting.

pub struct Console;

impl Console {
    pub fn print_info(msg: &str) {
        println!("[INFO] {}", msg);
    }

    pub fn print_warn(msg: &str) {
        eprintln!("[WARN] {}", msg);
    }

    pub fn print_error(msg: &str) {
        eprintln!("[ERROR] {}", msg);
    }
}
