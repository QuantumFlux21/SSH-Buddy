import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DesktopBehaviorSettings } from "./DesktopBehaviorSettings";
import type { AppSettings, DesktopBehaviorStatus } from "../lib/types";

const settings: AppSettings = {
  terminalPreference: "auto",
  safetyWarningsEnabled: true,
  startMinimized: false,
  closeToTray: false,
};

const available: DesktopBehaviorStatus = {
  autostartEnabled: false,
  autostartError: null,
  trayAvailable: true,
  trayError: null,
};

function renderSettings(overrides: Partial<Parameters<typeof DesktopBehaviorSettings>[0]> = {}) {
  const onSaveSettings = vi.fn(async () => undefined);
  const getDesktopBehaviorStatus = vi.fn(async () => available);
  const setAutostartEnabled = vi.fn(async (enabled: boolean) => ({ ...available, autostartEnabled: enabled }));

  render(
    <DesktopBehaviorSettings
      settings={settings}
      busy={false}
      onSaveSettings={onSaveSettings}
      getDesktopBehaviorStatus={getDesktopBehaviorStatus}
      setAutostartEnabled={setAutostartEnabled}
      {...overrides}
    />,
  );

  return { onSaveSettings, getDesktopBehaviorStatus, setAutostartEnabled };
}

describe("DesktopBehaviorSettings", () => {
  it("shows the persisted checkbox values and verified OS state", async () => {
    renderSettings({ settings: { ...settings, startMinimized: true, closeToTray: true } });

    expect(screen.getByLabelText("Start minimized")).toBeChecked();
    expect(screen.getByLabelText("Hide to system tray when closing")).toBeChecked();
    await waitFor(() => expect(screen.getByText("Login startup is disabled and verified.")).toBeInTheDocument());
    expect(screen.getByLabelText("Start SSH-Buddy when I sign in")).not.toBeChecked();
  });

  it("persists start minimized immediately, disables controls while busy, and explains fresh launches", async () => {
    let finishSave: (() => void) | undefined;
    const onSaveSettings = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          finishSave = resolve;
        }),
    );
    renderSettings({ onSaveSettings });
    await screen.findByText("Login startup is disabled and verified.");

    fireEvent.click(screen.getByLabelText("Start minimized"));
    expect(onSaveSettings).toHaveBeenCalledWith({ ...settings, startMinimized: true });
    expect(screen.getByLabelText("Hide to system tray when closing")).toBeDisabled();
    finishSave?.();

    await screen.findByText("Start-minimized preference saved for the next fresh process launch.");
    expect(screen.getByText(/next fresh process launch and creates a minimized taskbar window/i)).toBeInTheDocument();
  });

  it("refreshes the actual autostart state after every update", async () => {
    const getDesktopBehaviorStatus = vi
      .fn<() => Promise<DesktopBehaviorStatus>>()
      .mockResolvedValueOnce(available)
      .mockResolvedValueOnce({ ...available, autostartEnabled: true });
    const setAutostartEnabled = vi.fn(async () => ({ ...available, autostartEnabled: true }));
    renderSettings({ getDesktopBehaviorStatus, setAutostartEnabled });
    await screen.findByText("Login startup is disabled and verified.");

    fireEvent.click(screen.getByLabelText("Start SSH-Buddy when I sign in"));

    await waitFor(() => expect(getDesktopBehaviorStatus).toHaveBeenCalledTimes(2));
    expect(setAutostartEnabled).toHaveBeenCalledWith(true);
    expect(screen.getByLabelText("Start SSH-Buddy when I sign in")).toBeChecked();
    expect(screen.getByText("Login startup state verified.")).toBeInTheDocument();
  });

  it("recovers from an autostart operation failure without optimistic state drift", async () => {
    const getDesktopBehaviorStatus = vi.fn(async () => available);
    const setAutostartEnabled = vi.fn(async () => {
      throw new Error("registration denied");
    });
    renderSettings({ getDesktopBehaviorStatus, setAutostartEnabled });
    await screen.findByText("Login startup is disabled and verified.");

    fireEvent.click(screen.getByLabelText("Start SSH-Buddy when I sign in"));

    await screen.findByRole("alert");
    expect(screen.getByRole("alert")).toHaveTextContent("registration denied");
    expect(getDesktopBehaviorStatus).toHaveBeenCalledTimes(2);
    expect(screen.getByLabelText("Start SSH-Buddy when I sign in")).not.toBeChecked();
  });

  it("disables close to tray and explains an ineffective persisted preference", async () => {
    renderSettings({
      settings: { ...settings, closeToTray: true },
      getDesktopBehaviorStatus: vi.fn(async () => ({
        ...available,
        trayAvailable: false,
        trayError: "No status notifier host was found.",
      })),
    });

    expect(await screen.findByText(/No status notifier host was found/)).toBeInTheDocument();
    expect(screen.getByLabelText("Hide to system tray when closing")).toBeDisabled();
    expect(screen.getByText(/saved preference remains on, but closing will exit normally/i)).toBeInTheDocument();
  });

  it("persists close to tray immediately and reports save failures", async () => {
    const onSaveSettings = vi
      .fn<(next: AppSettings) => Promise<void>>()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error("database is read-only"));
    const { rerender } = render(
      <DesktopBehaviorSettings
        settings={settings}
        busy={false}
        onSaveSettings={onSaveSettings}
        getDesktopBehaviorStatus={async () => available}
        setAutostartEnabled={async () => available}
      />,
    );
    await screen.findByText("Login startup is disabled and verified.");

    fireEvent.click(screen.getByLabelText("Hide to system tray when closing"));
    await screen.findByText("Close-to-tray preference saved and applied.");
    expect(onSaveSettings).toHaveBeenCalledWith({ ...settings, closeToTray: true });

    rerender(
      <DesktopBehaviorSettings
        settings={{ ...settings, closeToTray: true }}
        busy={false}
        onSaveSettings={onSaveSettings}
        getDesktopBehaviorStatus={async () => available}
        setAutostartEnabled={async () => available}
      />,
    );
    fireEvent.click(screen.getByLabelText("Hide to system tray when closing"));
    expect(await screen.findByRole("alert")).toHaveTextContent("database is read-only");
  });
});
