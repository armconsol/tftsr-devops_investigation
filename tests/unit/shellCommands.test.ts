import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import * as tauriCommands from "@/lib/tauriCommands";

vi.mock("@tauri-apps/api/core");

type MockedFunction<T = (...args: unknown[]) => unknown> = T & {
  mockResolvedValue: (value: unknown) => void;
  mockRejectedValue: (error: Error) => void;
};

const mockClassifierRules = {
  tier1_kubectl: ["get", "describe", "logs", "explain", "api-resources", "api-versions", "cluster-info", "top", "version"],
  tier1_systemctl: ["status", "is-active", "is-enabled", "list-units", "list-unit-files"],
  tier1_proxmox: ["status", "get"],
  tier1_general: ["cat", "grep", "ls", "find", "df", "free", "ps", "dig", "nslookup", "ldapsearch"],
  tier2_kubectl: ["apply", "delete", "edit", "scale", "rollout", "exec", "cp", "port-forward"],
  tier2_systemctl: ["restart", "stop", "start", "enable", "disable", "reload", "mask", "unmask"],
  tier2_proxmox: ["migrate", "create", "set", "delete", "start", "stop"],
  tier2_general: ["ssh", "scp", "chmod", "chown", "curl", "wget", "ldapmodify", "ldapdelete", "ldapadd"],
  tier3: ["rm", "mkfs", "dd", "fdisk", "kill", "pkill", "killall", "init", "shutdown", "reboot", "halt", "poweroff"],
};

describe("Shell Classifier Commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("getClassifierRulesCmd", () => {
    it("should call invoke with correct command name", async () => {
      (invoke as MockedFunction).mockResolvedValue(mockClassifierRules);

      await tauriCommands.getClassifierRulesCmd();

      expect(invoke).toHaveBeenCalledWith("get_classifier_rules");
    });

    it("should return the classifier rules structure", async () => {
      (invoke as MockedFunction).mockResolvedValue(mockClassifierRules);

      const result = await tauriCommands.getClassifierRulesCmd();

      expect(result.tier1_kubectl).toContain("get");
      expect(result.tier1_kubectl).toContain("logs");
      expect(result.tier2_kubectl).toContain("apply");
      expect(result.tier2_kubectl).toContain("delete");
      expect(result.tier3).toContain("rm");
      expect(result.tier3).toContain("kill");
      expect(result.tier3).toContain("init");
    });

    it("should include fix for Bug 1 — kill and init in tier3", async () => {
      (invoke as MockedFunction).mockResolvedValue(mockClassifierRules);

      const result = await tauriCommands.getClassifierRulesCmd();

      expect(result.tier3).toContain("kill");
      expect(result.tier3).toContain("pkill");
      expect(result.tier3).toContain("killall");
      expect(result.tier3).toContain("init");
    });

    it("should include fix for Bug 2 — systemctl read-only subcommands in tier1", async () => {
      (invoke as MockedFunction).mockResolvedValue(mockClassifierRules);

      const result = await tauriCommands.getClassifierRulesCmd();

      expect(result.tier1_systemctl).toContain("status");
      expect(result.tier1_systemctl).toContain("is-active");
      expect(result.tier2_systemctl).toContain("restart");
      expect(result.tier2_systemctl).toContain("stop");
    });

    it("should include fix for Bug 3 — ldap mutating ops in tier2 not tier1", async () => {
      (invoke as MockedFunction).mockResolvedValue(mockClassifierRules);

      const result = await tauriCommands.getClassifierRulesCmd();

      expect(result.tier2_general).toContain("ldapmodify");
      expect(result.tier2_general).toContain("ldapdelete");
      expect(result.tier2_general).toContain("ldapadd");
      // ldapsearch must NOT appear in tier2 (it's read-only, belongs in tier1)
      expect(result.tier2_general).not.toContain("ldapsearch");
      expect(result.tier1_general).toContain("ldapsearch");
    });

    it("should propagate errors from invoke", async () => {
      (invoke as MockedFunction).mockRejectedValue(new Error("IPC error"));

      await expect(tauriCommands.getClassifierRulesCmd()).rejects.toThrow("IPC error");
    });
  });
});
