import React, { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert"
import { useAgentRpc } from "@/hooks/useAgentRpc"
import { formatAddress } from "@/lib/utils"
import { GUARDRAILS_PROGRAM_ID } from "@/lib/constants"
import { Settings as SettingsIcon, ShieldAlert, ShieldCheck, Server, Key, AlertTriangle } from "lucide-react"

export default function Settings() {
  const { data, setPaused } = useAgentRpc()
  const agent = data?.agent

  const isPaused = agent?.isPaused ?? false

  return (
    <div className="space-y-6 max-w-4xl">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          Guardrail Policies & Core Configuration
        </h1>
        <p className="text-xs text-muted-foreground font-mono mt-1">
          Solana on-chain Escrow caps, rate-limiting parameters, and emergency circuit breaker
        </p>
      </div>

      {/* Emergency Circuit Breaker */}
      <Card className={`border transition-all ${isPaused ? "border-destructive bg-destructive/5" : "border-border"}`}>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <ShieldAlert className={`h-5 w-5 ${isPaused ? "text-destructive animate-pulse" : "text-amber-500 dark:text-amber-400"}`} />
              <CardTitle className="text-base font-semibold">
                Emergency Circuit Breaker (set_paused)
              </CardTitle>
            </div>
            <Badge variant={isPaused ? "destructive" : "success"}>
              {isPaused ? "CIRCUIT BREAKER ACTIVE" : "NORMAL AUTONOMY"}
            </Badge>
          </div>
          <CardDescription>
            Freezes all voucher settlements immediately on-chain via the Anchor guardrail program. Even if the agent tries to sign vouchers, they will revert on-chain.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert variant={isPaused ? "destructive" : "warning"}>
            <ShieldAlert className="h-4 w-4" />
            <AlertTitle>
              {isPaused ? "Circuit Breaker Active" : "Autonomous Guardrails Active"}
            </AlertTitle>
            <AlertDescription>
              {isPaused
                ? "All voucher settlements are halted on-chain. Funds in Escrow PDA are physically locked."
                : "Agent is actively signing vouchers within daily spend limits."}
            </AlertDescription>
          </Alert>

          <div>
            <Button
              variant={isPaused ? "cyber" : "destructive"}
              onClick={() => setPaused(!isPaused)}
              className="text-xs font-semibold"
            >
              {isPaused ? "Deactivate Circuit Breaker (Resume Trading)" : "Engage Emergency Stop"}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* On-Chain Escrow Policies */}
      <Card className="border-border">
        <CardHeader>
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <ShieldCheck className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
            On-Chain Escrow Parameters
          </CardTitle>
          <CardDescription>
            Immutably enforced by Anchor Program ID: <span className="font-mono">{GUARDRAILS_PROGRAM_ID}</span>
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 text-xs font-mono">
          <div className="grid gap-3 sm:grid-cols-2">
            <div className="p-3 rounded-lg border border-border bg-muted/30 space-y-1">
              <span className="text-muted-foreground text-[11px] block">Daily Spend Cap</span>
              <span className="text-foreground text-sm font-bold">
                {agent ? (agent.dailyCapLamports / 1_000_000_000).toFixed(1) : 5.0} SOL (5,000,000,000 lamports)
              </span>
              <span className="text-[10px] text-muted-foreground block">
                Resets every 216,000 slots (~24 hours)
              </span>
            </div>

            <div className="p-3 rounded-lg border border-border bg-muted/30 space-y-1">
              <span className="text-muted-foreground text-[11px] block">Per-Transaction Cap</span>
              <span className="text-foreground text-sm font-bold">
                0.100 SOL (100,000,000 lamports)
              </span>
              <span className="text-[10px] text-muted-foreground block">
                Max allowable per batch settlement
              </span>
            </div>
          </div>

          <div className="space-y-2 pt-2 border-t border-border">
            <div className="flex justify-between">
              <span className="text-muted-foreground font-sans">Escrow PDA:</span>
              <span className="text-foreground">{agent?.escrowPda ?? "Escrow1111111111111111111111111111111111111111"}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground font-sans">Owner / Delegator:</span>
              <span className="text-foreground">{agent?.walletAddress ? formatAddress(agent.walletAddress, 6) : "AgeNT...5678"}</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Network & RPC Configuration */}
      <Card className="border-border">
        <CardHeader>
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <Server className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
            Backend RPC & Network
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 text-xs font-mono">
          <div className="flex items-center justify-between p-2.5 rounded bg-muted/30 border border-border">
            <div>
              <span className="font-semibold text-foreground block font-sans">Agent JSON-RPC Endpoint</span>
              <span className="text-muted-foreground text-[11px]">{agent?.rpcUrl ?? "http://127.0.0.1:8545"}</span>
            </div>
            <Badge variant="success">CONNECTED</Badge>
          </div>

          <div className="flex items-center justify-between p-2.5 rounded bg-muted/30 border border-border">
            <div>
              <span className="font-semibold text-foreground block font-sans">Solana Cluster</span>
              <span className="text-muted-foreground text-[11px]">http://127.0.0.1:8899 (Localnet / Devnet)</span>
            </div>
            <Badge variant="outline">LOCALNET</Badge>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
