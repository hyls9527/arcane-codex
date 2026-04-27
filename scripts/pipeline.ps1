#Requires -Version 5.1
<#
.SYNOPSIS
    完整 CI/CD 测试流水线 - Arcane Codex Tauri + React 项目。

.DESCRIPTION
    执行完整的构建流水线，包括：
      阶段 1: 快速预检查（TypeScript 类型检查 + 前端单元测试 + Rust 单元测试）
      阶段 2: Tauri 构建（编译 + 打包 NSIS 安装包）
      阶段 3: UI 自动化测试（对已安装的应用进行 UI 交互测试）

    输出 Markdown 格式的报告，包含每个阶段的 Pass/Fail 状态、错误详情和修复建议。

.PARAMETER ProjectRoot
    项目根目录路径（可选，默认为脚本所在目录的上两级）。

.PARAMETER Stage
    指定只运行某个阶段: "check" | "build" | "ui-test" | "all"（默认）。

.PARAMETER SkipUI
    跳过 UI 自动化测试阶段。

.PARAMETER Fast
    仅运行快速预检查（等同于 -Stage check）。

.PARAMETER OutputReport
    将报告输出到指定的 Markdown 文件路径。

.PARAMETER ReuseRelease
    跳过 Tauri 编译，直接使用已有的 release 二进制进行打包。

.EXAMPLE
    .\scripts\pipeline.ps1                                    # 运行完整流水线
    .\scripts\pipeline.ps1 -Fast                              # 仅运行预检查
    .\scripts\pipeline.ps1 -Stage check                       # 仅运行预检查
    .\scripts\pipeline.ps1 -Stage build                       # 仅运行构建
    .\scripts\pipeline.ps1 -Stage ui-test                     # 仅运行 UI 测试
    .\scripts\pipeline.ps1 -ReuseRelease                      # 跳过编译，仅打包
    .\scripts\pipeline.ps1 -OutputReport "report.md"           # 输出报告到文件
    .\scripts\pipeline.ps1 -SkipUI                            # 跳过 UI 测试
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$ProjectRoot = (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)),

    [ValidateSet("all", "check", "build", "ui-test")]
    [string]$Stage = "all",

    [switch]$SkipUI,

    [switch]$Fast,

    [string]$OutputReport,

    [switch]$ReuseRelease
)

# ============================================================
# 全局配置
# ============================================================
$ErrorActionPreference = 'Continue'
$Script:StageResults = @()
$Script:OverallExitCode = 0
$Script:StartTime = Get-Date
$Script:ReportLines = @()

# 路径配置
$FrontendDir = Join-Path $ProjectRoot 'frontend'
$BackendDir  = Join-Path $ProjectRoot 'src-tauri'
$ScriptsDir  = $PSScriptRoot
$ReleaseDir  = Join-Path $BackendDir 'target\release'
$BundleDir   = Join-Path $BackendDir 'target\release\bundle\nsis'

# Fast 模式等价于仅 check
if ($Fast -and $Stage -eq 'all') {
    $Stage = 'check'
}

# 颜色输出
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
    Write-Host "`n$('=' * 70)" -ForegroundColor Magenta
    Write-Host "  $Title" -ForegroundColor Magenta
    Write-Host $('=' * 70) -ForegroundColor Magenta
}

# 添加报告行
function Add-Report {
    param([string]$Line)
    $Script:ReportLines += $Line
}

# 记录阶段结果
function Record-Stage {
    param(
        [string]$Name,
        [string]$Status,
        [string]$Details = "",
        [TimeSpan]$Duration = (New-TimeSpan)
    )
    $Script:StageResults += @{
        Name     = $Name
        Status   = $Status
        Details  = $Details
        Duration = $Duration
    }
}

