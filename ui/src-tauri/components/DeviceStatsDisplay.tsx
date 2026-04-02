// DeviceStatsDisplay Component - M5 Phase 5 Real-time Bandwidth Metrics
// ======================================================================

import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

interface DeviceStats {
  ip: string
  current_usage: number
  peak_usage: number
  total_consumption: number
  bandwidth_limit: number
}

interface DeviceStatsDisplayProps {
  deviceIp: string
  onRefresh?: () => void
}

// Helper to format bytes to human-readable format
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// Helper to format bytes/sec to bandwidth format
const formatBandwidth = (bytesPerSec: number): string => {
  return formatBytes(bytesPerSec) + '/s'
}

export default function DeviceStatsDisplay({ deviceIp, onRefresh }: DeviceStatsDisplayProps) {
  const [stats, setStats] = useState<DeviceStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [autoRefresh, setAutoRefresh] = useState(true)

  // Fetch stats for this device
  const fetchStats = async () => {
    try {
      setLoading(true)
      setError(null)
      // Invoke the get_device_stats command from Tauri backend
      const response = await invoke<[number, number, number]>('get_device_stats', { 
        ip: deviceIp 
      })
      
      if (response && response.length === 3) {
        setStats({
          ip: deviceIp,
          current_usage: response[0],
          peak_usage: response[1],
          total_consumption: response[2],
          bandwidth_limit: 0 // Will be fetched separately if needed
        })
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      setError(`Failed to fetch stats: ${errorMsg}`)
      console.error('Stats fetch error:', err)
    } finally {
      setLoading(false)
    }
  }

  // Initial fetch
  useEffect(() => {
    fetchStats()
  }, [deviceIp])

  // Set up auto-refresh interval
  useEffect(() => {
    if (!autoRefresh) return

    const interval = setInterval(() => {
      fetchStats()
      onRefresh?.()
    }, 1000) // Refresh every 1 second

    return () => clearInterval(interval)
  }, [autoRefresh, deviceIp, onRefresh])

  if (loading && !stats) {
    return (
      <div className="device-stats-container">
        <div className="stats-loading">⏳ Loading stats...</div>
      </div>
    )
  }

  if (error && !stats) {
    return (
      <div className="device-stats-container">
        <div className="stats-error">❌ {error}</div>
        <button className="btn-retry" onClick={fetchStats}>Retry</button>
      </div>
    )
  }

  if (!stats) {
    return (
      <div className="device-stats-container">
        <div className="stats-empty">No stats available</div>
      </div>
    )
  }

  // Calculate usage percentage
  const usagePercent = stats.bandwidth_limit > 0 
    ? ((stats.current_usage / stats.bandwidth_limit) * 100).toFixed(1)
    : 'N/A'

  return (
    <div className="device-stats-container">
      <div className="stats-header">
        <h3>📊 Bandwidth Statistics</h3>
        <div className="stats-controls">
          <label className="auto-refresh-toggle">
            <input 
              type="checkbox" 
              checked={autoRefresh} 
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            <span>Auto-refresh</span>
          </label>
          <button className="btn-refresh" onClick={fetchStats} disabled={loading}>
            {loading ? '⏳' : '🔄'} Refresh
          </button>
        </div>
      </div>

      {error && (
        <div className="stats-warning">⚠️ {error}</div>
      )}

      <div className="stats-grid">
        {/* Current Usage */}
        <div className="stat-card current-usage">
          <div className="stat-label">Current Usage</div>
          <div className="stat-value">{formatBandwidth(stats.current_usage)}</div>
          <div className="stat-sublabel">Active bandwidth right now</div>
        </div>

        {/* Peak Usage */}
        <div className="stat-card peak-usage">
          <div className="stat-label">Peak Usage</div>
          <div className="stat-value">{formatBandwidth(stats.peak_usage)}</div>
          <div className="stat-sublabel">Highest rate observed</div>
        </div>

        {/* Total Consumption */}
        <div className="stat-card total-consumption">
          <div className="stat-label">Total Consumption</div>
          <div className="stat-value">{formatBytes(stats.total_consumption)}</div>
          <div className="stat-sublabel">All-time data transferred</div>
        </div>

        {/* Usage Percentage */}
        <div className="stat-card usage-percent">
          <div className="stat-label">Usage %</div>
          <div className="stat-value">{usagePercent}%</div>
          <div className="stat-sublabel">Current vs limit</div>
        </div>
      </div>

      {/* Progress visualization */}
      <div className="stats-progress">
        <div className="progress-label">
          <span>Current Usage vs Limit</span>
          <span className="progress-percent">{usagePercent}%</span>
        </div>
        <div className="progress-bar-container">
          <div className="progress-bar">
            <div 
              className={`progress-fill ${parseFloat(String(usagePercent)) > 80 ? 'warning' : parseFloat(String(usagePercent)) > 95 ? 'critical' : ''}`}
              style={{ width: `${Math.min(parseFloat(String(usagePercent)), 100)}%` }}
            ></div>
          </div>
        </div>
        <div className="progress-info">
          <span>Current: {formatBandwidth(stats.current_usage)}</span>
          <span>Limit: {formatBandwidth(stats.bandwidth_limit)}</span>
        </div>
      </div>

      {/* Usage trend indicator */}
      <div className="stats-trend">
        <div className="trend-item">
          <span className="trend-label">Trend</span>
          <span className="trend-indicator">
            {stats.current_usage > stats.peak_usage * 0.9 ? '📈 High' : 
             stats.current_usage > stats.peak_usage * 0.5 ? '➡️ Medium' : 
             '📉 Low'}
          </span>
        </div>
        <div className="trend-item">
          <span className="trend-label">Status</span>
          <span className={`trend-indicator ${usagePercent === 'N/A' || parseFloat(String(usagePercent)) < 100 ? 'active' : 'warning'}`}>
            {usagePercent === 'N/A' || parseFloat(String(usagePercent)) < 100 ? '✓ Active' : '⚠️ Warning'}
          </span>
        </div>
      </div>
    </div>
  )
}
