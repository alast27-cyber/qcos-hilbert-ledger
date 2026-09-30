Write-Host "--- [QCOS] Starting High-Load QLLM Benchmark & Pruning Test ---" -ForegroundColor Cyan

$gatewayUrl = "http://127.0.0.1:8081/api/v1/ledger/inject"
$metricsUrl = "http://127.0.0.1:8081/api/v1/ledger/metrics"

$agentCount = 10         # Number of concurrent agent simulation threads
$requestsPerAgent = 25   # Requests fired per agent (Total = 250 requests)

Write-Host "[1/3] Spawning $agentCount concurrent agents ($($agentCount * $requestsPerAgent) total requests)..." -ForegroundColor Yellow

$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()$jobs = @()

for ($i = 1; $i -le $agentCount; $i++) {
    $clientId = "benchmark_node_$i"
    
    $job = Start-Job -ScriptBlock {
        param($url, $id,$count)
        $successCount = 0$failCount = 0

        for ($req = 1; $req -le $count; $req++) {
            # Alternate amplitudes to intentionally trigger pruning rules (threshold P < 0.05)
            $re = if ($req \% 2 -eq 0) { 0.7071 } else { 0.1 }$im = if ($req \% 3 -eq 0) { 0.7071 } else { 0.0 }$body = @{
                client_id = $id
                mode_index = $req % 5
                amplitude_re = $re
                amplitude_im = $im
            } | ConvertTo-Json

            try {
                $response = Invoke-RestMethod -Uri $url -Method Post -Body$body -ContentType "application/json" -ErrorAction Stop
                $successCount++
            } catch {
                $failCount++
            }
        }
        return @{ Success = $successCount; Failed =$failCount }
    } -ArgumentList $gatewayUrl, $clientId,$requestsPerAgent

    $jobs +=$job
}

Write-Host "[2/3] Waiting for benchmark threads to finish execution..." -ForegroundColor Yellow
$jobResults = $jobs \vert{} Wait-Job \vert{} Receive-Job$jobs | Remove-Job

$stopwatch.Stop()
$elapsedSeconds =$stopwatch.Elapsed.TotalSeconds

# Aggregate results across threads
$totalSuccess = ($jobResults | Measure-Object -Property Success -Sum).Sum
$totalFailed = ($jobResults | Measure-Object -Property Failed -Sum).Sum
$totalRequests =$totalSuccess + $totalFailed$reqPerSec = [Math]::Round($totalRequests / $elapsedSeconds, 2)

Write-Host "`n--- Benchmark Performance Summary ---" -ForegroundColor Cyan
Write-Host "Total Time Elapsed : $([Math]::Round($elapsedSeconds, 3)) seconds"
Write-Host "Total Requests Sent: $totalRequests"
Write-Host "Successful Injections: $totalSuccess"
Write-Host "Failed Requests    : $totalFailed"
Write-Host "Throughput         : $reqPerSec req/sec" -ForegroundColor Green

Write-Host "`n[3/3] Fetching Final Engine Pruning Telemetry..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
Invoke-RestMethod -Uri $metricsUrl -Method Get | Format-List

Write-Host "--- [QCOS] High-Load Benchmark Complete! ---" -ForegroundColor Green