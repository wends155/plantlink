# start-backend.ps1
# Builds and starts the plantlink-cli backend for integration testing.

$ErrorActionPreference = "Stop"

Write-Host "Building plantlink-cli..."
cargo build -p plantlink-cli
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
    exit 1
}

Write-Host "Starting plantlink-cli backend in background..."
# Start the process and capture the object
$process = Start-Process -FilePath ".\target\debug\plantlink-cli.exe" -PassThru -NoNewWindow
$pidFile = ".\.backend.pid"
$process.Id | Out-File -FilePath $pidFile

Write-Host "Waiting for backend to become healthy on port 3000..."
$healthUrl = "http://localhost:3000/health"
$maxRetries = 20
$retryCount = 0
$isHealthy = $false

while ($retryCount -lt $maxRetries) {
    try {
        $response = Invoke-WebRequest -Uri $healthUrl -UseBasicParsing -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            $isHealthy = $true
            break
        }
    } catch {
        # Ignore connection refused while booting
    }
    
    Start-Sleep -Seconds 1
    $retryCount++
}

if (-not $isHealthy) {
    Write-Error "Backend failed to become healthy within 20 seconds. Aborting."
    Stop-Process -Id $process.Id -Force
    Remove-Item $pidFile -ErrorAction SilentlyContinue
    exit 1
}

Write-Host "Backend is up and running (PID: $($process.Id))."
exit 0
