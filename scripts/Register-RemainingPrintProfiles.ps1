param(
    [string]$ExePath = (Join-Path $PSScriptRoot '..\src-tauri\target\debug\adabarprint-tauri.exe'),
    [datetime]$FirstRun = (Get-Date).AddMinutes(3),
    [switch]$Register
)

$ErrorActionPreference = 'Stop'
$exe = (Get-Item -LiteralPath $ExePath).FullName
if ($FirstRun -le (Get-Date)) { throw 'FirstRun must be in the future.' }
$profiles = [ordered]@{ '003' = 'PromoStart'; '001' = 'PromoEnd'; '002' = 'NewProduct' }
$scheduledTime = $FirstRun
if ($Register) {
    $account = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
    $principal = New-ScheduledTaskPrincipal -UserId $account -LogonType Interactive -RunLevel Limited
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    $existing = @(Get-ScheduledTask -TaskPath '\')
}

foreach ($code in $profiles.Keys) {
    $name = "AdaBarprint-$code-$($profiles[$code])"
    $arguments = "/profile/$code"
    $equivalentArguments = @($arguments, "--profile $code", "--profile=$code", "/profile:$code", "-p $code", $code)
    $result = 'Preview'
    if ($Register) {
        # Also recognise tasks the operator already created with a Thai name.
        $duplicate = $existing | Where-Object {
            $_.TaskName -eq $name -or ($_.Actions | Where-Object {
                $_.Execute -and $_.Execute.Trim('"') -eq $exe -and
                $_.Arguments -and $_.Arguments.Trim() -in $equivalentArguments
            })
        }
        if ($duplicate) {
            $result = 'Skipped (already registered)'
        } else {
            $action = New-ScheduledTaskAction -Execute $exe -Argument $arguments -WorkingDirectory (Split-Path -Parent $exe)
            $trigger = New-ScheduledTaskTrigger -Once -At $scheduledTime
            Register-ScheduledTask -TaskPath '\' -TaskName $name -Action $action -Trigger $trigger -Principal $principal -Settings $settings | Out-Null
            $result = 'Registered'
        }
    }
    [pscustomobject]@{ Task = $name; Arguments = $arguments; PlannedTime = $scheduledTime.ToString('yyyy-MM-dd HH:mm:ss'); Result = $result }
    $scheduledTime = $scheduledTime.AddMinutes(3)
}
