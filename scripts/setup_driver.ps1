# NetShaper WFP Driver Setup Script
# ==================================
# This script registers and loads the NetShaper WFP kernel driver
# 
# Requirements:
# - Run as Administrator
# - WFP driver file at: wfp-callout\target\release\netshaper_wfp.sys
#
# Usage:
#   .\scripts\setup_driver.ps1
#
# After running:
# 1. Reboot the system (required for testsigning to take effect)
# 2. Run: sc start netshaper_wfp

param(
    [string]$DriverPath = "$PSScriptRoot\..\wfp-callout\target\release\netshaper_wfp.sys"
)

# Check if running as Administrator
$isAdmin = [bool]([System.Security.Principal.WindowsIdentity]::GetCurrent().Groups -match "S-1-5-32-544")
if (-not $isAdmin) {
    Write-Host "ERROR: This script must run as Administrator" -ForegroundColor Red
    Write-Host "Please open PowerShell as Administrator and try again" -ForegroundColor Yellow
    exit 1
}

Write-Host "NetShaper WFP Driver Setup" -ForegroundColor Cyan
Write-Host "===========================" -ForegroundColor Cyan
Write-Host ""

# Verify driver file exists
if (-not (Test-Path $DriverPath)) {
    Write-Host "ERROR: Driver file not found at: $DriverPath" -ForegroundColor Red
    Write-Host "Please compile the WFP driver first:" -ForegroundColor Yellow
    Write-Host "  cd wfp-callout" -ForegroundColor Yellow
    Write-Host "  cargo build --release" -ForegroundColor Yellow
    exit 1
}

$DriverFullPath = (Resolve-Path $DriverPath).Path
Write-Host "Driver path: $DriverFullPath" -ForegroundColor Green

# Step 1: Enable Test Signing
Write-Host ""
Write-Host "Step 1: Enabling Test Signing for unsigned drivers..." -ForegroundColor Yellow
$testeSigningOutput = bcdedit /set testsigning on 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Test signing enabled" -ForegroundColor Green
} else {
    Write-Host "⚠ Test signing already enabled or error occurred:" -ForegroundColor Yellow
    Write-Host $testSigningOutput
}

# Step 2: Create driver service
Write-Host ""
Write-Host "Step 2: Registering WFP driver as kernel service..." -ForegroundColor Yellow

# First, check if service already exists and remove it
$existingService = Get-Service -Name netshaper_wfp -ErrorAction SilentlyContinue
if ($existingService) {
    Write-Host "  Removing existing service..." -ForegroundColor Gray
    sc.exe delete netshaper_wfp | Out-Null
    Start-Sleep -Seconds 1
}

# Create the service
$createOutput = sc.exe create netshaper_wfp `
    type= kernel `
    start= demand `
    binPath= $DriverFullPath `
    DisplayName= "NetShaper WFP Filter" `
    2>&1

if ($LASTEXITCODE -eq 0 -or $createOutput -match "[SUCCESS]") {
    Write-Host "✓ Driver service registered successfully" -ForegroundColor Green
} else {
    Write-Host "✗ Failed to register driver service:" -ForegroundColor Red
    Write-Host $createOutput
    exit 1
}

# Step 3: Information for user
Write-Host ""
Write-Host "Setup Complete!" -ForegroundColor Green
Write-Host "===============" -ForegroundColor Green
Write-Host ""
Write-Host "IMPORTANT: Your system must be rebooted for test signing to take effect." -ForegroundColor Yellow
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Reboot your computer now:" -ForegroundColor White
Write-Host "   Restart-Computer -Force" -ForegroundColor Gray
Write-Host ""
Write-Host "2. After reboot, start the WFP driver:" -ForegroundColor White
Write-Host "   sc start netshaper_wfp" -ForegroundColor Gray
Write-Host ""
Write-Host "3. Verify the driver loaded successfully:" -ForegroundColor White
Write-Host "   sc query netshaper_wfp" -ForegroundColor Gray
Write-Host ""
Write-Host "4. To stop the driver:" -ForegroundColor White
Write-Host "   sc stop netshaper_wfp" -ForegroundColor Gray
Write-Host ""
Write-Host "To uninstall the driver completely:" -ForegroundColor Cyan
Write-Host "  sc delete netshaper_wfp" -ForegroundColor Gray
Write-Host "  bcdedit /set testsigning off" -ForegroundColor Gray
Write-Host "  (then reboot)" -ForegroundColor Gray
