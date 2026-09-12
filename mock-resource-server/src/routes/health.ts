import { Router, Request, Response } from "express";
import { PublicKey } from "@solana/web3.js";
import { DEFAULT_PYTH_FEEDS } from "../pyth";

export function createHealthRouter(providerPubkey: PublicKey, pricePerRequestLamports: bigint): Router {
  const router = Router();

  router.get("/", (_req: Request, res: Response) => {
    res.json({
      status: "ok",
      service: "agenticpay-mock-resource-server",
      timestamp: Math.floor(Date.now() / 1000),
      provider: providerPubkey.toBase58(),
      pricePerRequestLamports: pricePerRequestLamports.toString(),
      supportedSymbols: Object.keys(DEFAULT_PYTH_FEEDS),
      supportedFeeds: DEFAULT_PYTH_FEEDS,
    });
  });

  return router;
}
