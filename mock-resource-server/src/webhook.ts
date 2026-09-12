import { Router, Request, Response } from "express";

export interface SettlementNotification {
  signature: string;
  slot?: number;
  voucherCount?: number;
  totalLamports?: string;
  status: "confirmed" | "finalized";
}

/**
 * Webhook router for receiving settlement notices and Solana Pay transaction confirmations.
 */
export function createWebhookRouter(): Router {
  const router = Router();

  router.post("/settlement", (req: Request, res: Response) => {
    const { signature, status } = req.body as SettlementNotification;

    if (!signature) {
      res.status(400).json({ error: "Missing signature in settlement notification" });
      return;
    }

    console.log(`[Webhook] Received on-chain settlement notification: ${signature} (status: ${status || "confirmed"})`);

    res.json({
      received: true,
      signature,
      acknowledged_at: Math.floor(Date.now() / 1000),
    });
  });

  return router;
}
