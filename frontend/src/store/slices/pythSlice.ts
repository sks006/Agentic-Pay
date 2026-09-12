import { createSlice, createAsyncThunk, PayloadAction } from "@reduxjs/toolkit"
import type { PythPrice } from "@/lib/types"
import { fetchPythPrice } from "@/lib/api"
import type { RootState } from "../index"

export interface PythState {
  prices: Record<string, PythPrice>
  loading: Record<string, boolean>
  errors: Record<string, string | null>
  lastUpdated: Record<string, number>
}

const initialState: PythState = {
  prices: {},
  loading: {},
  errors: {},
  lastUpdated: {},
}

// Async Thunk
export const fetchPythPriceThunk = createAsyncThunk(
  "pyth/fetchPythPrice",
  async (symbol: string, { rejectWithValue }) => {
    try {
      const priceData = await fetchPythPrice(symbol)
      return { symbol: symbol.toUpperCase(), data: priceData }
    } catch (err) {
      return rejectWithValue({
        symbol: symbol.toUpperCase(),
        error: err instanceof Error ? err.message : `Failed to fetch price for ${symbol}`,
      })
    }
  }
)

export const pythSlice = createSlice({
  name: "pyth",
  initialState,
  reducers: {
    setManualPrice: (state, action: PayloadAction<PythPrice>) => {
      const sym = action.payload.symbol.toUpperCase()
      state.prices[sym] = action.payload
      state.lastUpdated[sym] = Date.now()
    },
  },
  extraReducers: (builder) => {
    builder.addCase(fetchPythPriceThunk.pending, (state, action) => {
      const sym = action.meta.arg.toUpperCase()
      state.loading[sym] = true
    })
    builder.addCase(fetchPythPriceThunk.fulfilled, (state, action) => {
      const { symbol, data } = action.payload
      state.prices[symbol] = data
      state.loading[symbol] = false
      state.errors[symbol] = null
      state.lastUpdated[symbol] = Date.now()
    })
    builder.addCase(fetchPythPriceThunk.rejected, (state, action) => {
      const sym = action.meta.arg.toUpperCase()
      state.loading[sym] = false
      const payload = action.payload as { symbol: string; error: string } | undefined
      state.errors[sym] = payload?.error || action.error.message || "Failed to fetch price"
    })
  },
})

export const { setManualPrice } = pythSlice.actions

// Selectors
export const selectPythPrices = (state: RootState) => state.pyth.prices
export const selectPythPrice = (symbol: string) => (state: RootState) =>
  state.pyth.prices[symbol.toUpperCase()] || null
export const selectPythLoading = (symbol: string) => (state: RootState) =>
  !!state.pyth.loading[symbol.toUpperCase()]
export const selectPythError = (symbol: string) => (state: RootState) =>
  state.pyth.errors[symbol.toUpperCase()] || null

export default pythSlice.reducer
