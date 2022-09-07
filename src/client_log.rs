use std::sync::Arc;

use serde::Serialize;
use tokio::sync::Mutex;

use crate::clientlog_grpc::ClientLogItem;

use super::{
    client_log_grpc_service::ClientLogGrpcService,
    client_log_single_threaded::ClientLogSingleThreaded,
};

pub struct ClientLog {
    client_log_data: Arc<Mutex<ClientLogSingleThreaded>>,
}

impl ClientLog {
    pub async fn new(grpc_url: String) -> Self {
        let client_log_grpc_service = ClientLogGrpcService::new(grpc_url).await;
        Self {
            client_log_data: Arc::new(Mutex::new(ClientLogSingleThreaded::new(
                client_log_grpc_service,
            ))),
        }
    }

    pub async fn write<T: Serialize>(
        &self,
        client_id: String,
        process_id: String,
        message: String,
        tech_data: T,
    ) {
        let mut write_access = self.client_log_data.lock().await;

        write_access.add(
            client_id,
            process_id,
            message,
            serde_json::to_string(&tech_data).unwrap(),
        );

        let client_log_grpc_service = write_access.get_client_log_grpc_service();

        if let Some(client_log_grpc_service) = client_log_grpc_service {
            tokio::spawn(reading_thread(
                self.client_log_data.clone(),
                client_log_grpc_service,
            ));
        }
    }
}

async fn reading_thread(
    client_log_data: Arc<Mutex<ClientLogSingleThreaded>>,
    client_log_grpc_service: ClientLogGrpcService,
) {
    loop {
        let items = {
            let mut write_access = client_log_data.lock().await;
            write_access.get()
        };

        match items {
            Some(items) => {
                write_to_grpc(&client_log_grpc_service, items).await;
            }
            None => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn write_to_grpc(client_log_grpc_service: &ClientLogGrpcService, items: Vec<ClientLogItem>) {
    loop {
        let result = client_log_grpc_service.write_log(items.clone()).await;

        match result {
            Ok(_) => {
                return;
            }
            Err(err) => my_logger::LOGGER.write_log(
                my_logger::LogLevel::Error,
                "WritingClientLog".to_string(),
                format!("Err:{}", err),
                None,
            ),
        }
    }
}
