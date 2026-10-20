import { existsSync } from "node:fs";
import { resolve } from "node:path";

export const app = resolve(import.meta.dirname, "..");
const nested = existsSync(resolve(app, "../Cargo.toml"));
export const core = resolve(
  process.env.OATHKEEPER_PATH ?? resolve(app, nested ? ".." : "../oathkeeper"),
);
export const lawlib = resolve(
  process.env.LAWLIB_PATH ??
    resolve(app, nested ? "../../lawlib" : "../lawlib"),
);
