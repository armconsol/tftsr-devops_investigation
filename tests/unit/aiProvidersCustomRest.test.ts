import { describe, it, expect } from "vitest";
import {
  CUSTOM_MODEL_OPTION,
  CUSTOM_REST_FORMAT,
  CUSTOM_REST_MODELS,
} from "@/pages/Settings/AIProviders";

describe("AIProviders Custom REST helpers", () => {
  it("custom_rest format constant has the correct value", () => {
    expect(CUSTOM_REST_FORMAT).toBe("custom_rest");
  });

  it("keeps openai api_format unchanged", () => {
    expect("openai").toBe("openai");
  });

  it("contains the guide model list and custom model option sentinel", () => {
    expect(CUSTOM_REST_MODELS).toContain("ChatGPT4o");
    expect(CUSTOM_REST_MODELS).toContain("VertexGemini");
    expect(CUSTOM_REST_MODELS).toContain("Gemini-3_Pro-Preview");
    expect(CUSTOM_MODEL_OPTION).toBe("__custom_model__");
  });
});
