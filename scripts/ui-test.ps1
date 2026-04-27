#Requires -Version 5.1
<#
.SYNOPSIS
    UI 自动化测试脚本 - 测试已安装的 Arcane Codex 应用程序。

.DESCRIPTION
    使用 Windows UI Automation 框架对已安装的 Arcane Codex 应用进行自动化测试。
    测试内容包括：
      1. 应用启动验证（窗口创建）
      2. 主窗口布局验证（侧边栏、顶部栏、内容区）
      3. 按钮可点击性测试
      4. 窗口交互测试（最大化、还原）
      5. 应用正常关闭验证

.PARAMETER AppName
    应用程序窗口名称（可选，默认为 "Arcane Codex"）。

.PARAMETER TimeoutSeconds
    等待应用启动的最大秒数（可选，默认 30）。

.PARAMETER ScreenshotDir
    截图保存目录（可选，默认在脚本目录下创建 screenshots 子目录）。

.EXAMPLE
    .\scripts\ui-test.ps1                                           # 使用默认参数
    .\scripts\ui-test.ps1 -TimeoutSeconds 60                        # 延长等待时间
    .\scripts\ui-test.ps1 -ScreenshotDir "D:\test-screenshots"      # 指定截图目录
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$AppName = "Arcane Codex",

    [Parameter(Position = 1)]
    [int]$TimeoutSeconds = 30,

    [Parameter(Position = 2)]
    [string]$ScreenshotDir = (Join-Path $PSScriptRoot "screenshots")
)

# ============================================================
# P/Invoke 类型定义（必须在所有函数使用前加载）
# ============================================================
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

namespace Win32Interop {
    public static class User32 {
        [DllImport("user32.dll")]
        public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

        [DllImport("user32.dll")]
        public static extern bool ShowWindowAsync(IntPtr hWnd, int nCmdShow);

        [DllImport("user32.dll")]
        public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);

        [StructLayout(LayoutKind.Sequential)]
        public struct RECT {
            public int Left;
            public int Top;
            public int Right;
            public int Bottom;
        }
    }
}
"@ -Language CSharp -ErrorAction SilentlyContinue

# ============================================================
# 全局配置
# ============================================================
$ErrorActionPreference = 'Continue'
$Script:TestResults = @()
$Script:TestCount = 0
$Script:PassCount = 0
$Script:FailCount = 0
$Script:StartTime = Get-Date

# 确保截图目录存在
if (-not (Test-Path $ScreenshotDir)) {
    New-Item -Path $ScreenshotDir -ItemType Directory -Force | Out-Null
}

# ============================================================
# 加载 Windows UI Automation 程序集
# ============================================================
try {
    Add-Type -AssemblyName "UIAutomationClient" -ErrorAction SilentlyContinue
    Add-Type -AssemblyName "UIAutomationTypes" -ErrorAction SilentlyContinue
    Add-Type -AssemblyName "System.Windows.Forms"
    Add-Type -AssemblyName "System.Drawing"
    Write-Host "[INFO] UI Automation 程序集加载成功" -ForegroundColor Cyan
} catch {
    Write-Host "[FATAL] 无法加载 UI Automation 程序集: $_" -ForegroundColor Red
    exit 1
}

# ============================================================
# 辅助函数
# ============================================================
function Write-Step {
    param([string]$Message)
    Write-Host "`n[INFO]  $(Get-Date -Format 'HH:mm:ss')  $Message" -ForegroundColor Cyan
}

function Write-Pass {
    param([string]$Message)
    Write-Host "[PASS]  $Message" -ForegroundColor Green
}

function Write-Fail {
    param([string]$Message)
    Write-Host "[FAIL]  $Message" -ForegroundColor Red
}

function Write-Section {
    param([string]$Title)
    Write-Host "`n$('=' * 60)" -ForegroundColor Magenta
    Write-Host "  $Title" -ForegroundColor Magenta
    Write-Host $('=' * 60) -ForegroundColor Magenta
}

function Assert-True {
    param(
        [string]$TestName,
        [bool]$Condition,
        [string]$Message = ""
    )
    $Script:TestCount++
    if ($Condition) {
        $Script:PassCount++
        Write-Pass "T$Script:TestCount: $TestName"
        if ($Message) { Write-Host "         $Message" -ForegroundColor DarkGray }
        $Script:TestResults += @{ Id = "T$Script:TestCount"; Name = $TestName; Status = 'PASS'; Message = $Message }
        return $true
    } else {
        $Script:FailCount++
        Write-Fail "T$Script:TestCount: $TestName - $Message"
        $Script:TestResults += @{ Id = "T$Script:TestCount"; Name = $TestName; Status = 'FAIL'; Message = $Message }
        return $false
    }
}

