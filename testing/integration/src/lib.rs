pub mod consensus;
pub mod pipeline;
pub mod rpc;

pub use consensus::*;
pub use pipeline::*;
pub use rpc::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_fixture() {
        let fixture = TestConsensusFixture::new();
        let session = fixture.consensus_manager.session();
        let vs = session.get_virtual_state().expect("virtual state exists");
        assert!(!vs.parents.is_empty());
        let tip = session.get_selected_chain_tip();
        assert!(tip.is_some());
    }
}
