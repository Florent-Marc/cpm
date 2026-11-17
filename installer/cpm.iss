#ifndef AppVersion
  #error AppVersion must be supplied by the release workflow
#endif

[Setup]
AppId={{6D10DD78-541C-4264-A754-12A96F4BE7A1}
AppName=CPM
AppVersion={#AppVersion}
AppPublisher=Florent-Marc
AppPublisherURL=https://github.com/Florent-Marc/cpm
AppSupportURL=https://github.com/Florent-Marc/cpm/issues
AppUpdatesURL=https://github.com/Florent-Marc/cpm/releases
DefaultDirName={localappdata}\Programs\CPM
DefaultGroupName=CPM
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir=..\release
OutputBaseFilename=cpm-setup-windows-x86_64
SetupIconFile=..\crates\cpm-app\assets\cpm.ico
UninstallDisplayIcon={app}\cpm.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Créer un raccourci sur le bureau"; Flags: unchecked

[Files]
Source: "..\target\release\cpm.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\CPM"; Filename: "{app}\cpm.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\CPM"; Filename: "{app}\cpm.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\cpm.exe"; Description: "Ouvrir CPM"; Flags: nowait postinstall skipifsilent
