import { describe, it, expect } from "vitest";
import { getDomainPrompt, DOMAINS, INCIDENT_RESPONSE_FRAMEWORK } from "@/lib/domainPrompts";

describe("Domain Prompts with Incident Response Framework", () => {
  it("exports INCIDENT_RESPONSE_FRAMEWORK constant", () => {
    expect(INCIDENT_RESPONSE_FRAMEWORK).toBeDefined();
    expect(typeof INCIDENT_RESPONSE_FRAMEWORK).toBe("string");
    expect(INCIDENT_RESPONSE_FRAMEWORK.length).toBeGreaterThan(100);
  });

  it("framework contains all 5 phases", () => {
    expect(INCIDENT_RESPONSE_FRAMEWORK).toContain("Phase 1: Detection & Evidence Gathering");
    expect(INCIDENT_RESPONSE_FRAMEWORK).toContain("Phase 2: Diagnosis & Hypothesis Testing");
    expect(INCIDENT_RESPONSE_FRAMEWORK).toContain("Phase 3: Root Cause Analysis with 5-Whys");
    expect(INCIDENT_RESPONSE_FRAMEWORK).toContain("Phase 4: Resolution & Prevention");
    expect(INCIDENT_RESPONSE_FRAMEWORK).toContain("Phase 5: Post-Incident Review");
  });

  it("framework contains the 3-Fix Rule", () => {
    expect(INCIDENT_RESPONSE_FRAMEWORK).toContain("3-Fix Rule");
  });

  it("framework contains communication practices", () => {
    expect(INCIDENT_RESPONSE_FRAMEWORK).toContain("Communication Practices");
  });

  it("all defined domains include incident response methodology", () => {
    for (const domain of DOMAINS) {
      const prompt = getDomainPrompt(domain.id);
      if (prompt) {
        expect(prompt).toContain("INCIDENT RESPONSE METHODOLOGY");
        expect(prompt).toContain("Phase 1:");
        expect(prompt).toContain("Phase 5:");
      }
    }
  });

  it("returns empty string for unknown domain", () => {
    expect(getDomainPrompt("nonexistent_domain")).toBe("");
    expect(getDomainPrompt("")).toBe("");
  });

  it("preserves existing Linux domain content", () => {
    const prompt = getDomainPrompt("linux");
    expect(prompt).toContain("senior Linux systems engineer");
    expect(prompt).toContain("RHEL");
    expect(prompt).toContain("INCIDENT RESPONSE METHODOLOGY");
  });

  it("preserves existing Kubernetes domain content", () => {
    const prompt = getDomainPrompt("kubernetes");
    expect(prompt).toContain("Kubernetes platform engineer");
    expect(prompt).toContain("k3s");
    expect(prompt).toContain("INCIDENT RESPONSE METHODOLOGY");
  });

  it("preserves existing Network domain content", () => {
    const prompt = getDomainPrompt("network");
    expect(prompt).toContain("network engineer");
    expect(prompt).toContain("Fortigate");
    expect(prompt).toContain("INCIDENT RESPONSE METHODOLOGY");
  });
});
