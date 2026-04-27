#Requires -Version 5.1
<#
.SYNOPSIS
    快速预构建检查脚本 - 在构建安装包之前运行所有验证。

.DESCRIPTION
    执行前端 TypeScript 类型检查、前端单元测试、后端 Rust 单元测试。
    所有检查通过后输出绿色 "ALL CHECKS PASSED"，否则输出红色错误详情。

.PARAMETER ProjectRoot
    项目根目录路径（可选，默认为脚本所在目录的上两级）。

.PARAMETER SkipFrontend
    跳过前端检查（类型检查 + 单元测试）。

.PARAMETER SkipBackend
    跳过后端 Rust 单元测试。

.EXAMPLE
    .\scripts\fast-check.ps1                          # 运行全部检查
    .\scripts\fast-check.ps1 -SkipBackend             # 仅检查前端
    .\scripts\fast-check.ps1 -ProjectRoot "D:\myapp"  # 指定项目根目录
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$ProjectRoot = (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)),

    [switch]$SkipFrontend,

    [switch]$SkipBackend
)

# ============================================================
# 全局配置
# ============================================================
$ErrorActionPreference = 'Continue'
$Script:ExitCode = 0
$Script:Results = @()
$Script:StartTime = Get-Date

$FrontendDir = Join-Path $ProjectRoot 'frontend'
$BackendDir  = Join-Path $ProjectRoot 'src-tauri'

# 颜色输出辅助函数
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

# ============================================================
# 前置条件验证
# ============================================================
Write-Section "快速预构建检查 (Fast Pre-build Check)"
Write-Host "项目根目录: $ProjectRoot" -ForegroundColor Yellow
Write-Host "开始时间:   $($Script:StartTime.ToString('yyyy-MM-dd HH:mm:ss'))" -ForegroundColor Yellow

# 验证项目结构
if (-not (Test-Path $FrontendDir)) {
    Write-Fail "前端目录不存在: $FrontendDir"
    exit 1
}
if (-not (Test-Path $BackendDir)) {
    Write-Fail "后端目录不存在: $BackendDir"
    exit 1
}

# ============================================================
# 阶段 1: 前端 TypeScript 类型检查
# ============================================================
if (-not $SkipFrontend) {
    Write-Section "阶段 1: 前端 TypeScript 类型检查"

    $stageName = "Frontend: TypeScript Type Check"
    $stageStart = Get-Date

    try {
        Push-Location $FrontendDir

        # 检查 node_modules 是否存在
        if (-not (Test-Path (Join-Path $FrontendDir 'node_modules'))) {
            Write-Host "[WARN] node_modules 不存在，正在安装依赖..." -ForegroundColor Yellow
            npm install --prefer-offline
            if ($LASTEXITCODE -ne 0) {
                Write-Fail "$stageName - npm install 失败"
                $Script:Results += @{ Name = $stageName; Status = 'FAIL'; Duration = ((Get-Date) - $stageStart); Error = 'npm install failed' }
                $Script:ExitCode = 1
                Pop-Location
                continue
            }
        }

        Write-Step "运行 npx tsc --noEmit ..."
        $tscOutput = & npx tsc --noEmit 2>&1
        $tscExitCode = $LASTEXITCODE

        if ($tscExitCode -ne 0) {
            $errorDetails = ($tscOutput | Where-Object { $_ -match 'error TS' }) -join "`n"
            Write-Fail "$stageName - 发现类型错误 ($tscExitCode)"
            if ($errorDetails) {
                Write-Host $errorDetails -ForegroundColor DarkRed
            }
            $Script:Results += @{ Name = $stageName; Status = 'FAIL'; Duration = ((Get-Date) - $stageStart); Error = $errorDetails }
            $Script:ExitCode = 1
        } else {
            Write-Pass "$stageName - 通过"
            $Script:Results += @{ Name = $stageName; Status = 'PASS'; Duration = ((Get-Date) - $stageStart); Error = $null }
        }

        Pop-Location
    } catch {
        Write-Fail "$stageName - 异常: $_"
        $Script:Results += @{ Name = $stageName; Status = 'FAIL'; Duration = ((Get-Date) - $stageStart); Error = $_.Exception.Message }
        $Script:ExitCode = 1
        if ($PWD.Path -eq $FrontendDir) { Pop-Location }
    }
} else {
    Write-Step "跳过前端检查 (-SkipFrontend)"
}

