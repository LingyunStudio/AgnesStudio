; AgnesStudio Inno Setup 安装脚本
; 用法：ISCC /DMyAppVersion="x.y.z" setup.iss
; 产出：dist\AgnesStudio-Setup-x.y.z.exe

#define MyAppName "AgnesStudio"
#define MyAppPublisher "LingyunStudio"
#define MyAppURL "https://agnes-ai.com/"
#define MyAppExeName "AgnesStudio.exe"

; AppId 固定，保证后续版本升级时被识别为同一程序
#define MyAppId "{{F3E8C7A1-9B2D-4F5E-8A3C-1D7E9B5F6A4C}}"

[Setup]
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\AgnesStudio
DefaultGroupName=AgnesStudio
AllowNoIcons=yes
OutputDir=dist
OutputBaseFilename=AgnesStudio-Setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
; 静默更新：自动关闭旧进程，安装后重启应用
CloseApplications=force
RestartApplications=yes
; 允许静默安装覆盖已存在文件
Uninstallable=yes
DisableProgramGroupPage=yes

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

[Files]
Source: "target\release\agnes-studio.exe"; DestDir: "{app}"; DestName: "AgnesStudio.exe"; Flags: ignoreversion

[Icons]
Name: "{group}\AgnesStudio"; Filename: "{app}\AgnesStudio.exe"
Name: "{group}\{cm:UninstallProgram,AgnesStudio}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\AgnesStudio"; Filename: "{app}\AgnesStudio.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\AgnesStudio.exe"; Description: "{cm:LaunchProgram,AgnesStudio}"; Flags: nowait postinstall

[Code]
function IsWebView2Installed: Boolean;
var
  RegKey: string;
begin
  Result := False;
  // Evergreen WebView2 Runtime 注册表位置
  RegKey := 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';
  if RegKeyExists(HKLM, RegKey) then
  begin
    Result := True;
    Exit;
  end;
  // 备选: per-user 安装
  if RegKeyExists(HKCU, RegKey) then
  begin
    Result := True;
    Exit;
  end;
end;

function InitializeSetup: Boolean;
begin
  Result := True;
end;

function InitializeUninstall: Boolean;
begin
  Result := True;
end;

// 安装完成后检查 WebView2，缺失则通过 PowerShell 下载并静默安装
procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
  PsCmd: string;
begin
  if CurStep = ssPostInstall then
  begin
    if not IsWebView2Installed then
    begin
      PsCmd := '-NoProfile -ExecutionPolicy Bypass -Command "& {' +
        '$out = \"$env:TEMP\MicrosoftEdgeWebview2Setup.exe\";' +
        'Invoke-WebRequest -Uri ''https://go.microsoft.com/fwlink/p/?LinkId=2124703'' -OutFile $out;' +
        'Start-Process -FilePath $out -ArgumentList ''/silent /install'' -Wait -NoNewWindow' +
        '}"';
      Exec('powershell.exe', PsCmd, '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    end;
  end;
end;

function NeedRestart: Boolean;
begin
  Result := False;
end;
