; Instalador de AFRWIDGETS (Inno Setup 6). Se instala para el usuario actual, sin permisos de administrador.
; Lo genera scripts/build.ps1; a mano: ISCC.exe installer\afrwidgets.iss

#define AppName "AFRWIDGETS"
#define AppVersion "1.0.0"
#define AppExe "afrwidgets.exe"

[Setup]
AppId={{6B0E2F7C-4A31-4C55-9E3B-AF7D1D0C2026}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=Pedro Gómez
AppPublisherURL=https://pedrogomez.dev/proyecto/afrwidgets
AppSupportURL=https://pedrogomez.dev/proyecto/afrwidgets
DefaultDirName={localappdata}\Programs\{#AppName}
DisableProgramGroupPage=yes
DisableDirPage=yes
DisableReadyPage=yes
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=AFRWIDGETS-Setup-{#AppVersion}
SetupIconFile=..\dist\previews\afrwidgets.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
WizardSizePercent=100
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
AppMutex=Local\AFRWIDGETS
CloseApplications=yes
LicenseFile=..\LICENSE

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "autostart"; Description: "Abrir AFRWIDGETS al iniciar Windows"; GroupDescription: "Opciones:"
Name: "desktopicon"; Description: "Crear un acceso directo en el escritorio"; GroupDescription: "Opciones:"; Flags: unchecked

[Files]
Source: "..\dist\app\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{userdesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#AppName}"; ValueData: """{app}\{#AppExe}"""; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{app}\{#AppExe}"; Description: "Abrir AFRWIDGETS ahora"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/IM {#AppExe} /F"; Flags: runhidden; RunOnceId: "CerrarAFRWIDGETS"

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\{#AppName}"
