export async function waitForApp(timeout = 10000) {
  await browser.waitUntil(
    async () => {
      try {
        const title = await browser.getTitle();
        return title.includes("TFTSR");
      } catch {
        return false;
      }
    },
    { timeout, timeoutMsg: "TFTSR app did not load within timeout" }
  );
}

export async function clickByText(text: string) {
  const el = await browser.$(`*=${text}`);
  await el.waitForDisplayed({ timeout: 5000 });
  await el.click();
}

export async function findByTestId(testId: string) {
  return browser.$(`[data-testid="${testId}"]`);
}

export async function typeInto(selector: string, text: string) {
  const el = await browser.$(selector);
  await el.click();
  await browser.keys(text.split(""));
}
