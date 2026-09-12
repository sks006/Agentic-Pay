import { useEffect, useState, useMemo } from "react"
import { useAppDispatch, useAppSelector } from "@/store"
import { fetchPythPriceThunk, selectPythPrice, selectPythLoading } from "@/store"

export function usePythProxy(symbol: string = "SOL") {
  const dispatch = useAppDispatch()
  const upperSymbol = symbol.toUpperCase()

  const priceSelector = useMemo(() => selectPythPrice(upperSymbol), [upperSymbol])
  const loadingSelector = useMemo(() => selectPythLoading(upperSymbol), [upperSymbol])

  const price = useAppSelector(priceSelector)
  const loading = useAppSelector(loadingSelector)

  const [history, setHistory] = useState<Array<{ time: string; price: number; confidence: number }>>([])

  useEffect(() => {
    let active = true

    const loadPrice = async () => {
      try {
        const result = await dispatch(fetchPythPriceThunk(upperSymbol)).unwrap()
        if (active && result?.data) {
          const nowStr = new Date().toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
          })
          setHistory((prev) => {
            const next = [
              ...prev,
              { time: nowStr, price: result.data.price, confidence: result.data.confidence },
            ]
            return next.slice(-20)
          })
        }
      } catch {
        // Handled by Redux
      }
    }

    void loadPrice()
    const interval = window.setInterval(loadPrice, 2000)

    return () => {
      active = false
      window.clearInterval(interval)
    }
  }, [dispatch, upperSymbol])

  return {
    price,
    history,
    loading,
  }
}
