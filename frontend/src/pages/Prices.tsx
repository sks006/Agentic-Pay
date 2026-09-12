import React, { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { PriceChart } from "@/components/PriceChart"
import { usePythProxy } from "@/hooks/usePythProxy"
import { PYTH_FEED_SYMBOLS, ASSET_METADATA, PythSymbol } from "@/lib/constants"
import { Activity, ShieldCheck, Zap, Database, TrendingUp } from "lucide-react"

export default function Prices() {
  const { price: solPrice } = usePythProxy("SOL")
  const { price: btcPrice } = usePythProxy("BTC")
  const { price: ethPrice } = usePythProxy("ETH")

  const assetPrices: Record<PythSymbol, typeof solPrice> = {
    SOL: solPrice,
    BTC: btcPrice,
    ETH: ethPrice,
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          Pyth Network Hermes Oracle Feeds
        </h1>
        <p className="text-xs text-muted-foreground font-mono mt-1">
          Cryptographically signed financial market feeds queried via HTTP 402 paywall
        </p>
      </div>

      {/* Asset Cards Overview */}
      <div className="grid gap-4 md:grid-cols-3">
        {PYTH_FEED_SYMBOLS.map((sym) => {
          const feed = assetPrices[sym]
          const meta = ASSET_METADATA[sym]
          return (
            <Card key={sym} className="border-border">
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <div className="flex items-center gap-2">
                  <span className="text-lg font-bold text-foreground font-mono">
                    {meta.icon}
                  </span>
                  <div>
                    <CardTitle className="text-sm font-semibold">{meta.name}</CardTitle>
                    <span className="text-[10px] text-muted-foreground font-mono">
                      {sym}/USD
                    </span>
                  </div>
                </div>
                <Badge variant="success" className="text-[10px] font-mono">
                  Live Hermes
                </Badge>
              </CardHeader>

              <CardContent className="space-y-2">
                <div className="text-2xl font-extrabold font-mono text-foreground">
                  ${feed ? feed.price.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) : "---"}
                </div>
                <div className="flex items-center justify-between text-xs font-mono text-muted-foreground pt-1 border-t border-border">
                  <span>Confidence: ±${feed ? feed.confidence.toFixed(3) : "---"}</span>
                  <span className="text-emerald-600 dark:text-emerald-400 font-semibold">
                    {feed?.confidenceBps ?? 5} bps
                  </span>
                </div>
              </CardContent>
            </Card>
          )
        })}
      </div>

      {/* Interactive Chart */}
      <PriceChart />

      {/* Architecture Rationale Card */}
      <Card className="border-border bg-muted/20">
        <CardHeader>
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <Database className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
            Why The Agent Uses x402 Micropayments for Pyth
          </CardTitle>
          <CardDescription>
            The economic rationale behind pay-per-query data acquisition
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-3 text-xs text-muted-foreground leading-relaxed">
          <div className="p-3 rounded-lg border border-border bg-card space-y-1">
            <span className="font-semibold text-foreground flex items-center gap-1.5">
              <Zap className="h-3.5 w-3.5 text-amber-500 dark:text-amber-400" />
              $0 Default Spend
            </span>
            <p>
              The agent listens to free public WebSockets by default. It opens its wallet to query high-precision Pyth feeds only when high volatility triggers an opportunity.
            </p>
          </div>

          <div className="p-3 rounded-lg border border-border bg-card space-y-1">
            <span className="font-semibold text-foreground flex items-center gap-1.5">
              <ShieldCheck className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
              Zero Human Approvals
            </span>
            <p>
              No credit cards, monthly API tiers, or pre-registered API keys. The agent signs a 105-byte voucher on demand and receives data in under 30ms.
            </p>
          </div>

          <div className="p-3 rounded-lg border border-border bg-card space-y-1">
            <span className="font-semibold text-foreground flex items-center gap-1.5">
              <TrendingUp className="h-3.5 w-3.5 text-sky-600 dark:text-cyan-400" />
              Mathematical EV Edge
            </span>
            <p>
              The fixed-point decision engine guarantees that data acquisition cost ($0.007) plus network fees never exceeds the expected trading edge.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
