# NetShaper Publishing Workflow

## 🚀 How to Publish a New Release

### **1. Prepare the Release (5 min)**

```bash
# Update version in Cargo.toml files
# Example: 0.1.0 → 0.2.0

# daemon/Cargo.toml
[package]
version = "0.2.0"

# ui/Cargo.toml
version = "0.2.0"

# ui/tauri.conf.json
"version": "0.2.0"
```

### **2. Create Release Notes**

Create file: `RELEASE_NOTES.md`

```markdown
# NetShaper v0.2.0

## ✨ New Features
- Real-time bandwidth monitoring
- Per-device rate limiting

## 🐛 Bug Fixes
- Fixed UI ↔ Daemon IPC protocol mismatch
- Improved error handling

## 📦 System Requirements
- Windows 10+ / macOS 10.15+ / Ubuntu 20.04+
- 200 MB free disk space

## 📥 Installation

### Windows
Download `NetShaper-0.2.0.msi` and run it.

### macOS
Download `NetShaper.dmg`, open it, drag NetShaper to Applications.

### Linux
Download `NetShaper-0.2.0.AppImage`, make it executable, and run it.

## 🔗 Installation Guide
See [INSTALL.md](./INSTALL.md) for detailed instructions.
```

### **3. Commit & Tag**

```bash
# Make sure all changes are committed
git add -A
git commit -m "Bump version to 0.2.0"

# Create version tag (triggers GitHub Actions)
git tag v0.2.0
git push origin main --tags

# GitHub Actions now:
# ✅ Builds Windows MSI
# ✅ Builds macOS DMG
# ✅ Builds Linux AppImage
# ✅ Creates GitHub Release
# ✅ Uploads all files
```

### **4. Verify Release (2 min)**

1. Go to [GitHub Releases](https://github.com/aksh08022006/Wifly/releases)
2. Check that all files uploaded:
   - ✅ `NetShaper-0.2.0.msi` (Windows)
   - ✅ `NetShaper.dmg` (macOS)
   - ✅ `NetShaper-0.2.0.AppImage` (Linux)
3. Test download link works

### **5. Announce (Optional)**

Post on social media:
```
🎉 NetShaper v0.2.0 is now available!

🔗 Download: https://github.com/aksh08022006/Wifly/releases/latest

What's new:
✨ Better stats display
🐛 Critical bug fixes
⚡ Faster performance

Try it now! It's free & open source 🚀
```

---

## 📊 Release Schedule Recommendations

**Option A: Rolling Releases (Publish ASAP)**
- Tag & release on every major feature completion
- E.g., v0.1.0 → v0.2.0 → v0.3.0
- Users always get latest
- ✅ Best for rapid development

**Option B: Monthly Releases**
- Tag & release on the 1st of each month
- Batch features & fixes together
- More stable, fewer updates

**Option C: When Ready**
- Release when you hit milestones
- E.g., v1.0.0 after full testing
- More controlled

---

## 🔗 Distribution Links

**Main Download Page:**
```
https://aksh08022006.github.io/Wifly/
```

**GitHub Releases:**
```
https://github.com/aksh08022006/Wifly/releases/latest
```

**Direct MSI Download:**
```
https://github.com/aksh08022006/Wifly/releases/download/v0.2.0/NetShaper-0.2.0.msi
```

---

## ✅ Checklist for Each Release

- [ ] Update version in all `Cargo.toml` files
- [ ] Update version in `tauri.conf.json`
- [ ] Write release notes in `RELEASE_NOTES.md`
- [ ] Commit: `git commit -m "Bump to v0.2.0"`
- [ ] Tag: `git tag v0.2.0`
- [ ] Push: `git push origin main --tags`
- [ ] Wait 5-10 minutes for GitHub Actions to build
- [ ] Check [Releases page](https://github.com/aksh08022006/Wifly/releases)
- [ ] Verify all 3 files uploaded (Windows, macOS, Linux)
- [ ] Test download links work
- [ ] Update website announcement (optional)
- [ ] Share on social media (optional)

---

## 🎓 Manual Build (If GitHub Actions Fails)

```bash
# Windows
cargo build --release
cd ui
npm run tauri build
# Check: ui/target/release/bundle/msi/

# macOS
cd ui
npm run tauri build
# Check: ui/target/release/bundle/macos/

# Linux
cd ui
npm run tauri build
# Check: ui/target/release/bundle/appimage/
```

---

## 💰 Monetization (Ads & Sponsorships)

**Website:**
- Add ad banners to `docs/index.html`
- Google AdSense (~$0.50-2 per 1000 views)
- Partner sponsorship section

**Examples:**
```html
<!-- Inside the .ad-section -->
<script async src="//pagead2.googlesyndication.com/pagead/js/adsbygoogle.js"></script>
<ins class="adsbygoogle"
     style="display:block"
     data-ad-client="ca-pub-xxxxxxxxxxxxxxxx"
     data-ad-slot="1234567890"
     data-ad-format="auto"
     data-full-width-responsive="true"></ins>
<script>
     (adsbygoogle = window.adsbygoogle || []).push({});
</script>
```

---

## 🔐 Security Considerations

Before releasing to public:

- [ ] Enable Windows code signing (optional but recommended)
- [ ] Test on clean Windows/Mac/Linux machines
- [ ] Run antivirus scan on built executables
- [ ] Verify no hardcoded credentials in code
- [ ] Check permissions (admin access justified?)
- [ ] Test uninstall removes all files

---

## 📞 Support & Feedback

Add issue template: `.github/ISSUE_TEMPLATE/bug_report.md`

```markdown
# Bug Report

## Describe the bug
[Your description here]

## System Info
- OS: [Windows 10/macOS/Linux]
- Version: [0.1.0, 0.2.0, etc]

## Steps to reproduce
1. ...
2. ...

## Expected behavior
[What should happen]

## Actual behavior
[What actually happens]

## Logs/Screenshots
[Paste any error messages]
```

---

## 🚀 First Release Checklist

- [ ] GitHub Actions workflow created ✅
- [ ] Website created in `docs/` ✅
- [ ] GitHub Pages enabled ✅
- [ ] Auto-updates configured ✅
- [ ] Release notes template created ✅
- [ ] Tested build locally on Windows
- [ ] Tested build locally on macOS
- [ ] Tested build locally on Linux
- [ ] All 3 installers working
- [ ] Tag pushed: `git push origin main --tags`
- [ ] Watch GitHub Actions build
- [ ] Verify release on GitHub Releases page
- [ ] Website visible at https://aksh08022006.github.io/Wifly/
- [ ] Share with Saksham for testing
- [ ] Announce v1.0.0 release! 🎉

