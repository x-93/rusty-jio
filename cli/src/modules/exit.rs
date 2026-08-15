use crate::result::CliResult;

pub async fn execute() -> CliResult<()> {
    std::process::exit(0);
}
