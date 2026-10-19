import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
test("dark theme keeps documentation and explorer readable", async ({
  page,
}, testInfo) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Toggle color theme" }).click();
  await expect(page.locator("html")).toHaveClass(/dark/);
  for (const [name, url] of [
    ["landing", "/"],
    ["reference", "/docs/language"],
    ["explorer", "/explorer"],
    ["sources", "/explorer/us/federal/tax/ira/2025?tab=sources"],
    ["code", "/explorer/private/household/demo?tab=source"],
    ["scenarios", "/explorer/us/federal/tax/ira/2025?tab=scenarios"],
  ]) {
    await page.goto(url);
    const result = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
      .analyze();
    expect(
      result.violations.map((v) => ({
        id: v.id,
        targets: v.nodes.map((n) => n.target),
      })),
      name,
    ).toEqual([]);
    await page.screenshot({
      path: testInfo.outputPath(`dark-${name}.png`),
      fullPage: true,
    });
  }
});
