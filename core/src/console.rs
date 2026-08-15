use std::io::{self, Write};

pub fn print_prompt(prompt: &str) {
    print!("{prompt}");
    let _ = io::stdout().flush();
}