# ============================================================
# 阶段 2: 前端单元测试
# ============================================================
if (-not $SkipFrontend) {
    Write-Section "阶段 2: 前端单元测试 (Vitest)"

    $stageName = "Frontend: Unit Tests (Vitest)"
    $stageStart = Get-Date

    try {
        Push-Location $FrontendDir

        Write-Step "运行 npx vitest run ..."
        $vitestOutput = & npx vitest run 2>&1
        $vitestExitCode = $LASTEXITCODE

        # 显示完整输出
        $vitestOutput | ForEach-Object { Write-Host $_ }

        if ($vitestExitCode -ne 0) {
            Write-Fail "$stageName - 测试失败 ($vitestExitCode)"
            $Script:Results += @{ Name = $stageName; Status = 'FAIL'; Duration = ((Get-Date) - $stageStart); Error = ($vitestOutput -join "`n") }
            $Script:ExitCode = 1
        } else {
            Write-Pass "$stageName - 通过"
            $Script:Results += @{ Name = $stageName; Status = 'PASS'; Duration = ((Get-Date) - $stageStart); Error = $null }
        }

        Pop-Location
    } catch {
        Write-Fail "$stageName - 异常: $_"
        $Script:Results += @{ Name = $stageName; Status = 'FAIL'; Duration = ((Get-Date) - $stageStart); Error = $_.Exception.Message }
        $Script:ExitCode = 1
        if ($PWD.Path -eq $FrontendDir) { Pop-Location }
    }
} else {
    Write-Step "跳过前端检查 (-SkipFrontend)"
}

# ============================================================
# 阶段 3: 后端 Rust 单元测试
# ============================================================
if (-not $SkipBackend) {
    Write-Section "阶段 3: 后端 Rust 单元测试"

    $stageName = "Backend: Rust Unit Tests (cargo test --lib)"
    $stageStart = Get-Date

    try {
        # 检查 cargo 是否可用
        $null = Get-Command cargo -ErrorAction SilentlyContinue
        if (-not $?) {
            Write-Fail "cargo 未找到，请确保 Rust toolchain 已安装"
            $Script:Results += @{ Name = $stageName; Status = 'FAIL'; Duration = ((Get-Date) - $stageStart); Error = 'cargo not found' }
            $Script:ExitCode = 1
        } else {
            Push-Location $BackendDir

            Write-Step "运行 cargo test --lib ..."
            # 使用 --no-fail-fast 来显示所有测试输出
            $cargoOutput = & cargo test --lib --no-fail-fast 2>&1
            $cargoExitCode = $LASTEXITCODE

            # 过滤并显示关键输出
            $cargoOutput | Where-Object {
                $_ -match 'test .* \.\.\.' -or
                $_ -match 'running \d+ test' -or
                $_ -match 'test result:' -or
                $_ -match 'FAILED' -or
                $_ -match 'error\[' -or
                $_ -match 'warning:'
            } | ForEach-Object {
                if ($_ -match 'FAILED|error\[') {
                    Write-Host $_ -ForegroundColor Red
                } elseif ($_ -match 'test result:.*passed') {
                    Write-Host $_ -ForegroundColor Green
                } else {
                    Write-Host $_
                }
            }

            if ($cargoExitCode -ne 0) {
                $errorDetails = ($cargoOutput | Where-Object { $_ -match 'FAILED|thread .* panicked|error\[' }) -join "`n"
                Write-Fail "$stageName - 测试失败 ($cargoExitCode)"
                if ($errorDetails) {
                    Write-Host $errorDetails -ForegroundColor DarkRed
                }
                $Script:Results += @{ Name = $stageName; Status = 'FAIL'; Duration = ((Get-Date) - $stageStart); Error = $errorDetails }
                $Script:ExitCode = 1
            } else {
                Write-Pass "$stageName - 通过"
                $Script:Results += @{ Name = $stageName; Status = 'PASS'; Duration = ((Get-Date) - $stageStart); Error = $null }
            }

            Pop-Location
        }
    } catch {
        Write-Fail "$stageName - 异常: $_"
        $Script:Results += @{ Name = $stageName; Status = 'FAIL'; Duration = ((Get-Date) - $stageStart); Error = $_.Exception.Message }
        $Script:ExitCode = 1
        if ($PWD.Path -eq $BackendDir) { Pop-Location }
    }
} else {
    Write-Step "跳过后端检查 (-SkipBackend)"
}

