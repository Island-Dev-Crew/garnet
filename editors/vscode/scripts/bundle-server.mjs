import { chmodSync, copyFileSync, existsSync, mkdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const extensionRoot = join(scriptDir, '..');
const repoRoot = join(extensionRoot, '..', '..');
const executableName = process.platform === 'win32' ? 'garnet-lsp.exe' : 'garnet-lsp';
const source = process.env.GARNET_LSP_BINARY || join(repoRoot, 'target', 'release', executableName);
const serverDir = join(extensionRoot, 'server');
const destination = join(serverDir, executableName);

if (!existsSync(source)) {
  console.error(
    `Missing ${source}. Run "cargo build -p garnet-lsp --release" before packaging the VS Code extension.`
  );
  process.exit(1);
}

const sourceStat = statSync(source);
if (!sourceStat.isFile()) {
  console.error(`Expected ${source} to be a file.`);
  process.exit(1);
}

mkdirSync(serverDir, { recursive: true });
copyFileSync(source, destination);
chmodSync(destination, 0o755);

console.log(`Bundled ${source} -> ${destination}`);