# ============================================================
# 报告生成函数
# ============================================================
function Generate-MarkdownReport {
    param([string]$OutputPath)

    $endTime = Get-Date
    $totalDuration = "{0:N1}" -f ($endTime - $Script:StartTime).TotalSeconds

    $report = @"
# Arcane Codex - 构建流水线报告

> 生成时间: $($endTime.ToString('yyyy-MM-dd HH:mm:ss'))
> 项目路径: $ProjectRoot
> 总耗时:   ${totalDuration} 秒

---

## 阶段结果总览

| 阶段 | 状态 | 耗时 | 详情 |
|------|------|------|------|
"@

    foreach ($r in $Script:StageResults) {
        $statusEmoji = if ($r.Status -eq 'PASS') { '✅ PASS' } else { '❌ FAIL' }
        $duration = if ($r.Duration.TotalSeconds -gt 0) { "{0:N1}s" -f $r.Duration.TotalSeconds } else { "N/A" }
        $detail = if ($r.Details) { " $($r.Details)" } else { "" }
        $report += "`n| $($r.Name) | $statusEmoji | $duration |$detail |"
    }

    $passCount = ($Script:StageResults | Where-Object { $_.Status -eq 'PASS' }).Count
    $failCount = ($Script:StageResults | Where-Object { $_.Status -eq 'FAIL' }).Count

    $report += @"


### 汇总

- **通过阶段**: $passCount
- **失败阶段**: $failCount
- **总体状态**: $(if ($failCount -eq 0) { '✅ 全部通过' } else { '❌ 存在失败' })

---

"@

    # 每个阶段的详细报告
    foreach ($r in $Script:StageResults) {
        $report += "## $($r.Name)`n`n"
        $report += "**状态**: $(if ($r.Status -eq 'PASS') { 'PASS' } else { 'FAIL' })`n`n"

        if ($r.Duration.TotalSeconds -gt 0) {
            $report += "**耗时**: $("{0:N1}s" -f $r.Duration.TotalSeconds)`n`n"
        }

        if ($r.Details) {
            $report += "### 详情`n`n"
            $report += "``````n"
            $report += "$($r.Details)`n"
            $report += "``````n"
        }

        if ($r.Status -eq 'FAIL') {
            $report += "### 修复建议`n`n"
            $report += Get-Recommendations -StageName $r.Name
            $report += "`n"
        }

        $report += "---`n`n"
    }

    # 修复建议汇总
    if ($failCount -gt 0) {
        $report += "## 修复建议汇总`n`n"
        foreach ($r in ($Script:StageResults | Where-Object { $_.Status -eq 'FAIL' })) {
            $report += Get-Recommendations -StageName $r.Name
            $report += "`n"
        }
    }

    # 输出到文件
    $report | Out-File -FilePath $OutputPath -Encoding UTF8
    Write-Host "报告已保存到: $OutputPath" -ForegroundColor Cyan
}

