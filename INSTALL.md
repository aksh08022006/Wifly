# NetShaper Installation Guide

## 🪟 Windows Installation (Recommended)

### **Step 1: Download**
1. Go to [NetShaper Releases](https://github.com/aksh08022006/Wifly/releases/latest)
2. Download `NetShaper-X.X.X.msi` (e.g., `NetShaper-0.2.0.msi`)
3. File size: ~60-100 MB

### **Step 2: Run Installer**
1. Double-click the downloaded `.msi` file
2. Click "Next" on the welcome screen
3. Accept the license terms
4. Choose installation location (default: `C:\Program Files\NetShaper`)
5. Click "Install"
6. Wait 1-2 minutes for installation
7. Click "Finish"

### **Step 3: Launch NetShaper**
**Option A: Start Menu**
- Open Start Menu
- Search "NetShaper"
- Click to launch

**Option B: Desktop Shortcut**
- Double-click the NetShaper icon on desktop

### **Step 4: Administrator Access**
- You'll see a prompt asking for administrator access
- Click "Yes" to allow (required for kernel integration)
- The daemon starts automatically in background
- Desktop dashboard appears after 2-3 seconds

### ⚡ **First Time Setup**
1. Dashboard opens showing "No Devices Connected"
2. Connect a device to your network (phone, laptop, etc.)
3. Device appears in the list after 5-10 seconds
4. Click "Approve" to allow it on network
5. Set bandwidth limit (e.g., 10 MB/s)
6. Stats update in real-time!

### 🔧 **Uninstall**
- Control Panel → Programs → Programs and Features
- Find "NetShaper"
- Click "Uninstall"
- Confirm when prompted
- Daemon automatically stops

---

## 🍎 macOS Installation

### **Step 1: Download**
1. Go to [NetShaper Releases](https://github.com/aksh08022006/Wifly/releases/latest)
2. Download `NetShaper.dmg` (~80 MB)
3. Wait for download to complete

### **Step 2: Install**
1. Double-click `NetShaper.dmg` to mount it
2. Drag "NetShaper" icon to "Applications" folder
3. Wait 1-2 minutes for copy to complete
4. Eject the DMG file

### **Step 3: Launch**
1. Open Applications folder
2. Find "NetShaper"
3. Right-click → "Open" (needed first time for unsigned app)
4. Click "Open" when security prompt appears

### **Step 4: Admin Access**
- Enter your password when prompted
- Required for kernel-level packet filtering
- Daemon runs in background after this

### 🔧 **Uninstall**
- Open Applications folder
- Right-click NetShaper
- Select "Move to Trash"
- Empty trash

---

## 🐧 Linux Installation

### **Ubuntu/Debian**

#### **Step 1: Download**
1. Go to [NetShaper Releases](https://github.com/aksh08022006/Wifly/releases/latest)
2. Download `NetShaper-X.X.X.AppImage` (~90 MB)

#### **Step 2: Make Executable**
```bash
cd ~/Downloads
chmod +x NetShaper-*.AppImage
```

#### **Step 3: Run**
```bash
sudo ./NetShaper-*.AppImage
```

Or create desktop shortcut:
```bash
# Copy to Applications
sudo cp NetShaper-*.AppImage /usr/local/bin/netshaper

# Make symlink for easy access
sudo ln -s /usr/local/bin/netshaper ~/Desktop/NetShaper
```

#### **Step 4: Admin Access**
- Enter sudo password when prompted
- Daemon starts automatically
- Dashboard appears

### **Fedora/RHEL**

```bash
# Download & make executable (same as Ubuntu)
chmod +x NetShaper-*.AppImage

# Run with sudo
sudo ./NetShaper-*.AppImage
```

### 🔧 **Uninstall**
```bash
# If installed to /usr/local/bin
sudo rm /usr/local/bin/netshaper
```

---

## ✅ System Requirements Check

Before installing, verify your system:

### **Windows**
- ✓ Windows 10 or later (64-bit only)
- ✓ 200 MB free disk space
- ✓ Administrator account access

**Check your version:**
1. Press `Win + R`
2. Type `winver`
3. Check Windows version (should be 10+)

### **macOS**
- ✓ macOS 10.15 (Catalina) or later
- ✓ 200 MB free disk space
- ✓ Administrator access

**Check your version:**
1. Click Apple menu → "About This Mac"
2. Check macOS version (should be 10.15+)

### **Linux**
- ✓ Ubuntu 20.04+ / Fedora 32+ / Debian 10+
- ✓ 200 MB free disk space
- ✓ sudo access

**Check version:**
```bash
lsb_release -a  # Ubuntu/Debian
cat /etc/redhat-release  # Fedora/RHEL
```

---

## 🆘 Troubleshooting

### **"Admin access required" prompt won't go away**
- Make sure you entered your password correctly
- Try closing and reopening the app
- Restart your computer

### **Dashboard won't load**
- Check if daemon is running
  - Windows: Task Manager → search "daemon"
  - Mac/Linux: `ps aux | grep daemon`
- Try closing and reopening the app
- Check you have 200+ MB free disk space

### **No devices appearing**
- Make sure device is connected to same network
- Device takes 5-10 seconds to appear
- Try reconnecting the device
- Check if firewall is blocking (ask admin)

### **Can't uninstall on Windows**
- Close the NetShaper app first
- Go to Control Panel → Programs
- Look for "NetShaper" in the list
- If not there, delete: `C:\Program Files\NetShaper\`

### **Error: "Pipe not found"**
- Make sure daemon is running first
- Try closing UI and reopening it
- Restart your computer if problem persists

---

## 📊 Verify Installation Worked

### **Windows**
1. Open NetShaper from Start Menu
2. Wait 3 seconds
3. Dashboard with "NetShaper" title appears ✅
4. No error messages ✅

### **macOS/Linux**
1. Run: `./NetShaper-*.AppImage`
2. Dashboard appears after 2-3 seconds ✅
3. Check daemon process: `ps aux | grep daemon` ✅

---

## 🔗 Get Help

- **Bug Report:** [GitHub Issues](https://github.com/aksh08022006/Wifly/issues)
- **Website:** [NetShaper](https://aksh08022006.github.io/Wifly/)
- **GitHub:** [Source Code](https://github.com/aksh08022006/Wifly)

---

## 📝 Uninstall Guide

### **Complete Cleanup (Advanced Users)**

**Windows:**
```batch
# Remove application files
rmdir /s C:\Program Files\NetShaper

# Remove from registry (only if needed)
reg delete "HKCU\Software\NetShaper" /f
```

**macOS:**
```bash
# Remove app
rm -rf /Applications/NetShaper.app

# Remove config (if exists)
rm -rf ~/.config/netshaper
```

**Linux:**
```bash
# Remove executable
sudo rm /usr/local/bin/netshaper

# Remove config
rm -rf ~/.config/netshaper
```

