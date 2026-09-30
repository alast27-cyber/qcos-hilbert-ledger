Write-Host "--- [QCOS] Starting Multi-Agent EAP High-Load Simulation ---" -ForegroundColor Cyan

$gatewayUrl = "http://127.0.0.1:8081/api/v1/ledger/inject"
$metricsUrl = "http://127.0.0.1:8081/api/v1/ledger/metrics"

# Number of concurrent agent jobs and requests per agent
$agentCount = 5
$requestsPerAgent = 20

Write-Host "[1/3] Launching $agentCount concurrent agents firing $requestsPerAgent requests each..." -ForegroundColor Yellow

$jobs = @()

for ($i = 1; $i -le $agentCount; $i++) {
    $clientId = "agent_q_node_0$i"
    
    $job = Start-Job -ScriptBlock {
        param($url, $id, $count)
        for ($req = 1; $req -le $count; $req++) {
            # Randomize amplitudes slightly to test threshold pruning bounds (P = re^2 + im^2)
            $re = [Math]::Round((Get-Random -Minimum 0.1 -Maximum 1.0), 4)
            $im = [Math]::Round((Get-Random -Minimum 0.0 -Maximum 0.5), 4)
            
            $body = @{
                client_id = $id
                mode_index = $req % 3 # Spread across modes 0, 1, 2
                amplitude_re = $re
                amplitude_im = $im
            } | ConvertTo-Json

            try {
                Invoke-RestMethod -Uri $url -Method Post -Body $body -ContentType "application/json" -ErrorAction Stop
            } catch {
                # Handle connection drops gracefully under heavy saturation
            }
        }
    } -ArgumentList $gatewayUrl, $clientId, $requestsPerAgent

    $jobs += $job
}

Write-Host "[2/3] Waiting for agent simulation threads to complete..." -ForegroundColor Yellow
$jobs | Wait-Job | Out-Job
$jobs | Remove-Job

Write-Host "[3/3] Fetching Final Telemetry & Pruning Efficiency..." -ForegroundColor Yellow
Start-Sleep -Seconds 1

$finalMetrics = Invoke-RestMethod -Uri $metricsUrl -Method Get
$finalMetrics | Format-List

Write-Host "--- [QCOS] High-Load Simulation Complete! ---" -ForegroundColor Green