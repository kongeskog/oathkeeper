import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("constitutional provision links lead to real queries and authored scenarios", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/explorer/constitution?provision=us.const.article.1.s7.c2");
  const selected = page.getByRole("region", { name: "Selected provision" });
  await expect(
    selected.getByRole("heading", {
      name: "Bicameral passage and presentment",
    }),
  ).toBeVisible();
  await expect(selected.locator("blockquote")).toContainText(
    "Sundays excepted",
  );
  await selected
    .getByRole("link", {
      name: "us.constitution.authority.signed_bill_path_requirements",
      exact: true,
    })
    .click();
  await expect(page).toHaveURL(/tab=source.*authority.oath/);
  await expect(page.getByRole("main")).toContainText("signature_timing_valid");
  await page.goBack();
  await selected.getByRole("link", { name: /authored scenarios/ }).click();
  await expect(page).toHaveURL(/tab=scenarios/);
  await expect(page.getByRole("main")).toContainText("undetermined");
  expect(errors).toEqual([]);
});

test("history links, authority filters and shareable constitutional navigation", async ({
  page,
}) => {
  await page.goto("/explorer/constitution");
  await page
    .getByLabel("Article or amendment")
    .selectOption("us.const.amendment.18");
  await expect(
    page
      .getByRole("navigation", { name: "Constitutional provisions" })
      .getByRole("button"),
  ).toHaveCount(3);
  await expect(
    page
      .getByRole("region", { name: "Selected provision" })
      .locator("blockquote"),
  ).toContainText("After one year");
  await page
    .getByRole("button", { name: "Amendment XXI, section 1", exact: true })
    .click();
  await expect(
    page
      .getByRole("region", { name: "Selected provision" })
      .locator("blockquote"),
  ).toContainText("is hereby repealed");
  await page.reload();
  await expect(
    page.getByRole("heading", {
      name: "Repeal of Amendment XVIII",
      exact: true,
    }),
  ).toBeVisible();
  await page
    .getByRole("tab", { name: "Authority relationships", exact: true })
    .click();
  await page.getByLabel("Relationship kind").selectOption("Repeals");
  await expect(page.getByRole("main").locator("article")).toHaveCount(3);
  await page.getByLabel("Institutional actor").selectOption("us.congress");
  await expect(page.getByText("No matching relationships.")).toBeVisible();
  await page.getByLabel("Relationship kind").selectOption("Authorizes");
  await page
    .getByRole("button", {
      name: "Article I, section 8, clause 9",
      exact: true,
    })
    .click();
  await expect(
    page
      .getByRole("region", { name: "Selected provision" })
      .locator("blockquote"),
  ).toHaveText("To constitute Tribunals inferior to the supreme Court;");
  await page
    .getByRole("textbox", { name: "Search constitutional provisions" })
    .fill("no_such_constitutional_text");
  await expect(
    page.getByRole("heading", { name: "No provisions found" }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Clear filters", exact: true })
    .click();
  await page
    .getByLabel("Encoding status")
    .selectOption("InterpretationRequired");
  await page.reload();
  await expect(page.getByLabel("Encoding status")).toHaveValue(
    "InterpretationRequired",
  );
  await expect(
    page
      .getByRole("navigation", { name: "Constitutional provisions" })
      .getByRole("button"),
  ).toHaveCount(8);
});

test("constitutional explorer accessibility, mobile layout and unknown IDs", async ({
  page,
}) => {
  await page.goto("/explorer/constitution?provision=us.const.article.3.s1.c1");
  await expect(
    page.getByRole("heading", { name: "Judicial vesting and tenure" }),
  ).toBeVisible();
  expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
  await page.screenshot({
    path: "test-results/constitution-desktop.png",
    fullPage: true,
  });
  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.evaluate(() => {
    if (document.activeElement instanceof HTMLElement)
      document.activeElement.blur();
    window.scrollTo(0, 0);
  });
  await page.screenshot({
    path: "test-results/constitution-mobile.png",
    fullPage: true,
  });
  await page.goto("/explorer/constitution?provision=not.a.provision");
  await expect(
    page.getByRole("heading", { name: "Unknown provision" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Return to inventory" }).click();
  await expect(
    page.getByRole("region", { name: "Selected provision" }),
  ).toBeVisible();
});
