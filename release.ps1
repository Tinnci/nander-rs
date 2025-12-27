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

# Check if cargo set-version is available
$cargoEditAvailable = $false
try {
    cargo set-version --version | Out-Null
    if ($LASTEXITCODE -eq 0) { $cargoEditAvailable = $true }
} catch {}

if (-not $cargoEditAvailable) {
    Write-Error "❌ 'cargo-edit' is required but not found. Please install it by running: cargo install cargo-edit"
    exit 1
}

Write-Host "🔧 Using cargo set-version to update manifest..." -ForegroundColor Cyan
cargo set-version $Version

# 2.1 Verify Update
$finalContent = Get-Content $cargoTomlPath -Raw
if ($finalContent -notmatch "version = ""$Version""") {
    Write-Error "Verification failed! Cargo.toml does not allow to contain version = ""$Version"""
    exit 1
}
Write-Host "✅ Verified Cargo.toml contains version $Version" -ForegroundColor Green

# 3. Update Cargo.lock
Write-Host "🛠️  Updating Cargo.lock..." -ForegroundColor Yellow
cargo check --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo check failed. Aborting release."
    exit 1
}

# 3.1 Run Tests
Write-Host "🧪 Running tests..." -ForegroundColor Yellow
cargo test --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Error "Tests failed. Aborting release."
    exit 1
}

# 4. Git Commit
$commitMsg = "chore: bump version to $Version"
Write-Host "📦 Committing changes..." -ForegroundColor Yellow
git add Cargo.toml Cargo.lock
git commit -m "$commitMsg"
if ($LASTEXITCODE -ne 0) {
    Write-Error "Git commit failed (perhaps no changes?). Aborting."
    exit 1
}

# 5. Git Tag
$tagName = "v$Version"
Write-Host "🏷️  creating tag: $tagName" -ForegroundColor Yellow
git tag -a $tagName -m "Release $tagName"

Write-Host "✨ Release $Version ready!" -ForegroundColor Green

# 6. Git Push
$confirmation = Read-Host "👉 Push changes and tags to origin? (y/N)"
if ($confirmation -match '^[Yy]$') {
    Write-Host "� Pushing changes..." -ForegroundColor Cyan
    git push
    git push origin $tagName
} else {
    Write-Host "Skipped pushing. You can push manually using: git push && git push origin $tagName" -ForegroundColor Gray
}
