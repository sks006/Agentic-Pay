import React from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"

interface StatCardProps {
  title: string
  value: string | number
  subtitle?: string
  icon?: React.ReactNode
  trend?: {
    value: string
    isPositive: boolean
  }
  badge?: string
  className?: string
}

export function StatCard({
  title,
  value,
  subtitle,
  icon,
  trend,
  badge,
  className,
}: StatCardProps) {
  return (
    <Card className={cn("relative overflow-hidden group border-border hover:border-primary/40 transition-all duration-300", className)}>
      <div className="absolute inset-x-0 -top-px h-px bg-gradient-to-r from-transparent via-primary/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-xs font-semibold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
          {icon}
          {title}
        </CardTitle>
        {badge && (
          <Badge variant="secondary" className="text-[10px] font-medium font-mono px-2 py-0.5">
            {badge}
          </Badge>
        )}
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold font-mono tracking-tight text-foreground flex items-baseline gap-2">
          {value}
          {trend && (
            <span
              className={cn(
                "text-xs font-semibold font-sans flex items-center",
                trend.isPositive ? "text-emerald-600 dark:text-emerald-400" : "text-rose-600 dark:text-rose-400"
              )}
            >
              {trend.isPositive ? "↑" : "↓"} {trend.value}
            </span>
          )}
        </div>
        {subtitle && (
          <p className="text-[11px] text-muted-foreground mt-1 flex items-center justify-between">
            <span>{subtitle}</span>
          </p>
        )}
      </CardContent>
    </Card>
  )
}
