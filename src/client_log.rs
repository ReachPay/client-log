use std::sync::Arc;

use serde::Serialize;
use service_sdk::{my_grpc_extensions::GrpcClientSettings, my_telemetry::MyTelemetryContext};
use tokio::sync::Mutex;

use super::{
    client_log_grpc_service::ClientLogGrpcService,
    client_log_single_threaded::ClientLogSingleThreaded,
};

pub struct ClientLog {
    client_log_data: Arc<Mutex<ClientLogSingleThreaded>>,
}

impl ClientLog {
    pub fn new(
        get_grpc_address: std::sync::Arc<dyn GrpcClientSettings + Send + Sync + 'static>,
    ) -> Self {
        let client_log_grpc_service = ClientLogGrpcService::new(get_grpc_address);
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
        my_telemetry: MyTelemetryContext,
    ) {
        let mut write_access = self.client_log_data.lock().await;

        write_access.add(
            client_id,
            process_id,
            message,
            serde_json::to_string(&tech_data).unwrap(),
            my_telemetry,
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
                let mut ctx = None;

                let mut to_publish = Vec::with_capacity(items.len());

                for (event, event_ctx) in items {
                    to_publish.push(event);
                    match &mut ctx {
                        None => {
                            ctx = Some(event_ctx);
                        }
                        Some(ctx) => ctx.merge_process(&event_ctx),
                    }
                }

                let _ = client_log_grpc_service
                    .write(to_publish, ctx.as_ref().unwrap())
                    .await;
            }
            None => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}
