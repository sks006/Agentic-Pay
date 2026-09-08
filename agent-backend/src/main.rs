use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use jsonrpsee::server::ServerBuilder;
use solana_client::nonblocking::rpc_client::RpcClient;

use agent_backend::config::Config;
use agent_backend::rpc::{AgentRpcServer, RpcServer};
use agent_backend::solana_rpc::AsyncSolanaProvider;
use agent_backend::state::{self, ServerState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cfg = Config::from_env()?;
    let solana_client = Arc::new(RpcClient::new(cfg.rpc_url.clone()));
    let rpc_client = Arc::new(AsyncSolanaProvider::new(
        solana_client,
        Duration::from_secs(cfg.nonce_ttl_secs),
    ));

    let engine_config = state::EngineConfig {
        max_in_flight_per_agent: cfg.max_in_flight_per_agent,
        nonce_ttl: Duration::from_secs(cfg.nonce_ttl_secs),
        max_rate_limit_units: cfg.max_rate_limit_units,
        rate_limit_per_sec: cfg.rate_limit_per_sec,
    };
    let state = Arc::new(ServerState::new(Some(engine_config)));
    state.clone().spawn_cleanup_task(Duration::from_secs(10));

    let rpc_impl = RpcServer::new(state, rpc_client);
    let rpc_service = AgentRpcServer::into_rpc(rpc_impl);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8545));
    let server = ServerBuilder::new().build(addr).await?;
    let handle = server.start(rpc_service);
    println!("JSON-RPC server started at http://{}", addr);
    handle.stopped().await;
    Ok(())
}
