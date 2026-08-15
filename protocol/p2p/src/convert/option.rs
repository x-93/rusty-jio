pub fn option_to_vec<T>(opt: Option<T>) -> Vec<T> {
    opt.into_iter().collect()
}

pub fn vec_to_option<T>(mut v: Vec<T>) -> Option<T> {
    if v.is_empty() {
        None
    } else {
        Some(v.remove(0))
    }
}
