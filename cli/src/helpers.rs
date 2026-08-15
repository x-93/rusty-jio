pub struct CliHelpers;

impl CliHelpers {
    pub fn format_sompi_to_jio(sompi: u64) -> String {
        format!("{:.8} JIO", sompi as f64 / 100_000_000.0)
    }
}
