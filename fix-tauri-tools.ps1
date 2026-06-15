# This script is designed to run before building a Tauri application if build with error like:
# Error failed to bundle project: `https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip: Connection Failed: Connect error:
# Error failed to bundle project: `https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.1/nsis_tauri_utils.dll: Connection Failed: Connect error:

# More info: https://github.com/tauri-apps/tauri/issues/7338

# 1. 删除之前的临时目录和目标目录，保证干净环境
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue temp
$tauriDir = Join-Path $env:USERPROFILE "AppData\Local\tauri"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $tauriDir "NSIS")
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue (Join-Path $tauriDir "WixTools314")

# 2. 创建临时目录
if (!(Test-Path temp)) { mkdir temp }
cd temp

# 3. 下载 wix 工具（WixTools314）
Invoke-WebRequest -Uri "https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip" -OutFile "wix314-binaries.zip"
Expand-Archive ./wix314-binaries.zip -DestinationPath ./WixTools314

# 4. 下载 NSIS
Invoke-WebRequest -Uri "https://github.com/tauri-apps/binary-releases/releases/download/nsis-3/nsis-3.zip" -OutFile "nsis-3.zip"
Expand-Archive ./nsis-3.zip -DestinationPath ./NSIS

# 5. 移动 NSIS 文件夹下的 nsis-3.* 子目录内容到 NSIS 根目录
$nsisSubDir = Get-ChildItem .\NSIS | Where-Object { $_.PSIsContainer -and $_.Name -like 'nsis-3.*' } | Select-Object -First 1
if ($nsisSubDir) {
    Move-Item -Path (Join-Path $nsisSubDir.FullName '*') -Destination .\NSIS -Force
    Remove-Item $nsisSubDir.FullName -Recurse -Force
}

# 6. 下载 NSIS-ApplicationID 插件并解压
Invoke-WebRequest -Uri "https://github.com/tauri-apps/binary-releases/releases/download/nsis-plugins-v0/NSIS-ApplicationID.zip" -OutFile "NSIS-ApplicationID.zip"
Expand-Archive .\NSIS-ApplicationID.zip -DestinationPath .\NSIS-ApplicationID

# 7. 移动插件文件到 NSIS\Plugins\x86-unicode
$pluginDir = ".\NSIS\Plugins\x86-unicode"
if (!(Test-Path $pluginDir)) { New-Item -ItemType Directory -Path $pluginDir | Out-Null }
if (Test-Path ".\NSIS-ApplicationID\Release") {
    Move-Item ".\NSIS-ApplicationID\Release\*" $pluginDir -Force
}

# 8. 下载 nsis_tauri_utils.dll v0.4.2 并重命名覆盖到插件目录
Invoke-WebRequest -Uri "https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.1/nsis_tauri_utils.dll" -OutFile "nsis_tauri_utils.dll"
if (Test-Path "nsis_tauri_utils.dll") {
    $additionalPluginDir = Join-Path $pluginDir "additional"
    if (!(Test-Path $additionalPluginDir)) { New-Item -ItemType Directory -Path $additionalPluginDir | Out-Null }

    Copy-Item "nsis_tauri_utils.dll" (Join-Path $pluginDir "nsis_tauri_utils.dll") -Force
    Copy-Item "nsis_tauri_utils.dll" (Join-Path $additionalPluginDir "nsis_tauri_utils.dll") -Force
    
    Remove-Item "nsis_tauri_utils.dll" -Force
}

# 9. 移动 NSIS 和 WixTools314 到用户目录
if (!(Test-Path $tauriDir)) { New-Item -ItemType Directory -Path $tauriDir | Out-Null }
Move-Item .\NSIS (Join-Path $tauriDir "NSIS") -Force -ErrorAction SilentlyContinue
Move-Item .\WixTools314 (Join-Path $tauriDir "WixTools314") -Force -ErrorAction SilentlyContinue

Write-Host "rm temp dir"

# 10. 清理临时文件和目录
Remove-Item .\NSIS-ApplicationID -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item .\nsis-3.zip -Force -ErrorAction SilentlyContinue
Remove-Item .\NSIS-ApplicationID.zip -Force -ErrorAction SilentlyContinue
Remove-Item .\wix314-binaries.zip -Force -ErrorAction SilentlyContinue
cd ..
Remove-Item .\temp -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "done"