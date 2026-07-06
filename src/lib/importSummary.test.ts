import { describe, expect, it } from "vitest";
import { formatImportPreviewSummary, formatImportResult, summarizeImportPreview } from "./importSummary";
import type { ImportCandidate } from "./types";

function candidate(overrides: Partial<ImportCandidate>): ImportCandidate {
  return {
    alias: "nas",
    name: "nas",
    host: "nas.local",
    port: 22,
    username: "admin",
    identityFile: null,
    proxyJump: null,
    warnings: [],
    selected: true,
    duplicate: false,
    skipped: false,
    ...overrides,
  };
}

describe("import summary helpers", () => {
  it("summarizes preview candidates", () => {
    const summary = summarizeImportPreview([
      candidate({ alias: "nas" }),
      candidate({ alias: "router", duplicate: true, selected: false, warnings: ["Duplicate"] }),
      candidate({ alias: "wildcard", skipped: true, selected: false, warnings: ["Skipped"] }),
    ]);

    expect(summary).toEqual({
      total: 3,
      ready: 1,
      duplicates: 1,
      skipped: 1,
      warnings: 2,
    });
    expect(formatImportPreviewSummary([
      candidate({ alias: "nas" }),
      candidate({ alias: "router", duplicate: true, selected: false, warnings: ["Duplicate"] }),
    ])).toBe("1 ready to import · 1 duplicate · 1 with warning");
  });

  it("formats apply results", () => {
    expect(formatImportResult({ imported: 2, skipped: 0, servers: [] })).toBe("Imported 2 servers from ~/.ssh/config.");
    expect(formatImportResult({ imported: 1, skipped: 1, servers: [] })).toBe(
      "Imported 1 server. Skipped 1 server that could not be imported.",
    );
    expect(formatImportResult({ imported: 0, skipped: 2, servers: [] })).toBe("No servers imported. Skipped 2 servers.");
  });
});
