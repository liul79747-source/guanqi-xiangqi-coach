param([switch]$Dev)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
if (Test-Path '.tools/toolchain/bin/cargo.exe') {
  $env:CARGO_HOME = Join-Path (Get-Location) '.tools/cargo'
  $env:PATH = "$(Join-Path (Get-Location) '.tools/toolchain/bin');$env:PATH"
} elseif (Test-Path '.tools/cargo/bin/cargo.exe') {
  $env:CARGO_HOME = Join-Path (Get-Location) '.tools/cargo'
  $env:RUSTUP_HOME = Join-Path (Get-Location) '.tools/rustup'
  $env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw '请安装 Rust MSVC 工具链和 Microsoft C++ Build Tools。' }
if ($Dev) { npm run desktop:dev } else { npm run desktop:build }
exit $LASTEXITCODE
