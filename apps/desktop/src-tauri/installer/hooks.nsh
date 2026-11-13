; Puts the bundled `gitvault` CLI on the user's PATH, so the Node library and terminals find it.
; On Linux the .deb installs the CLI to /usr/bin, which is already on PATH.
; The folder is passed through an environment variable so a quote in the path cannot break the script.

!macro NSIS_HOOK_POSTINSTALL
  System::Call 'Kernel32::SetEnvironmentVariable(t "GITVAULT_DIR", t "$INSTDIR")'
  nsExec::Exec `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$$d=$$env:GITVAULT_DIR; $$p=[Environment]::GetEnvironmentVariable('Path','User'); $$parts=@($$p -split ';' | Where-Object { $$_ }); if ($$parts -notcontains $$d) { [Environment]::SetEnvironmentVariable('Path', (($$parts + $$d) -join ';'), 'User') }"`
  Pop $0
  ; The short `gv` alias: a one-line shim next to gitvault.exe.
  FileOpen $0 "$INSTDIR\gv.cmd" w
  FileWrite $0 '@"%~dp0gitvault.exe" %*$\r$\n'
  FileClose $0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\gv.cmd"
  System::Call 'Kernel32::SetEnvironmentVariable(t "GITVAULT_DIR", t "$INSTDIR")'
  nsExec::Exec `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$$d=$$env:GITVAULT_DIR; $$p=[Environment]::GetEnvironmentVariable('Path','User'); $$parts=@($$p -split ';' | Where-Object { $$_ -and $$_ -ne $$d }); [Environment]::SetEnvironmentVariable('Path', ($$parts -join ';'), 'User')"`
  Pop $0
!macroend
