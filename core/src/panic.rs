use std::panic;

/// Registers a global diagnostic panic hook.
pub fn init_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("<unnamed>");

        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<Any>"
        };

        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "<unknown>".to_string());

        log::error!(
            "CRITICAL PANIC on thread '{}' at {}: {}\n{:?}",
            thread_name,
            location,
            payload,
            std::backtrace::Backtrace::capture()
        );

        default_hook(panic_info);
    }));
}
