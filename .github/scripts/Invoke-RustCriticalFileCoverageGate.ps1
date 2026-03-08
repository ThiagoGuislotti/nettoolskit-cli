param(
    [Parameter(Mandatory = $true)]
    [string]$ReportPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Test-Path -LiteralPath $ReportPath)) {
    throw "Coverage report not found: $ReportPath"
}

$report = Get-Content -LiteralPath $ReportPath -Raw | ConvertFrom-Json
if ($null -eq $report.data -or $report.data.Count -eq 0) {
    throw "Coverage report does not contain file data: $ReportPath"
}

$files = $report.data[0].files
if ($null -eq $files -or $files.Count -eq 0) {
    throw "Coverage report does not contain file entries: $ReportPath"
}

$budgets = @(
    @{
        Path = 'crates/cli/src/main.rs'
        MinLines = 5.0
        MinFunctions = 5.0
    },
    @{
        Path = 'crates/commands/manifest/src/ui/menu.rs'
        MinLines = 5.0
        MinFunctions = 5.0
    },
    @{
        Path = 'crates/otel/src/tracing_setup.rs'
        MinLines = 40.0
        MinFunctions = 50.0
    },
    @{
        Path = 'crates/orchestrator/src/execution/processor.rs'
        MinLines = 50.0
        MinFunctions = 60.0
    }
)

function Normalize-CoveragePath {
    param([string]$Path)

    return ($Path -replace '\\', '/').ToLowerInvariant()
}

function Resolve-CoverageEntry {
    param(
        [object[]]$CoverageFiles,
        [string]$TargetPath
    )

    $normalizedTarget = Normalize-CoveragePath -Path $TargetPath
    return $CoverageFiles | Where-Object {
        (Normalize-CoveragePath -Path $_.filename).EndsWith($normalizedTarget)
    } | Select-Object -First 1
}

$failures = New-Object System.Collections.Generic.List[string]
$summaryLines = New-Object System.Collections.Generic.List[string]
$summaryLines.Add('### Critical File Coverage Gate')
$summaryLines.Add('')
$summaryLines.Add('| File | Lines | Min | Functions | Min | Result |')
$summaryLines.Add('|---|---:|---:|---:|---:|---|')

foreach ($budget in $budgets) {
    $entry = Resolve-CoverageEntry -CoverageFiles $files -TargetPath $budget.Path
    if ($null -eq $entry) {
        $failures.Add("Missing coverage entry for $($budget.Path)")
        $summaryLines.Add("| `$($budget.Path)` | n/a | $($budget.MinLines)% | n/a | $($budget.MinFunctions)% | missing |")
        continue
    }

    $linePercent = [double]$entry.summary.lines.percent
    $functionPercent = [double]$entry.summary.functions.percent
    $result = 'pass'

    if ($linePercent -lt [double]$budget.MinLines) {
        $failures.Add(("{0}: line coverage {1:N2}% is below minimum {2:N2}%" -f $budget.Path, $linePercent, $budget.MinLines))
        $result = 'fail'
    }

    if ($functionPercent -lt [double]$budget.MinFunctions) {
        $failures.Add(("{0}: function coverage {1:N2}% is below minimum {2:N2}%" -f $budget.Path, $functionPercent, $budget.MinFunctions))
        $result = 'fail'
    }

    $summaryLines.Add(
        ("| `{0}` | {1:N2}% | {2:N2}% | {3:N2}% | {4:N2}% | {5} |" -f `
            $budget.Path, $linePercent, $budget.MinLines, $functionPercent, $budget.MinFunctions, $result)
    )
}

if ($env:GITHUB_STEP_SUMMARY) {
    Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY -Value ($summaryLines -join [Environment]::NewLine)
}

if ($failures.Count -gt 0) {
    $message = "Critical file coverage gate failed:{0}{1}" -f `
        [Environment]::NewLine, `
        ($failures -join [Environment]::NewLine)
    throw $message
}

Write-Host 'Critical file coverage gate passed.'