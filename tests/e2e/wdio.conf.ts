import { join } from "path";
import { spawn } from "child_process";
import type { Options } from "@wdio/types";

// Path to the tauri-driver binary
const tauriDriver = join(
  __dirname,
  "../../node_modules",
  ".bin",
  "tauri-driver"
);

// Path to the compiled TRCAA binary
const getBinaryPath = () => {
  const envPath = process.env.TAURI_BINARY_PATH;
  if (envPath) return envPath;

  const platform = process.platform;
  if (platform === "win32") {
    return join(__dirname, "../../src-tauri/target/release/trcaa.exe");
  }
  return join(__dirname, "../../src-tauri/target/release/trcaa");
};

let driverProcess: ReturnType<typeof spawn> | null = null;

export const config: Options.Testrunner = {
  hostname: "localhost",
  port: 4444,
  path: "/",
  specs: ["./specs/**/*.spec.ts"],
  exclude: [],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      browserName: "",
      "tauri:options": {
        application: getBinaryPath(),
      },
      acceptInsecureCerts: true,
    },
  ],
  logLevel: "info",
  bail: 0,
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  services: [
    {
      onPrepare: async () => {
        // Start tauri-driver before tests
        driverProcess = spawn(tauriDriver, [], {
          stdio: [null, process.stdout, process.stderr],
          env: process.env,
        });
        // Wait for driver to be ready
        await new Promise((resolve) => setTimeout(resolve, 2000));
      },
      onComplete: async () => {
        if (driverProcess) {
          driverProcess.kill();
        }
      },
    },
  ],

  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },
};
