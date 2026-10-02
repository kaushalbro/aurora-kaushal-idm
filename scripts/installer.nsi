; ==============================================================================
; AURORA Kaushal IDM - Modern NSIS Windows Installer Script
; Produces: dist/aurora-kaushal-idm-v0.1.0-setup.exe
; ==============================================================================

!define PRODUCT_NAME "AURORA IDM"
!define PRODUCT_FULL_NAME "AURORA Kaushal IDM"
!define PRODUCT_VERSION "0.1.0"
!define PRODUCT_PUBLISHER "AURORA Development Team"
!define PRODUCT_WEB_SITE "https://github.com/kaushalbro/aurora-kaushal-idm"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\aurora-gui.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKCU"

; Best LZMA Solid Compression
SetCompressor /SOLID lzma

; Request user-level permissions (No UAC prompt required, standard modern app)
RequestExecutionLevel user

; Includes
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"

; MUI Settings
!define MUI_ABORTWARNING
!define MUI_ICON "../apps/aurora-gui/resources/icon.ico"
!define MUI_UNICON "../apps/aurora-gui/resources/icon.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

; Finish page with option to run app immediately
!define MUI_FINISHPAGE_RUN "$INSTDIR\aurora-gui.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch AURORA IDM now"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Language
!insertmacro MUI_LANGUAGE "English"

Name "${PRODUCT_FULL_NAME}"
OutFile "../dist/aurora-kaushal-idm-v${PRODUCT_VERSION}-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\AURORA IDM"
InstallDirRegKey HKCU "Software\AURORA IDM" "InstallDir"
ShowInstDetails show
ShowUnInstDetails show

; ==============================================================================
; Installer Section
; ==============================================================================
Section "MainSection" SEC01
    ; 1. Terminate any running instances before overwriting
    DetailPrint "Terminating running AURORA instances..."
    ExecWait 'taskkill /F /IM aurora-gui.exe' $0
    Sleep 500

    ; 2. Clean previous installation directory
    SetOutPath "$INSTDIR"
    SetOverwrite on

    ; 3. Install binary & icon assets
    DetailPrint "Extracting application binaries..."
    File "../target/x86_64-pc-windows-gnu/release/aurora-gui.exe"
    File "../apps/aurora-gui/resources/icon.ico"
    File "../apps/aurora-gui/resources/icon-128.png"

    ; 4. Create Start Menu Shortcuts
    DetailPrint "Creating Start Menu shortcuts..."
    CreateDirectory "$SMPROGRAMS\AURORA IDM"
    CreateShortCut "$SMPROGRAMS\AURORA IDM\AURORA IDM.lnk" "$INSTDIR\aurora-gui.exe" "" "$INSTDIR\icon.ico" 0
    CreateShortCut "$SMPROGRAMS\AURORA IDM\Uninstall AURORA IDM.lnk" "$INSTDIR\Uninstall.exe" "" "$INSTDIR\Uninstall.exe" 0

    ; 5. Create Desktop Shortcut with Icon
    DetailPrint "Creating Desktop shortcut..."
    CreateShortCut "$DESKTOP\AURORA IDM.lnk" "$INSTDIR\aurora-gui.exe" "" "$INSTDIR\icon.ico" 0

    ; 6. Write Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; 7. Write Windows Registry Keys for Add/Remove Programs
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_FULL_NAME}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\icon.ico"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoModify" 1
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoRepair" 1

    ; Register custom URI scheme for browser extension integration
    WriteRegStr HKCU "Software\Classes\aurora" "" "URL:AURORA Protocol"
    WriteRegStr HKCU "Software\Classes\aurora" "URL Protocol" ""
    WriteRegStr HKCU "Software\Classes\aurora\DefaultIcon" "" "$INSTDIR\icon.ico"
    WriteRegStr HKCU "Software\Classes\aurora\shell\open\command" "" '"$INSTDIR\aurora-gui.exe" "%1"'

    ; Save install dir
    WriteRegStr HKCU "Software\AURORA IDM" "InstallDir" "$INSTDIR"
SectionEnd

; ==============================================================================
; Uninstaller Section
; ==============================================================================
Section "Uninstall"
    ; 1. Terminate running process
    DetailPrint "Closing running instances..."
    ExecWait 'taskkill /F /IM aurora-gui.exe' $0
    Sleep 500

    ; 2. Delete Shortcuts
    DetailPrint "Removing shortcuts..."
    Delete "$DESKTOP\AURORA IDM.lnk"
    Delete "$DESKTOP\AURORA Kaushal IDM.lnk"
    Delete "$SMPROGRAMS\AURORA IDM\AURORA IDM.lnk"
    Delete "$SMPROGRAMS\AURORA IDM\Uninstall AURORA IDM.lnk"
    RMDir "$SMPROGRAMS\AURORA IDM"
    RMDir "$SMPROGRAMS\AURORA Kaushal IDM"

    ; 3. Delete Application Files
    DetailPrint "Deleting installed files..."
    Delete "$INSTDIR\aurora-gui.exe"
    Delete "$INSTDIR\icon.ico"
    Delete "$INSTDIR\icon-128.png"
    Delete "$INSTDIR\icon.png"
    Delete "$INSTDIR\uninstall.bat"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"

    ; 4. Delete Registry Entries
    DetailPrint "Removing registry entries..."
    DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
    DeleteRegKey HKCU "Software\Classes\aurora"
    DeleteRegKey HKCU "Software\AURORA IDM"

    SetAutoClose true
SectionEnd
