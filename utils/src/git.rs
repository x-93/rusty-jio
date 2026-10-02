//! Git repository metadata query helpers.

pub fn git_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
