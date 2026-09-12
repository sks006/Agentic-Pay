import { Router, Request, Response } from "express";
import { PythService } from "../pyth";
import { bufferToPubkeyString } from "../voucher";

export function createPriceRouter(pythService: PythService): Router {
  const router = Router();

  router.get("/:symbol", async (req: Request, res: Response) => {
    const symbol = req.params.symbol;
    if (!symbol) {
      res.status(400).json({ error: "Missing required :symbol parameter" });
      return;
    }

    try {
      const priceData = await pythService.getPrice(symbol);
      const scalingFactor = Math.pow(10, -priceData.expo);
      const scaledPrice = Math.round(priceData.price * scalingFactor);
      const scaledConf = Math.round(priceData.conf * scalingFactor);

      const responsePayload: Record<string, any> = {
        symbol: priceData.symbol,
        price: scaledPrice,
        confidence: scaledConf,
        exponent: priceData.expo,
        publish_time: priceData.publishTime,
        formatted_price: priceData.formattedPrice,
        float_price: priceData.price,
        source: priceData.source,
        feed_id: priceData.feedId,
      };

      if (req.voucher) {
        responsePayload.payment = {
          status: "verified",
          agent: req.agentPubkey,
          provider: bufferToPubkeyString(req.voucher.provider),
          nonce: req.voucher.nonce.toString(),
          amount_lamports: req.voucher.amountLamports.toString(),
          expires_at: req.voucher.expiresAt.toString(),
        };
      }

      res.json(responsePayload);
    } catch (err) {
      res.status(404).json({
        error: "Price lookup failed",
        message: (err as Error).message,
      });
    }
  });

  return router;
}