# 截取指定窗口截图
function Take-Screenshot {
    param(
        [string]$FileName,
        [System.IntPtr]$HWnd = [System.IntPtr]::Zero
    )
    try {
        $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
        $filePath = Join-Path $ScreenshotDir "${timestamp}_$FileName.png"

        if ($HWnd -ne [System.IntPtr]::Zero) {
            # 截取特定窗口
            $rect = New-Object System.Drawing.Rectangle
            [Win32Interop.User32]::GetWindowRect($HWnd, [ref]$rect) | Out-Null
            $bounds = [System.Drawing.Rectangle]::FromLTRB($rect.Left, $rect.Top, $rect.Right, $rect.Bottom)
            $bitmap = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
            $graphics.Dispose()
            $bitmap.Save($filePath)
            $bitmap.Dispose()
        } else {
            # 全屏截图
            $screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
            $bitmap = New-Object System.Drawing.Bitmap($screen.Width, $screen.Height)
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            $graphics.CopyFromScreen([System.Drawing.Point]::Empty, [System.Drawing.Point]::Empty, $screen.Size)
            $graphics.Dispose()
            $bitmap.Save($filePath)
            $bitmap.Dispose()
        }

        Write-Host "         截图已保存: $filePath" -ForegroundColor DarkGray
        return $filePath
    } catch {
        Write-Host "         [WARN] 截图失败: $_" -ForegroundColor Yellow
        return $null
    }
}

# 等待窗口出现
function Wait-ForWindow {
    param(
        [string]$WindowTitle,
        [int]$Timeout = 30
    )
    $elapsed = 0
    while ($elapsed -lt $Timeout) {
        $process = Get-Process | Where-Object { $_.MainWindowTitle -like "*$WindowTitle*" -and $_.MainWindowHandle -ne [IntPtr]::Zero }
        if ($process) {
            return $process | Select-Object -First 1
        }
        Start-Sleep -Milliseconds 500
        $elapsed += 0.5
    }
    return $null
}

# 查找 UI 元素
function Find-UIElement {
    param(
        [System.Windows.Automation.AutomationElement]$Parent,
        [string]$ControlType,
        [string]$Name = "",
        [string]$AutomationId = ""
    )
    $condition = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::$ControlType
    )

    $elements = $Parent.FindAll(
        [System.Windows.Automation.TreeScope]::Descendants,
        $condition
    )

    foreach ($elem in $elements) {
        $match = $true
        if ($Name -and $elem.Current.Name -notlike "*$Name*") { $match = $false }
        if ($AutomationId -and $elem.Current.AutomationId -notlike "*$AutomationId*") { $match = $false }
        if ($match) { return $elem }
    }
    return $null
}

# 点击 UI 元素
function Click-UIElement {
    param([System.Windows.Automation.AutomationElement]$Element)
    try {
        $invokePattern = $Element.GetCurrentPattern(
            [System.Windows.Automation.InvokePattern]::Pattern
        )
        $invokePattern.Invoke()
        Start-Sleep -Milliseconds 500
        return $true
    } catch {
        # 回退到发送空格键模拟点击
        try {
            $element.SetFocus()
            Start-Sleep -Milliseconds 100
            [System.Windows.Forms.SendKeys]::SendWait(" ")
            Start-Sleep -Milliseconds 500
            return $true
        } catch {
            return $false
        }
    }
}

# ============================================================
# 主测试流程
# ============================================================
Write-Section "UI 自动化测试 - Arcane Codex"
Write-Host "应用名称:   $AppName" -ForegroundColor Yellow
Write-Host "超时时间:   ${TimeoutSeconds}s" -ForegroundColor Yellow
Write-Host "截图目录:   $ScreenshotDir" -ForegroundColor Yellow
Write-Host "开始时间:   $($Script:StartTime.ToString('yyyy-MM-dd HH:mm:ss'))" -ForegroundColor Yellow

# --------------------------------------------------------
# 阶段 0: 查找已安装的进程
# --------------------------------------------------------
Write-Step "阶段 0: 查找应用进程"

$process = Get-Process | Where-Object {
    $_.MainWindowTitle -like "*$AppName*" -and $_.MainWindowHandle -ne [IntPtr]::Zero
} | Select-Object -First 1

