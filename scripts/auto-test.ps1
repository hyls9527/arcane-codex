# 沙箱内自动测试脚本
# 此脚本会在 Windows Sandbox 启动后自动运行

$ErrorActionPreference = 'Continue'
$report = @()
$passCount = 0
$failCount = 0

function Write-Test {
    param([string]$Name, [bool]$Result, [string]$Detail = "")
    $status = if ($Result) { "PASS" } else { "FAIL" }
    $report += "[$status] $Name - $Detail"
    if ($Result) { $script:passCount++ } else { $script:failCount++ }
    Write-Host "[$status] $Name - $Detail"
}

# ===== 阶段 1: 安装应用 =====
Write-Host "`n=== 阶段 1: 安装应用 ===" -ForegroundColor Cyan
$installer = "C:\Users\WDAGUtilityAccount\Install\ArcaneCodex_0.1.0_x64-setup.exe"
if (Test-Path $installer) {
    Start-Process $installer -ArgumentList "/S" -Wait -PassThru | Out-Null
    Write-Test "安装程序存在" $true
} else {
    Write-Test "安装程序存在" $false "路径: $installer"
}

Start-Sleep -Seconds 5

# ===== 阶段 2: 启动验证 =====
Write-Host "`n=== 阶段 2: 启动验证 ===" -ForegroundColor Cyan
$appPath = "$env:LOCALAPPDATA\ArcaneCodex\arcane-codex.exe"
if (Test-Path $appPath) {
    Write-Test "应用文件存在" $true
    $proc = Start-Process $appPath -PassThru
    Start-Sleep -Seconds 5
    
    $proc = Get-Process -Name "arcane-codex" -ErrorAction SilentlyContinue
    Write-Test "应用进程启动" ($null -ne $proc) "PID: $($proc?.Id)"
    
    if ($proc) {
        Write-Test "窗口创建" ($proc.MainWindowHandle -ne [IntPtr]::Zero)
        
        # 检查是否报错退出
        Start-Sleep -Seconds 3
        $stillRunning = Get-Process -Name "arcane-codex" -ErrorAction SilentlyContinue
        Write-Test "应用持续运行" ($null -ne $stillRunning)
        
        # 关闭应用
        Stop-Process -Name "arcane-codex" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
        Write-Test "应用正常关闭" ($null -eq (Get-Process -Name "arcane-codex" -ErrorAction SilentlyContinue))
    }
} else {
    Write-Test "应用文件存在" $false "路径: $appPath"
}

# ===== 阶段 3: 快捷方式验证 =====
Write-Host "`n=== 阶段 3: 快捷方式验证 ===" -ForegroundColor Cyan
$startMenuLnk = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\ArcaneCodex.lnk"
Write-Test "开始菜单快捷方式" (Test-Path $startMenuLnk)

# ===== 阶段 4: 桌面快捷方式验证 =====
Write-Host "`n=== 阶段 4: 桌面图标验证 ===" -ForegroundColor Cyan
$desktopLnk = "$env:USERPROFILE\Desktop\ArcaneCodex.lnk"
if (Test-Path $desktopLnk) {
    Write-Test "桌面快捷方式" $true
    $iconPath = "C:\Users\WDAGUtilityAccount\Install\..\..\..\..\..\src-tauri\icons\icon.png"
    if (Test-Path $iconPath) {
        Write-Test "图标文件" $true
    } else {
        Write-Test "图标文件" $false "无法找到源图标"
    }
} else {
    Write-Test "桌面快捷方式" $false "桌面没有 ArcaneCodex 快捷方式"
}

# ===== 生成报告 =====
Write-Host "`n=== 测试报告 ===" -ForegroundColor Green
Write-Host "总计: $($passCount + $failCount) | 通过: $passCount | 失败: $failCount"
$report | ForEach-Object { Write-Host "  $_" }

# 保存报告
$report | Out-File "$env:USERPROFILE\Desktop\test-report.txt" -Encoding UTF8

if ($failCount -eq 0) {
    Write-Host "`n全部测试通过！" -ForegroundColor Green
} else {
    Write-Host "`n存在 $failCount 个失败项，请检查报告。" -ForegroundColor Red
}

Write-Host "`n按任意键退出沙箱..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