# ============================================================
# 结果汇总
# ============================================================
$Script:EndTime = Get-Date
$Script:TotalDuration = $Script:EndTime - $Script:StartTime

Write-Section "检查结果汇总"

$totalDuration = "{0:N1}" -f $Script:TotalDuration.TotalSeconds
Write-Host "总耗时: ${totalDuration} 秒" -ForegroundColor Yellow
Write-Host ""

$passCount = ($Script:Results | Where-Object { $_.Status -eq 'PASS' }).Count
$failCount = ($Script:Results | Where-Object { $_.Status -eq 'FAIL' }).Count
$skipCount = 0

if ($SkipFrontend) { $skipCount += 2 }
if ($SkipBackend) { $skipCount += 1 }

Write-Host "  通过 (PASS): $passCount" -ForegroundColor Green
Write-Host "  失败 (FAIL): $failCount" -ForegroundColor Red
if ($skipCount -gt 0) {
    Write-Host "  跳过 (SKIP): $skipCount" -ForegroundColor Yellow
}
Write-Host ""

# 详细结果表
Write-Host "阶段详情:" -ForegroundColor Cyan
Write-Host "  $('阶段名称'.PadRight(50)) $('状态'.PadRight(8)) 耗时"
Write-Host "  $('-' * 75)"

foreach ($r in $Script:Results) {
    $duration = "{0:N1}s" -f $r.Duration.TotalSeconds
    $statusColor = if ($r.Status -eq 'PASS') { 'Green' } else { 'Red' }
    $name = ($r.Name.PadRight(50)).Substring(0, [Math]::Min(50, $r.Name.Length))
    Write-Host "  $name " -NoNewline
    Write-Host ($r.Status.PadRight(8)) -ForegroundColor $statusColor -NoNewline
    Write-Host "  $duration"

    if ($r.Error) {
        # 尝试从错误信息中提取 file:line 格式的位置信息
        $locationMatch = [regex]::Match($r.Error, '--> (src/[^\s:]+:\d+:\d+)')
        if ($locationMatch.Success) {
            Write-Host "    位置: $($locationMatch.Groups[1].Value)" -ForegroundColor DarkYellow
        }
        # 显示 TypeScript 错误的位置
        $tsMatch = [regex]::Match($r.Error, '(src/[^:]+:\d+:\d+)')
        if ($tsMatch.Success) {
            Write-Host "    位置: $($tsMatch.Groups[1].Value)" -ForegroundColor DarkYellow
        }
    }
}

Write-Host ""

if ($Script:ExitCode -eq 0) {
    Write-Host "  ALL CHECKS PASSED - 可以继续进行构建" -ForegroundColor Green
    Write-Host ""
} else {
    Write-Host "  CHECKS FAILED - 请先修复上述错误再继续构建" -ForegroundColor Red
    Write-Host ""

    # 推荐修复步骤
    Write-Host "推荐修复步骤:" -ForegroundColor Cyan
    foreach ($r in ($Script:Results | Where-Object { $_.Status -eq 'FAIL' })) {
        Write-Host "  - $($r.Name):" -ForegroundColor Yellow
        if ($r.Name -match 'TypeScript') {
            Write-Host "    1. 运行 'cd frontend && npx tsc --noEmit' 查看完整错误列表"
            Write-Host "    2. 修复类型错误后重新运行此脚本"
        } elseif ($r.Name -match 'Vitest') {
            Write-Host "    1. 运行 'cd frontend && npx vitest run' 查看测试失败详情"
            Write-Host "    2. 修复失败的测试用例后重新运行此脚本"
        } elseif ($r.Name -match 'Rust') {
            Write-Host "    1. 运行 'cd src-tauri && cargo test --lib' 查看完整测试输出"
            Write-Host "    2. 修复失败的单元测试后重新运行此脚本"
        }
    }
    Write-Host ""
}

exit $Script:ExitCode
