// Sidebar Component
// =================

interface SidebarProps {
  open: boolean
}

export default function Sidebar({ open }: SidebarProps) {
  return (
    <aside className={`sidebar ${open ? 'open' : 'closed'}`}>
      <nav className="sidebar-nav">
        <div className="nav-section">
          <h3 className="nav-title">Dashboard</h3>
          <a href="#devices" className="nav-item active">
            📱 Devices
          </a>
          <a href="#bandwidth" className="nav-item">
            📊 Bandwidth
          </a>
        </div>
        <div className="nav-section">
          <h3 className="nav-title">Management</h3>
          <a href="#approve" className="nav-item">
            ✓ Pending Approvals
          </a>
          <a href="#blocked" className="nav-item">
            ✗ Blocked Devices
          </a>
        </div>
        <div className="nav-section">
          <h3 className="nav-title">Settings</h3>
          <a href="#settings" className="nav-item">
            ⚙️ Preferences
          </a>
          <a href="#logs" className="nav-item">
            📝 Activity Logs
          </a>
        </div>
      </nav>
    </aside>
  )
}
