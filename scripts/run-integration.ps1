$ErrorActionPreference = "Stop"

# Set the authentication token for the integration test runner
$env:PLANTLINK_AUTH_TOKEN = "test-secret"

# Navigate to the UI directory
Set-Location ui

# Run the playwright integration tests
npm run test:integration