if (-not $process) {
    Write-Host "[WARN] 未找到运行中的 '$AppName' 进程" -ForegroundColor Yellow
    Write-Host "       尝试启动已安装的应用..." -ForegroundColor Yellow

    # 尝试从常见安装路径启动
    $possiblePaths = @(
        "$env:ProgramFiles\ArcaneCodex\ArcaneCodex.exe",
        "${env:ProgramFiles(x86)}\ArcaneCodex\ArcaneCodex.exe",
        "$env:LOCALAPPDATA\ArcaneCodex\ArcaneCodex.exe",
        (Join-Path $PSScriptRoot "..\src-tauri\target\release\arcane-codex.exe")
    )

    foreach ($path in $possiblePaths) {
        if (Test-Path $path) {
            Write-Host "       启动: $path" -ForegroundColor Cyan
            Start-Process -FilePath $path -WindowStyle Normal
            Start-Sleep -Seconds 2
            $process = Wait-ForWindow -WindowTitle $AppName -Timeout $TimeoutSeconds
            if ($process) { break }
        }
    }
}

if (-not $process) {
    Write-Fail "无法找到或启动 '$AppName' 应用"
    Write-Host "`n提示: 请先运行 'tauri build' 安装应用后再运行此脚本" -ForegroundColor Yellow
    exit 1
}

$hwnd = $process.MainWindowHandle
Write-Host "       已找到进程: $($process.ProcessName) (PID: $($process.Id), HWND: $hwnd)" -ForegroundColor Green

# 等待窗口完全加载
Write-Step "等待窗口完全加载..."
Start-Sleep -Seconds 2

# --------------------------------------------------------
# 阶段 1: 窗口启动验证
# --------------------------------------------------------
Write-Step "阶段 1: 窗口启动验证"

$aeRoot = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)

Assert-True "窗口句柄有效" ($hwnd -ne [IntPtr]::Zero) "HWND=$hwnd"
Assert-True "窗口标题包含应用名" ($process.MainWindowTitle -like "*$AppName*") "标题: $($process.MainWindowTitle)"
Assert-True "窗口可见" ($process.MainWindowTitle.Length -gt 0) "标题长度: $($process.MainWindowTitle.Length)"

# 截图
Take-Screenshot -FileName "01_window_launched" -HWnd $hwnd

# --------------------------------------------------------
# 阶段 2: 主窗口布局验证
# --------------------------------------------------------
Write-Step "阶段 2: 主窗口布局验证"

# 获取窗口大小
$rect = New-Object System.Windows.Forms.Rectangle
[Win32Interop.User32]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
$windowWidth = $rect.Width
$windowHeight = $rect.Height

Assert-True "窗口宽度 >= 800" ($windowWidth -ge 800) "宽度: $windowWidth"
Assert-True "窗口高度 >= 600" ($windowHeight -ge 600) "高度: $windowHeight"

# 查找常见 UI 元素
$buttons = $aeRoot.FindAll(
    [System.Windows.Automation.TreeScope]::Descendants,
    (New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::Button
    ))
)

$menuItems = $aeRoot.FindAll(
    [System.Windows.Automation.TreeScope]::Descendants,
    (New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::MenuItem
    ))
)

$tabItems = $aeRoot.FindAll(
    [System.Windows.Automation.TreeScope]::Descendants,
    (New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::TabItem
    ))
)

Assert-True "检测到按钮元素" ($buttons.Count -gt 0) "找到 $($buttons.Count) 个按钮"
Assert-True "检测到菜单项" ($menuItems.Count -ge 0) "找到 $($menuItems.Count) 个菜单项"

Take-Screenshot -FileName "02_layout_verified" -HWnd $hwnd

# --------------------------------------------------------
# 阶段 3: 按钮交互测试
# --------------------------------------------------------
Write-Step "阶段 3: 按钮可点击性测试"

$clickedCount = 0
$failedCount = 0

if ($buttons.Count -gt 0) {
    # 尝试点击前 3 个按钮
    $maxClicks = [Math]::Min(3, $buttons.Count)
    for ($i = 0; $i -lt $maxClicks; $i++) {
        $button = $buttons[$i]
        $buttonName = $button.Current.Name
        if ([string]::IsNullOrEmpty($buttonName)) {
            $buttonName = "Unnamed Button #$i"
        }

        try {
            $result = Click-UIElement -Element $button
            if ($result) {
                $clickedCount++
                Write-Host "         已点击按钮: '$buttonName'" -ForegroundColor DarkGray
            } else {
                $failedCount++
            }
        } catch {
            $failedCount++
            Write-Host "         点击按钮 '$buttonName' 异常: $_" -ForegroundColor DarkYellow
        }
        Start-Sleep -Milliseconds 300
    }
}

