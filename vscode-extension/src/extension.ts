import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as fs from 'fs';
import * as path from 'path';

const execAsync = promisify(exec);

// Helper to resolve the correct Rust ContextFlow CLI command
function getCFCommand(subcommand: string): string {
    const wsRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || vscode.workspace.rootPath || "";
    const isWindows = process.platform === "win32";
    const exeName = isWindows ? "daemon-rust.exe" : "daemon-rust";

    // 1. Packaged production binary path inside the installed extension
    const platform = process.platform;
    const arch = process.arch;
    let platformDir = "";
    if (platform === "win32" && arch === "x64") {
        platformDir = "win32-x64";
    } else if (platform === "darwin") {
        platformDir = arch === "arm64" ? "darwin-arm64" : "darwin-x64";
    } else if (platform === "linux" && arch === "x64") {
        platformDir = "linux-x64";
    }

    const packagedBin = platformDir ? path.join(__dirname, "..", "bin", platformDir, exeName) : "";

    // 2. Development paths inside workspace
    const debugBin = `${wsRoot}/daemon-rust/target/debug/${exeName}`;
    const releaseBin = `${wsRoot}/daemon-rust/target/release/${exeName}`;

    if (packagedBin && fs.existsSync(packagedBin)) {
        return `"${packagedBin}" ${subcommand}`;
    } else if (fs.existsSync(releaseBin)) {
        return `"${releaseBin}" ${subcommand}`;
    } else if (fs.existsSync(debugBin)) {
        return `"${debugBin}" ${subcommand}`;
    } else {
        // Fallback to cargo run for development environments
        return `cargo run --manifest-path "${wsRoot}/daemon-rust/Cargo.toml" --quiet --bin daemon-rust -- ${subcommand}`;
    }
}

