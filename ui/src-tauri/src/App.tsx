// Main App Component - M5 Phase 3 Dashboard with IPC
// ==================================================

import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import DeviceList from '../components/DeviceList'
import Header from '../components/Header'
import Sidebar from '../components/Sidebar'
import '../styles/App.css'

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
  const [error, setError] = useState<string | null>(null)
  const [sidebarOpen, setSidebarOpen] = useState(true)

  // Fetch devices from daemon via Tauri IPC
  const fetchDevices = async () => {
    try {
      setLoading(true)
      setError(null)
      const response = await invoke<DeviceInfo[]>('list_devices')
      setDevices(response)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      setError(`Failed to load devices: ${errorMsg}`)
      console.error('Device fetch error:', err)
      
      // Fallback to mock data for development
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
      ])
    } finally {
      setLoading(false)
    }
  }

  // Load devices on component mount
  useEffect(() => {
    fetchDevices()
  }, [])

  // Approve device handler
  const handleApproveDevice = async (ip: string) => {
    try {
      await invoke('approve_device', { ip })
      // Refresh device list
      await fetchDevices()
    } catch (err) {
      setError(`Failed to approve device: ${err}`)
    }
  }

  // Deny device handler
  const handleDenyDevice = async (ip: string) => {
    try {
      await invoke('deny_device', { ip })
      // Refresh device list
      await fetchDevices()
    } catch (err) {
      setError(`Failed to deny device: ${err}`)
    }
  }

  return (
    <div className="app-container">
      <Header onMenuToggle={() => setSidebarOpen(!sidebarOpen)} />
      <div className="main-content">
        <Sidebar open={sidebarOpen} />
        <div className="content-area">
          {error && (
            <div className="error-banner">
              {error}
              <button onClick={() => setError(null)}>✕</button>
            </div>
          )}
          {loading ? (
            <div className="loading">Loading devices...</div>
          ) : (
            <DeviceList 
              devices={devices} 
              onSelectDevice={() => {}}
              onApprove={handleApproveDevice}
              onDeny={handleDenyDevice}
            />
          )}
        </div>
      </div>
    </div>
  )
}