function Get-Recommendations {
    param([string]$StageName)

    $recommendations = @{
        "Frontend: TypeScript Type Check" = @"
1. 运行 \`cd frontend && npx tsc --noEmit\` 查看完整错误列表
2. 根据错误提示修复类型问题
3. 注意检查 \`frontend/src/\` 目录下标注 \`file:line\` 的具体位置
4. 确认 \`tsconfig.json\` 和 \`tsconfig.app.json\` 配置正确
"@
        "Frontend: Unit Tests (Vitest)" = @"
1. 运行 \`cd frontend && npx vitest run\` 查看测试失败详情
2. 检查 \`frontend/src/test/\` 和 \`frontend/src/**/*.test.tsx\` 文件
3. 确认组件 Props 类型与测试用例匹配
4. 如需要更新快照，运行 \`npx vitest run -u\`
"@
        "Backend: Rust Unit Tests" = @"
1. 运行 \`cd src-tauri && cargo test --lib\` 查看完整测试输出
2. 关注 \`error[\` 和 \`thread .* panicked\` 错误行
3. 检查 \`src-tauri/src/\` 目录下标注的 \`file:line\` 位置
4. 确保数据库 schema 和模型定义一致
"@
        "Tauri Build" = @"
1. 确保前端构建成功（先运行 \`cd frontend && npm run build\`）
2. 检查 Rust 编译是否通过（\`cd src-tauri && cargo build --release\`）
3. 确认 \`src-tauri/tauri.conf.json\` 配置正确
4. 检查磁盘空间是否充足（完整构建可能需要数 GB）
5. 确认 WebView2 运行时已安装
"@
        "Package NSIS Installer" = @"
1. 确认 release 二进制存在: \`src-tauri\target\release\arcane-codex.exe\`
2. 运行 \`cd src-tauri && npx tauri build --bundles nsis\` 手动重试
3. 检查 NSIS 是否已正确安装
"@
        "UI Automation Tests" = @"
1. 确认应用已正确安装（检查 \`%ProgramFiles%\ArcaneCodex\`）
2. 手动启动应用确认能正常运行
3. 检查 UI 自动化依赖（Windows UI Automation 框架）
4. 查看 \`scripts/screenshots/\` 目录下的截图分析 UI 状态
5. 确认没有其他 Arcane Codex 实例在运行
"@
    }

    return $recommendations[$StageName] ?? "检查上述错误信息，修复后重新运行流水线。"
}

# ============================================================
# 流水线主体
# ============================================================
Write-Section "Arcane Codex 构建流水线"
Write-Host "项目根目录: $ProjectRoot" -ForegroundColor Yellow
Write-Host "执行阶段:   $Stage" -ForegroundColor Yellow
Write-Host "开始时间:   $($Script:StartTime.ToString('yyyy-MM-dd HH:mm:ss'))" -ForegroundColor Yellow

Add-Report "# Arcane Codex - 构建流水线报告"
Add-Report ""
Add-Report "> 开始时间: $($Script:StartTime.ToString('yyyy-MM-dd HH:mm:ss'))"
Add-Report "> 项目路径: $ProjectRoot"
Add-Report "> 执行阶段: $Stage"
Add-Report ""

# ============================================================
# 阶段 1: 快速预检查
# ============================================================
if ($Stage -in @("all", "check")) {
    Write-Section "阶段 1: 快速预检查 (Fast Feedback Loop)"

    $stageStart = Get-Date
    $checkErrors = @()

    # 1.1 前端 TypeScript 类型检查
    Write-Step "1.1 前端 TypeScript 类型检查"
    $tscStageStart = Get-Date

    try {
        Push-Location $FrontendDir
        $tscOutput = & npx tsc --noEmit 2>&1
        $tscExitCode = $LASTEXITCODE
        Pop-Location

        if ($tscExitCode -ne 0) {
            $errorDetails = ($tscOutput | Where-Object { $_ -match 'error TS' }) -join "`n"
            Write-Fail "TypeScript 类型检查失败"
            $checkErrors += "TypeScript: $errorDetails"

            # 尝试提取 file:line 位置
            $locations = [regex]::Matches($errorDetails, '(src/[^\s:]+:\d+:\d+)')
            if ($locations.Count -gt 0) {
                Write-Host "  错误位置:" -ForegroundColor DarkYellow
                $locations | ForEach-Object { Write-Host "    - $_" -ForegroundColor DarkYellow }
            }

            Record-Stage "Frontend: TypeScript Type Check" "FAIL" $errorDetails ((Get-Date) - $tscStageStart)
        } else {
            Write-Pass "TypeScript 类型检查通过"
            Record-Stage "Frontend: TypeScript Type Check" "PASS" "" ((Get-Date) - $tscStageStart)
        }
    } catch {
        Write-Fail "TypeScript 检查异常: $_"
        $checkErrors += "TypeScript Exception: $_"
        Record-Stage "Frontend: TypeScript Type Check" "FAIL" $_.Exception.Message ((Get-Date) - $tscStageStart)
        if ($PWD.Path -eq $FrontendDir) { Pop-Location }
    }

    # 1.2 前端单元测试
    Write-Step "1.2 前端单元测试 (Vitest)"
    $vitestStageStart = Get-Date

    try {
        Push-Location $FrontendDir
        $vitestOutput = & npx vitest run 2>&1
        $vitestExitCode = $LASTEXITCODE

        # 显示关键输出
        $vitestOutput | Where-Object {
            $_ -match 'Tests\s+\d+' -or
            $_ -match 'FAIL' -or
            $_ -match '✓' -or
            $_ -match '❌' -or
            $_ -match 'Error:'
        } | ForEach-Object {
            if ($_ -match 'FAIL|Error:') {
                Write-Host $_ -ForegroundColor Red
            } else {
                Write-Host $_
            }
        }

        Pop-Location

        if ($vitestExitCode -ne 0) {
            Write-Fail "Vitest 单元测试失败"
            $errorDetails = ($vitestOutput | Where-Object { $_ -match 'FAIL|Error:|at ' }) -join "`n"
            $checkErrors += "Vitest: $errorDetails"
            Record-Stage "Frontend: Unit Tests (Vitest)" "FAIL" $errorDetails ((Get-Date) - $vitestStageStart)
        } else {
            Write-Pass "Vitest 单元测试通过"
            Record-Stage "Frontend: Unit Tests (Vitest)" "PASS" "" ((Get-Date) - $vitestStageStart)
        }
    } catch {
        Write-Fail "Vitest 测试异常: $_"
        $checkErrors += "Vitest Exception: $_"
        Record-Stage "Frontend: Unit Tests (Vitest)" "FAIL" $_.Exception.Message ((Get-Date) - $vitestStageStart)
        if ($PWD.Path -eq $FrontendDir) { Pop-Location }
    }

    # 1.3 后端 Rust 单元测试
    Write-Step "1.3 后端 Rust 单元测试"
    $rustStageStart = Get-Date

    try {
        $null = Get-Command cargo -ErrorAction SilentlyContinue
        if (-not $?) {
            Write-Fail "cargo 未找到，跳过 Rust 测试"
            $checkErrors += "Rust: cargo not found in PATH"
            Record-Stage "Backend: Rust Unit Tests" "FAIL" "cargo not found" ((Get-Date) - $rustStageStart)
        } else {
            Push-Location $BackendDir
            $cargoOutput = & cargo test --lib --no-fail-fast 2>&1
            $cargoExitCode = $LASTEXITCODE

            # 显示关键输出
            $cargoOutput | Where-Object {
                $_ -match 'test .* \.\.\.' -or
                $_ -match 'running \d+ test' -or
                $_ -match 'test result:' -or
                $_ -match 'FAILED' -or
                $_ -match 'error\['
            } | ForEach-Object {
                if ($_ -match 'FAILED|error\[') {
                    Write-Host $_ -ForegroundColor Red
                } elseif ($_ -match 'test result:.*passed') {
                    Write-Host $_ -ForegroundColor Green
                } else {
                    Write-Host $_
                }
            }

            Pop-Location

            if ($cargoExitCode -ne 0) {
                $errorDetails = ($cargoOutput | Where-Object { $_ -match 'FAILED|thread .* panicked|error\[' }) -join "`n"
                Write-Fail "Rust 单元测试失败"

                # 尝试提取 file:line 位置
                $locations = [regex]::Matches($errorDetails, '--> (src/[^\s:]+:\d+:\d+)')
                if ($locations.Count -gt 0) {
                    Write-Host "  错误位置:" -ForegroundColor DarkYellow
                    $locations | ForEach-Object { Write-Host "    - $($_.Groups[1].Value)" -ForegroundColor DarkYellow }
                }

                $checkErrors += "Rust: $errorDetails"
                Record-Stage "Backend: Rust Unit Tests" "FAIL" $errorDetails ((Get-Date) - $rustStageStart)
            } else {
                Write-Pass "Rust 单元测试通过"
                Record-Stage "Backend: Rust Unit Tests" "PASS" "" ((Get-Date) - $rustStageStart)
            }
        }
    } catch {
        Write-Fail "Rust 测试异常: $_"
        $checkErrors += "Rust Exception: $_"
        Record-Stage "Backend: Rust Unit Tests" "FAIL" $_.Exception.Message ((Get-Date) - $rustStageStart)
        if ($PWD.Path -eq $BackendDir) { Pop-Location }
    }

    # 检查阶段 1 结果
    $checkStageDuration = (Get-Date) - $stageStart
    $checkPassCount = ($Script:StageResults | Where-Object { $_.Duration -eq $checkStageDuration -or $_.Name -match 'TypeScript|Vitest|Rust' } | Where-Object { $_.Status -eq 'PASS' }).Count

    if ($checkErrors.Count -gt 0) {
        Write-Host "`n  预检查存在 $($checkErrors.Count) 个失败项" -ForegroundColor Red
        if ($Stage -eq "check") {
            Write-Host "  流水线终止于预检查阶段" -ForegroundColor Red
        } else {
            Write-Host "  继续执行后续阶段（不阻塞）" -ForegroundColor Yellow
        }
    } else {
        Write-Host "`n  全部预检查通过，可以继续进行构建" -ForegroundColor Green
    }
}

