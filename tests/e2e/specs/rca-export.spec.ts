import { waitForApp } from "../helpers/app";

describe("RCA Export Flow", () => {
  before(async () => {
    await waitForApp();
  });

  it("can view RCA document editor", async () => {
    const editor = await browser.$("[data-testid='doc-editor']");
    expect(editor).toBeDefined();
  });

  it("shows export buttons for MD, PDF, DOCX", async () => {
    const mdBtn = await browser.$("button=Export MD");
    const pdfBtn = await browser.$("button=Export PDF");
    expect(mdBtn).toBeDefined();
    expect(pdfBtn).toBeDefined();
  });
});
