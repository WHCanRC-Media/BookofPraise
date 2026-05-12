; Inno Setup script for Book of Praise.
; Compile from the repo root with:
;   "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" /DAppVersion=YYYYMMDD installer\installer.iss
; The script consumes the dist\ tree assembled by .github/workflows/build.yml
; and emits installer\output\BookOfPraise-Setup.exe.

#define AppName       "Book of Praise"
#define AppPublisher  "Book of Praise"
#define AppExeName    "bop.exe"
#define AppId         "{B0B5F9C1-7A4D-4E2A-9D6F-3E5E0F1F2A3B}"

#ifndef AppVersion
  #define AppVersion "0.0.0-dev"
#endif

[Setup]
AppId={{#AppId}}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\BookOfPraise
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=output
OutputBaseFilename=BookOfPraise-Setup
SetupIconFile=..\assets\icon.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#AppExeName}
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\dist\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion
; Install GNU FreeSerif system-wide so resvg/fontdb finds it via load_system_fonts().
; Required for melisma underlines (Combining Half Marks U+FE27/FE28/FE2D); the
; older 2012 FreeSerif from GNU FTP lacked these glyphs, so CI now bundles the
; 2021 svn snapshot from Debian which includes (and properly tiles) them.
; Always overwrite: existing user installs may have the broken 2012 release that
; lacks Combining Half Marks (FE27/FE28/FE2D). uninsneveruninstall is kept so
; the font stays available to other apps after Book of Praise is removed.
Source: "fonts\FreeSerif.ttf"; DestDir: "{autofonts}"; FontInstall: "FreeSerif"; \
    Flags: uninsneveruninstall

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
