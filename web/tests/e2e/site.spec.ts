import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("landing navigation, theme persistence, and clipboard", async ({
  page,
  context,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Law, made explicit." }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Copy code", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Copied", exact: true }),
  ).toBeVisible();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toContain(
    "priority emergency_first",
  );
  await page.getByRole("button", { name: "Toggle color theme" }).click();
  await expect(page.locator("html")).toHaveClass(/dark/);
  await page.reload();
  await expect(page.locator("html")).toHaveClass(/dark/);
  await page.getByRole("link", { name: "Get started", exact: true }).click();
  await expect(page).toHaveURL(/\/docs\/getting-started/);
  expect(errors).toEqual([]);
});

test("all reference pages and guide links render", async ({ page }) => {
  for (const slug of [
    "getting-started",
    "language",
    "outcomes",
    "time",
    "formats",
    "verification",
    "encoding",
    "coverage",
    "deadlines",
    "deadline-review",
    "source-workflow",
    "wawd-answer",
    "constitutional-authority",
  ]) {
    const response = await page.goto(`/docs/${slug}`);
    expect(response?.status()).toBe(200);
    await expect(page.locator("main h1")).toBeVisible();
    const hrefs = await page
      .locator("article a")
      .evaluateAll((links) =>
        links
          .map((l) => l.getAttribute("href"))
          .filter((h): h is string => !!h && !h.startsWith("https://")),
      );
    for (const href of hrefs) {
      expect(href, `${slug}: relative links must be mapped`).toMatch(/^\/|^#/);
    }
  }
  await page.goto("/docs/language");
  await page
    .getByRole("textbox", { name: "Find a documentation topic" })
    .fill("calendar");
  await expect(
    page.getByRole("navigation", { name: "Documentation" }).getByRole("link"),
  ).toHaveCount(1);
  await page
    .getByRole("link", { name: "Dates & legal change", exact: true })
    .click();
  await expect(page.locator("main h1")).toHaveText("Dates & legal change");
});

test("explorer search, filters, empty state and URL restoration", async ({
  page,
}) => {
  await page.goto("/explorer");
  await page.getByRole("textbox", { name: "Search lawlib" }).fill("basis");
  await expect(
    page.getByRole("main").getByRole("link", { name: /IRA · tax year 2025/ }),
  ).toBeVisible();
  await expect(
    page.getByRole("main").getByRole("link", { name: /Household policy/ }),
  ).toHaveCount(0);
  await page.reload();
  await expect(
    page.getByRole("textbox", { name: "Search lawlib" }),
  ).toHaveValue("basis");
  await page.getByRole("button", { name: "Private", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "No packages found" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Clear all filters" }).click();
  await expect(
    page.getByRole("main").getByRole("link", { name: /Household policy/ }),
  ).toBeVisible();
});

test("source declarations, deep links, citations and authentic fixture amounts", async ({
  page,
}) => {
  await page.goto("/explorer/us/federal/tax/ira/2025");
  await page.getByRole("button", { name: /^taxable_conversion / }).click();
  await expect(page.getByRole("tab", { name: /^Source / })).toHaveAttribute(
    "aria-selected",
    "true",
  );
  await expect(page.locator('[data-highlight="true"]')).toContainText(
    "query taxable_conversion",
  );
  const deepLink = page.url();
  await page.reload();
  await expect(page).toHaveURL(deepLink);
  await page.getByRole("tab", { name: /Citations/ }).click();
  await expect(
    page.getByRole("link", { name: "Open original source" }),
  ).toHaveCount(6);
  await expect(page.getByText("Bytes matched", { exact: true })).toHaveCount(6);
  await page.getByRole("tab", { name: /Scenarios/ }).click();
  await page
    .getByRole("button", {
      name: "7000 basis 93000 remaining balance resolved",
    })
    .click();
  await expect(page.getByText("USD 6510", { exact: true })).toBeVisible();
  await page
    .getByRole("button", { name: "7000 basis no remaining balance resolved" })
    .click();
  await expect(page.getByText("USD 0", { exact: true })).toBeVisible();
  await page.getByRole("textbox", { name: "Search scenarios" }).fill("missing");
  await expect(
    page.getByRole("group", { name: "Scenarios" }).getByRole("button"),
  ).toHaveCount(1);
});

test("desktop accessibility and screenshots", async ({ page }, testInfo) => {
  for (const [name, url] of [
    ["landing", "/"],
    ["guide", "/docs/getting-started"],
    ["explorer", "/explorer"],
    ["inventory", "/explorer/inventory"],
    ["package", "/explorer/us/federal/constitution"],
    [
      "scenarios",
      "/explorer/us/federal/tax/ira/2025?tab=scenarios&scenario=7000_basis_93000_remaining_balance",
    ],
  ]) {
    await page.goto(url);
    await expect(page.locator("main h1")).toBeVisible();
    const result = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21aa"])
      .analyze();
    expect(
      result.violations,
      `${name}: ${JSON.stringify(result.violations.map((v) => ({ id: v.id, nodes: v.nodes.map((n) => n.target) })))}`,
    ).toEqual([]);
    await page.screenshot({
      path: testInfo.outputPath(`${name}.png`),
      fullPage: true,
    });
  }
});

test("mobile navigation and layouts remain within viewport", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const [name, url] of [
    ["landing", "/"],
    ["docs", "/docs/language"],
    ["explorer", "/explorer"],
    ["inventory", "/explorer/inventory"],
    ["source", "/explorer/private/household/demo?tab=source"],
    ["scenarios", "/explorer/us/federal/tax/ira/2025?tab=scenarios"],
  ]) {
    await page.goto(url);
    await expect(page.locator("main h1")).toBeVisible();
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth + 1,
      ),
      name,
    ).toBe(true);
    await page.screenshot({
      path: testInfo.outputPath(`mobile-${name}.png`),
      fullPage: true,
    });
  }
  await page.getByRole("button", { name: "Open navigation" }).click();
  await page
    .getByRole("navigation", { name: "Mobile navigation" })
    .getByRole("link", { name: "Documentation" })
    .click();
  await expect(page).toHaveURL(/\/docs\/language/);
  await page.getByRole("button", { name: "Documentation menu" }).click();
  await page
    .getByRole("navigation", { name: "Documentation" })
    .getByRole("link", { name: "Packages & CLI" })
    .click();
  await expect(page.locator("main h1")).toHaveText("Packages & CLI");
});

test("source downloads and missing routes have correct responses", async ({
  request,
}) => {
  for (const name of ["oathkeeper", "lawlib"]) {
    const r = await request.get(`/downloads/${name}-v0.1.0-dev.1.zip`);
    expect(r.ok()).toBe(true);
    expect((await r.body()).subarray(0, 2).toString()).toBe("PK");
  }
  expect((await request.get("/downloads/lawlib-v0.1.0-dev.2.zip")).ok()).toBe(
    true,
  );
  expect((await request.get("/downloads/lawlib-v0.1.0-dev.3.zip")).ok()).toBe(
    true,
  );
  expect((await request.get("/downloads/lawlib-v0.1.0-dev.4.zip")).ok()).toBe(
    true,
  );
  const source = await request.get(
    "/lawlib/private/household/demo/src/quiet_hours.oath",
  );
  expect(source.ok()).toBe(true);
  expect(await source.text()).toContain("query noise_status");
  expect((await request.get("/docs/not-a-page")).status()).toBe(404);
  expect((await request.get("/explorer/unknown")).status()).toBe(404);
});
