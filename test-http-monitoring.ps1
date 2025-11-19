# Quick Test Script for HTTP Request Monitoring on Windows
# This script tests the HTTP request monitoring feature

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "HTTP Request Monitoring Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Docker is running
Write-Host "Checking Docker..." -ForegroundColor Yellow
try {
    docker ps | Out-Null
    Write-Host "✓ Docker is running" -ForegroundColor Green
} catch {
    Write-Host "✗ Docker is not running. Please start Docker Desktop." -ForegroundColor Red
    exit 1
}

# Clean up any existing test container
Write-Host ""
Write-Host "Cleaning up old test containers..." -ForegroundColor Yellow
docker rm -f test-nginx 2>$null | Out-Null
Start-Sleep -Seconds 1

# Start a test container
Write-Host ""
Write-Host "Starting nginx test container..." -ForegroundColor Yellow
docker run -d -p 8080:80 --name test-nginx nginx
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Container started: test-nginx" -ForegroundColor Green
} else {
    Write-Host "✗ Failed to start container" -ForegroundColor Red
    exit 1
}

# Wait for container to be ready
Write-Host ""
Write-Host "Waiting for container to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

# Make HTTP requests
Write-Host ""
Write-Host "Making HTTP requests to generate traffic..." -ForegroundColor Yellow
$requestCount = 5
for ($i = 1; $i -le $requestCount; $i++) {
    try {
        $response = Invoke-WebRequest -Uri http://localhost:8080 -UseBasicParsing -ErrorAction Stop
        Write-Host "  Request $i/$requestCount - Status: $($response.StatusCode)" -ForegroundColor Gray
    } catch {
        Write-Host "  Request $i/$requestCount - Failed: $_" -ForegroundColor Red
    }
    Start-Sleep -Milliseconds 500
}

Write-Host ""
Write-Host "✓ Generated $requestCount HTTP requests" -ForegroundColor Green

# Check container logs
Write-Host ""
Write-Host "Checking container logs..." -ForegroundColor Yellow
$logs = docker logs test-nginx 2>&1 | Select-Object -Last 10
if ($logs) {
    Write-Host "Recent logs:" -ForegroundColor Gray
    $logs | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
} else {
    Write-Host "  No logs found" -ForegroundColor Yellow
}

# Instructions
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Test Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Make sure the backend server is running:" -ForegroundColor White
Write-Host "   cd backend" -ForegroundColor Gray
Write-Host "   cargo run" -ForegroundColor Gray
Write-Host ""
Write-Host "2. Open the frontend and go to APM page" -ForegroundColor White
Write-Host ""
Write-Host "3. Select container: test-nginx" -ForegroundColor White
Write-Host ""
Write-Host "4. Check the 'HTTP Requests' section" -ForegroundColor White
Write-Host ""
Write-Host "Note: On Windows, the system uses log parsing" -ForegroundColor Cyan
Write-Host "      (network capture requires WSL2)" -ForegroundColor Cyan
Write-Host ""
Write-Host "To clean up:" -ForegroundColor Yellow
Write-Host "  docker rm -f test-nginx" -ForegroundColor Gray
Write-Host ""

