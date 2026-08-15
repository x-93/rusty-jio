use crate::model::header::RpcHeader;
use jio_consensus_core::header::Header;

pub fn rpc_header_to_consensus(rpc_header: RpcHeader) -> Header {
    rpc_header
}

pub fn consensus_header_to_rpc(header: Header) -> RpcHeader {
    header
}