# ============================================================
# 阶段 2: Tauri 构建
# ============================================================
if ($Stage -in @("all", "build")) {
    Write-Section "阶段 2: Tauri 构建"

    # 检查 tauri cli 是否可用
    $null = Get-Command npx -ErrorAction SilentlyContinue
    if (-not $?) {
        Write-Fail "npx 未找到，无法执行 Tauri 构建"
        Record-Stage "Tauri Build" "FAIL" "npx not found"
    } else {
        if ($ReuseRelease) {
            # 仅打包模式：跳过编译，直接使用已有 release 二进制
            Write-Step "模式: 仅打包（跳过编译，使用已有 release 二进制）"

            $releaseBinary = Join-Path $ReleaseDir 'arcane-codex.exe'
            if (-not (Test-Path $releaseBinary)) {
                Write-Fail "release 二进制不存在: $releaseBinary"
                Write-Host "  请先运行完整构建或手动编译: cd src-tauri && cargo build --release" -ForegroundColor Yellow
                Record-Stage "Package NSIS Installer" "FAIL" "Release binary not found"
            } else {
                $buildStart = Get-Date
                Write-Host "  发现 release 二进制: $releaseBinary" -ForegroundColor Green
                Write-Host "  文件大小: $([Math]::Round((Get-Item $releaseBinary).Length / 1MB, 2)) MB" -ForegroundColor Green

                Write-Step "运行 npx tauri build --bundles nsis ..."
                Push-Location $BackendDir

                try {
                    # 设置环境变量跳过前端构建（因为已有 release 二进制）
                    $env:TAURI_SKIP_FRONTEND_BUILD = "true"
                    $buildOutput = & npx tauri build --bundles nsis 2>&1
                    $buildExitCode = $LASTEXITCODE
                    Remove-Item Env:\TAURI_SKIP_FRONTEND_BUILD -ErrorAction SilentlyContinue

                    $buildOutput | Where-Object {
                        $_ -match 'Finished|error|warn|Building| Bundling'
                    } | ForEach-Object {
                        if ($_ -match 'error') {
                            Write-Host $_ -ForegroundColor Red
                        } elseif ($_ -match 'warn') {
                            Write-Host $_ -ForegroundColor Yellow
                        } else {
                            Write-Host $_
                        }
                    }

                    Pop-Location

                    if ($buildExitCode -ne 0) {
                        $errorDetails = ($buildOutput | Where-Object { $_ -match 'error' }) -join "`n"
                        Write-Fail "NSIS 打包失败"
                        Record-Stage "Package NSIS Installer" "FAIL" $errorDetails ((Get-Date) - $buildStart)
                    } else {
                        Write-Pass "NSIS 打包成功"
                        Record-Stage "Package NSIS Installer" "PASS" "" ((Get-Date) - $buildStart)

                        # 查找生成的安装包
                        if (Test-Path $BundleDir) {
                            $installers = Get-ChildItem $BundleDir -Filter "*.exe"
                            if ($installers) {
                                Write-Host "`n  生成的安装包:" -ForegroundColor Green
                                $installers | ForEach-Object {
                                    $sizeMB = [Math]::Round($_.Length / 1MB, 2)
                                    Write-Host "    - $_.Name ($sizeMB MB)" -ForegroundColor Green
                                }
                            }
                        }
                    }
                } catch {
                    Write-Fail "打包异常: $_"
                    Record-Stage "Package NSIS Installer" "FAIL" $_.Exception.Message ((Get-Date) - $buildStart)
                    Pop-Location -ErrorAction SilentlyContinue
                }
            }
        } else {
            # 完整构建模式
            Write-Step "模式: 完整构建（编译前端 + 编译 Rust + 打包）"
            $buildStart = Get-Date

            Push-Location $BackendDir

            try {
                $buildOutput = & npx tauri build --bundles nsis 2>&1
                $buildExitCode = $LASTEXITCODE

                # 显示关键输出
                $buildOutput | Where-Object {
                    $_ -match 'Finished|error|warn|Building| Bundling|Compiling'
                } | ForEach-Object {
                    if ($_ -match 'error') {
                        Write-Host $_ -ForegroundColor Red
                    } elseif ($_ -match 'warn') {
                        Write-Host $_ -ForegroundColor Yellow
                    } else {
                        Write-Host $_
                    }
                }

                Pop-Location

                if ($buildExitCode -ne 0) {
                    $errorDetails = ($buildOutput | Where-Object { $_ -match 'error' }) -join "`n"
                    Write-Fail "Tauri 构建失败"
                    Record-Stage "Tauri Build" "FAIL" $errorDetails ((Get-Date) - $buildStart)
                } else {
                    Write-Pass "Tauri 构建成功"
                    Record-Stage "Tauri Build" "PASS" "" ((Get-Date) - $buildStart)

                    # 查找生成的安装包
                    if (Test-Path $BundleDir) {
                        $installers = Get-ChildItem $BundleDir -Filter "*.exe"
                        if ($installers) {
                            Write-Host "`n  生成的安装包:" -ForegroundColor Green
                            $installers | ForEach-Object {
                                $sizeMB = [Math]::Round($_.Length / 1MB, 2)
                                Write-Host "    - $_.Name ($sizeMB MB)" -ForegroundColor Green
                            }
                        }
                    }
                }
            } catch {
                Write-Fail "构建异常: $_"
                Record-Stage "Tauri Build" "FAIL" $_.Exception.Message ((Get-Date) - $buildStart)
                Pop-Location -ErrorAction SilentlyContinue
            }
        }
    }
}

