// DeviceCard Component
// ====================

import React from 'react'

interface DeviceInfo {
  ip: string
  hostname: string | null
  approved: boolean
  enrolled_at: string
  bandwidth_limit: number
  current_usage: number
}

interface DeviceCardProps {
  device: DeviceInfo
  onSelect: (device: DeviceInfo) => void
}

export default function DeviceCard({ device, onSelect }: DeviceCardProps) {
  const bandwidthMB = (device.bandwidth_limit / 1_000_000).toFixed(1)
  const usageMB = (device.current_usage / 1_000_000).toFixed(1)
  const usagePercent = device.bandwidth_limit > 0 ? ((device.current_usage / device.bandwidth_limit) * 100).toFixed(0) : 0

  return (
    <div className={`device-card ${device.approved ? 'approved' : 'pending'}`} onClick={() => onSelect(device)}>
      <div className="card-header">
        <div className="device-icon">📱</div>
        <div className="device-info">
          <h4 className="device-name">{device.hostname || device.ip}</h4>
          <p className="device-ip">{device.ip}</p>
        </div>
        <div className={`status-badge ${device.approved ? 'approved' : 'pending'}`}>
          {device.approved ? '✓ Approved' : '⏳ Pending'}
        </div>
      </div>

      <div className="card-bandwidth">
        <div className="bandwidth-label">
          <span>Bandwidth: {usageMB} MB/s / {bandwidthMB} MB/s</span>
          <span className="usage-percent">{usagePercent}%</span>
        </div>
        <div className="progress-bar">
          <div className="progress-fill" style={{ width: `${usagePercent}%` }}></div>
        </div>
      </div>

      <div className="card-footer">
        <small className="enrolled-date">Enrolled: {new Date(device.enrolled_at).toLocaleDateString()}</small>
      </div>
    </div>
  )
}
