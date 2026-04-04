import { useState } from 'react'
import '../styles/ServerSettings.css'

interface ServerSettingsProps {
  onClose: () => void
  onConnect: (host: string, port: number) => void
  defaultHost: string
  defaultPort: number
}

export default function ServerSettings({
  onClose,
  onConnect,
  defaultHost,
  defaultPort,
}: ServerSettingsProps) {
  const [host, setHost] = useState(defaultHost)
  const [port, setPort] = useState(defaultPort.toString())
  const [error, setError] = useState<string | null>(null)
  const [testing, setTesting] = useState(false)

  const handleConnect = async () => {
    if (!host.trim()) {
      setError('Please enter a server IP address')
      return
    }

    const portNum = parseInt(port)
    if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
      setError('Port must be between 1 and 65535')
      return
    }

    setTesting(true)
    setError(null)

    try {
      // Test connection to daemon
      const response = await fetch(`http://${host}:${portNum}/`, {
        method: 'GET',
        timeout: 5000,
      })

      if (response.ok || response.status === 404) {
        // 404 is OK, it means the server is responding
        onConnect(host, portNum)
        onClose()
      } else {
        setError(`Server returned error: ${response.status}`)
      }
    } catch (err) {
      setError(
        `Cannot reach server at ${host}:${port}. Make sure the daemon is running.`
      )
    } finally {
      setTesting(false)
    }
  }

  return (
    <div className="server-settings-modal">
      <div className="server-settings-content">
        <h2>Server Connection Settings</h2>
        <p>Enter your daemon server IP address and port</p>

        <div className="form-group">
          <label htmlFor="host">Server IP Address</label>
          <input
            id="host"
            type="text"
            placeholder="e.g., 172.17.44.89"
            value={host}
            onChange={(e) => setHost(e.target.value)}
            disabled={testing}
          />
        </div>

        <div className="form-group">
          <label htmlFor="port">Port</label>
          <input
            id="port"
            type="number"
            placeholder="8080"
            value={port}
            onChange={(e) => setPort(e.target.value)}
            min="1"
            max="65535"
            disabled={testing}
          />
        </div>

        {error && <div className="error-message">{error}</div>}

        <div className="button-group">
          <button
            className="btn-primary"
            onClick={handleConnect}
            disabled={testing}
          >
            {testing ? 'Testing Connection...' : 'Connect'}
          </button>
          <button className="btn-secondary" onClick={onClose} disabled={testing}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}
