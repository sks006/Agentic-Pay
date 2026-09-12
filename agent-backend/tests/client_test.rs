use std::sync::Arc;
use agent_backend::client::{PaymentTerms, X402Client};
use agent_backend::wallet::AgentWallet;

#[tokio::test]
async fn test_client_voucher_creation_and_signing() {
    let wallet = Arc::new(
        AgentWallet::from_hex_secret(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap(),
    );

    let client = X402Client::new("http://127.0.0.1:8080", wallet.clone()).unwrap();

    let terms = PaymentTerms {
        price: 1000,
        network: "solana".into(),
        recipient: "11111111111111111111111111111111".into(),
        ttl: 60,
        scheme: "agenticpay_x402v1".into(),
    };

    // Verify signing succeeds and produces valid canonical length and nonces
    let voucher1 = client.sign_voucher_for_terms(&terms).unwrap();
    let voucher2 = client.sign_voucher_for_terms(&terms).unwrap();

    assert_eq!(voucher1.amount_lamports, 1000);
    assert_eq!(voucher1.nonce + 1, voucher2.nonce);
    assert_eq!(voucher1.agent, wallet.pubkey_bytes());
}

#[tokio::test]
#[ignore] // Requires the resource server running on localhost:8080
async fn test_fetch_price_with_x402() {
    let secret = std::env::var("AGENT_SECRET_KEY")
        .unwrap_or_else(|_| "0000000000000000000000000000000000000000000000000000000000000001".into());
    let wallet = Arc::new(AgentWallet::from_hex_secret(&secret).unwrap());
    let client = X402Client::new("http://127.0.0.1:8080", wallet).unwrap();
    let feed = client.fetch_price("BTC").await.unwrap();
    assert!(feed.price > 0);
}
