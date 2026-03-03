# Script to cleanly kill the backend using the stored PID
$pidFile = ".\.backend.pid"

if (Test-Path $pidFile) {
    $processId = Get-Content $pidFile
    Write-Host "Stopping backend (PID: $processId)..."
    try {
        Stop-Process -Id $processId -Force -ErrorAction Stop
        Write-Host "Backend stopped."
    } catch {
        Write-Warning "Failed to stop process or it is already dead."
    }
    Remove-Item $pidFile
} else {
    Write-Host "No .backend.pid file found."
}
