$ErrorActionPreference = 'Stop'

$installDir = "C:\ProgramData\Molestus"
$modelsDir = "D:\Molestus\models"

Write-Host "Installing Molestus..."

if (!(Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

if (!(Test-Path $modelsDir)) {
    New-Item -ItemType Directory -Force -Path $modelsDir | Out-Null
}

$repoOwner = "turtle170"
$repoName = "Molestus"
$apiUrl = "https://api.github.com/repos/$repoOwner/$repoName/releases/latest"

try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{"User-Agent"="PowerShell"}
    $asset = $release.assets | Where-Object { $_.name -eq "Molestus.exe" }
    
    if ($asset) {
        $downloadUrl = $asset.browser_download_url
        $exePath = Join-Path $installDir "Molestus.exe"
        Write-Host "Downloading Molestus.exe from $downloadUrl..."
        Invoke-WebRequest -Uri $downloadUrl -OutFile $exePath
        Write-Host "Downloaded binary successfully."
    } else {
        Write-Host "Could not find Molestus.exe in the latest release."
    }
} catch {
    Write-Host "Error fetching release from GitHub: $_. You might need to build it manually."
}

# Download models
$modelUrls = @(
    "https://huggingface.co/unsloth/Qwen3-VL-2B-Instruct-1M-GGUF/resolve/main/Qwen3-VL-2B-Instruct-1M-UD-Q6_K_XL.gguf",
    "https://huggingface.co/unsloth/Qwen3-VL-2B-Instruct-1M-GGUF/resolve/main/mmproj-F16.gguf"
)

foreach ($url in $modelUrls) {
    $fileName = Split-Path $url -Leaf
    $destination = Join-Path $modelsDir $fileName
    if (!(Test-Path $destination)) {
        Write-Host "Downloading $fileName to $modelsDir (This may take a while)..."
        Invoke-WebRequest -Uri $url -OutFile $destination
    } else {
        Write-Host "$fileName already exists in $modelsDir, skipping download."
    }
}

# Create shortcut on Desktop
$wshShell = New-Object -ComObject WScript.Shell
$desktopPath = [Environment]::GetFolderPath("Desktop")
$shortcut = $wshShell.CreateShortcut((Join-Path $desktopPath "Molestus.lnk"))
$shortcut.TargetPath = Join-Path $installDir "Molestus.exe"
$shortcut.WorkingDirectory = $installDir
$shortcut.Save()

Write-Host "Installation complete! Shortcut created on Desktop."
