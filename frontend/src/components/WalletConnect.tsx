import React, { useState } from "react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { formatAddress, formatLamports } from "@/lib/utils"
import { Wallet, ChevronDown, Check, ShieldCheck, ExternalLink } from "lucide-react"

interface WalletConnectProps {
  walletAddress?: string
  balanceSol?: number
  network?: string
}

export function WalletConnect({
  walletAddress = "AgeNT7xK89abCdeFGhiJKLMnoPQRSTuvwxYz12345678",
  balanceSol = 14.82,
  network = "Solana Localnet",
}: WalletConnectProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(walletAddress)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="relative">
      <Button
        variant="outline"
        size="sm"
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 font-mono text-xs border-border hover:border-primary/50"
      >
        <div className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
        <span className="font-semibold text-emerald-600 dark:text-emerald-400">
          {balanceSol.toFixed(2)} SOL
        </span>
        <span className="text-muted-foreground">•</span>
        <span className="text-muted-foreground">{formatAddress(walletAddress, 3)}</span>
        <ChevronDown className="h-3 w-3 text-muted-foreground" />
      </Button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-72 rounded-xl border border-border bg-popover text-popover-foreground p-4 shadow-xl backdrop-blur-md z-50 animate-in fade-in-50 zoom-in-95 space-y-3">
          <div className="flex items-center justify-between pb-2 border-b border-border">
            <div className="flex items-center gap-2">
              <Wallet className="h-4 w-4 text-primary" />
              <span className="font-semibold text-xs text-foreground">Agent Wallet</span>
            </div>
            <Badge variant="outline" className="text-[10px] text-muted-foreground">
              {network}
            </Badge>
          </div>

          <div className="space-y-2 text-xs">
            <div>
              <span className="text-muted-foreground text-[11px]">Public Key</span>
              <div className="flex items-center justify-between mt-1 bg-muted p-2 rounded border border-border font-mono">
                <span className="text-foreground text-[11px] truncate mr-2">
                  {walletAddress}
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleCopy}
                  className="h-6 px-1.5 text-xs text-muted-foreground hover:text-foreground shrink-0"
                >
                  {copied ? <Check className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" /> : "Copy"}
                </Button>
              </div>
            </div>

            <div className="flex justify-between py-1 text-xs">
              <span className="text-muted-foreground">Escrow Balance</span>
              <span className="font-mono font-semibold text-emerald-600 dark:text-emerald-400">5.000 SOL</span>
            </div>

            <div className="flex justify-between py-1 text-xs">
              <span className="text-muted-foreground">Guardrail Daily Cap</span>
              <span className="font-mono text-foreground font-medium">5.0 SOL</span>
            </div>
          </div>

          <div className="pt-2 border-t border-border flex items-center justify-between text-[11px] text-muted-foreground">
            <span className="flex items-center gap-1 text-emerald-600 dark:text-emerald-400">
              <ShieldCheck className="h-3.5 w-3.5" /> Caps Enforced On-Chain
            </span>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsOpen(false)}
              className="h-6 px-2 text-xs text-muted-foreground hover:text-foreground"
            >
              Close
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
