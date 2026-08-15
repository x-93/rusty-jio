use jio_consensus_core::tx::Transaction;

pub trait TransactionExtension {
    fn summary(&self) -> String;
}

impl TransactionExtension for Transaction {
    fn summary(&self) -> String {
        format!(
            "Inputs: {}, Outputs: {}, Subnetwork: {:?}",
            self.inputs.len(),
            self.outputs.len(),
            self.subnetwork_id
        )
    }
}
