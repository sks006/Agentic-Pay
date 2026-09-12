import React, { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Progress } from "@/components/ui/progress"
import type { AgentInfo } from "@/lib/types"
import { formatAddress, formatLamports } from "@/lib/utils"
import { ShieldCheck, ShieldAlert, Cpu, HardDrive, Check, Copy } from "lucide-react"

interface Props {
  agent: AgentInfo
  onTogglePause?: (paused: boolean) => void
}

export function AgentStatus({ agent, onTogglePause }: Props) {
  const online = agent.status === "running"
  const [copied, setCopied] = useState(false)

  const copyAddress = () => {
    navigator.clipboard.writeText(agent.walletAddress)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const hours = Math.floor(agent.uptimeSeconds / 3600)
  const minutes = Math.floor((agent.uptimeSeconds % 3600) / 60)
  const seconds = agent.uptimeSeconds % 60

  const spentSol = agent.spentTodayLamports / 1_000_000_000
  const dailyCapSol = agent.dailyCapLamports / 1_000_000_000
  const capPercent = Math.min(100, Math.round((spentSol / (dailyCapSol || 1)) * 100))
  const inFlightPercent = Math.min(100, (agent.inFlightCount / (agent.maxInFlight || 1)) * 100)

  return (
    <Card className="border-border">
      <CardHeader className="flex flex-row items-center justify-between pb-3">
        <CardTitle className="flex items-center gap-2 text-sm font-semibold">
          <Cpu className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
          Agent Core Engine
        </CardTitle>
        <Badge
          variant={
            agent.isPaused
              ? "destructive"
              : online
              ? "success"
              : "secondary"
          }
          className="flex items-center gap-1.5"
        >
          <span
            className={`h-1.5 w-1.5 rounded-full ${
              agent.isPaused
                ? "bg-rose-500 animate-ping"
                : online
                ? "bg-emerald-500 animate-pulse"
                : "bg-muted-foreground"
            }`}
          />
          {agent.isPaused ? "EMERGENCY PAUSED" : agent.status.toUpperCase()}
        </Badge>
      </CardHeader>

      <CardContent className="space-y-4 text-sm">
        {/* Agent Name & ID */}
        <div className="flex items-center justify-between">
          <span className="text-muted-foreground text-xs font-medium">Identifier</span>
          <span className="font-semibold text-foreground font-mono text-xs">
            {agent.name}
          </span>
        </div>

        {/* Wallet Address */}
        <div className="flex items-center justify-between">
          <span className="text-muted-foreground text-xs font-medium">Wallet Pubkey</span>
          <div className="flex items-center gap-1.5">
            <span className="font-mono text-xs bg-muted px-2 py-0.5 rounded border border-border text-emerald-700 dark:text-emerald-400 font-semibold">
              {formatAddress(agent.walletAddress, 4)}
            </span>
            <Button
              variant="ghost"
              size="icon"
              onClick={copyAddress}
              className="h-6 w-6 text-muted-foreground hover:text-foreground"
              title="Copy public key"
            >
              {copied ? (
                <Check className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
              ) : (
                <Copy className="h-3.5 w-3.5" />
              )}
            </Button>
          </div>
        </div>

        {/* Uptime */}
        <div className="flex items-center justify-between">
          <span className="text-muted-foreground text-xs font-medium">Runtime</span>
          <span className="font-mono text-xs text-foreground">
            {hours}h {minutes}m {seconds}s
          </span>
        </div>

        {/* In-Flight Rate Limit Gauge */}
        <div className="space-y-1.5 pt-1 border-t border-border">
          <div className="flex justify-between text-xs">
            <span className="text-muted-foreground flex items-center gap-1">
              <HardDrive className="h-3.5 w-3.5 text-sky-600 dark:text-cyan-400" />
              In-Flight Vouchers
            </span>
            <span className="font-mono text-xs font-semibold">
              {agent.inFlightCount} / {agent.maxInFlight}
            </span>
          </div>
          <Progress
            value={inFlightPercent}
            className="h-1.5 bg-muted"
            indicatorClassName="bg-sky-600 dark:bg-cyan-400"
          />
        </div>

        {/* Daily Spending Cap Guardrail */}
        <div className="space-y-1.5 pt-1 border-t border-border">
          <div className="flex justify-between text-xs">
            <span className="text-muted-foreground flex items-center gap-1">
              {agent.isPaused ? (
                <ShieldAlert className="h-3.5 w-3.5 text-rose-500" />
              ) : (
                <ShieldCheck className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
              )}
              Daily Guardrail Cap
            </span>
            <span className="font-mono text-xs font-semibold">
              {spentSol.toFixed(3)} / {dailyCapSol.toFixed(1)} SOL ({capPercent}%)
            </span>
          </div>
          <Progress
            value={capPercent}
            className="h-1.5 bg-muted"
            indicatorClassName={
              capPercent > 80
                ? "bg-rose-500"
                : capPercent > 50
                ? "bg-amber-500"
                : "bg-emerald-600 dark:bg-emerald-500"
            }
          />
        </div>

        {/* Emergency Pause Toggle */}
        {onTogglePause && (
          <div className="pt-2">
            <Button
              variant={agent.isPaused ? "cyber" : "destructive"}
              size="sm"
              className="w-full text-xs"
              onClick={() => onTogglePause(!agent.isPaused)}
            >
              {agent.isPaused ? "Resume Agent Trading" : "Emergency Pause Guardrail"}
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
