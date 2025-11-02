use async_trait::async_trait;
use solana_client::{
    client_error::ClientError, nonblocking::rpc_client::RpcClient,
};
use tracing::debug;


#[async_trait]
#[mockall::automock] 
pub trait SolanaRpc: Send + Sync {
    async fn get_slot(&self) -> Result<u64, ClientError>;
    async fn get_blocks(&self, start_slot: u64, end_slot: u64) -> Result<Vec<u64>, ClientError>;
}


pub struct RpcClientWrapper {
    client: RpcClient,
}


impl RpcClientWrapper {
    pub fn new(url: String) -> Self {
        Self {
            client: RpcClient::new(url),
        }
    }
}


#[async_trait]
impl SolanaRpc for RpcClientWrapper {
    async fn get_slot(&self) -> Result<u64, ClientError> {
        debug!("RPC CALL: getSlot");
        self.client.get_slot().await
    }

    async fn get_blocks(&self, start_slot: u64, end_slot: u64) -> Result<Vec<u64>, ClientError> {
        debug!("RPC CALL: getBlocks ({}..={})", start_slot, end_slot);
        self.client.get_blocks(start_slot, Some(end_slot)).await
    }
}