$ErrorActionPreference = "Stop"

# Set the authentication token for the frontend build
$env:VITE_AUTH_TOKEN = "test-secret"

# Navigate to the UI directory
Set-Location ui

# Build the frontend assets
npm run build
