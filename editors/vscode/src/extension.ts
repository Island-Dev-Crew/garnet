import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

type ServerCommand = {
  command: string;
  args?: string[];
  cwd?: string;
};

export function activate(context: vscode.ExtensionContext): void {
  const command = resolveServerCommand(context);
  const serverOptions: ServerOptions = {
    command: command.command,
    args: command.args ?? [],
    options: command.cwd ? { cwd: command.cwd } : undefined
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'garnet' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.garnet')
    }
  };

  client = new LanguageClient('garnet-lsp', 'Garnet LSP', serverOptions, clientOptions);
  context.subscriptions.push(client);
  context.subscriptions.push(
    vscode.commands.registerCommand('garnet.addCapsAnnotation', () =>
      applyNamedCodeAction(vscode.CodeActionKind.QuickFix, 'Add `@caps()`')
    ),
    vscode.commands.registerCommand('garnet.refactorLongParameterList', () =>
      applyNamedCodeAction(vscode.CodeActionKind.RefactorRewrite, 'Refactor long parameter list')
    ),
    vscode.commands.registerCommand('garnet.addReturnTypeAnnotation', () =>
      applyNamedCodeAction(vscode.CodeActionKind.QuickFix, 'Add return type')
    )
  );
  void client.start();
}

export function deactivate(): Thenable<void> | undefined {
  const running = client;
  client = undefined;
  return running?.stop();
}

function resolveServerCommand(context: vscode.ExtensionContext): ServerCommand {
  const configured = vscode.workspace
    .getConfiguration('garnet')
    .get<string>('lsp.path');
  if (configured && configured.trim().length > 0) {
    return { command: configured.trim() };
  }

  if (process.env.GARNET_LSP && process.env.GARNET_LSP.trim().length > 0) {
    return { command: process.env.GARNET_LSP.trim() };
  }

  const executable = executableName();
  const bundled = path.join(context.extensionPath, 'server', executable);
  if (fs.existsSync(bundled)) {
    return { command: bundled };
  }

  for (const folder of vscode.workspace.workspaceFolders ?? []) {
    const workspaceServer = path.join(folder.uri.fsPath, 'target', 'release', executable);
    if (fs.existsSync(workspaceServer)) {
      return { command: workspaceServer, cwd: folder.uri.fsPath };
    }
  }

  return { command: executable };
}

function executableName(): string {
  return process.platform === 'win32' ? 'garnet-lsp.exe' : 'garnet-lsp';
}

async function applyNamedCodeAction(
  kind: vscode.CodeActionKind,
  titleFragment: string
): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== 'garnet') {
    return;
  }

  const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>(
    'vscode.executeCodeActionProvider',
    editor.document.uri,
    editor.selection,
    kind.value
  );
  const action = actions?.find(candidate => candidate.title.includes(titleFragment));
  if (!action) {
    await vscode.commands.executeCommand('editor.action.codeAction', { kind: kind.value });
    return;
  }

  if (action.edit) {
    await vscode.workspace.applyEdit(action.edit);
  }
  if (action.command) {
    await vscode.commands.executeCommand(
      action.command.command,
      ...(action.command.arguments ?? [])
    );
  }
}