export function activate(context: vscode.ExtensionContext) {
  console.log('ContextFlow extension is now active (Rust Core Integrated)');

  const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  statusBarItem.command = 'contextflow.showMenu';
  statusBarItem.text = '$(sync~spin) CF: Checking...';
  statusBarItem.show();
  context.subscriptions.push(statusBarItem);

  // 주기적으로 세션 상태 업데이트
  const updateStatus = async () => {
    try {
      const { stdout } = await execAsync(getCFCommand('status'), { cwd: vscode.workspace.rootPath });
      if (stdout.includes('Active Session:') && !stdout.includes('Active Session: None')) {
        statusBarItem.text = '$(pulse) CF: Active';
        statusBarItem.tooltip = 'ContextFlow is tracking your work. Click for menu.';
      } else {
        statusBarItem.text = '$(circle-slash) CF: Idle';
        statusBarItem.tooltip = 'ContextFlow is idle. Click to start.';
      }
    } catch (e) {
      statusBarItem.text = '$(error) CF: Offline';
    }
  };

  setInterval(updateStatus, 15000);
  updateStatus();

  // 명령어 등록: 메뉴 보기
  let menuCmd = vscode.commands.registerCommand('contextflow.showMenu', async () => {
    const items = [
      { label: '$(comment-discussion) Show Context Summary', command: 'contextflow.showSummary' },
      { label: '$(wrench) Fix with AI (Agentic Fix)', command: 'contextflow.fix' },
      { label: '$(target) Set Current Goal', command: 'contextflow.setGoal' },
      { label: '📚 Show Knowledge Base', command: 'contextflow.showKnowledge' },
      { label: '$(list-ordered) Generate Execution Plan', command: 'contextflow.showPlan' },
      { label: '$(debug-restart) Restart Session', command: 'contextflow.startSession' }
    ];
    const selection = await vscode.window.showQuickPick(items, { placeHolder: 'ContextFlow Actions' });
    if (selection) {
      vscode.commands.executeCommand(selection.command);
    }
  });

  // 명령어 등록: 자동 수정
  let fixCmd = vscode.commands.registerCommand('contextflow.fix', async () => {
    const terminal = vscode.window.activeTerminal || vscode.window.createTerminal('ContextFlow');
    terminal.show();
    terminal.sendText(getCFCommand('fix'));
  });

  // 명령어 등록: 요약 보기 (Webview 고도화)
  let summarizeCmd = vscode.commands.registerCommand('contextflow.showSummary', async () => {
    vscode.window.withProgress({
      location: vscode.ProgressLocation.Notification,
      title: "ContextFlow: Analyzing context...",
      cancellable: false
    }, async () => {
      try {
        const { stdout } = await execAsync(getCFCommand('summarize'), { cwd: vscode.workspace.rootPath });
        const panel = vscode.window.createWebviewPanel('cfSummary', 'ContextFlow Summary', vscode.ViewColumn.Two, { enableScripts: true });
        
        panel.webview.html = getWebviewContent('Context Summary', stdout);
      } catch (e: any) {
        vscode.window.showErrorMessage(`Summary failed: ${e.message}`);
      }
    });
  });

  // 명령어 등록: 계획 생성
  let planCmd = vscode.commands.registerCommand('contextflow.showPlan', async () => {
    vscode.window.withProgress({
      location: vscode.ProgressLocation.Notification,
      title: "ContextFlow: Generating plan...",
      cancellable: false
    }, async () => {
      try {
        const { stdout } = await execAsync(getCFCommand('plan'), { cwd: vscode.workspace.rootPath });
        const panel = vscode.window.createWebviewPanel('cfPlan', 'Execution Plan', vscode.ViewColumn.Two, { enableScripts: true });
        panel.webview.html = getWebviewContent('Execution Plan', stdout);
      } catch (e: any) {
        vscode.window.showErrorMessage(`Planning failed: ${e.message}`);
      }
    });
  });

  // 명령어 등록: 목표 설정
  let setGoalCmd = vscode.commands.registerCommand('contextflow.setGoal', async () => {
    const goal = await vscode.window.showInputBox({ prompt: 'What is your current goal?' });
    if (goal) {
      try {
        // Rust daemon config set
        await execAsync(getCFCommand(`config --set "goal=${goal}"`), { cwd: vscode.workspace.rootPath });
        vscode.window.showInformationMessage(`Goal set successfully: ${goal}`);
        updateStatus();
      } catch (e: any) {
        vscode.window.showErrorMessage(`Failed to set goal: ${e.message}`);
      }
    }
  });

  // 명령어 등록: 지식 관리 조회
  let knowledgeCmd = vscode.commands.registerCommand('contextflow.showKnowledge', async () => {
    try {
      const { stdout } = await execAsync(getCFCommand('knowledge --list'), { cwd: vscode.workspace.rootPath });
      const panel = vscode.window.createWebviewPanel('cfKnowledge', 'ContextFlow Knowledge Base', vscode.ViewColumn.Two, { enableScripts: true });
      panel.webview.html = getWebviewContent('Knowledge Base Items', stdout);
    } catch (e: any) {
      vscode.window.showErrorMessage(`Failed to load Knowledge Base: ${e.message}`);
    }
  });

  // 명령어 등록: 세션 시작
  let startSessionCmd = vscode.commands.registerCommand('contextflow.startSession', async () => {
    try {
      await execAsync(getCFCommand('start'), { cwd: vscode.workspace.rootPath });
      vscode.window.showInformationMessage('ContextFlow Rust Core Daemon Started');
      updateStatus();
    } catch (e: any) {
      vscode.window.showErrorMessage(`Failed to start session: ${e.message}`);
    }
  });

  context.subscriptions.push(menuCmd, summarizeCmd, fixCmd, planCmd, setGoalCmd, knowledgeCmd, startSessionCmd);
}

function getWebviewContent(title: string, content: string) {
  return `<!DOCTYPE html>
  <html lang="en">
  <head>
    <meta charset="UTF-8">
    <style>
      body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; padding: 20px; line-height: 1.6; color: var(--vscode-foreground); background-color: var(--vscode-editor-background); }
      .container { max-width: 800px; margin: 0 auto; }
      h1 { color: var(--vscode-textLink-foreground); border-bottom: 1px solid var(--vscode-widget-border); padding-bottom: 10px; }
      .card { background: var(--vscode-editor-inactiveSelectionBackground); padding: 20px; border-radius: 8px; border-left: 4px solid var(--vscode-textLink-foreground); margin-bottom: 20px; }
      pre { white-space: pre-wrap; font-size: 1.1em; }
      .footer { font-size: 0.8em; opacity: 0.7; margin-top: 40px; text-align: center; }
    </style>
  </head>
  <body>
    <div class="container">
      <h1>${title}</h1>
      <div class="card">
        <pre>${content}</pre>
      </div>
      <div class="footer">Generated by ContextFlow Engine</div>
    </div>
  </body>
  </html>`;
}

export function deactivate() {}