# ============================================================
# 阶段 3: UI 自动化测试
# ============================================================
if ($Stage -in @("all", "ui-test") -and -not $SkipUI) {
    Write-Section "阶段 3: UI 自动化测试"

    $uiTestScript = Join-Path $ScriptsDir 'ui-test.ps1'

    if (-not (Test-Path $uiTestScript)) {
        Write-Fail "UI 测试脚本不存在: $uiTestScript"
        Record-Stage "UI Automation Tests" "FAIL" "Script not found: $uiTestScript"
    } else {
        $uiStart = Get-Date

        try {
            Write-Step "执行 UI 自动化测试..."
            & $uiTestScript
            $uiExitCode = $LASTEXITCODE

            if ($uiExitCode -ne 0) {
                Write-Fail "UI 自动化测试存在失败项"
                Record-Stage "UI Automation Tests" "FAIL" "See UI test output above" ((Get-Date) - $uiStart)
            } else {
                Write-Pass "UI 自动化测试全部通过"
                Record-Stage "UI Automation Tests" "PASS" "" ((Get-Date) - $uiStart)
            }
        } catch {
            Write-Fail "UI 测试异常: $_"
            Record-Stage "UI Automation Tests" "FAIL" $_.Exception.Message ((Get-Date) - $uiStart)
        }
    }
} elseif ($SkipUI) {
    Write-Step "跳过 UI 自动化测试 (-SkipUI)"
}

