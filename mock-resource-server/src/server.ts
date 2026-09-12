import express, { Request, Response, NextFunction } from "express";
import cors from "cors";
import dotenv from "dotenv";
import { PythService } from "./pyth";
import {
  createPaywallMiddleware,
  getOrCreateProviderKeypair,
  PaywallConfig,
} from "./paywall";
import { createHealthRouter } from "./routes/health";
import { createPriceRouter } from "./routes/price";
import { createWebhookRouter } from "./webhook";

dotenv.config();

export function createApp(customConfig?: Partial<PaywallConfig>) {
  const app = express();

  const providerKeypair = customConfig?.providerKeypair ?? getOrCreateProviderKeypair();
  const pricePerRequestLamports =
    customConfig?.pricePerRequestLamports ??
    BigInt(process.env.PRICE_PER_REQUEST_LAMPORTS || "1000");
  const voucherTtlSecs = customConfig?.voucherTtlSecs ?? 60;

  const paywallConfig: PaywallConfig = {
    providerKeypair,
    pricePerRequestLamports,
    voucherTtlSecs,
  };

  const pythService = new PythService(process.env.PYTH_HERMES_URL);
  const paywallMiddleware = createPaywallMiddleware(paywallConfig);

  // Standard middlewares
  app.use(cors());
  app.use(express.json());

  // Request logger
  app.use((req: Request, _res: Response, next: NextFunction) => {
    const timestamp = new Date().toISOString();
    console.log(`[${timestamp}] ${req.method} ${req.originalUrl}`);
    next();
  });

  // Public un-gated routes
  app.use(
    "/health",
    createHealthRouter(providerKeypair.publicKey, pricePerRequestLamports)
  );
  app.use("/webhook", createWebhookRouter());

  // x402-gated routes
  app.use("/price", paywallMiddleware, createPriceRouter(pythService));

  // Catch-all 404 handler
  app.use((_req: Request, res: Response) => {
    res.status(404).json({ error: "Endpoint not found" });
  });

  // Global error handler
  app.use((err: Error, _req: Request, res: Response, _next: NextFunction) => {
    console.error("[ServerError]", err);
    res.status(500).json({ error: "Internal Server Error", message: err.message });
  });

  return {
    app,
    paywallConfig,
    pythService,
  };
}

// Auto-start server when executed directly
if (process.env.NODE_ENV !== "test") {
  const port = parseInt(process.env.PORT || "8080", 10);
  const { app, paywallConfig } = createApp();

  app.listen(port, () => {
    console.log("==================================================");
    console.log(`🚀 Mock Resource Server running on port ${port}`);
    console.log(`🔑 Provider Public Key: ${paywallConfig.providerKeypair.publicKey.toBase58()}`);
    console.log(`💰 Price per Request: ${paywallConfig.pricePerRequestLamports} lamports`);
    console.log(`🌐 Health check: http://127.0.0.1:${port}/health`);
    console.log(`🔒 Gated endpoint: http://127.0.0.1:${port}/price/BTC`);
    console.log("==================================================");
  });
}
