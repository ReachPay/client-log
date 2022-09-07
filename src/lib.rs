mod client_log;
mod client_log_grpc_service;
mod client_log_single_threaded;

pub mod clientlog_grpc {
    tonic::include_proto!("clientlog");
}

pub use client_log::ClientLog;
