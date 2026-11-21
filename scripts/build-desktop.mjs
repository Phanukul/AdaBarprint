import { spawnSync } from 'node:child_process';
import { mkdirSync, copyFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
for (const [command, args] of [
  [process.platform === 'win32' ? 'npm.cmd' : 'npm', ['run', 'build']],
  ['cargo', ['build', '--manifest-path', 'src-tauri/Cargo.toml', '--release', '--bin', 'autoprint_agent']],
]) {
  // The agent must compile before it can be included as an installer resource.
  const env = command === 'cargo' ? { ...process.env, TAURI_CONFIG: JSON.stringify({ bundle: { resources: [] } }) } : process.env;
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit', shell: process.platform === 'win32', env });
  if (result.error || result.status !== 0) process.exit(result.status || 1);
}
const destination = path.join(root, 'src-tauri', 'bundle-agent');
mkdirSync(destination, { recursive: true });
copyFileSync(path.join(root, 'src-tauri/target/release/autoprint_agent.exe'), path.join(destination, 'autoprint_agent.exe'));
copyFileSync(path.join(root, 'scripts/Register-AutoPrintAgent.ps1'), path.join(destination, 'Register-AutoPrintAgent.ps1'));
