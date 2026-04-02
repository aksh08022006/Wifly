// Main App Component - M5 Dashboard
// ==================================

import React, { useState, useEffect } from 'react'
import DeviceList from './components/DeviceList'
import Header from './components/Header'
import Sidebar from './components/Sidebar'
import './styles/App.css'

interface DeviceInfo {
  ip: string
  hostname: string | null
  approved: boolean
  enrolled_at: string
  bandwidth_limit: number
  current_usage: number
}

export default function App() {
  const [devices, setDevices] = useState<DeviceInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedDevice, setSelectedDevice] = useState<DeviceInfo | null>(null)
  const [sidebarOpen, setSidebarOpen] = useState(true)

  // Mock data - will be replaced with daemon IPC in Phase 3
  useEffect(() => {
    setTimeout(() => {
      setDevices([
        {
          ip: '192.168.1.100',
          hostname: 'iPhone-12',
          approved: true,
          enrolled_at: '2026-04-03T15:30:45Z',
          bandwidth_limit: 10_000_000,
          current_usage: 2_500_000,
        },
        {
          ip: '192.168.1.101',
          hostname: 'MacBook-Pro',
          approved: false,
          enrolled_at: '2026-04-03T16:00:00Z',
          bandwidth_limit: 0,
          current_usage: 0,
        },
        {
          ip: '192.168.1.102',
          hostname: 'iPad-Air',
          approved: true,
          enrolled_at: '2026-04-02T10:15:30Z',
          bandwidth_limit: 5_000_000,
          current_usage: 1_200_000,
        },
      ])
      setLoading(false)
    }, 500)
  }, [])

  return (
    <div className="app-container">
      <Header onMenuToggle={() => setSidebarOpen(!sidebarOpen)} />
      <div className="main-content">
        <Sidebar open={sidebarOpen} />
        <div className="content-area">
          {loading ? (
            <div className="loading">Loading devices...</div>
          ) : (
            <DeviceList devices={devices} onSelectDevice={setSelectedDevice} />
          )}
        </div>
      </div>
    </div>
  )
}
