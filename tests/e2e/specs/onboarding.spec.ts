import { waitForApp, clickByText } from "../helpers/app";

describe("Onboarding Flow", () => {
  before(async () => {
    await waitForApp();
  });

  it("loads the dashboard on first launch", async () => {
    const title = await browser.getTitle();
    expect(title).toContain("TRCAA");
  });

  it("shows navigation sidebar", async () => {
    const sidebar = await browser.$("[data-testid='sidebar']");
    await sidebar.waitForDisplayed({ timeout: 5000 });
    expect(await sidebar.isDisplayed()).toBe(true);
  });

  it("can navigate to AI Providers settings", async () => {
    await clickByText("AI Providers");
    const heading = await browser.$("h1");
    await heading.waitForDisplayed({ timeout: 3000 });
    const text = await heading.getText();
    expect(text).toContain("AI Provider");
  });

  it("can navigate to New Issue page", async () => {
    await clickByText("New Issue");
    const domainGrid = await browser.$("[data-testid='domain-grid']");
    await domainGrid.waitForDisplayed({ timeout: 3000 });
    expect(await domainGrid.isDisplayed()).toBe(true);
  });
});
