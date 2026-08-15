use std::env;

pub fn is_test_environment() -> bool {
    env::var("JIOPAD_TEST").is_ok() || cfg!(test)
}

pub fn get_data_dir() -> String {
    env::var("JIOPAD_DATA_DIR").unwrap_or_else(|_| "./data".to_string())
}
