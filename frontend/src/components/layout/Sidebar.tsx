import React from "react"
import { NavLink } from "react-router-dom"
import { Badge } from "@/components/ui/badge"
import {
  LayoutDashboard,
  Receipt,
  Activity,
  BrainCircuit,
  Settings,
  Shield,
  Zap,
} from "lucide-react"

const links = [
  { label: "Dashboard", path: "/dashboard", icon: LayoutDashboard },
  { label: "Voucher Payments", path: "/payments", icon: Receipt },
  { label: "Pyth Prices", path: "/prices", icon: Activity },
  { label: "Agent Decisions", path: "/decisions", icon: BrainCircuit },
  { label: "Settings & Caps", path: "/settings", icon: Settings },
]

export function Sidebar() {
  return (
    <aside className="hidden md:flex w-64 flex-col border-r border-border bg-card/95 backdrop-blur-xl shrink-0">
      {/* Brand Header */}
      <div className="h-16 flex items-center gap-3 px-6 border-b border-border">
        <div className="h-8 w-8 rounded-lg bg-emerald-600 dark:bg-emerald-500 flex items-center justify-center text-white font-bold shadow-sm">
          <Zap className="h-4 w-4 fill-current" />
        </div>
        <div>
          <h1 className="font-extrabold text-sm tracking-tight text-foreground flex items-center gap-1.5">
            AgenticPay
            <Badge variant="secondary" className="text-[10px] font-mono px-1.5 py-0 font-bold">
              v1.0
            </Badge>
          </h1>
          <p className="text-[10px] text-muted-foreground font-mono">
            Autonomous x402 Console
          </p>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 px-3 py-4">
        <div className="px-3 pb-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground font-mono">
          Control Center
        </div>
        {links.map((link) => {
          const Icon = link.icon
          return (
            <NavLink
              key={link.path}
              to={link.path}
              className={({ isActive }) =>
                `flex items-center gap-3 rounded-lg px-3 py-2 text-xs font-medium transition-all ${
                  isActive
                    ? "bg-primary text-primary-foreground shadow-sm font-semibold"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                }`
              }
            >
              <Icon className="h-4 w-4" />
              <span>{link.label}</span>
            </NavLink>
          )
        })}
      </nav>

      {/* Footer Guardrail Badge */}
      <div className="p-4 m-3 rounded-xl bg-muted/60 border border-border space-y-2">
        <div className="flex items-center gap-1.5 text-xs font-semibold text-foreground">
          <Shield className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
          <span>Anchor Guardrails</span>
        </div>
        <p className="text-[11px] text-muted-foreground leading-snug">
          On-chain spend caps physically prevent infinite drain or compromise.
        </p>
      </div>
    </aside>
  )
}
