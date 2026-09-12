import React from "react"
import { createBrowserRouter, Navigate, Outlet } from "react-router-dom"
import { AppShell } from "@/components/layout/AppShell"
import Dashboard from "@/pages/Dashboard"
import Payments from "@/pages/Payments"
import Prices from "@/pages/Prices"
import Decisions from "@/pages/Decisions"
import Settings from "@/pages/Settings"

function RootLayout() {
  return (
    <AppShell>
      <Outlet />
    </AppShell>
  )
}

export const router = createBrowserRouter([
  {
    path: "/",
    element: <RootLayout />,
    children: [
      {
        index: true,
        element: <Navigate to="/dashboard" replace />,
      },
      {
        path: "dashboard",
        element: <Dashboard />,
      },
      {
        path: "payments",
        element: <Payments />,
      },
      {
        path: "prices",
        element: <Prices />,
      },
      {
        path: "decisions",
        element: <Decisions />,
      },
      {
        path: "settings",
        element: <Settings />,
      },
    ],
  },
])

export default function App() {
  return null
}
