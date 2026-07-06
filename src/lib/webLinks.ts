import type { WebLink, WebLinkInput } from "./types";

export interface WebLinkFormModel {
  id?: string | null;
  label: string;
  url: string;
}

export type WebLinkFormErrors = Partial<Record<"label" | "url", string>>;

export const newWebLinkDraft = (link?: WebLink | null): WebLinkFormModel => ({
  id: link?.id ?? null,
  label: link?.label ?? "",
  url: link?.url ?? "",
});

export function toWebLinkInput(form: WebLinkFormModel): WebLinkInput {
  return {
    label: form.label.trim(),
    url: form.url.trim(),
  };
}

export function validateWebLinkForm(form: WebLinkFormModel): WebLinkFormErrors {
  const errors: WebLinkFormErrors = {};

  if (!form.label.trim()) {
    errors.label = "Label is required.";
  }

  const url = form.url.trim();
  if (!url) {
    errors.url = "URL is required.";
  } else {
    try {
      const parsed = new URL(url);
      if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
        errors.url = "URL must start with http:// or https://.";
      } else if (!parsed.hostname) {
        errors.url = "URL must include a host.";
      } else if (parsed.username || parsed.password) {
        errors.url = "URL must not include embedded credentials.";
      }
    } catch {
      errors.url = "URL must be valid.";
    }
  }

  return errors;
}

export function hasWebLinkFormErrors(errors: WebLinkFormErrors) {
  return Object.values(errors).some(Boolean);
}
