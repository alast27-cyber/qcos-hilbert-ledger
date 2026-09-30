param (
    [Parameter(Mandatory=$true)]
    [ValidateSet("start", "stop", "status", "restart")]
    [string]$Action
)

$dir = $PSScriptRoot

function Stop-QCOSServices {
    Write-Host "Stopping all QCOS services..." -ForegroundColor Yellow
    $ports = @(8080..8090) + 9090
    Get-NetTCPConnection -ErrorAction SilentlyContinue | Where-Object { $ports -contains $_.LocalPort } | ForEach-Object {
        Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue
    }
    Stop-Process -Name "qcos_web_gateway", "qcos_ledger_monitor", "qcos-hilbert-ledger" -Force -ErrorAction SilentlyContinue
    Write-Host "Services stopped cleanly." -ForegroundColor Green
}

function Start-QCOSServices {
    Stop-QCOSServices
    Write-Host "Starting QCOS DQHHL Ecosystem..." -ForegroundColor Cyan
    
    # 1. Start Core Daemon
    $daemon = Start-Process "$dir\target\release\qcos-hilbert-ledger.exe" -PassThru
    Write-Host "Daemon started (PID: $($daemon.Id))." -ForegroundColor Green
    Start-Sleep -Seconds 2

    # 2. Start Web Gateway
    $gateway = Start-Process "$dir\target\release\qcos_web_gateway.exe" -PassThru
    Write-Host "Web Gateway started (PID: $($gateway.Id))." -ForegroundColor Green
    Start-Sleep -Seconds 1

    # 3. Launch CLI Monitor Window cleanly
    Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$dir'; cargo run --release --bin qcos_ledger_monitor"
    Write-Host "CLI Monitor window opened." -ForegroundColor Green
}

function Get-QCOSStatus {
    Write-Host "=== QCOS ECOSYSTEM STATUS ===" -ForegroundColor Cyan
    Get-Process -Name "qcos-hilbert-ledger", "qcos_web_gateway", "qcos_ledger_monitor" -ErrorAction SilentlyContinue | Format-Table Id, ProcessName, CPU, WorkingSet -AutoSize
    
    8080..8090 | ForEach-Object {
        try {
            $res = Invoke-RestMethod -Uri "http://127.0.0.1:${_}/api/v1/ledger/status" -ErrorAction Stop
            Write-Host "REST API active on Port ${_} -> Status: $($res.daemon_status) | Consensus: $($res.consensus_score)" -ForegroundColor Green
        } catch {}
    }
}

switch ($Action) {
    "start"   { Start-QCOSServices }
    "stop"    { Stop-QCOSServices }
    "restart" { Start-QCOSServices }
    "status"  { Get-QCOSStatus }
}
