import type { ImportCandidate, ImportResult } from "./types";

export interface ImportPreviewSummary {
  total: number;
  ready: number;
  duplicates: number;
  skipped: number;
  warnings: number;
}

export function summarizeImportPreview(candidates: ImportCandidate[]): ImportPreviewSummary {
  return candidates.reduce(
    (summary, candidate) => ({
      total: summary.total + 1,
      ready: summary.ready + (candidate.selected && !candidate.duplicate && !candidate.skipped ? 1 : 0),
      duplicates: summary.duplicates + (candidate.duplicate ? 1 : 0),
      skipped: summary.skipped + (candidate.skipped ? 1 : 0),
      warnings: summary.warnings + (candidate.warnings.length > 0 ? 1 : 0),
    }),
    { total: 0, ready: 0, duplicates: 0, skipped: 0, warnings: 0 },
  );
}

export function formatImportPreviewSummary(candidates: ImportCandidate[]) {
  const summary = summarizeImportPreview(candidates);
  if (summary.total === 0) {
    return "No concrete Host aliases were found in ~/.ssh/config.";
  }

  const parts = [`${summary.ready} ready to import`];
  if (summary.duplicates > 0) {
    parts.push(`${summary.duplicates} duplicate${summary.duplicates === 1 ? "" : "s"}`);
  }
  if (summary.skipped > 0) {
    parts.push(`${summary.skipped} skipped`);
  }
  if (summary.warnings > 0) {
    parts.push(`${summary.warnings} with warning${summary.warnings === 1 ? "" : "s"}`);
  }

  return parts.join(" · ");
}

export function formatImportResult(result: ImportResult) {
  if (result.imported > 0 && result.skipped > 0) {
    return `Imported ${serverCount(result.imported)}. Skipped ${serverCount(result.skipped)} that could not be imported.`;
  }

  if (result.imported > 0) {
    return `Imported ${serverCount(result.imported)} from ~/.ssh/config.`;
  }

  if (result.skipped > 0) {
    return `No servers imported. Skipped ${serverCount(result.skipped)}.`;
  }

  return "No servers were imported.";
}

function serverCount(count: number) {
  return `${count} server${count === 1 ? "" : "s"}`;
}
