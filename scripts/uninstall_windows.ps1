# Onevox Windows Uninstaller
# Run this script in PowerShell

param(
    [switch]$KeepConfig = $false,
    [switch]$Force = $false
)

# Define paths
$onevoxDir = "$env:LOCALAPPDATA\onevox"
$configDir = "$env:APPDATA\onevox\onevox\config"
$dataDir = "$env:APPDATA\onevox\onevox\data"
$cacheDir = "$env:LOCALAPPDATA\onevox\onevox\cache"
$logsDir = "$env:APPDATA\onevox\onevox\data\logs"
$serviceName = "Onevox"

# Colors for output
function Write-Info {
    param([string]$message)
    Write-Host "[INFO] $message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$message)
    Write-Host "[WARN] $message" -ForegroundColor Yellow
}

function Write-Error-Message {
    param([string]$message)
    Write-Host "[ERROR] $message" -ForegroundColor Red
}

# Check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Confirm uninstall
function Confirm-Uninstall {
    Write-Host ""
    Write-Host "╔════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║   Onevox Windows Uninstaller           ║" -ForegroundColor Cyan
    Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "This will remove Onevox and all its data." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "The following will be deleted:"
    Write-Host "  • Binary: $onevoxDir" -ForegroundColor Gray
    Write-Host "  • Config: $configDir" -ForegroundColor Gray
    Write-Host "  • Data: $dataDir" -ForegroundColor Gray
    Write-Host "  • Cache: $cacheDir" -ForegroundColor Gray
    Write-Host "  • Service: $serviceName (if registered)" -ForegroundColor Gray
    Write-Host ""
    
    if (-not $Force) {
        $response = Read-Host "Continue? (y/N)"
        if ($response -ne 'y' -and $response -ne 'Y') {
            Write-Info "Uninstall cancelled"
            exit 0
        }
    }
}

# Stop and remove Windows service
function Remove-Service {
    try {
        $service = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
        
        if ($service) {
            Write-Info "Stopping service..."
            
            if ($service.Status -eq 'Running') {
                if (Test-Administrator) {
                    Stop-Service -Name $serviceName -Force -ErrorAction SilentlyContinue
                    Write-Info "Service stopped"
                } else {
                    Write-Warn "Service is running but script is not running as Administrator"
                    Write-Warn "Please run PowerShell as Administrator to stop the service, or stop it manually:"
                    Write-Host "  sc.exe stop $serviceName" -ForegroundColor Gray
                }
            }
            
            if (Test-Administrator) {
                Write-Info "Removing service..."
                sc.exe delete $serviceName | Out-Null
                Write-Info "Service removed"
            } else {
                Write-Warn "Cannot remove service without Administrator privileges"
                Write-Warn "Please run PowerShell as Administrator and execute:"
                Write-Host "  sc.exe delete $serviceName" -ForegroundColor Gray
            }
        }
    } catch {
        Write-Warn "Could not query/remove service: $_"
    }
}

# Remove from PATH environment variable
function Remove-FromPath {
    Write-Info "Removing from PATH..."
    
    try {
        # User PATH
        $userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
        if ($userPath -like "*$onevoxDir*") {
            $newUserPath = ($userPath -split ';' | Where-Object { $_ -ne $onevoxDir }) -join ';'
            [Environment]::SetEnvironmentVariable("Path", $newUserPath, [EnvironmentVariableTarget]::User)
            Write-Info "Removed from user PATH"
        }
        
        # System PATH (requires admin)
        if (Test-Administrator) {
            $systemPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::Machine)
            if ($systemPath -like "*$onevoxDir*") {
                $newSystemPath = ($systemPath -split ';' | Where-Object { $_ -ne $onevoxDir }) -join ';'
                [Environment]::SetEnvironmentVariable("Path", $newSystemPath, [EnvironmentVariableTarget]::Machine)
                Write-Info "Removed from system PATH"
            }
        }
    } catch {
        Write-Warn "Could not remove from PATH: $_"
    }
}

# Remove files and directories
function Remove-Files {
    Write-Info "Removing files..."
    
    # Remove binary
    if (Test-Path $onevoxDir) {
        try {
            Remove-Item -Path $onevoxDir -Recurse -Force -ErrorAction Stop
            Write-Info "Removed binary and installation directory"
        } catch {
            Write-Error-Message "Failed to remove $onevoxDir : $_"
        }
    }
    
    # Remove config (unless KeepConfig flag is set)
    if (-not $KeepConfig) {
        if (Test-Path $configDir) {
            try {
                Remove-Item -Path $configDir -Recurse -Force -ErrorAction Stop
                Write-Info "Removed config"
            } catch {
                Write-Error-Message "Failed to remove $configDir : $_"
            }
        }
    } else {
        Write-Info "Keeping config directory (--KeepConfig flag set)"
    }
    
    # Remove data
    if (Test-Path $dataDir) {
        try {
            Remove-Item -Path $dataDir -Recurse -Force -ErrorAction Stop
            Write-Info "Removed data"
        } catch {
            Write-Error-Message "Failed to remove $dataDir : $_"
        }
    }
    
    # Remove cache
    if (Test-Path $cacheDir) {
        try {
            Remove-Item -Path $cacheDir -Recurse -Force -ErrorAction Stop
            Write-Info "Removed cache"
        } catch {
            Write-Error-Message "Failed to remove $cacheDir : $_"
        }
    }
    
    # Clean up empty parent directories
    $appDataOnevox = "$env:APPDATA\onevox"
    if ((Test-Path $appDataOnevox) -and ((Get-ChildItem $appDataOnevox -Recurse | Measure-Object).Count -eq 0)) {
        Remove-Item -Path $appDataOnevox -Force -ErrorAction SilentlyContinue
    }
}

# Main uninstall function
function Main {
    Confirm-Uninstall
    
    Write-Host ""
    
    # Stop and remove service
    Remove-Service
    
    # Remove from PATH
    Remove-FromPath
    
    # Remove files
    Remove-Files
    
    Write-Host ""
    Write-Info "✅ Onevox uninstalled successfully"
    Write-Host ""
    
    if (-not (Test-Administrator)) {
        Write-Warn "Note: Some operations may require Administrator privileges"
        Write-Warn "If the service was not removed, run this script as Administrator"
    }
    
    Write-Host "You may need to restart your terminal or log out and back in for PATH changes to take effect." -ForegroundColor Gray
    Write-Host ""
}

# Run main function
Main
