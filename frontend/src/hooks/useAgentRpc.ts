import { useCallback, useEffect, useState } from "react"
import type { AgentSnapshot, Voucher } from "@/lib/types"
import { fetchAgentSnapshot, toggleEmergencyPause, triggerPaymentSimulation } from "@/lib/api"

export function useAgentRpc() {
  const [data, setData] = useState<AgentSnapshot | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    try {
      const snapshot = await fetchAgentSnapshot()
      setData(snapshot)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to communicate with Agent RPC")
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void refresh()
    const interval = window.setInterval(refresh, 2500)
    return () => window.clearInterval(interval)
  }, [refresh])

  const setPaused = useCallback(async (paused: boolean) => {
    await toggleEmergencyPause(paused)
    void refresh()
  }, [refresh])

  const simulatePayment = useCallback(async (lamports?: number): Promise<Voucher> => {
    const vch = await triggerPaymentSimulation(lamports)
    void refresh()
    return vch
  }, [refresh])

  return {
    data,
    loading,
    error,
    refresh,
    setPaused,
    simulatePayment,
  }
}
