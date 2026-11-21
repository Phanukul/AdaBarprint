; Print-owned machine data. Never bundle, copy, alter or delete StoreFront DBs.
!macro NSIS_HOOK_PREINSTALL
  ; Reuse Tauri's path-scoped process check before replacing the worker.
  !insertmacro CheckIfAppIsRunning "$INSTDIR\agent\autoprint_agent.exe" "AdaBarprint Agent"
  Push $0
  Push $1
  Push $R8
  ReadEnvStr $R8 "ProgramData"
  ${If} $R8 == ""
    MessageBox MB_ICONSTOP|MB_OK "Windows ProgramData is unavailable."
    Abort
  ${EndIf}
  CreateDirectory "$R8\AdaPos6.0\AdaBarprint\Data"
  CreateDirectory "$R8\AdaPos6.0\AdaBarprint\Exports"
  CreateDirectory "$R8\AdaPos6.0\AdaBarprint\Logs"
  CreateDirectory "$R8\AdaPos6.0\AdaBarprint\Backups"
  nsExec::ExecToStack '"$SYSDIR\icacls.exe" "$R8\AdaPos6.0\AdaBarprint" /inheritance:e /grant "*S-1-5-32-545:(OI)(CI)M"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONSTOP|MB_OK "Unable to prepare AdaBarprint data directory."
    Abort
  ${EndIf}
  Pop $R8
  Pop $1
  Pop $0
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; The per-machine installer runs elevated; a scheduled task created here
  ; would bind DPAPI/printers to the installing administrator. HKLM Run starts
  ; the worker in the actual interactive user's session instead, at next logon.
  ; No RunOnce/Exec: installation must not start a print job or enable Auto.
  ClearErrors
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "AdaBarprintAgent" '$"$INSTDIR\agent\autoprint_agent.exe$"'
  ${If} ${Errors}
    MessageBox MB_ICONSTOP|MB_OK "Unable to configure AdaBarprint background startup."
    Abort
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro CheckIfAppIsRunning "$INSTDIR\agent\autoprint_agent.exe" "AdaBarprint Agent"
  DeleteRegValue HKLM "Software\Microsoft\Windows\CurrentVersion\Run" "AdaBarprintAgent"
!macroend

; Uninstall intentionally leaves user layouts, jobs and backups intact.
