// DeviceList Component - With Action Handlers
// ============================================

import React from 'react'
import DeviceCard from './DeviceCard'

interface DeviceInfo {
  ip: string
  hostname: string | null
  approved: boolean
  enrolled_at: string
  bandwidth_limit: number
  current_usage: number
}

interface DeviceListProps {
  devices: DeviceInfo[]
  onSelectDevice: (device: DeviceInfo) => void
  onApprove?: (ip: string) => Promise<void>
  onDeny?: (ip: string) => Promise<void>
}

export default function DeviceList({ devices, onSelectDevice, onApprove, onDeny }: DeviceListProps) {
  const approvedDevices = devices.filter((d) => d.approved)
  const pendingDevices = devices.filter((d) => !d.approved)

  return (
    <div className="device-list">
      <div className="list-header">
        <h2>Connected Devices</h2>
        <div className="list-stats">
          <span className="stat-badge approved">{approvedDevices.length} Approved</span>
          <span className="stat-badge pending">{pendingDevices.length} Pending</span>
        </div>
      </div>

      {approvedDevices.length > 0 && (
        <section className="device-section">
          <h3 className="section-title">Approved Devices</h3>
          <div className="device-grid">
            {approvedDevices.map((device) => (
              <DeviceCard 
                key={device.ip} 
                device={device} 
                onSelect={onSelectDevice}
                onApprove={onApprove}
                onDeny={onDeny}
              />
            ))}
          </div>
        </section>
      )}

      {pendingDevices.length > 0 && (
        <section className="device-section">
          <h3 className="section-title">Pending Approval</h3>
          <div className="device-grid">
            {pendingDevices.map((device) => (
              <DeviceCard 
                key={device.ip} 
                device={device} 
                onSelect={onSelectDevice}
                onApprove={onApprove}
                onDeny={onDeny}
              />
            ))}
          </div>
        </section>
      )}

      {devices.length === 0 && <div className="empty-state">No devices connected</div>}
    </div>
  )
}
