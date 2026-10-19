import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("source inventory exposes pins and keeps formalization separate", async ({
  page,
  request,
}, testInfo) => {
  await page.goto("/explorer");
  await page
    .getByRole("link", { name: "Browse the U.S. Code inventory" })
    .click();
  await expect(page.locator("main h1")).toHaveText(
    "U.S. Code source inventory",
  );
  await expect(
    page.getByText("58 of 58 documents", { exact: true }),
  ).toBeVisible();
  await page
    .getByLabel("Search titles, filenames, or identifiers")
    .fill("usc26.xml");
  await expect(
    page.getByText("1 of 58 documents", { exact: true }),
  ).toBeVisible();
  const row = page.locator("tbody tr");
  await expect(row).toHaveCount(1);
  await expect(row).toContainText("INTERNAL REVENUE CODE");
  await expect(row.getByText("Not assessed", { exact: true })).toHaveCount(2);
  await expect(row.getByText("Unreviewed", { exact: true })).toBeVisible();
  await row.getByText("Identifiers and hashes").click();
  await expect(row.getByText("/us/usc/t26", { exact: true })).toBeVisible();
  const download = await request.get("/inventory/usc-119-121.json");
  expect(download.ok()).toBe(true);
  expect((await download.json()).source.sha256).toBe(
    "c6122974e951003437ece5a998aa7c0b2cdc0f5703dfc78d6135e1fc9e729142",
  );
  await page.screenshot({
    path: testInfo.outputPath("inventory-filtered.png"),
    fullPage: true,
  });
  await page.getByRole("button", { name: "Toggle color theme" }).click();
  expect(
    (
      await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze()
    ).violations,
  ).toEqual([]);
  await page
    .getByLabel("Search titles, filenames, or identifiers")
    .fill("not-a-title");
  await expect(page.getByText("No documents match this search.")).toBeVisible();
  await page
    .getByRole("link", { name: "Rule 6(a) deadline package", exact: true })
    .click();
  await expect(page.locator("main h1")).toHaveText("Federal civil deadlines");
  await page.getByRole("tab", { name: /Scenarios/ }).click();
  await page
    .getByRole("textbox", { name: "Search scenarios" })
    .fill("missing_holiday");
  await expect(
    page.getByRole("group", { name: "Scenarios" }).getByRole("button"),
  ).toHaveCount(1);
});
