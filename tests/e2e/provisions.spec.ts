import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { readFile } from "node:fs/promises";
const rule6 =
  "/explorer/provisions?release=119-121&file=usc28a.xml&key=1515f8cf1e5b19e1b5e644cf";
test("verified provision text, notes, encoding link and draft download", async ({
  page,
}, info) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto(rule6);
  const article = page.getByRole("article", { name: "Selected provision" });
  await expect(
    article.getByRole("heading", { name: /Computing and Extending Time/ }),
  ).toBeVisible();
  await expect(article).toContainText("exclude the day of the event");
  await expect(
    article.getByRole("link", { name: "frcp.6a.forward_days", exact: true }),
  ).toBeVisible();
  await article.getByText(/^Notes \(/).click();
  await expect(article).toContainText("Committee Notes");
  await article.getByText(/^Notes \(/).click();
  await page
    .getByLabel("Question to formalize")
    .fill("When does this deadline expire?");
  await page
    .getByRole("button", { name: "Preview draft", exact: true })
    .click();
  await expect(
    page.getByText("Unimplemented · Legal review: Unreviewed", { exact: true }),
  ).toBeVisible();
  const downloaded = page.waitForEvent("download");
  await page.getByRole("button", { name: "Download draft bundle" }).click();
  const download = await downloaded;
  const path = info.outputPath("oath-draft.json");
  await download.saveAs(path);
  const bundle = JSON.parse(await readFile(path, "utf8"));
  expect(bundle.format).toBe("lawlib-draft-bundle/1");
  expect(bundle.files["src/main.oath"]).not.toMatch(/^rule /m);
  expect(
    JSON.parse(bundle.files["tests/unimplemented.json"]).expected.status,
  ).toBe("undetermined");
  expect(bundle.files["src/main.oath"]).toContain(
    "/us/usc/t28a/courtRules/Civil/rule6",
  );
  expect(
    (
      await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
        .analyze()
    ).violations,
  ).toEqual([]);
  await page.screenshot({
    path: info.outputPath("provision-draft.png"),
    fullPage: true,
  });
  expect(errors).toEqual([]);
});
test("search, references, historical changes and source integrity rejection", async ({
  page,
}) => {
  await page.goto("/explorer/changes");
  await expect(
    page.getByRole("heading", { name: "Source changes", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "draft.us.commerce.section9801",
      exact: true,
    }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Earlier passage" }).first().click();
  await expect(
    page.getByRole("article", { name: "Selected provision" }),
  ).toBeVisible();
  await expect(page.getByLabel("Pinned release")).toHaveValue("119-118");
  await page.goto(
    "/explorer/provisions?file=usc28a.xml&ref=" +
      encodeURIComponent("/us/usc/t28a/courtRules/Civil/rule6/a"),
  );
  const rows = page.getByRole("button", { name: /Rule 6\./ });
  await expect(rows).toHaveCount(2);
  await page
    .getByLabel("Find by number, heading, or locator")
    .fill("Computing");
  await expect(rows).toHaveCount(1);
  await rows.click();
  await expect(
    page.getByRole("article", { name: "Selected provision" }),
  ).toBeVisible();
  await page.route("**/provisions/119-121/*.index.json.gz", (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/gzip",
      body: Buffer.from("tampered"),
    }),
  );
  await page.goto(rule6);
  await expect(page.locator('main [role="alert"]')).toContainText(
    "integrity check failed",
  );
  await expect(
    page.getByRole("article", { name: "Selected provision" }),
  ).toHaveCount(0);
});
test("provision explorer fits mobile and handles missing data", async ({
  page,
}, info) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(rule6);
  await expect(
    page.getByRole("article", { name: "Selected provision" }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: info.outputPath("provision-mobile.png"),
    fullPage: true,
  });
  await page.goto("/explorer/provisions?release=unknown");
  await expect(page.locator('main [role="alert"]')).toContainText(
    "Unknown pinned release",
  );
});
