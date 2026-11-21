param(
    [string]$AgentPath = (Join-Path $PSScriptRoot 'autoprint_agent.exe'),
    [string]$DatabaseDirectory = '',
    [switch]$Unregister
)
$ErrorActionPreference = 'Stop'
$taskName = 'AdaBarprint-BranchAgent'
if ($Unregister) {
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue
    return
}
$resolvedAgent = (Resolve-Path -LiteralPath $AgentPath).Path
$workspaceDirectory = Split-Path -Parent $resolvedAgent
$account = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
# Runs in the signed-in user's printer session, without an administrator account.
# The Agent's exclusive OS file lock prevents multiple print workers for the same DB.
$actionArguments = @{}
if ($DatabaseDirectory) {
    $resolvedDatabase = (Resolve-Path -LiteralPath $DatabaseDirectory).Path
    $actionArguments.Argument = '--database-directory "' + $resolvedDatabase + '"'
}
$action = New-ScheduledTaskAction -Execute $resolvedAgent -WorkingDirectory $workspaceDirectory @actionArguments
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $account
$principal = New-ScheduledTaskPrincipal -UserId $account -LogonType Interactive -RunLevel Limited
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -MultipleInstances IgnoreNew `
    -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) `
    -ExecutionTimeLimit ([TimeSpan]::Zero) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
Write-Output 'Registered AdaBarprint Agent for this Windows user at sign-in. It starts on the next sign-in.'
