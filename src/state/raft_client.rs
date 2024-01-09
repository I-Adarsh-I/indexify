use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::indexify_raft::raft_api_client::RaftApiClient;

pub struct RaftClient {
    clients: Arc<Mutex<HashMap<String, RaftApiClient<Channel>>>>,
}

impl RaftClient {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get(&self, addr: &str) -> Result<RaftApiClient<Channel>> {
        let mut clients = self.clients.lock().await;
        if let Some(client) = clients.get(addr) {
            return Ok(client.clone());
        }

        let client = RaftApiClient::connect(format!("http://{}", addr))
            .await
            .map_err(|e| anyhow!("unable to connect to raft: {}", e))?;
        clients.insert(addr.to_string(), client.clone());
        Ok(client)
    }
}
