import { useCallback, useEffect, useState } from "react";
import { RefreshCw } from "lucide-react";
import { api } from "../lib/api";
import type { AppSettings, DesktopBehaviorStatus } from "../lib/types";

type DesktopSetting = "startMinimized" | "closeToTray";
type PendingOperation = "autostart" | DesktopSetting | "status" | null;

export interface DesktopBehaviorSettingsProps {
  settings: AppSettings;
  busy: boolean;
  onSaveSettings: (settings: AppSettings) => Promise<void>;
  getDesktopBehaviorStatus?: () => Promise<DesktopBehaviorStatus>;
  setAutostartEnabled?: (enabled: boolean) => Promise<DesktopBehaviorStatus>;
}

function errorMessage(cause: unknown) {
  return cause instanceof Error ? cause.message : String(cause);
}

export function DesktopBehaviorSettings({
  settings,
  busy,
  onSaveSettings,
  getDesktopBehaviorStatus = api.getDesktopBehaviorStatus,
  setAutostartEnabled = api.setAutostartEnabled,
}: DesktopBehaviorSettingsProps) {
  const [status, setStatus] = useState<DesktopBehaviorStatus | null>(null);
  const [pending, setPending] = useState<PendingOperation>("status");
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const refreshStatus = useCallback(async () => {
    setPending((current) => current ?? "status");
    try {
      const next = await getDesktopBehaviorStatus();
      setStatus(next);
      setError(next.autostartError);
    } catch (cause: unknown) {
      const nextError = `Could not query desktop behavior: ${errorMessage(cause)}`;
      setStatus((current) => ({
        autostartEnabled: null,
        autostartError: nextError,
        trayAvailable: current?.trayAvailable ?? false,
        trayError: current?.trayError ?? null,
      }));
      setError(nextError);
    } finally {
      setPending((current) => (current === "status" ? null : current));
    }
  }, [getDesktopBehaviorStatus]);

  useEffect(() => {
    void refreshStatus();
  }, [refreshStatus]);

  async function updateAutostart(enabled: boolean) {
    setPending("autostart");
    setError(null);
    setMessage(null);
    let operationError: string | null = null;

    try {
      const attemptedStatus = await setAutostartEnabled(enabled);
      setStatus(attemptedStatus);
      operationError = attemptedStatus.autostartError;
    } catch (cause: unknown) {
      operationError = errorMessage(cause);
    }

    try {
      const verifiedStatus = await getDesktopBehaviorStatus();
      setStatus(verifiedStatus);
      const nextError = operationError ?? verifiedStatus.autostartError;
      setError(nextError);
      if (!nextError) {
        setMessage(
          verifiedStatus.autostartEnabled === enabled
            ? "Login startup state verified."
            : `Login startup did not match the requested ${enabled ? "enabled" : "disabled"} state.`,
        );
      }
    } catch (cause: unknown) {
      const verificationError = `Could not verify login startup after the update: ${errorMessage(cause)}`;
      setStatus((current) => ({
        autostartEnabled: null,
        autostartError: verificationError,
        trayAvailable: current?.trayAvailable ?? false,
        trayError: current?.trayError ?? null,
      }));
      setError([operationError, verificationError].filter(Boolean).join(" "));
    } finally {
      setPending(null);
    }
  }

  async function updateSetting(name: DesktopSetting, enabled: boolean) {
    setPending(name);
    setError(null);
    setMessage(null);
    try {
      await onSaveSettings({ ...settings, [name]: enabled });
      setMessage(
        name === "startMinimized"
          ? "Start-minimized preference saved for the next fresh process launch."
          : "Close-to-tray preference saved and applied.",
      );
    } catch (cause: unknown) {
      setError(errorMessage(cause));
    } finally {
      setPending(null);
    }
  }

  const disabled = busy || pending !== null;
  const autostartState = status?.autostartEnabled;
  const trayAvailable = status?.trayAvailable ?? false;

  return (
    <section className="panel edit-panel desktop-behavior-settings" aria-labelledby="desktop-behavior-heading">
      <div>
        <h2 id="desktop-behavior-heading">Desktop behavior</h2>
        <p className="muted">These preferences apply immediately unless noted otherwise.</p>
      </div>

      <label className="check-row">
        <input
          type="checkbox"
          checked={autostartState === true}
          disabled={disabled}
          onChange={(event) => void updateAutostart(event.target.checked)}
        />
        Start SSH-Buddy when I sign in
      </label>
      <div className="desktop-setting-detail" aria-live="polite">
        {pending === "status" ? <span className="field-hint">Checking the operating-system registration…</span> : null}
        {pending !== "status" && autostartState === true ? <span className="field-hint">Login startup is enabled and verified.</span> : null}
        {pending !== "status" && autostartState === false ? <span className="field-hint">Login startup is disabled and verified.</span> : null}
        {pending !== "status" && autostartState == null ? <span className="field-error">Login startup state is unknown.</span> : null}
        <span className="field-hint">AppImage users must keep the AppImage at a stable path before enabling login startup.</span>
        <button className="button ghost compact-button" type="button" disabled={disabled} onClick={() => void refreshStatus()}>
          <RefreshCw size={15} />
          Refresh actual state
        </button>
      </div>

      <label className="check-row">
        <input
          type="checkbox"
          checked={settings.startMinimized}
          disabled={disabled}
          onChange={(event) => void updateSetting("startMinimized", event.target.checked)}
        />
        Start minimized
      </label>
      <span className="field-hint">Applies on the next fresh process launch and creates a minimized taskbar window.</span>

      <label className="check-row">
        <input
          type="checkbox"
          checked={settings.closeToTray}
          disabled={disabled || !trayAvailable}
          onChange={(event) => void updateSetting("closeToTray", event.target.checked)}
        />
        Hide to system tray when closing
      </label>
      <span className="field-hint">Use the tray menu to reopen SSH-Buddy or quit it explicitly.</span>
      {!trayAvailable ? (
        <div className="desktop-warning" role="status">
          <strong>System tray unavailable.</strong> {status?.trayError ?? "Tray initialization has not completed."}
          {settings.closeToTray ? " Your saved preference remains on, but closing will exit normally." : " Closing will exit normally."}
        </div>
      ) : null}

      {message ? <div className="status-banner success desktop-status">{message}</div> : null}
      {error ? <div className="error-text desktop-status" role="alert">{error}</div> : null}
    </section>
  );
}
