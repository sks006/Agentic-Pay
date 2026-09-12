import { useCallback, useEffect } from "react"
import type { Voucher } from "@/lib/types"
import { useAppDispatch, useAppSelector } from "@/store"
import {
  fetchAgentData,
  togglePauseAction,
  simulatePaymentAction,
  selectAgentSnapshot,
  selectAgentLoading,
  selectAgentError,
  selectIsPausing,
  selectIsSimulating,
} from "@/store"

export function useAgentRpc() {
  const dispatch = useAppDispatch()
  const data = useAppSelector(selectAgentSnapshot)
  const loading = useAppSelector(selectAgentLoading)
  const error = useAppSelector(selectAgentError)
  const isPausing = useAppSelector(selectIsPausing)
  const isSimulating = useAppSelector(selectIsSimulating)

  const refresh = useCallback(async () => {
    try {
      await dispatch(fetchAgentData()).unwrap()
    } catch {
      // Error handled by Redux state
    }
  }, [dispatch])

  useEffect(() => {
    // Initial fetch
    void refresh()
    // Periodic refresh
    const interval = window.setInterval(refresh, 2500)
    return () => window.clearInterval(interval)
  }, [refresh])

  const setPaused = useCallback(
    async (paused: boolean) => {
      return await dispatch(togglePauseAction(paused)).unwrap()
    },
    [dispatch]
  )

  const simulatePayment = useCallback(
    async (lamports?: number): Promise<Voucher> => {
      return await dispatch(simulatePaymentAction(lamports)).unwrap()
    },
    [dispatch]
  )

  return {
    data,
    loading,
    error,
    refresh,
    setPaused,
    simulatePayment,
    isPausing,
    isSimulating,
  }
}
