import net from "node:net";
import { spawn } from "node:child_process";
import process from "node:process";
import { syncPdfjsAssets } from "./sync-pdfjs-assets.mjs";

syncPdfjsAssets();

const isWin = process.platform === "win32";
const children = [];
let shuttingDown = false;

function startChild(name, command, args) {
  const child = spawn(command, args, {
    stdio: "inherit",
    shell: isWin,
  });
  children.push(child);
  child.on("exit", (code) => {
    if (!shuttingDown) {
      shuttingDown = true;
      process.stderr.write(`[${name}] exited with code ${code ?? 0}\n`);
      stopAll(code ?? 0);
    }
  });
  return child;
}

function stopAll(exitCode = 0) {
  for (const child of children) {
    if (!child.pid || child.killed) continue;
    if (isWin) {
      spawn("taskkill", ["/pid", String(child.pid), "/T", "/F"], {
        stdio: "ignore",
      });
    } else {
      child.kill("SIGTERM");
    }
  }
  process.exit(exitCode);
}

process.on("SIGINT", () => {
  if (!shuttingDown) {
    shuttingDown = true;
    stopAll(0);
  }
});

process.on("SIGTERM", () => {
  if (!shuttingDown) {
    shuttingDown = true;
    stopAll(0);
  }
});

function canConnect(host, port) {
  return new Promise((resolve) => {
    const socket = net.createConnection({ host, port });
    socket.once("connect", () => {
      socket.destroy();
      resolve(true);
    });
    socket.once("error", () => {
      socket.destroy();
      resolve(false);
    });
  });
}

async function waitForPort(host, port) {
  while (!shuttingDown) {
    if (await canConnect(host, port)) {
      return true;
    }
    await new Promise((r) => globalThis.setTimeout(r, 400));
  }
  return false;
}

process.stdout.write(
  "[dev:web] Building & starting Rust API on http://127.0.0.1:3000 (waiting before opening browser)...\n"
);

startChild("dev:api", "cargo", [
  "run",
  "--manifest-path",
  "src-tauri/Cargo.toml",
  "--bin",
  "web_server",
]);

waitForPort("127.0.0.1", 3000).then((ready) => {
  if (!ready || shuttingDown) return;
  process.stdout.write(
    "[dev:web] API ready! Starting Vite Frontend (http://localhost:5173/AdaBarprint)...\n"
  );
  startChild("dev:ui", "npx", ["vite", "--open", "/AdaBarprint"]);
});
