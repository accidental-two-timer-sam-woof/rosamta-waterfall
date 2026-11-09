# Creates symbolic links for the P3 test fixtures, as the administrator.
#
# With developer mode off, a standard user cannot create symbolic links, so
# tests/p3_symlink.rs moves links from this pool into its scratch layout
# instead. Moving a link needs no privilege. The pool is on the system drive,
# the same volume as the standard user's %TEMP%, because a rename cannot
# cross volumes.
#
# Usage (Windows PowerShell 5.1, elevated): ./ci/make-link-pool.ps1

$ErrorActionPreference = 'Stop'

$pool = Join-Path "$env:SystemDrive\" 'rosamta-pilot-links'
# Link targets are relative to home\a in the test layout.
$targets = @{
    outside = '..\..\outside\target'
    inside  = '..\inside\target'
}

# BUILTIN\Users may move the links out: Modify, inherited by each link as
# it is created.
New-Item -ItemType Directory -Force -Path $pool | Out-Null
icacls $pool /grant '*S-1-5-32-545:(OI)(CI)M' /Q | Out-Null
if ($LASTEXITCODE -ne 0) { throw "icacls $pool failed" }

foreach ($kind in $targets.Keys) {
    $dir = Join-Path $pool $kind
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    foreach ($i in 1..8) {
        $link = Join-Path $dir ('{0:d2}' -f $i)
        cmd /c mklink "$link" "$($targets[$kind])" | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "mklink $link failed" }
    }
}
Get-ChildItem -Recurse $pool | Select-Object FullName, LinkType, Target | Format-Table -AutoSize
