import React from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import type { AgentEvent } from "@/lib/types"
import { Terminal, Radio } from "lucide-react"

interface LiveFeedProps {
  events?: AgentEvent[]
}

export function LiveFeed({ events = [] }: LiveFeedProps) {
  const getEventBadge = (type: AgentEvent["type"]) => {
    switch (type) {
      case "price_update":
        return <Badge variant="info" className="text-[9px] py-0 px-1.5">PYTH</Badge>
      case "voucher":
        return <Badge variant="warning" className="text-[9px] py-0 px-1.5">x402 VOUCHER</Badge>
      case "decision":
        return <Badge variant="success" className="text-[9px] py-0 px-1.5">DECISION</Badge>
      case "transaction":
        return <Badge variant="purple" className="text-[9px] py-0 px-1.5">ON-CHAIN</Badge>
      case "system":
      default:
        return <Badge variant="outline" className="text-[9px] py-0 px-1.5">SYSTEM</Badge>
    }
  }

  return (
    <Card className="border-border">
      <CardHeader className="flex flex-row items-center justify-between pb-3">
        <div className="flex items-center gap-2">
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <Terminal className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
            Live Autonomous Telemetry Feed
          </CardTitle>
          <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground font-mono">
            <Radio className="h-3 w-3 text-emerald-500 animate-pulse" />
            <span>Streaming</span>
          </div>
        </div>
        <Badge variant="outline" className="text-[10px] font-mono">
          Loop A & Loop B Interleaved
        </Badge>
      </CardHeader>

      <CardContent>
        <div className="rounded-lg bg-muted/40 border border-border p-3.5 font-mono text-xs max-h-52 overflow-y-auto space-y-2">
          {events.length === 0 ? (
            <div className="text-muted-foreground/60 py-4 text-center">
              Listening to local event bus...
            </div>
          ) : (
            events.map((evt) => {
              const time = new Date(evt.timestamp).toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
                second: "2-digit",
              })
              return (
                <div
                  key={evt.id}
                  className="flex items-start gap-2.5 leading-relaxed text-muted-foreground hover:text-foreground transition-colors group"
                >
                  <span className="text-muted-foreground/50 shrink-0 text-[11px]">
                    {time}
                  </span>
                  <div className="shrink-0 pt-0.5">{getEventBadge(evt.type)}</div>
                  <span className="text-foreground/90 font-medium">
                    {evt.message}
                  </span>
                </div>
              )
            })
          )}
        </div>
      </CardContent>
    </Card>
  )
}
