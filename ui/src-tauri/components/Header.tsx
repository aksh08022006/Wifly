// Header Component
// ================

interface HeaderProps {
  onMenuToggle: () => void
  onSettingsClick: () => void
}

export default function Header({ onMenuToggle, onSettingsClick }: HeaderProps) {
  return (
    <header className="header">
      <div className="header-content">
        <button className="menu-button" onClick={onMenuToggle}>
          ☰
        </button>
        <h1 className="header-title">NetShaper</h1>
        <div className="header-status">
          <span className="status-indicator online">● Online</span>
          <button className="settings-button" onClick={onSettingsClick} title="Server Settings">
            ⚙️
          </button>
        </div>
      </div>
    </header>
  )
}
