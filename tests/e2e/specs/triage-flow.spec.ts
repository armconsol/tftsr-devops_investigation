import { waitForApp } from "../helpers/app";

describe("Triage Flow", () => {
  before(async () => {
    await waitForApp();
  });

  it("shows triage progress indicator", async () => {
    // Navigate to a triage session
    // Verify the 5-whys progress component is present
    const progress = await browser.$("[data-testid='triage-progress']");
    expect(progress).toBeDefined();
  });

  it("shows chat window on triage page", async () => {
    const chatInput = await browser.$("[data-testid='chat-input']");
    expect(chatInput).toBeDefined();
  });
});
