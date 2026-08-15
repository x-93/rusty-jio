pub struct CliNotifier;

impl CliNotifier {
    pub fn notify(msg: &str) {
        println!("[NOTIFICATION] {}", msg);
    }
}
