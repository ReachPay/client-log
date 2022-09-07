use std::time::Duration;

use tonic::transport::Channel;

use crate::clientlog_grpc::{client_log_service_client::ClientLogServiceClient, *};

pub struct ClientLogGrpcService {
    timeout: Duration,
    channel: Channel,
}

impl ClientLogGrpcService {
    pub async fn new(grpc_address: String) -> Self {
        let channel = Channel::from_shared(grpc_address)
            .unwrap()
            .connect()
            .await
            .unwrap();
        Self {
            timeout: Duration::from_secs(10),
            channel,
        }
    }
    fn create_grpc_service(&self) -> ClientLogServiceClient<Channel> {
        ClientLogServiceClient::new(self.channel.clone())
    }

    pub async fn write_log(&self, item: Vec<ClientLogItem>) -> Result<(), String> {
        let mut client = self.create_grpc_service();

        let future = client.write(futures::stream::iter(item));

        let result = tokio::time::timeout(self.timeout, future).await;

        if result.is_err() {
            return Err(String::from("Timeout"));
        }

        let result = result.unwrap();

        if let Err(err) = result {
            return Err(err.to_string());
        }

        Ok(())
    }
}
