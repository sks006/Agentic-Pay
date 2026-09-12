import * as React from "react"
import { ResponsiveContainer } from "recharts"
import { cn } from "@/lib/utils"

export type ChartConfig = Record<
  string,
  {
    label?: React.ReactNode
    icon?: React.ComponentType
    color?: string
  }
>

const ChartContext = React.createContext<{ config: ChartConfig } | null>(null)

export function useChart() {
  const context = React.useContext(ChartContext)
  if (!context) {
    throw new Error("useChart must be used within a <ChartContainer />")
  }
  return context
}

export const ChartContainer = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div"> & {
    config: ChartConfig
    children: React.ComponentProps<typeof ResponsiveContainer>["children"]
  }
>(({ id, className, children, config, ...props }, ref) => {
  const uniqueId = React.useId()
  const chartId = `chart-${id || uniqueId.replace(/:/g, "")}`

  return (
    <ChartContext.Provider value={{ config }}>
      <div
        data-chart={chartId}
        ref={ref}
        className={cn(
          "flex aspect-video justify-center text-xs [&_.recharts-cartesian-axis-tick_text]:fill-muted-foreground [&_.recharts-cartesian-grid_line[stroke='#ccc']]:stroke-border/40 [&_.recharts-curve.recharts-tooltip-cursor]:stroke-border [&_.recharts-dot[stroke='#fff']]:stroke-transparent [&_.recharts-layer]:outline-none [&_.recharts-polar-grid_[stroke='#ccc']]:stroke-border [&_.recharts-radial-bar-background-sector]:fill-muted [&_.recharts-rectangle.recharts-tooltip-cursor]:fill-muted [&_.recharts-reference-line_[stroke='#ccc']]:stroke-border [&_.recharts-sector[stroke='#fff']]:stroke-transparent [&_.recharts-sector]:outline-none [&_.recharts-surface]:outline-none",
          className
        )}
        {...props}
      >
        <ChartStyle id={chartId} config={config} />
        <ResponsiveContainer width="100%" height="100%">
          {children}
        </ResponsiveContainer>
      </div>
    </ChartContext.Provider>
  )
})
ChartContainer.displayName = "ChartContainer"

const ChartStyle = ({ id, config }: { id: string; config: ChartConfig }) => {
  const colorConfig = Object.entries(config).filter(
    ([_, config]) => config.color
  )

  if (!colorConfig.length) {
    return null
  }

  return (
    <style
      dangerouslySetInnerHTML={{
        __html: `
[data-chart="${id}"] {
${colorConfig
  .map(([key, itemConfig]) => {
    return `  --color-${key}: ${itemConfig.color};`
  })
  .join("\n")}
}
`,
      }}
    />
  )
}

export function ChartTooltip({
  active,
  payload,
  label,
  content,
  ...props
}: any) {
  if (content) {
    return React.cloneElement(content, { active, payload, label, ...props })
  }
  return <ChartTooltipContent active={active} payload={payload} label={label} {...props} />
}

export function ChartTooltipContent({
  active,
  payload,
  className,
  indicator = "dot",
  hideLabel = false,
  label,
}: {
  active?: boolean
  payload?: any[]
  className?: string
  indicator?: "dot" | "line" | "dashed"
  hideLabel?: boolean
  label?: string
}) {
  const { config } = useChart()

  if (!active || !payload?.length) {
    return null
  }

  return (
    <div
      className={cn(
        "grid min-w-[8rem] items-start gap-1.5 rounded-lg border border-border/80 bg-background/95 px-3 py-2 text-xs shadow-xl backdrop-blur-md",
        className
      )}
    >
      {!hideLabel && label && (
        <div className="font-mono text-[11px] font-medium text-muted-foreground">
          {label}
        </div>
      )}
      <div className="grid gap-1.5">
        {payload.map((item: any, index: number) => {
          const key = item.dataKey || item.name || "value"
          const itemConfig = config[key] || {}
          const indicatorColor = item.color || item.fill || "var(--chart-1)"

          return (
            <div
              key={index}
              className="flex w-full items-center justify-between gap-4"
            >
              <div className="flex items-center gap-1.5">
                {indicator === "dot" && (
                  <div
                    className="h-2 w-2 rounded-full"
                    style={{ backgroundColor: indicatorColor }}
                  />
                )}
                <span className="text-muted-foreground">
                  {itemConfig.label || item.name || key}
                </span>
              </div>
              <span className="font-mono font-semibold text-foreground">
                {typeof item.value === "number"
                  ? `$${item.value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
                  : item.value}
              </span>
            </div>
          )
        })}
      </div>
    </div>
  )
}
