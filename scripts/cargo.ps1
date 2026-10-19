$CargoArgs = @($args)
$ErrorActionPreference = 'Stop'
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) { $cargo = Get-Item (Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe') }
$cargoPath = if ($cargo.Source) { $cargo.Source } else { $cargo.FullName }
$rustcPath = Join-Path (Split-Path $cargoPath) 'rustc.exe'
$repoRoot = Split-Path $PSScriptRoot
$oldPath = $env:PATH
$oldFlags = $env:RUSTFLAGS
Push-Location $repoRoot
try {
    $env:PATH = (Split-Path $cargoPath) + ';' + $env:PATH
    $toolchain = '1.98.1'
    $gnuVersion = & $rustcPath +stable-x86_64-pc-windows-gnu --version 2>$null
    if ($LASTEXITCODE -eq 0 -and $gnuVersion -match '^rustc 1\.98\.1 ') {
        $toolchain = 'stable-x86_64-pc-windows-gnu'
        $dlltool = $env:OATH_DLLTOOL
        if (-not $dlltool) {
            $workspaceTool = Join-Path $repoRoot '..\..\work\toolchain\llvm-mingw-20260616-ucrt-x86_64\bin\llvm-dlltool.exe'
            if (Test-Path -LiteralPath $workspaceTool) { $dlltool = (Resolve-Path -LiteralPath $workspaceTool).Path }
        }
        if ($dlltool) { $env:RUSTFLAGS = ($oldFlags + ' -C dlltool=' + $dlltool).Trim() }
    }
    & $cargoPath ('+' + $toolchain) @CargoArgs
    $code = $LASTEXITCODE
} finally {
    Pop-Location
    $env:PATH = $oldPath
    $env:RUSTFLAGS = $oldFlags
}
exit $code
