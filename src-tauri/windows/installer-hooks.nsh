; Ivily Desktop NSIS hooks: migrates a per-user "Wakai Desktop" (<= 1.1.0) installation.
;
; Wakai and Ivily share the identifier com.wakai.desktop, so the user data in
; %APPDATA%\com.wakai.desktop is NOT touched here. Only the old program folder, its shortcuts,
; its uninstall entry and its autostart value are moved over to the Ivily identity.
; See docs/IVILY_BRAND_MIGRATION.md.

!define LEGACY_PRODUCT "Wakai Desktop"
!define LEGACY_UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Wakai Desktop"
!define LEGACY_MANUKEY "Software\Wakai Desktop"
!define LEGACY_RUNKEY "Software\Microsoft\Windows\CurrentVersion\Run"

Var LegacyDir

; Finds the legacy install folder (uninstall entry first, default location as fallback).
!macro IvilyFindLegacy
  Push $0
  Push $1
  StrCpy $LegacyDir ""
  ReadRegStr $0 HKCU "${LEGACY_UNINSTKEY}" "InstallLocation"
  ${If} $0 != ""
    StrCpy $1 $0 1
    ${If} $1 == '"'
      StrCpy $0 $0 -1 1
    ${EndIf}
    StrCpy $LegacyDir $0
  ${ElseIf} ${FileExists} "$LOCALAPPDATA\${LEGACY_PRODUCT}\wakai-desktop.exe"
    StrCpy $LegacyDir "$LOCALAPPDATA\${LEGACY_PRODUCT}"
  ${EndIf}
  ; Same folder as the new install (should not happen): nothing to migrate.
  ${If} $LegacyDir == $INSTDIR
    StrCpy $LegacyDir ""
  ${EndIf}
  Pop $1
  Pop $0
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro IvilyFindLegacy
  ${If} $LegacyDir != ""
  ${AndIf} ${FileExists} "$LegacyDir\wakai-desktop.exe"
    ; A Wakai instance started from the old folder must be closed before its files are removed.
    !insertmacro CheckIfAppIsRunning "$LegacyDir\wakai-desktop.exe" "${LEGACY_PRODUCT}"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ${If} $LegacyDir != ""
    DetailPrint "Wakai Desktop kurulumu Ivily Desktop'a taşınıyor"

    ; Shortcuts: during updates the template only refreshes shortcuts that already exist under
    ; the NEW name, so replace each legacy shortcut with an Ivily one.
    ${If} ${FileExists} "$SMPROGRAMS\${LEGACY_PRODUCT}.lnk"
      Delete "$SMPROGRAMS\${LEGACY_PRODUCT}.lnk"
      ${IfNot} ${FileExists} "$SMPROGRAMS\${PRODUCTNAME}.lnk"
        CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
        !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
      ${EndIf}
    ${EndIf}
    ${If} ${FileExists} "$DESKTOP\${LEGACY_PRODUCT}.lnk"
      Delete "$DESKTOP\${LEGACY_PRODUCT}.lnk"
      ${IfNot} ${FileExists} "$DESKTOP\${PRODUCTNAME}.lnk"
        CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
        !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
      ${EndIf}
    ${EndIf}

    ; Autostart: the app re-registers itself under the new name on its next start when
    ; "Windows ile başlat" is on; the legacy value would start the removed executable.
    DeleteRegValue HKCU "${LEGACY_RUNKEY}" "${LEGACY_PRODUCT}"

    ; Program files: only the files the Wakai installer created. RMDir without /r removes the
    ; folder only if it is empty, so nothing else the user put there is deleted.
    ; If the old executable is still locked (the updater's process is exiting), Ivily removes
    ; it on its next start (src-tauri/src/legacy.rs).
    Delete "$LegacyDir\wakai-desktop.exe"
    Delete "$LegacyDir\wakai-desktop.exe.old"
    Delete "$LegacyDir\uninstall.exe"
    RMDir "$LegacyDir"

    DeleteRegKey HKCU "${LEGACY_UNINSTKEY}"
    DeleteRegKey HKCU "${LEGACY_MANUKEY}\${LEGACY_PRODUCT}"
    DeleteRegKey /ifempty HKCU "${LEGACY_MANUKEY}"
  ${EndIf}
!macroend
