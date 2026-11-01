; Inno Setup Script for CubeLauncher
; Generated for Innatecube / CubeLauncher

#define MyAppName "CubeLauncher"
#define MyAppVersion "5.3.1"
#define MyAppPublisher "Innatecube"
#define MyAppURL "https://github.com/Innatecube/CubeLauncher"
#define MyAppExeName "CubeLauncher.exe"
#define MyAppId "{8B781E7A-F75C-409A-B03C-43F15ECA00A1}"

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
DefaultDirName={code:GetDefaultInstallDir}
UsePreviousAppDir=yes
UsePreviousTasks=yes
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
CloseApplicationsFilter=*{#MyAppExeName}*
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

[Files]
Source: "..\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "installer_banner.bmp"; DestDir: "{app}"; Flags: ignoreversion
Source: "installer_banner_2x.bmp"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
var
  IsUpgrade: Boolean;
  ExistingInstallDir: String;
  ExistingVersion: String;

function GetDefaultInstallDir(Param: String): String;
begin
  if ExistingInstallDir <> '' then
    Result := ExistingInstallDir
  else
    Result := ExpandConstant('{localappdata}\Programs\{#MyAppName}');
end;

function InitializeSetup(): Boolean;
var
  AppKey: String;
  PrevPath: String;
  PrevVer: String;
  ExeFile: String;
begin
  Result := True;
  IsUpgrade := False;
  ExistingInstallDir := '';
  ExistingVersion := '';

  AppKey := 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#MyAppId}_is1';

  // 1. Check HKCU (Current User uninstall registry)
  if RegQueryStringValue(HKCU, AppKey, 'Inno Setup: App Path', PrevPath) or
     RegQueryStringValue(HKCU, AppKey, 'InstallLocation', PrevPath) then
  begin
    if PrevPath <> '' then
    begin
      ExeFile := AddBackslash(PrevPath) + '{#MyAppExeName}';
      if DirExists(PrevPath) or FileExists(ExeFile) then
      begin
        IsUpgrade := True;
        ExistingInstallDir := PrevPath;
        if RegQueryStringValue(HKCU, AppKey, 'DisplayVersion', PrevVer) then
          ExistingVersion := PrevVer;
      end;
    end;
  end;

  // 2. Check HKLM (Local Machine uninstall registry if not found in HKCU)
  if not IsUpgrade then
  begin
    if RegQueryStringValue(HKLM, AppKey, 'Inno Setup: App Path', PrevPath) or
       RegQueryStringValue(HKLM, AppKey, 'InstallLocation', PrevPath) then
    begin
      if PrevPath <> '' then
      begin
        ExeFile := AddBackslash(PrevPath) + '{#MyAppExeName}';
        if DirExists(PrevPath) or FileExists(ExeFile) then
        begin
          IsUpgrade := True;
          ExistingInstallDir := PrevPath;
          if RegQueryStringValue(HKLM, AppKey, 'DisplayVersion', PrevVer) then
            ExistingVersion := PrevVer;
        end;
      end;
    end;
  end;

  // 3. Fallback: Check standard LocalAppData directory if executable exists
  if not IsUpgrade then
  begin
    PrevPath := ExpandConstant('{localappdata}\Programs\{#MyAppName}');
    ExeFile := AddBackslash(PrevPath) + '{#MyAppExeName}';
    if FileExists(ExeFile) then
    begin
      IsUpgrade := True;
      ExistingInstallDir := PrevPath;
    end;
  end;

  // If we found the directory but don't have the version yet, get it from the executable
  if IsUpgrade and (ExistingVersion = '') and (ExistingInstallDir <> '') then
  begin
    ExeFile := AddBackslash(ExistingInstallDir) + '{#MyAppExeName}';
    if FileExists(ExeFile) then
    begin
      GetVersionNumbersString(ExeFile, PrevVer);
      ExistingVersion := PrevVer;
    end;
  end;
end;

procedure InitializeWizard();
begin
  if IsUpgrade and (ExistingInstallDir <> '') then
  begin
    WizardForm.DirEdit.Text := ExistingInstallDir;
  end;
end;

function ShouldSkipPage(PageID: Integer): Boolean;
begin
  Result := False;
  if IsUpgrade then
  begin
    // When updating, skip directory selection, task selection, and ready page
    // so clicking Update on the welcome screen updates immediately in-place.
    if (PageID = wpSelectDir) or (PageID = wpSelectTasks) or (PageID = wpReady) then
      Result := True;
  end;
end;

procedure CurPageChanged(CurPageID: Integer);
var
  UpdateMsg: String;
begin
  if IsUpgrade then
  begin
    if CurPageID = wpWelcome then
    begin
      WizardForm.WelcomeLabel1.Caption := 'Welcome to the CubeLauncher Update Wizard';
      
      UpdateMsg := 'Setup detected an existing installation of CubeLauncher';
      if ExistingVersion <> '' then
        UpdateMsg := UpdateMsg + ' (version ' + ExistingVersion + ')';
      UpdateMsg := UpdateMsg + ' at:' + #13#10 + ExistingInstallDir + #13#10#13#10 +
                   'Setup will now update CubeLauncher to version {#MyAppVersion}.' + #13#10#13#10 +
                   'Click Update to proceed with the update.';

      WizardForm.WelcomeLabel2.Caption := UpdateMsg;
      WizardForm.NextButton.Caption := '&Update';
    end
    else if CurPageID = wpInstalling then
    begin
      WizardForm.StatusLabel.Caption := 'Updating CubeLauncher files...';
    end
    else if CurPageID = wpFinished then
    begin
      WizardForm.FinishedHeadingLabel.Caption := 'CubeLauncher Updated Successfully';
      WizardForm.FinishedLabel.Caption :=
        'CubeLauncher has been updated to version {#MyAppVersion}.' + #13#10#13#10 +
        'Click Finish to exit Setup.';
    end;
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
begin
  Result := '';
  // Close any running instance of CubeLauncher before file replacement
  Exec('taskkill.exe', '/IM {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Sleep(250);
  // If still running, force terminate so update doesn't fail with file locked error
  Exec('taskkill.exe', '/F /IM {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Sleep(150);
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  CommonProgramsDir: String;
  CommonDesktopDir: String;
  UserDesktopDir: String;
  AppKey: String;
  PrevPath: String;
begin
  if CurStep = ssPostInstall then
  begin
    CommonProgramsDir := ExpandConstant('{commonprograms}');
    CommonDesktopDir := ExpandConstant('{commondesktop}');
    UserDesktopDir := ExpandConstant('{userdesktop}');

    // Clean up any legacy / stale Start Menu shortcuts
    DeleteFile(AddBackslash(CommonProgramsDir) + 'CubeLauncher.lnk');
    DeleteFile(AddBackslash(CommonProgramsDir) + 'Cube Launcher.lnk');

    // Clean up any legacy / stale Desktop shortcuts
    DeleteFile(AddBackslash(CommonDesktopDir) + 'Cube Launcher.lnk');
    DeleteFile(AddBackslash(UserDesktopDir) + 'Cube Launcher.lnk');

    // Remove legacy Program Files copies if they exist and are not current install dir
    if Uppercase(WizardDirValue) <> Uppercase('C:\Program Files\CubeLauncher') then
    begin
      DeleteFile('C:\Program Files\CubeLauncher\CubeLauncher.exe');
      DeleteFile('C:\Program Files\CubeLauncher\unins000.exe');
      DeleteFile('C:\Program Files\CubeLauncher\unins000.dat');
      RemoveDir('C:\Program Files\CubeLauncher');
      RegDeleteKeyIncludingSubkeys(HKLM, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#MyAppId}_is1');
    end;

    if Uppercase(WizardDirValue) <> Uppercase('C:\Program Files\Cube Launcher') then
    begin
      DeleteFile('C:\Program Files\Cube Launcher\Cube Launcher.exe');
      DeleteFile('C:\Program Files\Cube Launcher\uninstall.exe');
      RemoveDir('C:\Program Files\Cube Launcher');
      RegDeleteKeyIncludingSubkeys(HKLM, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\Cube Launcher');
    end;

    // Append /SILENT to UninstallString so Windows Settings/Control Panel automatically runs the custom uninstaller wizard
    AppKey := 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#MyAppId}_is1';
    if RegQueryStringValue(HKCU, AppKey, 'UninstallString', PrevPath) then
    begin
      if Pos('/SILENT', UpperCase(PrevPath)) = 0 then
        RegWriteStringValue(HKCU, AppKey, 'UninstallString', PrevPath + ' /SILENT');
    end;
    if RegQueryStringValue(HKLM, AppKey, 'UninstallString', PrevPath) then
    begin
      if Pos('/SILENT', UpperCase(PrevPath)) = 0 then
        RegWriteStringValue(HKLM, AppKey, 'UninstallString', PrevPath + ' /SILENT');
    end;
  end;
end;

var
  DeleteAppData: Boolean;

function ShowCustomUninstallWizard(): Boolean;
var
  UninstForm: TSetupForm;
  BannerImg: TBitmapImage;
  BannerPath: String;
  TitleLabel: TNewStaticText;
  DescLabel: TNewStaticText;
  DeleteCheckBox: TNewCheckBox;
  DeleteNoteLabel: TNewStaticText;
  BottomBevel: TBevel;
  UninstallBtn: TNewButton;
  CancelBtn: TNewButton;
  ButtonWidth: Integer;
  LeftContent: Integer;
begin
  Result := False;
  DeleteAppData := False;

  // Window matches standard Inno Setup modern wizard form dimensions
  UninstForm := CreateCustomForm(ScaleX(497), ScaleY(360), False, False);
  try
    UninstForm.Caption := 'CubeLauncher Uninstall';
    UninstForm.BorderStyle := bsDialog;
    UninstForm.Position := poScreenCenter;

    // 1. Full-height Left Banner Image (matches installer welcome page)
    BannerPath := ExpandConstant('{app}\installer_banner_2x.bmp');
    if not FileExists(BannerPath) then
      BannerPath := ExpandConstant('{app}\installer_banner.bmp');

    if FileExists(BannerPath) then
    begin
      BannerImg := TBitmapImage.Create(UninstForm);
      BannerImg.Parent := UninstForm;
      BannerImg.Left := 0;
      BannerImg.Top := 0;
      BannerImg.Width := ScaleX(164);
      BannerImg.Height := ScaleY(314);
      BannerImg.Stretch := True;
      BannerImg.Bitmap.LoadFromFile(BannerPath);
      LeftContent := ScaleX(180);
    end
    else
    begin
      LeftContent := ScaleX(24);
    end;

    // 2. Welcome Title Label
    TitleLabel := TNewStaticText.Create(UninstForm);
    TitleLabel.Parent := UninstForm;
    TitleLabel.Left := LeftContent;
    TitleLabel.Top := ScaleY(16);
    TitleLabel.Width := UninstForm.ClientWidth - LeftContent - ScaleX(20);
    TitleLabel.Font.Size := 12;
    TitleLabel.Font.Style := [fsBold];
    TitleLabel.Caption := 'Welcome to the CubeLauncher Uninstall Wizard';
    TitleLabel.AutoSize := True;

    // 3. Description Label
    DescLabel := TNewStaticText.Create(UninstForm);
    DescLabel.Parent := UninstForm;
    DescLabel.Left := LeftContent;
    DescLabel.Top := ScaleY(72);
    DescLabel.Width := UninstForm.ClientWidth - LeftContent - ScaleX(24);
    DescLabel.Caption :=
      'This wizard will remove CubeLauncher from your computer.' + #13#10#13#10 +
      'Please make sure CubeLauncher is completely closed before continuing.' + #13#10#13#10 +
      'Click Uninstall to begin removal, or Cancel to exit.';
    DescLabel.AutoSize := True;

    // 4. Delete Application Data Checkbox
    DeleteCheckBox := TNewCheckBox.Create(UninstForm);
    DeleteCheckBox.Parent := UninstForm;
    DeleteCheckBox.Left := LeftContent;
    DeleteCheckBox.Top := ScaleY(180);
    DeleteCheckBox.Width := UninstForm.ClientWidth - LeftContent - ScaleX(24);
    DeleteCheckBox.Height := ScaleY(22);
    DeleteCheckBox.Font.Style := [fsBold];
    DeleteCheckBox.Caption := 'Delete Application data';
    DeleteCheckBox.Checked := False;

    // 5. Explanatory Note for Checkbox
    DeleteNoteLabel := TNewStaticText.Create(UninstForm);
    DeleteNoteLabel.Parent := UninstForm;
    DeleteNoteLabel.Left := LeftContent + ScaleX(20);
    DeleteNoteLabel.Top := ScaleY(206);
    DeleteNoteLabel.Width := UninstForm.ClientWidth - LeftContent - ScaleX(44);
    DeleteNoteLabel.Caption :=
      'Permanently deletes all Minecraft instances, worlds, mods, accounts, ' +
      'and configurations stored in %APPDATA%\CubeLauncher.';
    DeleteNoteLabel.AutoSize := True;

    // 6. Bottom Bevel Separator
    BottomBevel := TBevel.Create(UninstForm);
    BottomBevel.Parent := UninstForm;
    BottomBevel.Left := 0;
    BottomBevel.Top := ScaleY(314);
    BottomBevel.Width := UninstForm.ClientWidth;
    BottomBevel.Height := ScaleY(2);

    // 7. Uninstall & Cancel Buttons
    UninstallBtn := TNewButton.Create(UninstForm);
    UninstallBtn.Parent := UninstForm;
    UninstallBtn.Caption := '&Uninstall';
    UninstallBtn.ModalResult := mrOk;
    UninstallBtn.Default := True;

    CancelBtn := TNewButton.Create(UninstForm);
    CancelBtn.Parent := UninstForm;
    CancelBtn.Caption := 'Cancel';
    CancelBtn.ModalResult := mrCancel;
    CancelBtn.Cancel := True;

    ButtonWidth := UninstForm.CalculateButtonWidth([UninstallBtn.Caption, CancelBtn.Caption]);
    if ButtonWidth < ScaleX(75) then
      ButtonWidth := ScaleX(75);

    CancelBtn.Width := ButtonWidth;
    CancelBtn.Height := ScaleY(24);
    CancelBtn.Left := UninstForm.ClientWidth - ButtonWidth - ScaleX(12);
    CancelBtn.Top := ScaleY(323);

    UninstallBtn.Width := ButtonWidth;
    UninstallBtn.Height := ScaleY(24);
    UninstallBtn.Left := CancelBtn.Left - ButtonWidth - ScaleX(8);
    UninstallBtn.Top := ScaleY(323);

    UninstForm.ActiveControl := UninstallBtn;

    // Show the modal wizard dialog
    if UninstForm.ShowModal() = mrOk then
    begin
      Result := True;
      DeleteAppData := DeleteCheckBox.Checked;
    end;
  finally
    UninstForm.Free();
  end;
end;

function InitializeUninstall(): Boolean;
var
  ResultCode: Integer;
begin
  // When executed normally without /SILENT, re-launch with /SILENT so Inno Setup's
  // default message box is bypassed and our custom wizard UI controls the flow.
  if not UninstallSilent then
  begin
    Exec(ExpandConstant('{uninstallexe}'), '/SILENT', '', SW_SHOW, ewNoWait, ResultCode);
    Result := False;
    Exit;
  end;

  Result := ShowCustomUninstallWizard();

  if Result then
  begin
    // Terminate any running CubeLauncher process before uninstallation starts
    Exec('taskkill.exe', '/F /IM {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
    Sleep(150);
  end;
end;

procedure InitializeUninstallProgressForm();
begin
  UninstallProgressForm.PageNameLabel.Caption := 'Uninstalling CubeLauncher';
  UninstallProgressForm.PageDescriptionLabel.Caption := 'Please wait while CubeLauncher is removed from your computer.';
  UninstallProgressForm.StatusLabel.Caption := 'Removing files and configurations...';
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  AppDataDir: String;
  OldAppDataDir: String;
  LocalAppDataDir: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    if DeleteAppData then
    begin
      AppDataDir := ExpandConstant('{userappdata}\{#MyAppName}');
      if DirExists(AppDataDir) then
        DelTree(AppDataDir, True, True, True);

      OldAppDataDir := ExpandConstant('{userappdata}\PandoraLauncher');
      if DirExists(OldAppDataDir) then
        DelTree(OldAppDataDir, True, True, True);

      LocalAppDataDir := ExpandConstant('{localappdata}\{#MyAppName}');
      if DirExists(LocalAppDataDir) then
        DelTree(LocalAppDataDir, True, True, True);
    end;
  end
  else if CurUninstallStep = usDone then
  begin
    if DeleteAppData then
      MsgBox('CubeLauncher and all application data were successfully removed from your computer.', mbInformation, MB_OK)
    else
      MsgBox('CubeLauncher was successfully removed from your computer.', mbInformation, MB_OK);
  end;
end;
