!macro NSIS_HOOK_POSTINSTALL
  CreateShortCut "$INSTDIR\帧印.lnk" "$INSTDIR\frame-mark.exe" "" "$INSTDIR\frame-mark.exe" 0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\帧印.lnk"
!macroend
; Retain Tauri's previous-location restoration and explicit /D selection.
Function FrameMarkInstallDirectory
  Push $0
  ReadRegStr $0 HKLM "Software\peakedge\帧印" ""
  ${If} $0 == ""
  ${AndIf} $INSTDIR == "$PROGRAMFILES64\帧印"
    StrCpy $INSTDIR "$PROGRAMFILES64\framemark"
  ${EndIf}
  ; Never silently create a machine-wide copy beside an old per-user installation.
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\帧印" "UninstallString"
  ${If} $0 != ""
    IfSilent +2
      MessageBox MB_OK|MB_ICONEXCLAMATION "检测到旧版当前用户安装。请先通过 Windows 设置卸载旧版（不要勾选删除用户数据），再安装新版。水印信息、历史和设置将继续保留。"
    SetErrorLevel 3
    Quit
  ${EndIf}
  Pop $0
FunctionEnd

!define MUI_CUSTOMFUNCTION_GUIINIT FrameMarkInstallDirectory

!macro NSIS_HOOK_PREINSTALL
  ${If} ${Silent}
    Call FrameMarkInstallDirectory
  ${EndIf}
  ; Tauri selects OUTDIR before this hook; changing INSTDIR alone splits the installation.
  ${If} $OUTDIR != $INSTDIR
    Push $0
    StrCpy $0 $OUTDIR
    SetOutPath $INSTDIR
    RMDir $0 ; Only removes the obsolete empty directory; never removes user files.
    Pop $0
  ${EndIf}
!macroend
