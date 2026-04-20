$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if (-not $env:VCPKG_ROOT) {
    $env:VCPKG_ROOT = 'C:\vcpkg'
}
if (-not $env:VCPKG_DEFAULT_TRIPLET) {
    $env:VCPKG_DEFAULT_TRIPLET = 'x64-windows'
}

$vcpkgBin = Join-Path $env:VCPKG_ROOT ("installed/{0}/bin" -f $env:VCPKG_DEFAULT_TRIPLET)
$isStaticTriplet = $env:VCPKG_DEFAULT_TRIPLET -like '*-static'

Write-Host '=== Build ==='
cargo build --release --examples

if ($isStaticTriplet) {
    Write-Host "Triplet '$($env:VCPKG_DEFAULT_TRIPLET)' is static; no DLL copy is needed."
} elseif (Test-Path $vcpkgBin) {
    $exampleDir = Join-Path (Get-Location) 'target/release/examples'
    foreach ($dll in @('zlib1.dll', 'qhull_r.dll')) {
        $src = Join-Path $vcpkgBin $dll
        if (Test-Path $src) {
            Copy-Item -Force $src $exampleDir
        }
    }
} else {
    Write-Host "Vcpkg bin directory not found at '$vcpkgBin'; skipping DLL copy."
}

Write-Host ''
Write-Host '=== cargo test --release ==='
cargo test --release

Write-Host ''
Write-Host '=== Benchmarks ==='
if ($env:RUN_BENCH -eq '1') {
    cargo bench
} else {
    Write-Host '(skipped - set RUN_BENCH=1 para correr)'
}

Write-Host ''
Write-Host '=== TODO PASO OK ==='
