import { describe, expect, it } from "vitest";
import { newWebLinkDraft, toWebLinkInput, validateWebLinkForm } from "./webLinks";

describe("web link helpers", () => {
  it("validates required fields", () => {
    expect(validateWebLinkForm(newWebLinkDraft())).toEqual({
      label: "Label is required.",
      url: "URL is required.",
    });
  });

  it("allows only http and https URLs", () => {
    expect(validateWebLinkForm({ label: "Router", url: "https://router.local" })).toEqual({});
    expect(validateWebLinkForm({ label: "Router", url: "http://10.0.0.1/admin" })).toEqual({});
    expect(validateWebLinkForm({ label: "Bad", url: "file:///etc/passwd" }).url).toBe(
      "URL must start with http:// or https://.",
    );
    expect(validateWebLinkForm({ label: "Bad", url: "javascript:alert(1)" }).url).toBe(
      "URL must start with http:// or https://.",
    );
    expect(validateWebLinkForm({ label: "Bad", url: "https://user:secret@example.com" }).url).toBe(
      "URL must not include embedded credentials.",
    );
  });

  it("trims values before saving", () => {
    expect(toWebLinkInput({ label: "  NAS  ", url: "  https://nas.local  " })).toEqual({
      label: "NAS",
      url: "https://nas.local",
    });
  });
});
