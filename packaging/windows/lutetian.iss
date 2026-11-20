#define MyAppName "Lutetian"
#define MyAppDisplayName "Lutetian Studio"
#define MyAppPublisher "hamides-dev"
#define MyAppURL "https://github.com/hamides-dev/lutetian"
#define MyAppExeName "lutetian.exe"
#define MyAppMcpExeName "lutetian-mcp.exe"

; x-release-please-start-version
#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif
; x-release-please-end

[Setup]
AppId={{C3A9175E-6D2F-4E91-9A4B-88E2B571A3C4}
AppName={#MyAppDisplayName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppDisplayName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppDisplayName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
OutputBaseFilename=lutetian-setup-windows-x86_64
SetupIconFile=..\..\assets\icons\lutetian.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
CloseApplications=yes
RestartApplications=yes
ChangesAssociations=yes
AppMutex=LutetianStudioSingleInstanceMutex

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\target\release\{#MyAppMcpExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\antigravity-plugin\*"; DestDir: "{app}\antigravity-plugin"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppDisplayName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Uninstall {#MyAppDisplayName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppDisplayName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppDisplayName}"; Flags: nowait postinstall skipifsilent
Filename: "{app}\{#MyAppExeName}"; Flags: nowait runasoriginaluser; Check: HasRelaunchParam

[Code]
function HasRelaunchParam(): Boolean;
var
  I: Integer;
begin
  Result := False;
  for I := 1 to ParamCount do
  begin
    if CompareText(ParamStr(I), '/RELAUNCH') = 0 then
    begin
      Result := True;
      Exit;
    end;
  end;
end;
