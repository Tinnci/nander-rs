<#
.SYNOPSIS
    Automated release script for nander-rs.
    Updates Cargo.toml version, creates a git commit, and tags the release.

.DESCRIPTION
    This script performs the following actions:
    1. Validate the version format (x.y.z).
    2. Update the `version` field in Cargo.toml.
    3. Run `cargo check` to update Cargo.lock.
    4. Git commit the changes.
    5. Git tag the new version.

.PARAMETER Version
    The new version number (e.g., 0.5.2).

.EXAMPLE
    .\release.ps1 0.5.2
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

# 1. Validate version format
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Invalid version format. Please use semantic versioning (e.g., 0.5.2)."
    exit 1
}

Write-Host "🚀 Preparing release for version: $Version" -ForegroundColor Cyan

# 2. Update Cargo.toml
$cargoTomlPath = Join-Path $PSScriptRoot "Cargo.toml"
$content = Get-Content $cargoTomlPath -Raw
$newContent = $content -replace '^version = ".*"', "version = ""$Version"""

if ($content -eq $newContent) {
    Write-Warning "Cargo.toml already has version $Version. Skipping file update."
} else {
    Set-Content -Path $cargoTomlPath -Value $newContent -Encoding UTF8
    Write-Host "✅ Updated Cargo.toml" -ForegroundColor Green
}

# 3. Update Cargo.lock
Write-Host "🛠️  Updating Cargo.lock..." -ForegroundColor Yellow
cargo check --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo check failed. Aborting release."
    exit 1
}

# 4. Git Commit
$commitMsg = "chore: bump version to $Version"
Write-Host "📦 Committing changes..." -ForegroundColor Yellow
git add Cargo.toml Cargo.lock
git commit -m "$commitMsg"

# 5. Git Tag
$tagName = "v$Version"
Write-Host "🏷️  creating tag: $tagName" -ForegroundColor Yellow
git tag -a $tagName -m "Release $tagName"

Write-Host "✨ Release $Version ready!" -ForegroundColor Green
Write-Host "👉 Run 'git push origin main --tags' to publish." -ForegroundColor Cyan
