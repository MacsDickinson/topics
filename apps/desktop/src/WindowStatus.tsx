import { useEffect, useRef, useState } from "react";
import { useDismissOnOutsidePress } from "./boardui/use-dismiss-on-outside-press";
import { errorHelp } from "./ipc";
import { Button, Icon } from "./ui";
import type { ApiError, RuntimeStatus } from "./workspace-types";

export function WindowStatus({
  runtime,
  error,
  notice,
  bootError,
  opening,
  running,
  dismissError,
  dismissNotice,
  checkRuntime,
}: {
  runtime: RuntimeStatus | null;
  error: ApiError | null;
  notice: string;
  bootError: ApiError | null;
  opening: boolean;
  running: boolean;
  dismissError: () => void;
  dismissNotice: () => void;
  checkRuntime: () => Promise<void>;
}) {
  const [open, setOpen] = useState(false);
  const [checking, setChecking] = useState(false);
  const container = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  useDismissOnOutsidePress(open, () => setOpen(false), [container]);

  function close() {
    setOpen(false);
    trigger.current?.focus();
  }

  useEffect(() => {
    if (!open) return;
    // A mouse press on selectable text or a WebKit button can clear focus.
    // Close when focus actually arrives on another control.
    function focusChanged(event: FocusEvent) {
      if (event.target instanceof Element && !container.current?.contains(event.target))
        setOpen(false);
    }
    function escape(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        close();
      }
    }
    document.addEventListener("focusin", focusChanged);
    document.addEventListener("keydown", escape);
    return () => {
      document.removeEventListener("focusin", focusChanged);
      document.removeEventListener("keydown", escape);
    };
  }, [open]);

  const runtimeLabel = runtime?.available
    ? ["Codex", runtime.version, "installed"].filter(Boolean).join(" ")
    : runtime?.message;
  const summary = bootError
    ? "Workspace unavailable"
    : error
    ? error.message
    : notice ||
      (running
        ? "Analysis in progress"
        : opening
        ? "Opening your workspace…"
        : runtimeLabel) ||
      "Checking runtime…";
  const tone =
    bootError || error
      ? "danger"
      : notice || running || opening
      ? "info"
      : runtime?.available
      ? "ready"
      : "muted";

  return (
    <header className="window-chrome" aria-label="Workspace status">
      <div
        className="window-chrome-drag"
        aria-hidden="true"
        data-tauri-drag-region
      />
      <div ref={container} className="window-status">
        <div className={`window-status-line ${tone}`}>
          <Icon name={tone === "ready" ? "check" : "info"} />
          <span
            id="workspace-status-summary"
            className="window-status-summary"
            role={error && !bootError ? "alert" : undefined}
            aria-live={error && !bootError ? "assertive" : "polite"}
          >
            {summary}
            {error && !bootError && <span className="sr-only"> {errorHelp(error)}</span>}
          </span>
          <button
            ref={trigger}
            type="button"
            className="window-status-trigger"
            aria-label="Workspace status details"
            aria-describedby="workspace-status-summary"
            aria-expanded={open}
            aria-controls={open ? "workspace-status-details" : undefined}
            onClick={() => setOpen(!open)}
          >
            <Icon name="chevron" />
          </button>
        </div>
        {open && (
          <section
            id="workspace-status-details"
            aria-label="Status details"
            className="window-status-details"
          >
            {bootError ? (
              <p>{bootError.message}</p>
            ) : (
              <>
                {error && (
                  <>
                    <p>
                      <strong>{error.message}</strong>
                    </p>
                    {errorHelp(error) && <p>{errorHelp(error)}</p>}
                    {error.details != null && (
                      <details>
                        <summary>Related changes</summary>
                        <pre>{JSON.stringify(error.details, null, 2)}</pre>
                      </details>
                    )}
                    <Button
                      variant="quiet"
                      onClick={() => {
                        close();
                        dismissError();
                      }}
                    >
                      Dismiss error
                    </Button>
                  </>
                )}
                {notice && (
                  <div>
                    <p>{notice}</p>
                    <Button
                      variant="quiet"
                      onClick={() => {
                        close();
                        dismissNotice();
                      }}
                    >
                      Dismiss notification
                    </Button>
                  </div>
                )}
                {running && <p>Analysis in progress</p>}
                {runtimeLabel && <p>{runtimeLabel}</p>}
                {!runtimeLabel && !running && !error && !notice && (
                  <p>{summary}</p>
                )}
                {runtime && !runtime.available && (
                  <Button
                    busy={checking}
                    onClick={async () => {
                      trigger.current?.focus();
                      setChecking(true);
                      try {
                        await checkRuntime();
                      } finally {
                        setChecking(false);
                      }
                    }}
                  >
                    Check runtime again
                  </Button>
                )}
              </>
            )}
          </section>
        )}
      </div>
    </header>
  );
}