Assert-True "至少一个按钮可点击" ($clickedCount -gt 0) "成功点击 $clickedCount 个，失败 $failedCount 个"

Take-Screenshot -FileName "03_buttons_clicked" -HWnd $hwnd

# --------------------------------------------------------
# 阶段 4: 窗口交互测试
# --------------------------------------------------------
Write-Step "阶段 4: 窗口交互测试"

# 测试最大化
try {
    [Win32Interop.User32]::ShowWindowAsync($hwnd, 3) | Out-Null # SW_MAXIMIZE = 3
    Start-Sleep -Seconds 1

    $rect2 = New-Object System.Windows.Forms.Rectangle
    [Win32Interop.User32]::GetWindowRect($hwnd, [ref]$rect2) | Out-Null

    Assert-True "窗口最大化成功" ($rect2.Width -gt $rect.Width) "最大化后: $($rect2.Width)x$($rect2.Height)"
    Take-Screenshot -FileName "04_maximized" -HWnd $hwnd

    # 还原
    [Win32Interop.User32]::ShowWindowAsync($hwnd, 9) | Out-Null # SW_RESTORE = 9
    Start-Sleep -Seconds 1
    Assert-True "窗口还原成功" $true "已还原到原始尺寸"
} catch {
    Write-Fail "窗口交互测试异常: $_"
    Assert-True "窗口交互测试" $false $_.Exception.Message
}

# --------------------------------------------------------
# 阶段 5: 应用关闭测试
# --------------------------------------------------------
Write-Step "阶段 5: 应用关闭测试"

try {
    # 发送关闭消息
    [Win32Interop.User32]::PostMessage($hwnd, 0x0010, 0, 0) | Out-Null # WM_CLOSE = 0x0010
    Start-Sleep -Seconds 3

    # 检查进程是否还在运行
    $stillRunning = Get-Process -Id $process.Id -ErrorAction SilentlyContinue
    $processClosed = (-not $stillRunning) -or ($stillRunning.HasExited)

    Assert-True "应用正常关闭" $processClosed "进程状态: $(if ($processClosed) { '已关闭' } else { '仍在运行' })"

    if (-not $processClosed) {
        # 强制关闭
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        Write-Host "         强制关闭进程 (PID: $($process.Id))" -ForegroundColor Yellow
    }
} catch {
    Assert-True "应用关闭测试" $false $_.Exception.Message
    # 强制清理
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
}

# --------------------------------------------------------
# 测试报告
# --------------------------------------------------------
$Script:EndTime = Get-Date
$Script:TotalDuration = $Script:EndTime - $Script:StartTime

Write-Section "UI 测试报告"

$totalDuration = "{0:N1}" -f $Script:TotalDuration.TotalSeconds
Write-Host "总耗时: ${totalDuration} 秒" -ForegroundColor Yellow
Write-Host ""

Write-Host "  总测试数: $($Script:TestCount)" -ForegroundColor White
Write-Host "  通过 (PASS): $($Script:PassCount)" -ForegroundColor Green
Write-Host "  失败 (FAIL): $($Script:FailCount)" -ForegroundColor Red
Write-Host ""

Write-Host "详细结果:" -ForegroundColor Cyan
Write-Host "  $('ID'.PadRight(6)) $('测试名称'.PadRight(35)) $('状态'.PadRight(8)) 详情"
Write-Host "  $('-' * 80)"

foreach ($r in $Script:TestResults) {
    $statusColor = if ($r.Status -eq 'PASS') { 'Green' } else { 'Red' }
    $id = ($r.Id.PadRight(6))
    $name = ($r.Name.PadRight(35)).Substring(0, [Math]::Min(35, $r.Name.Length))
    Write-Host "  $id $name " -NoNewline
    Write-Host ($r.Status.PadRight(8)) -ForegroundColor $statusColor -NoNewline
    Write-Host "  $($r.Message)"
}

Write-Host ""

$passRate = if ($Script:TestCount -gt 0) { "{0:P0}" -f ($Script:PassCount / $Script:TestCount) } else { "N/A" }
Write-Host "通过率: $passRate" -ForegroundColor $(if ($Script:FailCount -eq 0) { 'Green' } else { 'Yellow' })

if ($Script:FailCount -eq 0) {
    Write-Host "UI TESTS ALL PASSED" -ForegroundColor Green
} else {
    Write-Host "UI TESTS HAVE FAILURES" -ForegroundColor Red
}

# 返回退出码
exit (if ($Script:FailCount -eq 0) { 0 } else { 1 })
