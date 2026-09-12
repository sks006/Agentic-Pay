export const DOMAIN_SEPARATOR = "agenticpay_x402v1"
export const CANONICAL_MSG_LENGTH = 105
export const PAYMENT_SIGNATURE_WIRE_LENGTH = 170
export const GUARDRAILS_PROGRAM_ID = "Grd1111111111111111111111111111111111111111"

export const DEFAULT_RPC_ENDPOINT = "http://127.0.0.1:8545"
export const DEFAULT_PYTH_ENDPOINT = "http://127.0.0.1:8080"

export const PYTH_FEED_SYMBOLS = ["SOL", "BTC", "ETH"] as const
export type PythSymbol = (typeof PYTH_FEED_SYMBOLS)[number]

export const ASSET_METADATA: Record<PythSymbol, { name: string; icon: string; color: string }> = {
  SOL: {
    name: "Solana",
    icon: "◎",
    color: "#10b981",
  },
  BTC: {
    name: "Bitcoin",
    icon: "₿",
    color: "#f59e0b",
  },
  ETH: {
    name: "Ethereum",
    icon: "Ξ",
    color: "#8b5cf6",
  },
}
