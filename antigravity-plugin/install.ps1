<#
.SYNOPSIS
  One-command setup script for the Lutetian Google Antigravity Plugin on Windows.
.DESCRIPTION
  1. Builds (or locates) lutetian-mcp.exe
  2. Installs the 5 Lutetian SKILL.md modules into ~/.gemini/antigravity/skills/
  3. Copies lutetian-mcp.exe into %LOCALAPPDATA%\Programs\Lutetian\ (and adds to PATH if missing)
  4. Writes a ready-to-paste Antigravity MCP config snippet
#>
$ErrorActionPreference = "Stop"

$PluginRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $PluginRoot
$InstallBinDir = Join-Path $env:LOCALAPPDATA "Programs\Lutetian"
$SkillsDest = Join-Path $env:USERPROFILE ".gemini\antigravity\skills"

Write-Host "[Lutetian Plugin Setup] Installing skills and MCP server..." -ForegroundColor Cyan

# 1. Locate or build lutetian-mcp.exe
$Candidates = @(
    (Join-Path $PluginRoot "lutetian-mcp.exe"),
    (Join-Path $InstallBinDir "lutetian-mcp.exe"),
    (Join-Path $RepoRoot "target\release\lutetian-mcp.exe"),
    (Join-Path $RepoRoot "target\debug\lutetian-mcp.exe")
)

$McpExe = $Candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $McpExe -and (Test-Path (Join-Path $RepoRoot "Cargo.toml"))) {
    Write-Host "Building lutetian-mcp in release mode..." -ForegroundColor Yellow
    Push-Location $RepoRoot
    cargo build --release -p lutetian-mcp
    Pop-Location
    $McpExe = Join-Path $RepoRoot "target\release\lutetian-mcp.exe"
}

if (-not $McpExe -or -not (Test-Path $McpExe)) {
    throw "Could not find or build lutetian-mcp.exe."
}

New-Item -ItemType Directory -Force -Path $InstallBinDir | Out-Null
$InstalledMcp = Join-Path $InstallBinDir "lutetian-mcp.exe"
if ($McpExe -ne $InstalledMcp) {
    Copy-Item $McpExe $InstalledMcp -Force
}

# 2. Install SKILL.md directories into Antigravity user skills directory
New-Item -ItemType Directory -Force -Path $SkillsDest | Out-Null
$SkillDirs = Get-ChildItem -Path (Join-Path $PluginRoot "skills") -Directory
foreach ($dir in $SkillDirs) {
    $dest = Join-Path $SkillsDest $dir.Name
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    Copy-Item (Join-Path $dir.FullName "*") $dest -Recurse -Force
    Write-Host "  + Installed skill: $($dir.Name)" -ForegroundColor Green
}

# 3. Emit resolved mcp_config.resolved.json
$ResolvedJson = @{
    mcpServers = @{
        lutetian = @{
            command = $InstalledMcp
            args    = @()
        }
    }
} | ConvertTo-Json -Depth 4

$OutConfig = Join-Path $PluginRoot "mcp_config.resolved.json"
$ResolvedJson | Set-Content -Path $OutConfig -Encoding UTF8

Write-Host "`nSetup complete!" -ForegroundColor Green
Write-Host "MCP binary : $InstalledMcp"
Write-Host "Skills dir : $SkillsDest"
Write-Host "Config file: $OutConfig`n"
Write-Host "Paste this into Antigravity's mcp_config.json:" -ForegroundColor Cyan
Write-Host $ResolvedJson
