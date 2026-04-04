// Header Component
// ================

interface HeaderProps {
  onMenuToggle: () => void
}

export default function Header({ onMenuToggle }: HeaderProps) {
  return (
    <header className="header">
      <div className="header-content">
        <button className="menu-button" onClick={onMenuToggle}>
          ☰
        </button>
        <h1 className="header-title">NetShaper</h1>
        <div className="header-status">
          <span className="status-indicator online">● Online</span>
        </div>
      </div>
    </header>
  )
}
