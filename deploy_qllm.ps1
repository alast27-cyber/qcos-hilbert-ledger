Write-Host "--- [QCOS] Initiating QLLM & Hilbert Ledger Deployment ---" -ForegroundColor Cyan

# 1. Terminate any lingering instances or background jobs
Get-Job | Remove-Job -Force -ErrorAction SilentlyContinue
Stop-Process -Name "qcos_web_gateway", "qcos-hilbert-ledger" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# 2. Compile optimized release binaries
Write-Host "[1/4] Compiling release binaries..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed! Please check code errors." -ForegroundColor Red
    exit
}

# 3. Launch Core Daemons as background jobs
Write-Host "[2/4] Launching Hilbert Ledger Engine & REST Gateway daemons..." -ForegroundColor Yellow
Start-Job -Name EngineJob -ScriptBlock { 
    Set-Location 'C:\Program Files (x86)\QCOS_AI_Core\qcos-standalone\hilbert-ledger'
    .\target\release\qcos-hilbert-ledger.exe 
}

Start-Job -Name GatewayJob -ScriptBlock { 
    Set-Location 'C:\Program Files (x86)\QCOS_AI_Core\qcos-standalone\hilbert-ledger'
    .\target\release\qcos_web_gateway.exe 
}

Start-Sleep -Seconds 3

# 4. Run Smoke Tests & Telemetry Verification
Write-Host "[3/4] Running system health & connectivity smoke tests..." -ForegroundColor Yellow

Write-Host "`n--- Status Endpoint ---"
Invoke-RestMethod -Uri "http://127.0.0.1:8081/api/v1/ledger/status" -Method Get

Write-Host "`n--- Testing EAP State Injection (Superposition Node 01) ---"
$injectBody = @{
    client_id = "agent_q_deploy_node"
    mode_index = 0
    amplitude_re = 0.7071
    amplitude_im = 0.7071
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://127.0.0.1:8081/api/v1/ledger/inject" -Method Post -Body $injectBody -ContentType "application/json"

Write-Host "`n--- Fetching Real-Time Pruning Metrics ---"
Invoke-RestMethod -Uri "http://127.0.0.1:8081/api/v1/ledger/metrics" -Method Get

Write-Host "`n--- [QCOS] QLLM Deployment Complete & Operational! ---" -ForegroundColor Green