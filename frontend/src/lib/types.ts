export type AgentStatus = "running" | "stopped" | "error" | "connecting"

export type DecisionAction = "BUY" | "SELL" | "PAY" | "HOLD"

export type DecisionStatus = "pending" | "approved" | "rejected" | "executed"

export interface AgentInfo {
  id: string
  name: string
  status: AgentStatus
  uptimeSeconds: number
  walletAddress: string
  inFlightCount: number
  maxInFlight: number
  rpcUrl: string
  programId: string
  escrowPda: string
  dailyCapLamports: number
  spentTodayLamports: number
  isPaused: boolean
}

export interface WalletBalance {
  currency: string
  amount: number
  usdValue: number
  symbol: string
}

export interface Voucher {
  id: string
  nonce: number
  issuer: string
  recipient: string
  amountLamports: number
  amountSol: number
  amountUsd: number
  expiresAt: string
  expiresAtTimestamp: number
  status: "active" | "redeemed" | "expired" | "settling"
  signature: string
  canonicalBytesHex?: string
}

export interface PythPrice {
  symbol: string
  price: number
  confidence: number
  confidenceBps: number
  publishTime: number
  emaPrice?: number
  exponent: number
  status?: string
}

export interface Decision {
  id: string
  action: DecisionAction
  asset: string
  reason: string
  amount?: number
  price?: number
  status: DecisionStatus
  timestamp: string
  edgeBps?: number
  feeBps?: number
  confidenceBps?: number
  oracleLatencyMs?: number
}

export interface Payment {
  id: string
  recipient: string
  amount: number
  currency: string
  amountUsd: number
  status: "pending" | "confirmed" | "failed"
  timestamp: string
  signature?: string
  batchIndex?: number
}

export interface AgentEvent {
  id: string
  type: "price_update" | "decision" | "payment" | "transaction" | "system" | "voucher"
  message: string
  timestamp: string
  data?: Record<string, any>
}

export interface AgentSnapshot {
  agent: AgentInfo
  balances: WalletBalance[]
  vouchers: Voucher[]
  decisions: Decision[]
  payments: Payment[]
  events: AgentEvent[]
}
