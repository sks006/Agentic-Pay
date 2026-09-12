import React, { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import type { Voucher } from "@/lib/types"
import { formatAddress, timeAgo } from "@/lib/utils"
import { Receipt, Send, CheckCircle2, Clock, AlertTriangle } from "lucide-react"

interface PaymentLogProps {
  vouchers: Voucher[]
  onSimulateVoucher?: (amountLamports?: number) => Promise<Voucher>
}

export function PaymentLog({ vouchers, onSimulateVoucher }: PaymentLogProps) {
  const [simulating, setSimulating] = useState(false)

  const handleSimulate = async () => {
    if (!onSimulateVoucher) return
    try {
      setSimulating(true)
      await onSimulateVoucher(50_000)
    } finally {
      setSimulating(false)
    }
  }

  const getVoucherBadge = (status: Voucher["status"]) => {
    switch (status) {
      case "active":
        return <Badge variant="success">ACTIVE (UNSETTLED)</Badge>
      case "settling":
        return <Badge variant="warning">SETTLING BATCH</Badge>
      case "redeemed":
        return <Badge variant="outline">REDEEMED ON-CHAIN</Badge>
      case "expired":
        return <Badge variant="destructive">EXPIRED</Badge>
    }
  }

  return (
    <Card className="border-border">
      <CardHeader className="flex flex-row items-center justify-between pb-3">
        <div className="space-y-1">
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <Receipt className="h-4 w-4 text-amber-500 dark:text-amber-400" />
            x402 Voucher Log
          </CardTitle>
          <p className="text-xs text-muted-foreground">
            Off-chain Ed25519 vouchers awaiting on-chain batch settlement
          </p>
        </div>

        {onSimulateVoucher && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleSimulate}
            disabled={simulating}
            className="text-xs flex items-center gap-1.5 border-border hover:border-primary"
          >
            <Send className="h-3 w-3" />
            {simulating ? "Signing..." : "Simulate 402 Pay"}
          </Button>
        )}
      </CardHeader>

      <CardContent className="space-y-2.5">
        {vouchers.length === 0 ? (
          <div className="py-8 text-center text-xs text-muted-foreground font-mono">
            No active vouchers in queue.
          </div>
        ) : (
          vouchers.slice(0, 6).map((voucher) => (
            <div
              key={voucher.id}
              className="flex items-center justify-between p-2.5 rounded-lg border border-border bg-muted/30 hover:bg-muted/60 transition-all text-xs"
            >
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <span className="font-mono font-bold text-foreground">
                    #{voucher.nonce}
                  </span>
                  <span className="font-mono text-emerald-600 dark:text-emerald-400 font-semibold">
                    {voucher.amountLamports.toLocaleString()} lamports
                  </span>
                  <span className="text-[11px] text-muted-foreground">
                    (${voucher.amountUsd.toFixed(4)})
                  </span>
                </div>
                <div className="flex items-center gap-2 text-[11px] font-mono text-muted-foreground">
                  <span>To: {formatAddress(voucher.recipient, 4)}</span>
                  <span>•</span>
                  <span>Expires: {timeAgo(voucher.expiresAt)}</span>
                </div>
              </div>

              <div className="flex flex-col items-end gap-1">
                {getVoucherBadge(voucher.status)}
                <span className="font-mono text-[10px] text-muted-foreground">
                  Sig: {voucher.signature.slice(0, 8)}...
                </span>
              </div>
            </div>
          ))
        )}
      </CardContent>
    </Card>
  )
}
