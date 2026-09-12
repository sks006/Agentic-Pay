import React, { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { useAgentRpc } from "@/hooks/useAgentRpc"
import { formatAddress, timeAgo } from "@/lib/utils"
import { DOMAIN_SEPARATOR, CANONICAL_MSG_LENGTH, PAYMENT_SIGNATURE_WIRE_LENGTH } from "@/lib/constants"
import { Receipt, Send, CheckCircle2, Shield, Code, Sparkles } from "lucide-react"

export default function Payments() {
  const { data, simulatePayment } = useAgentRpc()
  const [selectedVoucherId, setSelectedVoucherId] = useState<string | null>(null)
  const [simulating, setSimulating] = useState(false)

  const vouchers = data?.vouchers ?? []
  const selectedVoucher = vouchers.find((v) => v.id === selectedVoucherId) || vouchers[0]

  const handleSimulate = async () => {
    try {
      setSimulating(true)
      const vch = await simulatePayment(50_000)
      if (vch?.id) {
        setSelectedVoucherId(vch.id)
      }
    } catch (err) {
      console.error("Simulation failed:", err)
    } finally {
      setSimulating(false)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Voucher Settlements & x402 Cryptography
          </h1>
          <p className="text-xs text-muted-foreground font-mono mt-1">
            Loop B on-chain batch settlement & 105-byte canonical message inspector
          </p>
        </div>

        <Button
          variant="cyber"
          size="sm"
          onClick={handleSimulate}
          disabled={simulating}
          className="text-xs flex items-center gap-2"
        >
          <Send className="h-3.5 w-3.5" />
          {simulating ? "Signing Voucher..." : "Sign 50,000 Lamport Voucher"}
        </Button>
      </div>

      {/* Protocol Architecture Banner */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card className="border-border">
          <CardHeader className="pb-2">
            <CardTitle className="text-xs text-muted-foreground uppercase font-mono">
              Canonical Byte Layout
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xl font-bold font-mono text-foreground">
              {CANONICAL_MSG_LENGTH} Bytes
            </div>
            <p className="text-[11px] text-muted-foreground mt-1">
              Domain separator + Nonce + Agent + Provider + Amount + TTL
            </p>
          </CardContent>
        </Card>

        <Card className="border-border">
          <CardHeader className="pb-2">
            <CardTitle className="text-xs text-muted-foreground uppercase font-mono">
              Wire Format Header
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xl font-bold font-mono text-sky-600 dark:text-cyan-400">
              {PAYMENT_SIGNATURE_WIRE_LENGTH} Bytes
            </div>
            <p className="text-[11px] text-muted-foreground mt-1">
              Base64(105B Canonical + 64B Ed25519 Sig + 1B Retry Count)
            </p>
          </CardContent>
        </Card>

        <Card className="border-border">
          <CardHeader className="pb-2">
            <CardTitle className="text-xs text-muted-foreground uppercase font-mono">
              Domain Separator
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-lg font-bold font-mono text-emerald-600 dark:text-emerald-400">
              {DOMAIN_SEPARATOR}
            </div>
            <p className="text-[11px] text-muted-foreground mt-1">
              Cryptographically eliminates cross-protocol replay attacks
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Selected Voucher Canonical Inspector */}
      {selectedVoucher && (
        <Card className="border-border glass-panel">
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Code className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
                <CardTitle className="text-sm font-semibold">
                  Canonical 105-Byte Message Inspector: Voucher #{selectedVoucher.nonce}
                </CardTitle>
              </div>
              <Badge variant="success">Ed25519 Verified</Badge>
            </div>
            <CardDescription className="font-mono text-xs">
              This exact 105-byte canonical buffer is validated by Anchor instruction sysvars on Solana.
            </CardDescription>
          </CardHeader>

          <CardContent className="space-y-4 text-xs font-mono">
            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-2">
              <div className="p-2.5 rounded-lg bg-muted/40 border border-border">
                <span className="text-[10px] text-muted-foreground block">0..17 (17B)</span>
                <span className="font-semibold text-emerald-600 dark:text-emerald-400 truncate block">
                  {DOMAIN_SEPARATOR}
                </span>
                <span className="text-[10px] text-muted-foreground">Domain Sep</span>
              </div>

              <div className="p-2.5 rounded-lg bg-muted/40 border border-border">
                <span className="text-[10px] text-muted-foreground block">17..25 (8B LE)</span>
                <span className="font-semibold text-sky-600 dark:text-cyan-400 block">
                  #{selectedVoucher.nonce}
                </span>
                <span className="text-[10px] text-muted-foreground">Monotonic Nonce</span>
              </div>

              <div className="p-2.5 rounded-lg bg-muted/40 border border-border">
                <span className="text-[10px] text-muted-foreground block">25..57 (32B)</span>
                <span className="font-semibold text-foreground truncate block">
                  {formatAddress(selectedVoucher.issuer, 3)}
                </span>
                <span className="text-[10px] text-muted-foreground">Agent Pubkey</span>
              </div>

              <div className="p-2.5 rounded-lg bg-muted/40 border border-border">
                <span className="text-[10px] text-muted-foreground block">57..89 (32B)</span>
                <span className="font-semibold text-foreground truncate block">
                  {formatAddress(selectedVoucher.recipient, 3)}
                </span>
                <span className="text-[10px] text-muted-foreground">Provider Pubkey</span>
              </div>

              <div className="p-2.5 rounded-lg bg-muted/40 border border-border">
                <span className="text-[10px] text-muted-foreground block">89..97 (8B LE)</span>
                <span className="font-semibold text-amber-600 dark:text-amber-400 block">
                  {selectedVoucher.amountLamports}
                </span>
                <span className="text-[10px] text-muted-foreground">Lamports</span>
              </div>

              <div className="p-2.5 rounded-lg bg-muted/40 border border-border">
                <span className="text-[10px] text-muted-foreground block">97..105 (8B LE)</span>
                <span className="font-semibold text-purple-600 dark:text-purple-400 block">
                  +{Math.max(0, selectedVoucher.expiresAtTimestamp - Math.floor(Date.now()/1000))}s
                </span>
                <span className="text-[10px] text-muted-foreground">Expiry TTL</span>
              </div>
            </div>

            <div className="p-3 rounded-lg bg-muted/60 border border-border space-y-1">
              <span className="text-muted-foreground text-[11px] block">Ed25519 Detached Signature (64 Bytes)</span>
              <span className="text-foreground break-all text-[11px] font-mono">
                {selectedVoucher.signature}
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Vouchers Table */}
      <Card className="border-border">
        <CardHeader>
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <Receipt className="h-4 w-4 text-amber-500 dark:text-amber-400" />
            Voucher Settlement Ledger
          </CardTitle>
          <CardDescription>
            All issued vouchers and their on-chain settlement progress.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-20">Nonce</TableHead>
                <TableHead>Recipient</TableHead>
                <TableHead>Amount</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>TTL / Expiry</TableHead>
                <TableHead className="text-right">Action</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {vouchers.map((v) => (
                <TableRow
                  key={v.id}
                  className={selectedVoucher?.id === v.id ? "bg-muted/40" : ""}
                >
                  <TableCell className="font-mono font-bold">#{v.nonce}</TableCell>
                  <TableCell className="font-mono text-xs">
                    {formatAddress(v.recipient, 4)}
                  </TableCell>
                  <TableCell className="font-mono">
                    <span className="text-emerald-600 dark:text-emerald-400 font-semibold">
                      {v.amountLamports.toLocaleString()} lamports
                    </span>{" "}
                    <span className="text-xs text-muted-foreground">
                      (${v.amountUsd.toFixed(4)})
                    </span>
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant={
                        v.status === "active"
                          ? "success"
                          : v.status === "settling"
                          ? "warning"
                          : "outline"
                      }
                      className="text-[10px]"
                    >
                      {v.status.toUpperCase()}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground font-mono">
                    {timeAgo(v.expiresAt)}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-xs font-mono text-primary hover:text-primary"
                      onClick={() => setSelectedVoucherId(v.id)}
                    >
                      Inspect 105B
                    </Button>
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
