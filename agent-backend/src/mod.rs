mod config;
mod error;
mod solana_rpc;     // <-- changed from solana_client
mod state;
mod rpc;

use config::Config;
use solana_rpc::SolanaRpcAdapter;
use state::ServerState;
use rpc::{RpcServer, AgentRpcServer};
use std::sync::Arc;
use std::net::SocketAddr;
use jsonrpsee::server::ServerBuilder;