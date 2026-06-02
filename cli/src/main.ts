#!/usr/bin/env node
/**
 * TRCAA CLI - Command-line interface for TRCAA IT Triage & RCA
 *
 * Note: The CLI provides basic operations. For full functionality,
 * use the TRCAA desktop GUI application.
 */

const args = process.argv.slice(2);
const command = args[0];

function printHelp() {
  console.log(`
TRCAA CLI v0.1.0 — IT Triage & RCA Tool

Usage: trcaa-cli <command> [options]

Commands:
  analyze <log-file>          Analyze a log file for issues
    --domain, -d <domain>     IT domain (linux, windows, network, k8s, db, virt, hw, obs)
    --provider, -p <name>     AI provider to use

  export <issue-id> <format>  Export an issue document
    format: md, pdf, docx

  config set <key> <value>    Set a configuration value
  config get <key>            Get a configuration value
  config list                 List all configuration

  version                     Show version information
  help                        Show this help message

Examples:
  trcaa-cli analyze /var/log/syslog --domain linux
  trcaa-cli export abc-123 pdf
  trcaa-cli config set active_provider ollama

Note: For full AI-powered triage, launch the TRCAA desktop application.
`);
}

function printVersion() {
  console.log("TRCAA CLI v0.1.0");
  console.log("Part of the TRCAA IT Triage & RCA Desktop Application");
}

switch (command) {
  case "analyze": {
    const logFile = args[1];
    if (!logFile) {
      console.error("Error: log file path required");
      console.error("Usage: trcaa-cli analyze <log-file>");
      process.exit(1);
    }
    const domainIdx = args.findIndex((a) => a === "--domain" || a === "-d");
    const domain = domainIdx >= 0 ? args[domainIdx + 1] : "linux";
    console.log(`Analyzing: ${logFile}`);
    console.log(`Domain: ${domain}`);
    console.log("\nFor AI-powered analysis, launch the TRCAA desktop application.");
    console.log("The GUI provides: PII detection, 5-whys triage, RCA generation.");
    break;
  }

  case "export": {
    const issueId = args[1];
    const format = args[2];
    if (!issueId || !format) {
      console.error("Usage: trcaa-cli export <issue-id> <format>");
      process.exit(1);
    }
    if (!["md", "pdf", "docx"].includes(format)) {
      console.error("Error: format must be one of: md, pdf, docx");
      process.exit(1);
    }
    console.log(`Export issue ${issueId} as ${format.toUpperCase()}`);
    console.log("Launch the TRCAA app to access the export functionality.");
    break;
  }

  case "config": {
    const subcommand = args[1];
    switch (subcommand) {
      case "set":
        console.log(`Configuration: ${args[2]} = ${args[3]}`);
        console.log("Note: Configuration is managed by the TRCAA desktop application.");
        break;
      case "get":
        console.log(`Getting config key: ${args[2]}`);
        break;
      case "list":
        console.log("Configuration is stored in the TRCAA app data directory.");
        console.log("Launch the app and go to Settings to view/edit configuration.");
        break;
      default:
        console.error(`Unknown config subcommand: ${subcommand}`);
    }
    break;
  }

  case "version":
    printVersion();
    break;

  case "help":
  case "--help":
  case "-h":
  case undefined:
    printHelp();
    break;

  default:
    console.error(`Unknown command: ${command}`);
    printHelp();
    process.exit(1);
}