# ============================================================
# 最终报告
# ============================================================
$Script:EndTime = Get-Date
$totalDuration = "{0:N1}" -f ($Script:EndTime - $Script:StartTime).TotalSeconds

Write-Section "流水线报告 (Pipeline Report)"

Write-Host "总耗时: ${totalDuration} 秒" -ForegroundColor Yellow
Write-Host ""

$allPass = ($Script:StageResults | Where-Object { $_.Status -eq 'FAIL' }).Count -eq 0

# 表格输出
Write-Host "阶段结果:" -ForegroundColor Cyan
Write-Host "  $('阶段名称'.PadRight(40)) $('状态'.PadRight(10)) 耗时"
Write-Host "  $('-' * 65)"

foreach ($r in $Script:StageResults) {
    $duration = if ($r.Duration.TotalSeconds -gt 0) { "{0:N1}s" -f $r.Duration.TotalSeconds } else { "-" }
    $statusColor = if ($r.Status -eq 'PASS') { 'Green' } else { 'Red' }
    $statusSymbol = if ($r.Status -eq 'PASS') { '[PASS]' } else { '[FAIL]' }
    $name = ($r.Name.PadRight(40)).Substring(0, [Math]::Min(40, $r.Name.Length))

    Write-Host "  $name " -NoNewline
    Write-Host $statusSymbol -ForegroundColor $statusColor -NoNewline
    Write-Host (" " * (10 - $statusSymbol.Length)) -NoNewline
    Write-Host "  $duration"

    if ($r.Details -and $r.Status -eq 'FAIL') {
        # 显示 file:line 信息
        $fileMatches = [regex]::Matches($r.Details, '--> (src/[^\s:]+:\d+:\d+)')
        if ($fileMatches.Count -gt 0) {
            Write-Host "    错误位置:" -ForegroundColor DarkYellow
            $fileMatches | ForEach-Object {
                Write-Host "      $($_.Groups[1].Value)" -ForegroundColor DarkYellow
            }
        }
        $tsMatches = [regex]::Matches($r.Details, '(src/[^\s:]+:\d+:\d+)')
        if ($tsMatches.Count -gt 0) {
            Write-Host "    错误位置:" -ForegroundColor DarkYellow
            $tsMatches | ForEach-Object {
                Write-Host "      $($_.Groups[1].Value)" -ForegroundColor DarkYellow
            }
        }
    }
}

