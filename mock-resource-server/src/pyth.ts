import { HermesClient } from "@pythnetwork/hermes-client";
import { LRUCache } from "lru-cache";

export const DEFAULT_PYTH_FEEDS: Record<string, string> = {
  BTC: "0xe62df6e110f78005f12b4fce2f688b04f32d5ffb5f5da89453feed78dd595a4d",
  ETH: "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
  SOL: "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
};

export interface PythPriceData {
  symbol: string;
  feedId: string;
  price: number;
  conf: number;
  expo: number;
  publishTime: number;
  formattedPrice: string;
  source: "hermes-live" | "cache" | "fallback-mock";
}

export class PythService {
  private client: HermesClient;
  private cache: LRUCache<string, PythPriceData>;
  private hermesUrl: string;

  constructor(
    hermesUrl: string = process.env.PYTH_HERMES_URL || "https://hermes.pyth.network",
    cacheTtlMs: number = 5000
  ) {
    this.hermesUrl = hermesUrl;
    this.client = new HermesClient(this.hermesUrl, {});
    this.cache = new LRUCache<string, PythPriceData>({
      max: 100,
      ttl: cacheTtlMs,
    });
  }

  /**
   * Resolve symbol string to Pyth Feed ID.
   */
  public resolveFeedId(symbol: string): string {
    const normalized = symbol.toUpperCase().replace("/USD", "").replace("USD", "");
    if (DEFAULT_PYTH_FEEDS[normalized]) {
      return DEFAULT_PYTH_FEEDS[normalized];
    }
    if (symbol.startsWith("0x") && symbol.length === 66) {
      return symbol;
    }
    throw new Error(`Unsupported symbol or invalid feed ID: ${symbol}`);
  }

  /**
   * Fetch price for a symbol or feed ID, using cache if available.
   */
  public async getPrice(symbol: string): Promise<PythPriceData> {
    const normalizedSymbol = symbol.toUpperCase().replace("/USD", "").replace("USD", "");
    const feedId = this.resolveFeedId(symbol);

    const cached = this.cache.get(feedId);
    if (cached) {
      return { ...cached, source: "cache" };
    }

    try {
      const response = await this.client.getLatestPriceUpdates([feedId]);
      if (response && response.parsed && response.parsed.length > 0) {
        const update = response.parsed[0];
        const rawPrice = BigInt(update.price.price);
        const expo = update.price.expo;
        const rawConf = BigInt(update.price.conf);
        const publishTime = update.price.publish_time;

        const numericPrice = Number(rawPrice) * Math.pow(10, expo);
        const numericConf = Number(rawConf) * Math.pow(10, expo);

        const data: PythPriceData = {
          symbol: normalizedSymbol,
          feedId,
          price: numericPrice,
          conf: numericConf,
          expo,
          publishTime,
          formattedPrice: numericPrice.toFixed(Math.abs(expo) > 4 ? 4 : Math.abs(expo)),
          source: "hermes-live",
        };

        this.cache.set(feedId, data);
        return data;
      }
    } catch (err) {
      console.warn(`[PythService] Hermes lookup failed for ${symbol}:`, (err as Error).message);
    }

    // Fallback simulated price for offline development / test environments
    const fallback = this.getFallbackPrice(normalizedSymbol, feedId);
    this.cache.set(feedId, fallback);
    return fallback;
  }

  private getFallbackPrice(symbol: string, feedId: string): PythPriceData {
    let basePrice = 64000;
    if (symbol === "ETH") basePrice = 3400;
    if (symbol === "SOL") basePrice = 145;

    // Small deterministic variation based on current minute
    const variation = (Date.now() % 60000) / 1000;
    const price = basePrice + variation * 0.1;
    const expo = -8;
    const conf = price * 0.0005;

    return {
      symbol,
      feedId,
      price,
      conf,
      expo,
      publishTime: Math.floor(Date.now() / 1000),
      formattedPrice: price.toFixed(2),
      source: "fallback-mock",
    };
  }
}
