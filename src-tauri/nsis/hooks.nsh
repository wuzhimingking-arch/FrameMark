!macro NSIS_HOOK_POSTINSTALL
  CreateShortCut "$INSTDIR\帧印.lnk" "$INSTDIR\frame-mark.exe" "" "$INSTDIR\frame-mark.exe" 0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\帧印.lnk"
!macroend
