import { useEffect, useState } from "react"
import type { PythPrice } from "@/lib/types"
import { fetchPythPrice } from "@/lib/api"

export function usePythProxy(symbol: string = "SOL") {
  const [price, setPrice] = useState<PythPrice | null>(null)
  const [history, setHistory] = useState<Array<{ time: string; price: number; confidence: number }>>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let active = true

    async function loadPrice() {
      try {
        const data = await fetchPythPrice(symbol)
        if (active) {
          setPrice(data)
          const nowStr = new Date().toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
          })
          setHistory((prev) => {
            const next = [...prev, { time: nowStr, price: data.price, confidence: data.confidence }]
            return next.slice(-20) // Keep last 20 tick points
          })
        }
      } finally {
        if (active) {
          setLoading(false)
        }
      }
    }

    void loadPrice()
    const interval = window.setInterval(loadPrice, 2000)

    return () => {
      active = false
      window.clearInterval(interval)
    }
  }, [symbol])

  return {
    price,
    history,
    loading,
  }
}
