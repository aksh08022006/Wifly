#requires -RunAsAdministrator
<#
  NetShaper / Wifly - Milestone 0 Automated Setup Script
  For: Saksham (Windows Driver & UI Development)
  
  This script automates complete M0 setup on Windows.
  RUN WITH ADMINISTRATOR PRIVILEGES
#>

param(
    [switch]$SkipTestSigning = $false,
    [switch]$SkipToolchain = $false,
    [switch]$NoRestart = $false
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "NetShaper M0 Setup - Windows (Saksham)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Colors for output
$successColor = "Green"
$errorColor = "Red"
$infoColor = "Cyan"
$warnColor = "Yellow"

# Function: Log output
function Write-Status {
    param([string]$Message, [ValidateSet("Success", "Error", "Info", "Warn")]$Type = "Info")
    $color = switch($Type) {
        "Success" { $successColor }
        "Error" { $errorColor }
        "Warn" { $warnColor }
        default { $infoColor }
    }
    Write-Host "[$Type] $Message" -ForegroundColor $color
}

# ===== STEP 1: Enable Test Signing =====
if (-not $SkipTestSigning) {
    Write-Host "`n[Step 1] Enabling Test Signing..." -ForegroundColor Cyan
    try {
        $testsigningStatus = bcdedit | Select-String "testsigning"
        if ($testsigningStatus -match "Yes") {
            Write-Status "Test signing already enabled" "Success"
        } else {
            Write-Status "Enabling test signing (requires restart)" "Info"
            bcdedit /set testsigning on | Out-Null
            Write-Status "Test signing enabled" "Success"
            $restartNeeded = $true
        }
    } catch {
        Write-Status "Failed to enable test signing: $_" "Error"
        exit 1
    }
} else {
    Write-Status "Skipping test signing (already configured?)" "Warn"
}

# ===== STEP 2: Install Rust =====
if (-not $SkipToolchain) {
    Write-Host "`n[Step 2] Installing Rust Toolchain..." -ForegroundColor Cyan
    try {
        $rustInstalled = rustc --version 2>$null
        if ($rustInstalled) {
            Write-Status "Rust already installed: $rustInstalled" "Success"
        } else {
            Write-Status "Installing Rustup..." "Info"
            winget install Rustlang.Rustup --accept-package-agreements --accept-source-agreements -h --force
            Write-Status "Rust installed" "Success"
        }
    } catch {
        Write-Status "Failed to install Rust: $_" "Error"
        exit 1
    }
}

# Refresh PATH for current session
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

# ===== STEP 3: Configure Rust for Windows MSVC =====
Write-Host "`n[Step 3] Configuring Rust MSVC Toolchain..." -ForegroundColor Cyan
try {
    Write-Status "Installing x86_64-pc-windows-msvc..." "Info"
    & "$env:USERPROFILE\.cargo\bin\rustup" toolchain install stable-x86_64-pc-windows-msvc 2>&1 | Where-Object { $_ -match "(installed|unchanged)" }
    & "$env:USERPROFILE\.cargo\bin\rustup" default stable-x86_64-pc-windows-msvc 2>&1 | Where-Object { $_ -match "set to" }
    Write-Status "MSVC toolchain configured" "Success"
} catch {
    Write-Status "Failed to configure Rust: $_" "Warn"
}

# ===== STEP 4: Install Windows Driver Kit =====
Write-Host "`n[Step 4] Installing Windows Driver Kit (WDK)..." -ForegroundColor Cyan
try {
    # WDK download: https://docs.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk
    Write-Status "WDK installation" "Info"
    Write-Status "Manual step: Download WDK from https://docs.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk" "Warn"
    Write-Status "Or run: winget install Microsoft.WindowsDriverKit" "Warn"
} catch {
    Write-Status "WDK note: May need manual installation" "Warn"
}

# ===== STEP 5: Install Visual C++ Build Tools =====
Write-Host "`n[Step 5] Installing Visual Studio Build Tools with C++ Workload..." -ForegroundColor Cyan
try {
    Write-Status "Downloading VS Build Tools..." "Info"
    $vsBootstrapperUrl = "https://aka.ms/vs/17/release/vs_buildtools.exe"
    $vsBootstrapperPath = "$env:TEMP\vs_buildtools.exe"
    
    (New-Object Net.WebClient).DownloadFile($vsBootstrapperUrl, $vsBootstrapperPath)
    
    if (Test-Path $vsBootstrapperPath) {
        Write-Status "Installing Visual Studio Build Tools..." "Info"
        & $vsBootstrapperPath `
            --quiet `
            --norestart `
            --wait `
            --add Microsoft.VisualStudio.Workload.NativeDesktop `
            --add Microsoft.VisualStudio.Component.Windows10SDK `
            --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64
        
        Write-Status "Visual Studio Build Tools installed" "Success"
    } else {
        Write-Status "Failed to download VS Build Tools" "Error"
    }
} catch {
    Write-Status "Error with VS Build Tools installation: $_" "Error"
}

# ===== STEP 6: Install LLVM (optional, faster linking) =====
Write-Host "`n[Step 6] Installing LLVM..." -ForegroundColor Cyan
try {
    Write-Status "Installing LLVM..." "Info"
    winget install LLVM.LLVM --accept-package-agreements --accept-source-agreements -h --force | Out-Null
    Write-Status "LLVM installed" "Success"
} catch {
    Write-Status "LLVM installation optional, skipped" "Warn"
}

# Refresh PATH again
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

# ===== STEP 7: Verify Installation =====
Write-Host "`n[Step 7] Verifying Installation..." -ForegroundColor Cyan
try {
    $rustVersion = & "$env:USERPROFILE\.cargo\bin\rustc" --version
    Write-Status "Rustc: $rustVersion" "Success"
    
    $cargoVersion = & "$env:USERPROFILE\.cargo\bin\cargo" --version
    Write-Status "Cargo: $cargoVersion" "Success"
} catch {
    Write-Status "Warning: Could not verify Rust (may need PATH refresh)" "Warn"
}

# ===== STEP 8: Clone / Navigate to Wifly repo =====
Write-Host "`n[Step 8] Setting up Wifly Repository..." -ForegroundColor Cyan
$wiflyPath = "$env:USERPROFILE\Wifly"
if (Test-Path $wiflyPath) {
    Write-Status "Wifly repository found: $wiflyPath" "Success"
    cd $wiflyPath
} else {
    Write-Status "Wifly directory not found at: $wiflyPath" "Error"
    Write-Status "Please ensure the repository is cloned to: $wiflyPath" "Warn"
    exit 1
}

# ===== STEP 9: Test Build =====
Write-Host "`n[Step 9] Testing Build..." -ForegroundColor Cyan
try {
    Write-Status "Running: cargo build -p proto (simple test)" "Info"
    & "$env:USERPROFILE\.cargo\bin\cargo" build -p proto 2>&1 | Select-Object -Last 20
    Write-Status "Build test completed - check output above for errors" "Success"
} catch {
    Write-Status "Build failed - may need additional configuration" "Error"
}

# ===== COMPLETION =====
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Setup Complete" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`nNext Steps:" -ForegroundColor Cyan
Write-Host "1. If test signing was enabled, RESTART your computer now" -ForegroundColor Yellow
Write-Host "2. Open new PowerShell window (non-admin)"
Write-Host "3. Navigate to: cd $wiflyPath"
Write-Host "4. Test Cargo: cargo build --workspace"
Write-Host "5. Create M0 setup PR: git checkout -b saksham/milestone-0-setup"
Write-Host ""

if ($restartNeeded -and -not $NoRestart) {
    Write-Host "Restarting computer in 30 seconds...`n" -ForegroundColor Yellow
    Write-Host "Press Ctrl+C to cancel restart" -ForegroundColor Yellow
    Start-Sleep -Seconds 30
    Restart-Computer -Force
}
