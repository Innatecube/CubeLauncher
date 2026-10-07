; Inno Setup Script for CubeLauncher
; Generated for Innatecube / CubeLauncher

#define MyAppName "CubeLauncher"
#define MyAppVersion "5.3.1"
#define MyAppPublisher "Innatecube"
#define MyAppURL "https://github.com/Innatecube/CubeLauncher"
#define MyAppExeName "CubeLauncher.exe"

[Setup]
; Unique application GUID for clean upgrades and uninstall
AppId={{8B781E7A-F75C-409A-B03C-43F15ECA00A1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}

; Show the Welcome Page on startup (matches reference design with full-height left banner)
DisableWelcomePage=no
DisableProgramGroupPage=yes

; Default installation directory in LocalAppData (no administrator prompt required)
DefaultDirName={localappdata}\Programs\{#MyAppName}
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=commandline

; Output settings
OutputDir=..\dist
OutputBaseFilename=CubeLauncher-Setup
Compression=lzma2/ultra64
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible

; Visual styling & custom branding matching reference mockup
WizardStyle=modern
SetupIconFile=windows.ico
WizardImageFile=installer_banner.bmp,installer_banner_2x.bmp
WizardSmallImageFile=installer_small.bmp,installer_small_2x.bmp

; Application integration & uninstall details
UninstallDisplayIcon={app}\{#MyAppExeName}
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

[Files]
Source: "..\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
