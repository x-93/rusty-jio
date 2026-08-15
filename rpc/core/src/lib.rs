pub mod api;
pub mod convert;
pub mod model;
pub mod notify;

pub use api::*;
pub use model::*;
pub use notify::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_ops_enum() {
        let op = RpcApiOps::SubmitBlock;
        assert_eq!(op, RpcApiOps::SubmitBlock);
    }
}
