import React, { useState, useEffect } from "react"
import { WalletConnect } from "@/components/WalletConnect"
import { useAgentRpc } from "@/hooks/useAgentRpc"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Activity, Sun, Moon } from "lucide-react"

export function Topbar() {
  const { data } = useAgentRpc()
  const agent = data?.agent
  const isOnline = agent?.status === "running"
  const isPaused = agent?.isPaused

  const [isDark, setIsDark] = useState(() => {
    return document.documentElement.classList.contains("dark")
  })

  const toggleTheme = () => {
    const next = !isDark
    setIsDark(next)
    if (next) {
      document.documentElement.classList.add("dark")
      localStorage.setItem("theme", "dark")
    } else {
      document.documentElement.classList.remove("dark")
      localStorage.setItem("theme", "light")
    }
  }

  return (
    <header className="h-16 border-b border-border bg-card/90 backdrop-blur-md px-6 flex items-center justify-between sticky top-0 z-40">
      {/* Left Status Beacon */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-muted/60 border border-border text-xs">
          <span
            className={`h-2 w-2 rounded-full ${
              isPaused
                ? "bg-rose-500 animate-ping"
                : isOnline
                ? "bg-emerald-500 animate-pulse"
                : "bg-muted-foreground"
            }`}
          />
          <span className="font-semibold font-mono text-foreground">
            {isPaused
              ? "Agent: Circuit Breaker Paused"
              : isOnline
              ? "Agent: Autonomous Online"
              : "Agent: Connecting..."}
          </span>
        </div>

        <Badge variant="outline" className="hidden sm:inline-flex text-[11px] font-mono text-muted-foreground border-border">
          Solana Devnet / Localnet
        </Badge>
      </div>

      {/* Right Controls */}
      <div className="flex items-center gap-3">
        <Button
          variant="outline"
          size="icon"
          onClick={toggleTheme}
          title={isDark ? "Switch to light mode" : "Switch to dark mode"}
          className="h-8 w-8 text-muted-foreground hover:text-foreground"
        >
          {isDark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
        </Button>

        <WalletConnect
          walletAddress={agent?.walletAddress}
          balanceSol={data?.balances[0]?.amount}
        />
      </div>
    </header>
  )
}
