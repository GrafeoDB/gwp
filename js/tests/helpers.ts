/** Test helpers for integration tests. */

import { execSync, spawn, type ChildProcess } from "child_process";
import { existsSync } from "fs";
import { resolve } from "path";
import net from "net";

const REPO_ROOT = resolve(__dirname, "../..");

function findFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.listen(0, "127.0.0.1", () => {
      const addr = server.address();
      if (addr && typeof addr === "object") {
        const port = addr.port;
        server.close(() => resolve(port));
      } else {
        reject(new Error("could not get port"));
      }
    });
  });
}

function waitForPort(port: number, timeout = 10_000): Promise<void> {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + timeout;
    const tryConnect = () => {
      if (Date.now() > deadline) {
        reject(new Error(`server did not start on port ${port}`));
        return;
      }
      const sock = net.createConnection(port, "127.0.0.1");
      sock.on("connect", () => {
        sock.destroy();
        resolve();
      });
      sock.on("error", () => {
        setTimeout(tryConnect, 100);
      });
    };
    tryConnect();
  });
}

let serverProcess: ChildProcess | null = null;
let serverEndpoint: string | null = null;

export async function getTestServer(): Promise<string> {
  if (serverEndpoint) return serverEndpoint;

  const binary = resolve(
    REPO_ROOT,
    process.platform === "win32"
      ? "target/release/gwp-test-server.exe"
      : "target/release/gwp-test-server",
  );

  if (!existsSync(binary)) {
    throw new Error(`gwp-test-server not found at ${binary}`);
  }

  const port = await findFreePort();
  serverProcess = spawn(binary, [String(port)], {
    stdio: "pipe",
  });

  await waitForPort(port);
  serverEndpoint = `localhost:${port}`;
  return serverEndpoint;
}

export function stopTestServer(): void {
  if (serverProcess) {
    serverProcess.kill();
    serverProcess = null;
    serverEndpoint = null;
  }
}
