//! Global panic hook.

pub fn init_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("\n==========================================");
        eprintln!("FATAL: Unexpected panic in Jio node!");
        if let Some(location) = info.location() {
            eprintln!("Location: {}:{}:{}", location.file(), location.line(), location.column());
        }
        if let Some(msg) = info.payload().downcast_ref::<&str>() {
            eprintln!("Message: {}", msg);
        } else if let Some(msg) = info.payload().downcast_ref::<String>() {
            eprintln!("Message: {}", msg);
        }
        eprintln!("==========================================\n");
        default_hook(info);
    }));
}
