# Runs the pilot test binaries as a newly created standard (non-admin) user.
#
# GitHub-hosted Windows runners execute every step as an elevated
# administrator, which can create symbolic links and hides privilege-dependent
# behavior (DC-001). The pilot results must come from a standard user.
#
# Usage (from prototypes/pilot, Windows PowerShell 5.1):
#   ./ci/run-as-standard-user.ps1 -Log test.log

param([Parameter(Mandatory = $true)][string]$Log)

$ErrorActionPreference = 'Stop'

# Build every test binary as the runner and list their paths. Windows
# PowerShell turns a native command's stderr into errors, so cargo's progress
# output must not meet 'Stop'.
$ErrorActionPreference = 'Continue'
$artifacts = cargo test --locked --no-run --message-format=json 2>$null
$ErrorActionPreference = 'Stop'
if ($LASTEXITCODE -ne 0) { throw "cargo test --no-run failed" }
$exes = $artifacts |
    ForEach-Object { $_ | ConvertFrom-Json } |
    Where-Object { $_.reason -eq 'compiler-artifact' -and $_.profile.test -and $_.executable } |
    ForEach-Object { $_.executable }
if (-not $exes) { throw "no test binaries found" }

# A throwaway local account in the Users group only.
$user = 'pilotuser'
$plain = 'Pw-' + [guid]::NewGuid().ToString('N') + '!a1'
$password = ConvertTo-SecureString $plain -AsPlainText -Force
New-LocalUser -Name $user -Password $password -PasswordNeverExpires | Out-Null
Add-LocalGroupMember -Group 'Users' -Member $user
$credential = New-Object System.Management.Automation.PSCredential("$env:COMPUTERNAME\$user", $password)

# The account must be able to run the binaries; it writes only to its own
# profile (the tests use its %TEMP%).
$target = (Resolve-Path 'target').Path
icacls $target /grant "${user}:(OI)(CI)RX" /Q | Out-Null
try { Start-Service seclogon } catch { Write-Warning "seclogon: $_" }

$failed = @()
foreach ($exe in $exes) {
    $base = [IO.Path]::GetFileNameWithoutExtension($exe)
    $out = Join-Path $env:RUNNER_TEMP "$base.out.txt"
    $err = Join-Path $env:RUNNER_TEMP "$base.err.txt"
    $process = Start-Process -FilePath $exe -ArgumentList '--nocapture', '--test-threads=1' `
        -Credential $credential -LoadUserProfile -WorkingDirectory $target `
        -RedirectStandardOutput $out -RedirectStandardError $err -Wait -PassThru
    $text = (Get-Content -Raw $out) + (Get-Content -Raw $err)
    Add-Content -Path $Log -Value "== $base (exit $($process.ExitCode))`n$text"
    Write-Output "== $base (exit $($process.ExitCode))"
    Write-Output $text
    if ($process.ExitCode -ne 0 -or $text -notmatch 'test result: ok') { $failed += $base }
}

# The results only count if they really came from the standard user.
$logText = Get-Content -Raw $Log
if ($logText -notmatch "identity=user=$user elevated=false") {
    throw "tests did not run as non-elevated $user"
}
if ($failed) { throw "failed: $($failed -join ', ')" }
