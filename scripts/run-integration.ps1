$ErrorActionPreference = "Stop"
$env:PLANTLINK_AUTH_TOKEN = "test-secret"

# Step 1: Build UI
Write-Host "Building UI assets..."
Push-Location ui
npm run build
Pop-Location

# Step 2: Build and start backend
Write-Host "Building plantlink-cli..."
cargo build -p plantlink-cli

Write-Host "Starting backend (auth token: test-secret)..."
$process = Start-Process -FilePath ".\target\debug\plantlink-cli.exe" `
    -ArgumentList "--auth-token", "test-secret" -PassThru -NoNewWindow
$pidFile = ".\.backend.pid"
$process.Id | Out-File -FilePath $pidFile

# Step 3: Wait for health
$maxRetries = 20
$isHealthy = $false
for ($i = 0; $i -lt $maxRetries; $i++) {
    try {
        $r = Invoke-WebRequest -Uri "http://localhost:3000/health" -UseBasicParsing -ErrorAction Stop
        if ($r.StatusCode -eq 200) { $isHealthy = $true; break }
    } catch { }
    Start-Sleep -Seconds 1
}
if (-not $isHealthy) {
    Write-Error "Backend failed health check within 20s"
    Stop-Process -Id $process.Id -Force
    Remove-Item $pidFile -ErrorAction SilentlyContinue
    exit 1
}
Write-Host "Backend healthy (PID: $($process.Id))"

# Step 4: Run tests with guaranteed cleanup
$testExitCode = 0
try {
    Push-Location ui
    # Call playwright directly to ensure we capture the exit code correctly
    npx playwright test --config playwright.integration.config.js
    $testExitCode = $LASTEXITCODE
    Pop-Location
} finally {
    Write-Host "Cleaning up backend..."
    if (Test-Path $pidFile) {
        $storedPid = Get-Content $pidFile
        try { Stop-Process -Id $storedPid -Force -ErrorAction Stop } catch { }
        Remove-Item $pidFile -ErrorAction SilentlyContinue
    }
}
exit $testExitCode
