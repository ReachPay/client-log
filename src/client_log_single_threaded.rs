use service_sdk::{
    my_telemetry::MyTelemetryContext, rust_extensions::date_time::DateTimeAsMicroseconds,
};

use crate::clientlog_grpc::ClientLogItem;

use super::client_log_grpc_service::ClientLogGrpcService;

pub struct ClientLogSingleThreaded {
    pub items: Vec<(ClientLogItem, MyTelemetryContext)>,
    client_log_grpc_service: Option<ClientLogGrpcService>,
}

impl ClientLogSingleThreaded {
    pub fn new(client_log_grpc_service: ClientLogGrpcService) -> Self {
        Self {
            items: vec![],
            client_log_grpc_service: Some(client_log_grpc_service),
        }
    }

    pub fn add(
        &mut self,
        client_id: String,
        process_id: String,
        message: String,
        tech_data: String,
        my_telemetry_context: MyTelemetryContext,
    ) {
        let item = ClientLogItem {
            timestamp: DateTimeAsMicroseconds::now().unix_microseconds,
            client_id,
            process_id,
            message,
            tech_data: serde_json::to_string(&tech_data).unwrap(),
        };

        self.items.push((item, my_telemetry_context));
    }

    pub fn get(&mut self) -> Option<Vec<(ClientLogItem, MyTelemetryContext)>> {
        if self.items.len() == 0 {
            return None;
        }

        let to_publish = std::mem::replace(&mut self.items, vec![]);
        Some(to_publish)
    }

    pub fn get_client_log_grpc_service(&mut self) -> Option<ClientLogGrpcService> {
        self.client_log_grpc_service.take()
    }
}
