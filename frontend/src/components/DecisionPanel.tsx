import React from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import type { Decision } from "@/lib/types"
import { timeAgo } from "@/lib/utils"
import { BrainCircuit, CheckCircle2, XCircle, Clock, ArrowRight } from "lucide-react"

interface DecisionPanelProps {
  decisions: Decision[]
}

export function DecisionPanel({ decisions }: DecisionPanelProps) {
  const getActionBadge = (action: Decision["action"]) => {
    switch (action) {
      case "BUY":
        return <Badge variant="success">BUY</Badge>
      case "SELL":
        return <Badge variant="destructive">SELL</Badge>
      case "PAY":
        return <Badge variant="info">PAY (x402)</Badge>
      case "HOLD":
        return <Badge variant="warning">HOLD</Badge>
      default:
        return <Badge variant="outline">{action}</Badge>
    }
  }

  const getStatusIcon = (status: Decision["status"]) => {
    switch (status) {
      case "executed":
        return <CheckCircle2 className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
      case "approved":
        return <CheckCircle2 className="h-3.5 w-3.5 text-sky-600 dark:text-cyan-400" />
      case "rejected":
        return <XCircle className="h-3.5 w-3.5 text-rose-500 dark:text-rose-400" />
      case "pending":
      default:
        return <Clock className="h-3.5 w-3.5 text-amber-500 dark:text-amber-400 animate-spin" />
    }
  }

  return (
    <Card className="border-border">
      <CardHeader className="flex flex-row items-center justify-between pb-3">
        <div className="space-y-1">
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <BrainCircuit className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
            Fixed-Point Decision Engine
          </CardTitle>
          <p className="text-xs text-muted-foreground">
            Pure deterministic EV calculations & guardrail validation
          </p>
        </div>
        <Badge variant="outline" className="text-[10px] font-mono">
          Integer Fixed-Point
        </Badge>
      </CardHeader>

      <CardContent className="space-y-3">
        {decisions.length === 0 ? (
          <div className="py-8 text-center text-xs text-muted-foreground font-mono">
            Awaiting incoming market signals...
          </div>
        ) : (
          decisions.slice(0, 5).map((decision) => (
            <div
              key={decision.id}
              className="p-3 rounded-lg border border-border bg-muted/30 hover:bg-muted/60 transition-all space-y-2 group"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  {getActionBadge(decision.action)}
                  <span className="font-semibold text-xs font-mono text-foreground">
                    {decision.asset}
                  </span>
                  {decision.price && (
                    <span className="text-xs font-mono text-muted-foreground">
                      @ ${decision.price.toLocaleString()}
                    </span>
                  )}
                </div>

                <div className="flex items-center gap-2 text-xs font-mono">
                  <span className="text-[11px] text-muted-foreground">
                    {timeAgo(decision.timestamp)}
                  </span>
                  <div className="flex items-center gap-1">
                    {getStatusIcon(decision.status)}
                    <span className="text-[10px] uppercase font-semibold text-muted-foreground">
                      {decision.status}
                    </span>
                  </div>
                </div>
              </div>

              {/* Rationale description */}
              <p className="text-xs text-muted-foreground leading-relaxed">
                {decision.reason}
              </p>

              {/* Micro Metrics bar if available */}
              {(decision.edgeBps !== undefined || decision.feeBps !== undefined) && (
                <div className="flex items-center gap-3 pt-1 border-t border-border/50 text-[10px] font-mono text-muted-foreground">
                  {decision.edgeBps !== undefined && (
                    <span className="flex items-center gap-1">
                      Edge:{" "}
                      <strong className={decision.edgeBps > 15 ? "text-emerald-600 dark:text-emerald-400" : "text-amber-600 dark:text-amber-400"}>
                        +{decision.edgeBps} bps
                      </strong>
                    </span>
                  )}
                  {decision.feeBps !== undefined && (
                    <span>
                      Fee: <strong className="text-foreground">{decision.feeBps} bps</strong>
                    </span>
                  )}
                  {decision.confidenceBps !== undefined && (
                    <span>
                      Conf: <strong className="text-sky-600 dark:text-cyan-400">{decision.confidenceBps} bps</strong>
                    </span>
                  )}
                  {decision.oracleLatencyMs !== undefined && (
                    <span className="ml-auto text-muted-foreground">
                      {decision.oracleLatencyMs}ms
                    </span>
                  )}
                </div>
              )}
            </div>
          ))
        )}
      </CardContent>
    </Card>
  )
}
