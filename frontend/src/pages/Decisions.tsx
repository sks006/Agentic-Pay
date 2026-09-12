import React, { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { useAgentRpc } from "@/hooks/useAgentRpc"
import { timeAgo } from "@/lib/utils"
import type { Decision } from "@/lib/types"
import { BrainCircuit, CheckCircle2, XCircle, Calculator, ArrowRight, Shield } from "lucide-react"

export default function Decisions() {
  const { data } = useAgentRpc()
  const [filterAction, setFilterAction] = useState<string>("ALL")

  const decisions = data?.decisions ?? []
  const filteredDecisions =
    filterAction === "ALL"
      ? decisions
      : decisions.filter((d) => d.action === filterAction)

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
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          Autonomous Decision Engine & Math Audits
        </h1>
        <p className="text-xs text-muted-foreground font-mono mt-1">
          Zero-float integer fixed-point expected value (EV) evaluation logs
        </p>
      </div>

      {/* Math Formula Card */}
      <Card className="border-border glass-panel">
        <CardHeader>
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <Calculator className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
            Deterministic Fixed-Point Decision Rule
          </CardTitle>
          <CardDescription>
            Implemented in pure integer arithmetic (`agent-backend/src/decision.rs`) to prevent floating-point non-determinism.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="p-4 rounded-xl bg-muted/50 border border-border font-mono text-xs sm:text-sm text-center text-emerald-700 dark:text-emerald-400 font-semibold">
            Net Edge = Signal Edge (bps) - ( Data Cost (bps) + Network Fee (bps) )
          </div>

          <div className="grid gap-3 sm:grid-cols-3 text-xs text-muted-foreground">
            <div className="p-3 rounded-lg border border-border bg-muted/30">
              <span className="font-semibold text-foreground block mb-1">
                1. Confidence Filter
              </span>
              If Pyth confidence interval exceeds <strong>20 bps</strong>, data is discarded as too noisy.
            </div>
            <div className="p-3 rounded-lg border border-border bg-muted/30">
              <span className="font-semibold text-foreground block mb-1">
                2. Data Justification
              </span>
              Data query fee is authorized only if predicted price move justifies the lamport cost.
            </div>
            <div className="p-3 rounded-lg border border-border bg-muted/30">
              <span className="font-semibold text-foreground block mb-1">
                3. On-Chain Caps
              </span>
              Even if decision engine approves, on-chain Anchor contract enforces daily cap limits.
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Decisions Ledger */}
      <Card className="border-border">
        <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div>
            <CardTitle className="text-base font-semibold flex items-center gap-2">
              <BrainCircuit className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
              Decision Audit Trail
            </CardTitle>
            <CardDescription>
              Chronological ledger of every evaluation performed by the autonomous agent.
            </CardDescription>
          </div>

          {/* Filters via shadcn Tabs */}
          <Tabs value={filterAction} onValueChange={setFilterAction}>
            <TabsList className="h-8">
              {["ALL", "BUY", "PAY", "HOLD"].map((act) => (
                <TabsTrigger key={act} value={act} className="text-xs px-2.5 py-1">
                  {act}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        </CardHeader>

        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Action</TableHead>
                <TableHead>Asset / Target</TableHead>
                <TableHead>Mathematical Rationale</TableHead>
                <TableHead>Edge / Fee</TableHead>
                <TableHead>Outcome</TableHead>
                <TableHead className="text-right">Time</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredDecisions.map((d) => (
                <TableRow key={d.id}>
                  <TableCell>{getActionBadge(d.action)}</TableCell>
                  <TableCell className="font-mono font-semibold text-xs text-foreground">
                    {d.asset}
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground max-w-md">
                    {d.reason}
                  </TableCell>
                  <TableCell className="font-mono text-xs">
                    {d.edgeBps !== undefined && (
                      <span className="text-emerald-600 dark:text-emerald-400 font-semibold">
                        +{d.edgeBps} bps
                      </span>
                    )}
                    {d.feeBps !== undefined && (
                      <span className="text-muted-foreground ml-1">
                        (-{d.feeBps} bps fee)
                      </span>
                    )}
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1.5">
                      {d.status === "executed" || d.status === "approved" ? (
                        <CheckCircle2 className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
                      ) : (
                        <XCircle className="h-3.5 w-3.5 text-rose-500 dark:text-rose-400" />
                      )}
                      <span className="text-[11px] font-mono uppercase font-semibold text-muted-foreground">
                        {d.status}
                      </span>
                    </div>
                  </TableCell>
                  <TableCell className="text-right text-xs text-muted-foreground font-mono">
                    {timeAgo(d.timestamp)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
