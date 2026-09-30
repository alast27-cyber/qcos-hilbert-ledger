# Requires -RunAsAdministrator
Param (
    [string]$TargetDir = "C:\Program Files (x86)\QCOS_AI_Core\qcos-standalone\hilbert-ledger",
    [string]$TaskName = "QCOS_Hilbert_Ledger_Daemon",
    [string]$IssPath = ".\installer.iss",
    [string]$OutputDir = ".\Output"
)

$ErrorActionPreference = "Stop"

Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "   QCOS HILBERT LEDGER BUILD & INSTALLER PIPELINE   " -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan

# Step 1: Stop running instances to release executable file locks
Write-Host "[1/7] Stopping existing ledger processes..." -ForegroundColor Yellow
Stop-Process -Name "qcos-hilbert-ledger" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Step 2: Compile optimized release binary via Cargo
Write-Host "[2/7] Compiling release binary via Cargo..." -ForegroundColor Yellow
cargo build --release --bin qcos-hilbert-ledger
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo compilation failed! Aborting deployment pipeline."
    exit 1
}

# Step 3: Locate Inno Setup Compiler (ISCC.exe)
Write-Host "[3/7] Locating Inno Setup Compiler (ISCC.exe)..." -ForegroundColor Yellow
$IsccPaths = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
    (Get-Command "ISCC.exe" -ErrorAction SilentlyContinue).Path
) | Where-Object { $_ -and (Test-Path $_) }

$IsccExe = $IsccPaths | Select-Object -First 1

if (-not $IsccExe) {
    Write-Error "ISCC.exe not found! Please install Inno Setup 6 or add ISCC.exe to your system PATH."
    exit 1
}
Write-Host " -> Found ISCC compiler at: $IsccExe" -ForegroundColor Green

# Step 4: Compile installer.iss into setup bundle executable
Write-Host "[4/7] Compiling installer setup package..." -ForegroundColor Yellow
if (-not (Test-Path $IssPath)) {
    Write-Error "Inno Setup file not found at: $IssPath"
    exit 1
}

& $IsccExe /Q $IssPath
if ($LASTEXITCODE -ne 0) {
    Write-Error "Inno Setup compilation failed!"
    exit 1
}

$SetupBundle = Get-ChildItem -Path $OutputDir -Filter "QCOS_Hilbert_Ledger_Setup_v*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($SetupBundle) {
    Write-Host " -> Setup bundle generated successfully: $($SetupBundle.FullName)" -ForegroundColor Green
} else {
    Write-Warning "Setup bundle compiled, but output file search returned no matches in $OutputDir."
}

# Step 5: Sync binary to deployment target directory
Write-Host "[5/7] Syncing compiled release binary to target directory..." -ForegroundColor Yellow
if (-not (Test-Path -Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$SourceExe = ".\target\release\qcos-hilbert-ledger.exe"
$TargetExe = Join-Path $TargetDir "qcos-hilbert-ledger.exe"

Copy-Item -Path $SourceExe -Destination $TargetExe -Force
Write-Host " -> Deployed binary to: $TargetExe" -ForegroundColor Green

# Step 6: Register Windows Task Scheduler Auto-Start Service
Write-Host "[6/7] Registering Windows Scheduled Task for system startup..." -ForegroundColor Yellow
$Action = New-ScheduledTaskAction -Execute $TargetExe
$Trigger = New-ScheduledTaskTrigger -AtStartup
$Principal = New-ScheduledTaskPrincipal -UserId "NT AUTHORITY\SYSTEM" -LogonType ServiceAccount -RunLevel Highest

Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Principal $Principal -Force | Out-Null
Write-Host " -> Registered Scheduled Task: $TaskName" -ForegroundColor Green

# Step 7: Launch process and verify IPC pipe initialization
Write-Host "[7/7] Launching background daemon and verifying pipe..." -ForegroundColor Yellow
Start-Process -FilePath $TargetExe -WindowStyle Hidden
Start-Sleep -Seconds 1

$PipeExists = Test-Path "\\.\pipe\qcos_hilbert_ledger"
if ($PipeExists) {
    Write-Host "====================================================" -ForegroundColor Green
    Write-Host " SUCCESS: QCOS Hilbert Ledger Deployed & Active!" -ForegroundColor Green
    Write-Host " Target Pipe   : \\.\pipe\qcos_hilbert_ledger" -ForegroundColor Green
    if ($SetupBundle) {
        Write-Host " Setup Bundle  : $($SetupBundle.FullName)" -ForegroundColor Green
    }
    Write-Host "====================================================" -ForegroundColor Green
} else {
    Write-Host "====================================================" -ForegroundColor Red
    Write-Host " WARNING: Service launched, but pipe was not detected." -ForegroundColor Red
    Write-Host "====================================================" -ForegroundColor Red
}