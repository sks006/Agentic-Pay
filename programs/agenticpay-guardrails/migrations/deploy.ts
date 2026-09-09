// Migrations are an early feature. Currently, they're just an Anchor script.
import * as anchor from "@coral-xyz/anchor";

module.exports = async function (provider: anchor.AnchorProvider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  console.log("Deploying agenticpay-guardrails program to cluster:", provider.connection.rpcEndpoint);
};
