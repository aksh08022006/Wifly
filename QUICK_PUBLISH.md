# 🚀 Quick Start: Publish NetShaper v1.0.0 Right Now

## ⏱️ Takes 20 Minutes Total

### **Step 1: Update Version (3 files)**

Open these files and change version to `1.0.0`:

**File 1:** `daemon/Cargo.toml`
```toml
[package]
version = "1.0.0"  ← Change this
```

**File 2:** `ui/Cargo.toml`
```toml
[package]
version = "1.0.0"  ← Change this
```

**File 3:** `ui/tauri.conf.json`
```json
{
  "version": "1.0.0"  ← Change this
}
```

### **Step 2: Commit & Tag (Copy-Paste)**

```bash
cd /Users/akshkaushik/Desktop/Waifu/netshaper

# Commit version bump
git add -A
git commit -m "Release v1.0.0"

# Tag it (this triggers GitHub Actions!)
git tag v1.0.0

# Push to GitHub
git push origin develop
git push origin v1.0.0
```

### **Step 3: Wait 15 Minutes**

GitHub Actions automatically:
- ✅ Builds Windows MSI
- ✅ Builds macOS DMG
- ✅ Builds Linux AppImage
- ✅ Uploads to GitHub Release

### **Step 4: Verify Success (2 min)**

1. Go to: https://github.com/aksh08022006/Wifly/releases
2. Look for "v1.0.0" at the top
3. Should show 3 files:
   - `NetShaper-1.0.0.msi` (Windows)
   - `NetShaper.dmg` (macOS)
   - `NetShaper-1.0.0.AppImage` (Linux)

✅ **Done! Your software is now public and downloadable!**

---

## 🌐 Where Users Download

**Website (Auto-Updated):**
```
https://aksh08022006.github.io/Wifly/
```

**Direct Links:**
```
Windows: https://github.com/.../releases/latest
macOS:   https://github.com/.../releases/latest
Linux:   https://github.com/.../releases/latest
```

---

## 🎉 Publishing Updates Later

**Every update takes same 20 minutes:**

```bash
# Update version (edit 3 files, change to 1.0.1)
# Then:

git add -A
git commit -m "Release v1.0.1"
git tag v1.0.1
git push origin develop
git push origin v1.0.1

# Wait 15 minutes → New version is live! ✅
```

---

## 📝 Release Notes Template

Create `RELEASE_NOTES.md` before tagging:

```markdown
# NetShaper v1.0.0

## Features
- ✨ Real-time bandwidth monitoring
- ⚡ Per-device rate limiting
- 🖥️ Beautiful dashboard interface

## Improvements
- 🐛 Fixed UI ↔ Daemon IPC protocol mismatch
- 📊 Better stats tracking
- 🎨 Improved UI responsiveness

## Requirements
- Windows 10+ / macOS 10.15+ / Ubuntu 20.04+
- 200 MB disk space
- Administrator access

## Installation
See [INSTALL.md](./INSTALL.md)

## Download
- 🪟 Windows: NetShaper-1.0.0.msi
- 🍎 macOS: NetShaper.dmg
- 🐧 Linux: NetShaper-1.0.0.AppImage
```

---

## ✅ Checklist

- [ ] Updated daemon/Cargo.toml to v1.0.0
- [ ] Updated ui/Cargo.toml to v1.0.0
- [ ] Updated ui/tauri.conf.json to v1.0.0
- [ ] Committed: `git commit -m "Release v1.0.0"`
- [ ] Tagged: `git tag v1.0.0`
- [ ] Pushed: `git push origin develop && git push origin v1.0.0`
- [ ] Waited 15 minutes for GitHub Actions
- [ ] Verified files on GitHub Releases page
- [ ] Website shows download buttons ✅
- [ ] Tested one installer works ✅
- [ ] **LIVE! 🚀**

---

**That's it! You're done.** Your software is now publicly available like Slack, Discord, RStudio, etc.

