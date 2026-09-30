#define MyAppName "QCOS Hilbert Ledger"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "QCOS AI Core"
#define MyAppExeName "qcos-hilbert-ledger.exe"
#define MyAppPath "C:\Program Files (x86)\QCOS_AI_Core\qcos-standalone\hilbert-ledger"

[Setup]
AppId={{D3E14A89-3F71-4B93-9021-9E22A1A8B1A2}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={#MyAppPath}
DefaultGroupName={#MyAppName}
OutputBaseFilename=QCOS_Hilbert_Ledger_Setup_v{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKLM; Subkey: "SOFTWARE\QCOS_AI_Core\HilbertLedger"; ValueType: string; ValueName: "InstallDir"; ValueData: "{app}"; Flags: uninsdeletekey
Root: HKLM; Subkey: "SOFTWARE\QCOS_AI_Core\HilbertLedger"; ValueType: string; ValueName: "Version"; ValueData: "{#MyAppVersion}"; Flags: uninsdeletekey

[Run]
; Kill existing running instance before starting newly installed binary
Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -Command ""Stop-Process -Name 'qcos-hilbert-ledger' -Force -ErrorAction SilentlyContinue"""; Flags: runhidden
Filename: "{app}\{#MyAppExeName}"; Description: "Launch QCOS Hilbert Ledger Daemon"; Flags: nowait postinstall runhidden

[UninstallRun]
Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -Command ""Stop-Process -Name 'qcos-hilbert-ledger' -Force -ErrorAction SilentlyContinue"""; Flags: runhidden