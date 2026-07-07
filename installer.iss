; Inno Setup 安装脚本
; 编译: iscc installer.iss /dMyAppVersion=1.1.0 /dExeSource=target\release\ra2-clicker.exe

#define MyAppName "ra2-clicker"
#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif
#ifndef ExeSource
  #define ExeSource "target\release\ra2-clicker.exe"
#endif
#define MyAppPublisher "cmixed"
#define MyAppURL "https://github.com/cmixed/ra2-clicker"
#define MyAppExeName "ra2-clicker.exe"
#ifndef Arch
  #define Arch "x86_64"
#endif

[Setup]
AppId={{B8F4C3E2-1A2B-4C5D-8E7F-6A5B4C3D2E1F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=.
OutputBaseFilename=ra2-clicker-{#MyAppVersion}-{#Arch}-setup
Compression=lzma2
SolidCompression=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
PrivilegesRequired=admin

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"; GroupDescription: "快捷方式："; Flags: checkedonce

[Files]
Source: "{#ExeSource}"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "运行 {#MyAppName}"; Flags: nowait postinstall skipifsilent
