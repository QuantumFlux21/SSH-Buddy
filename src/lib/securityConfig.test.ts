import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

type StringMap = Record<string, string>;

const repositoryFile = (path: string) => resolve(process.cwd(), path);

async function readJson<T>(path: string): Promise<T> {
  return JSON.parse(await readFile(repositoryFile(path), "utf8")) as T;
}

describe("Tauri security configuration", () => {
  it("uses a local-only production CSP with only Tauri IPC connectivity", async () => {
    const config = await readJson<{
      app: { security: { csp: StringMap; devCsp: StringMap } };
    }>("src-tauri/tauri.conf.json");
    const { csp, devCsp } = config.app.security;

    expect(csp).toMatchObject({
      "default-src": "'self'",
      "connect-src": "ipc: http://ipc.localhost",
      "script-src": "'self'",
      "style-src": "'self' 'unsafe-inline'",
      "font-src": "'none'",
      "img-src": "'self'",
      "object-src": "'none'",
      "media-src": "'none'",
      "frame-src": "'none'",
      "worker-src": "'none'",
      "manifest-src": "'none'",
      "base-uri": "'none'",
      "form-action": "'none'",
      "frame-ancestors": "'none'",
    });
    expect(Object.values(csp).join(" ")).not.toContain("https:");
    expect(Object.values(csp).join(" ")).not.toContain("data:");
    expect(Object.values(csp).join(" ")).not.toContain("'unsafe-eval'");
    expect(devCsp["connect-src"]).toBe(
      "ipc: http://ipc.localhost ws://127.0.0.1:1420",
    );
  });

  it("exposes clipboard write only and keeps URL opening backend-owned", async () => {
    const capability = await readJson<{ permissions: string[] }>(
      "src-tauri/capabilities/default.json",
    );
    const commands = await readFile(repositoryFile("src-tauri/src/commands.rs"), "utf8");
    const main = await readFile(repositoryFile("src-tauri/src/main.rs"), "utf8");

    expect(capability.permissions).toEqual([
      "core:default",
      "clipboard-manager:allow-write-text",
    ]);
    expect(capability.permissions.some((permission) => permission.startsWith("opener:"))).toBe(false);
    expect(capability.permissions.some((permission) => permission.startsWith("shell:"))).toBe(false);
    expect(capability.permissions.some((permission) => permission.startsWith("fs:"))).toBe(false);
    expect(commands).toContain("validate_web_link_url(&link.url)?");
    expect(commands).toContain(".open_url(link.url, None::<&str>)");
    expect(main).toContain(".open_js_links_on_click(false)");
  });
});
