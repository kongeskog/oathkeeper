import current from "@/content/provisions-119-121.json";
import historical from "@/content/provisions-119-118.json";
export type ContentRef = { path: string; sha256: string; bytes: number };
export type SourceDocument = {
  file: string;
  title: string;
  sha256: string;
  bytes: number;
  count: number;
  index: ContentRef;
};
export type Catalog = {
  format: string;
  source: {
    release: string;
    url: string;
    sha256: string;
    [key: string]: unknown;
  };
  count: number;
  documents: SourceDocument[];
};
export type UnitRow = {
  key: string;
  locator: string | null;
  kind: string;
  number: string;
  heading: string;
  content_sha256: string;
  chunk: string;
  note_count: number;
  reference_count: number;
};
export type Provision = UnitRow & {
  element: number;
  body: string;
  hierarchy: {
    kind: string;
    locator: string;
    number: string;
    heading: string;
  }[];
  notes: {
    topic: string | null;
    role: string | null;
    group: string | null;
    text: string;
  }[];
  credits: string[];
  references: { href: string; text: string; context: string }[];
};
export type ProvisionIndex = {
  format: string;
  source: { file: string; sha256: string };
  units: UnitRow[];
  chunks: ContentRef[];
};
export const catalogs: Record<string, Catalog> = {
  "119-121": current,
  "119-118": historical,
};
export function provisionHref(release: string, file: string, key?: string) {
  return (
    "/explorer/provisions?" +
    new URLSearchParams({ release, file, ...(key ? { key } : {}) })
  );
}
async function boundedRead(stream: ReadableStream<Uint8Array>, max: number) {
  const reader = stream.getReader(),
    chunks: Uint8Array[] = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.length;
      if (size > max) throw Error("Source data exceeds the reader limit");
      chunks.push(value);
    }
  } catch (e) {
    await reader.cancel();
    throw e;
  } finally {
    reader.releaseLock();
  }
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const c of chunks) {
    bytes.set(c, offset);
    offset += c.length;
  }
  return bytes;
}
export async function checkedContent<T>(
  release: string,
  ref: ContentRef,
  signal?: AbortSignal,
): Promise<T> {
  if (
    !catalogs[release] ||
    !ref ||
    !/^[A-Za-z0-9.-]+\.json\.gz$/.test(ref.path)
  )
    throw Error("Invalid source reference");
  const response = await fetch(`/provisions/${release}/${ref.path}`, {
    signal,
  });
  if (!response.ok || !response.body)
    throw Error(
      "Source data is unavailable. Please retry or use the pinned archive.",
    );
  const bytes = await boundedRead(
    response.body,
    Math.min(ref.bytes, 64 * 1024 ** 2),
  );
  const digest = Array.from(
    new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)),
  )
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  if (bytes.length !== ref.bytes || digest !== ref.sha256)
    throw Error(
      "Source integrity check failed. This data will not be displayed.",
    );
  const stream = new Blob([bytes])
    .stream()
    .pipeThrough(new DecompressionStream("gzip"));
  return JSON.parse(
    new TextDecoder("utf-8", { fatal: true }).decode(
      await boundedRead(stream, 128 * 1024 ** 2),
    ),
  ) as T;
}
