import { createSlice, createAsyncThunk, PayloadAction } from "@reduxjs/toolkit"
import type { AgentSnapshot, AgentEvent, Voucher, WalletBalance } from "@/lib/types"
import { fetchAgentSnapshot, toggleEmergencyPause, triggerPaymentSimulation } from "@/lib/api"
import type { RootState } from "../index"

export interface AgentState {
  data: AgentSnapshot | null
  loading: boolean
  error: string | null
  lastUpdated: number | null
  isPausing: boolean
  isSimulating: boolean
}

const initialState: AgentState = {
  data: null,
  loading: true,
  error: null,
  lastUpdated: null,
  isPausing: false,
  isSimulating: false,
}

// Async Thunks
export const fetchAgentData = createAsyncThunk(
  "agent/fetchAgentData",
  async (_, { rejectWithValue }) => {
    try {
      const snapshot = await fetchAgentSnapshot()
      return snapshot
    } catch (err) {
      return rejectWithValue(
        err instanceof Error ? err.message : "Failed to communicate with Agent RPC"
      )
    }
  }
)

export const togglePauseAction = createAsyncThunk(
  "agent/togglePause",
  async (paused: boolean, { dispatch, rejectWithValue }) => {
    try {
      const result = await toggleEmergencyPause(paused)
      // Refetch snapshot to guarantee synchronization with backend/simulation state
      await dispatch(fetchAgentData()).unwrap()
      return result
    } catch (err) {
      return rejectWithValue(
        err instanceof Error ? err.message : "Failed to toggle emergency pause"
      )
    }
  }
)

export const simulatePaymentAction = createAsyncThunk<Voucher, number | undefined>(
  "agent/simulatePayment",
  async (lamports = 50_000, { dispatch, rejectWithValue }) => {
    try {
      const voucher = await triggerPaymentSimulation(lamports)
      // Refetch snapshot to guarantee all lists, events, and totals are updated
      await dispatch(fetchAgentData()).unwrap()
      return voucher
    } catch (err) {
      return rejectWithValue(
        err instanceof Error ? err.message : "Failed to simulate voucher payment"
      )
    }
  }
)

export const agentSlice = createSlice({
  name: "agent",
  initialState,
  reducers: {
    clearError: (state) => {
      state.error = null
    },
    addManualEvent: (state, action: PayloadAction<AgentEvent>) => {
      if (state.data) {
        state.data.events.unshift(action.payload)
      }
    },
    addVoucher: (state, action: PayloadAction<Voucher>) => {
      if (state.data) {
        state.data.vouchers.unshift(action.payload)
      }
    },
    updateBalances: (state, action: PayloadAction<WalletBalance[]>) => {
      if (state.data) {
        state.data.balances = action.payload
      }
    },
    setLocalPaused: (state, action: PayloadAction<boolean>) => {
      if (state.data) {
        state.data.agent.isPaused = action.payload
      }
    },
  },
  extraReducers: (builder) => {
    // fetchAgentData
    builder.addCase(fetchAgentData.pending, (state) => {
      // Keep loading indicator true on initial load
      if (!state.data) {
        state.loading = true
      }
    })
    builder.addCase(fetchAgentData.fulfilled, (state, action) => {
      state.data = action.payload
      state.loading = false
      state.error = null
      state.lastUpdated = Date.now()
    })
    builder.addCase(fetchAgentData.rejected, (state, action) => {
      state.loading = false
      state.error = (action.payload as string) || action.error.message || "RPC connection failure"
    })

    // togglePauseAction
    builder.addCase(togglePauseAction.pending, (state) => {
      state.isPausing = true
    })
    builder.addCase(togglePauseAction.fulfilled, (state, action) => {
      state.isPausing = false
      if (state.data) {
        state.data.agent.isPaused = action.payload
      }
    })
    builder.addCase(togglePauseAction.rejected, (state, action) => {
      state.isPausing = false
      state.error = (action.payload as string) || "Failed to toggle circuit breaker"
    })

    // simulatePaymentAction
    builder.addCase(simulatePaymentAction.pending, (state) => {
      state.isSimulating = true
    })
    builder.addCase(simulatePaymentAction.fulfilled, (state) => {
      state.isSimulating = false
    })
    builder.addCase(simulatePaymentAction.rejected, (state, action) => {
      state.isSimulating = false
      state.error = (action.payload as string) || "Failed to simulate payment"
    })
  },
})

// Action Creators
export const {
  clearError,
  addManualEvent,
  addVoucher,
  updateBalances,
  setLocalPaused,
} = agentSlice.actions

// Selectors
export const selectAgentSnapshot = (state: RootState) => state.agent.data
export const selectAgentInfo = (state: RootState) => state.agent.data?.agent
export const selectBalances = (state: RootState) => state.agent.data?.balances ?? []
export const selectVouchers = (state: RootState) => state.agent.data?.vouchers ?? []
export const selectDecisions = (state: RootState) => state.agent.data?.decisions ?? []
export const selectPayments = (state: RootState) => state.agent.data?.payments ?? []
export const selectEvents = (state: RootState) => state.agent.data?.events ?? []
export const selectIsPaused = (state: RootState) => state.agent.data?.agent?.isPaused ?? false
export const selectAgentLoading = (state: RootState) => state.agent.loading
export const selectAgentError = (state: RootState) => state.agent.error
export const selectAgentLastUpdated = (state: RootState) => state.agent.lastUpdated
export const selectIsSimulating = (state: RootState) => state.agent.isSimulating
export const selectIsPausing = (state: RootState) => state.agent.isPausing

export default agentSlice.reducer
