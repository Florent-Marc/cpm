param(
    [int]$OldPid,
    [string]$TargetPath,
    [string]$DownloadPath,
    [string]$BackupPath
)
$ErrorActionPreference = 'Stop'
$stage = Split-Path -Parent $BackupPath
$replacementInstalled = $false
try {
    $oldProcess = Get-Process -Id $OldPid -ErrorAction SilentlyContinue
    if ($oldProcess) { $oldProcess.WaitForExit() }
    # Attendre aussi une éventuelle libération tardive du fichier par Windows.
    $moved = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        try {
            Move-Item -LiteralPath $TargetPath -Destination $BackupPath
            $moved = $true
            break
        } catch { Start-Sleep -Milliseconds 300 }
    }
    if (-not $moved) { throw 'Le programme reste verrouille par Windows.' }
    try {
        Move-Item -LiteralPath $DownloadPath -Destination $TargetPath
        $replacementInstalled = $true
    } catch {
        Move-Item -LiteralPath $BackupPath -Destination $TargetPath
        throw
    }
    $newProcess = Start-Process -FilePath $TargetPath -WorkingDirectory (Split-Path -Parent $TargetPath) -PassThru
    Start-Sleep -Milliseconds 1500
    if ($newProcess.HasExited -and $newProcess.ExitCode -ne 0) { throw 'La nouvelle version ne demarre pas.' }
    Remove-Item -LiteralPath $stage -Recurse -Force -ErrorAction SilentlyContinue
} catch {
    $errorText = $_ | Out-String
    $errorText | Out-File -LiteralPath (Join-Path $stage 'update-error.txt') -Encoding utf8
    if (Test-Path -LiteralPath $BackupPath) {
        if ($replacementInstalled -and (Test-Path -LiteralPath $TargetPath)) {
            Move-Item -LiteralPath $TargetPath -Destination (Join-Path $stage 'failed-update.exe') -ErrorAction SilentlyContinue
        }
        if (-not (Test-Path -LiteralPath $TargetPath)) {
            Move-Item -LiteralPath $BackupPath -Destination $TargetPath -ErrorAction SilentlyContinue
        }
    }
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.MessageBox]::Show("La mise a jour a echoue. Les fichiers de recuperation sont dans : $stage", 'CPM - Mise a jour') | Out-Null
    if (Test-Path -LiteralPath $TargetPath) { Start-Process -FilePath $TargetPath }
}
