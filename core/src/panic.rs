use log::error;
use std::panic;

#[allow(deprecated)]
pub fn configure_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        error!("APPLICATION PANIC: {info}");
        default_hook(info);
    }));
}
