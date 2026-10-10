#define AppVersion GetEnv('GOSH_VERSION')
[Setup]
AppId={{C77CF2E4-5927-4AF0-A56B-48AB92215DD5}
AppName=Gosh USB Creator
AppVersion={#AppVersion}
DefaultDirName={autopf}\Gosh USB Creator
DefaultGroupName=Gosh USB Creator
OutputDir=..\..\release
OutputBaseFilename=gosh-usb-creator-windows
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=admin
UninstallDisplayIcon={app}\gosh_usb_creator.exe
LicenseFile=..\..\LICENSE
[Files]
Source: "..\..\build\windows\x64\runner\Release\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion
[Icons]
Name: "{group}\Gosh USB Creator"; Filename: "{app}\gosh_usb_creator.exe"
