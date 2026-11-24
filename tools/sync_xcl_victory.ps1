[CmdletBinding()]
param(
    [switch]$Once
)

$ErrorActionPreference = 'Stop'

$mirrorRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\mirrors'))
$logRoot = Join-Path $mirrorRoot 'logs'
$statusPath = Join-Path $mirrorRoot 'sync-status.json'
$taskName = 'GTA6-VR-XCL-Victory-Sync'

$repositories = @(
    [pscustomobject]@{
        Name = 'xcl'
        Url = 'https://github.com/ivall/xcl.git'
        Branch = 'main'
        Path = Join-Path $mirrorRoot 'xcl'
    },
    [pscustomobject]@{
        Name = 'victory'
        Url = 'https://github.com/Marvisak/VIctory.git'
        Branch = 'main'
        Path = Join-Path $mirrorRoot 'victory'
    }
)

New-Item -ItemType Directory -Path $mirrorRoot -Force | Out-Null
New-Item -ItemType Directory -Path $logRoot -Force | Out-Null

function Write-SyncLog {
    param([string]$Message)

    $line = "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss zzz')] $Message"
    Add-Content -LiteralPath (Join-Path $logRoot 'sync.log') -Value $line -Encoding utf8
}

function Assert-SafeMirrorPath {
    param([string]$Path)

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $rootWithSeparator = $mirrorRoot.TrimEnd('\') + [System.IO.Path]::DirectorySeparatorChar
    if (-not $fullPath.StartsWith($rootWithSeparator, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to modify a path outside the mirror root: $fullPath"
    }
}

function Invoke-GitCommand {
    param([string[]]$Arguments)

    Write-SyncLog ("git " + ($Arguments -join ' '))
    $output = & git @Arguments 2>&1
    foreach ($line in $output) {
        Write-SyncLog ([string]$line)
    }
    if ($LASTEXITCODE -ne 0) {
        throw "git exited with code $LASTEXITCODE"
    }
}

function Sync-Repository {
    param($Repository)

    Assert-SafeMirrorPath -Path $Repository.Path
    $gitPath = Join-Path $Repository.Path '.git'

    if (-not (Test-Path -LiteralPath $gitPath)) {
        if (Test-Path -LiteralPath $Repository.Path) {
            Write-SyncLog "Removing non-repository mirror directory $($Repository.Path)"
            Remove-Item -LiteralPath $Repository.Path -Recurse -Force
        }

        $parentPath = Split-Path -Parent $Repository.Path
        New-Item -ItemType Directory -Path $parentPath -Force | Out-Null
        Invoke-GitCommand -Arguments @(
            'clone',
            '--origin', 'origin',
            '--branch', $Repository.Branch,
            '--recurse-submodules',
            $Repository.Url,
            $Repository.Path
        )
    }

    Invoke-GitCommand -Arguments @('-C', $Repository.Path, 'fetch', '--prune', 'origin')
    Invoke-GitCommand -Arguments @('-C', $Repository.Path, 'reset', '--hard', "origin/$($Repository.Branch)")
    Invoke-GitCommand -Arguments @('-C', $Repository.Path, 'clean', '-fdx')

    if (Test-Path -LiteralPath (Join-Path $Repository.Path '.gitmodules')) {
        Invoke-GitCommand -Arguments @('-C', $Repository.Path, 'submodule', 'sync', '--recursive')
        Invoke-GitCommand -Arguments @('-C', $Repository.Path, 'submodule', 'update', '--init', '--recursive')
    }

    $head = (& git -C $Repository.Path rev-parse HEAD).Trim()
    $remote = (& git -C $Repository.Path config --get remote.origin.url).Trim()
    Write-SyncLog "$($Repository.Name) synchronized at $head"

    return [pscustomobject]@{
        Name = $Repository.Name
        Path = $Repository.Path
        Remote = $remote
        Branch = $Repository.Branch
        Commit = $head
        Status = 'ok'
        UpdatedAt = (Get-Date).ToString('o')
    }
}

function Sync-AllRepositories {
    $results = [System.Collections.Generic.List[object]]::new()
    foreach ($repository in $repositories) {
        try {
            $results.Add((Sync-Repository -Repository $repository))
        }
        catch {
            Write-SyncLog "$($repository.Name) failed: $($_.Exception.Message)"
            $results.Add([pscustomobject]@{
                Name = $repository.Name
                Path = $repository.Path
                Remote = $repository.Url
                Branch = $repository.Branch
                Commit = $null
                Status = 'error'
                Error = $_.Exception.Message
                UpdatedAt = (Get-Date).ToString('o')
            })
        }
    }

    $results | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $statusPath -Encoding utf8
    if (@($results | Where-Object Status -eq 'error').Count -gt 0) {
        return $false
    }
    return $true
}

Write-SyncLog 'sync cycle started'
$cycleOk = Sync-AllRepositories
$cycleMessage = if ($cycleOk) { 'sync cycle completed' } else { 'sync cycle completed with errors' }
Write-SyncLog $cycleMessage

if ($Once) {
    if (-not $cycleOk) {
        exit 1
    }
    exit 0
}

while ($true) {
    Start-Sleep -Seconds 600
    Write-SyncLog 'scheduled sync cycle started'
    $cycleOk = Sync-AllRepositories
    $cycleMessage = if ($cycleOk) { 'scheduled sync cycle completed' } else { 'scheduled sync cycle completed with errors' }
    Write-SyncLog $cycleMessage
}