Write-Host ""

$passCount = ($Script:StageResults | Where-Object { $_.Status -eq 'PASS' }).Count
$failCount = ($Script:StageResults | Where-Object { $_.Status -eq 'FAIL' }).Count
$passRate = if ($Script:StageResults.Count -gt 0) { "{0:P0}" -f ($passCount / $Script:StageResults.Count) } else { "N/A" }

Write-Host "通过: $passCount | 失败: $failCount | 通过率: $passRate" -ForegroundColor $(if ($failCount -eq 0) { 'Green' } else { 'Red' })
Write-Host ""

if ($allPass) {
    Write-Host "  ALL STAGES PASSED" -ForegroundColor Green
    $Script:OverallExitCode = 0
} else {
    Write-Host "  SOME STAGES FAILED" -ForegroundColor Red

    Write-Host "`n修复建议:" -ForegroundColor Cyan
    foreach ($r in ($Script:StageResults | Where-Object { $_.Status -eq 'FAIL' })) {
        Write-Host "  [$($r.Name)]" -ForegroundColor Yellow
        Get-Recommendations -StageName $r.Name | ForEach-Object {
            if ($_ -match '^\d+\.') {
                Write-Host "    $_"
            }
        }
    }
    $Script:OverallExitCode = 1
}

Write-Host ""

# 生成 Markdown 报告
if ($OutputReport) {
    try {
        Generate-MarkdownReport -OutputPath $OutputReport
    } catch {
        Write-Host "报告生成失败: $_" -ForegroundColor Yellow
    }
}

# 默认在 all 模式下生成报告
if ($Stage -eq "all" -and -not $OutputReport) {
    $defaultReportPath = Join-Path $ProjectRoot "pipeline-report-$(Get-Date -Format 'yyyyMMdd-HHmmss').md"
    try {
        Generate-MarkdownReport -OutputPath $defaultReportPath
    } catch {
        Write-Host "默认报告生成失败: $_" -ForegroundColor Yellow
    }
}

exit $Script:OverallExitCode
