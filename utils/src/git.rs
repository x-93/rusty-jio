pub const GIT_COMMIT_HASH: Option<&'static str> = option_env!("GIT_COMMIT_HASH");
pub const GIT_BRANCH: Option<&'static str> = option_env!("GIT_BRANCH");

pub fn version_info() -> String {
    let pkg_version = env!("CARGO_PKG_VERSION");
    match (GIT_COMMIT_HASH, GIT_BRANCH) {
        (Some(commit), Some(branch)) => format!("{pkg_version}-{branch}+{commit}"),
        (Some(commit), None) => format!("{pkg_version}+{commit}"),
        _ => pkg_version.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let v = version_info();
        assert!(!v.is_empty());
    }
}
