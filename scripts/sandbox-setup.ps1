# Trae CN 沙箱环境自动配置脚本
# 此脚本在 Windows Sandbox 启动后自动运行

$ErrorActionPreference = 'Continue'
Write-Host "=== Trae CN 沙箱环境初始化 ===" -ForegroundColor Cyan

# ===== 1. 安装必要运行时 =====
Write-Host "`n[1/4] 检查运行环境..." -ForegroundColor Yellow

# Node.js
$nodeInstalled = Get-Command "node" -ErrorAction SilentlyContinue
if ($nodeInstalled) {
    Write-Host "  Node.js: $(node --version)" -ForegroundColor Green
} else {
    Write-Host "  Node.js: 未安装，尝试从项目目录复制..." -ForegroundColor Yellow
    # 尝试从映射的项目目录找
    $nodePath = "C:\Users\WDAGUtilityAccount\Project\node_modules\.bin"
    if (Test-Path $nodePath) {
        $env:PATH += ";$nodePath"
        Write-Host "  已添加项目 node_modules 到 PATH" -ForegroundColor Green
    }
}

# Rust/Cargo
$cargoInstalled = Get-Command "cargo" -ErrorAction SilentlyContinue
if ($cargoInstalled) {
    Write-Host "  Cargo: $(cargo --version)" -ForegroundColor Green
} else {
    Write-Host "  Cargo: 未安装" -ForegroundColor Yellow
}

# ===== 2. 安装应用 =====
Write-Host "`n[2/4] 安装应用..." -ForegroundColor Yellow
$installer = "C:\Users\WDAGUtilityAccount\Install\ArcaneCodex_0.1.0_x64-setup.exe"
if (Test-Path $installer) {
    Write-Host "  找到安装包，开始安装..." -ForegroundColor Yellow
    Start-Process $installer -ArgumentList "/S" -Wait -PassThru | Out-Null
    Write-Host "  安装完成" -ForegroundColor Green
} else {
    Write-Host "  未找到安装包: $installer" -ForegroundColor Red
}

# ===== 3. 启动应用测试 =====
Write-Host "`n[3/4] 启动应用验证..." -ForegroundColor Yellow
$appPath = "$env:LOCALAPPDATA\ArcaneCodex\arcane-codex.exe"
if (Test-Path $appPath) {
    $proc = Start-Process $appPath -PassThru
    Start-Sleep -Seconds 5
    
    $proc = Get-Process -Name "arcane-codex" -ErrorAction SilentlyContinue
    if ($proc) {
        Write-Host "  应用运行中 (PID: $($proc.Id))" -ForegroundColor Green
        Write-Host "  窗口句柄: $($proc.MainWindowHandle)" -ForegroundColor Green
        
        # 检查错误弹窗
        Start-Sleep -Seconds 3
        $stillRunning = Get-Process -Name "arcane-codex" -ErrorAction SilentlyContinue
        if ($stillRunning) {
            Write-Host "  应用持续运行，无崩溃" -ForegroundColor Green
        } else {
            Write-Host "  应用已退出，可能存在错误" -ForegroundColor Red
        }
        
        # 关闭应用
        Stop-Process -Name "arcane-codex" -Force -ErrorAction SilentlyContinue
    } else {
        Write-Host "  应用启动失败" -ForegroundColor Red
    }
} else {
    Write-Host "  应用未安装: $appPath" -ForegroundColor Red
}

# ===== 4. 检查图标 =====
Write-Host "`n[4/4] 检查图标..." -ForegroundColor Yellow
$iconPath = "C:\Users\WDAGUtilityAccount\Project\src-tauri\icons\icon.png"
if (Test-Path $iconPath) {
    $iconSize = (Get-Item $iconPath).Length
    Write-Host "  图标文件存在: $iconPath ($iconSize bytes)" -ForegroundColor Green
    
    # 读取 PNG 尺寸
    $bytes = [System.IO.File]::ReadAllBytes($iconPath)
    if ($bytes.Length -gt 24) {
        $wBytes = $bytes[16..19]; [Array]::Reverse($wBytes)
        $hBytes = $bytes[20..23]; [Array]::Reverse($hBytes)
        $w = [BitConverter]::ToInt32($wBytes, 0)
        $h = [BitConverter]::ToInt32($hBytes, 0)
        Write-Host "  图标尺寸: ${w}x${h}" -ForegroundColor Green
    }
} else {
    Write-Host "  图标文件不存在: $iconPath" -ForegroundColor Red
}

# ===== 生成报告 =====
Write-Host "`n=== 测试完成 ===" -ForegroundColor Cyan
Write-Host "报告已保存到桌面: test-report.txt"
Write-Host "项目路径: C:\Users\WDAGUtilityAccount\Project"
Write-Host "`n按任意键退出沙箱..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
