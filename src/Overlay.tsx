import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import type { FightSummary, MeterState } from "./App";
import { liveRate, useLiveDuration } from "./useLiveDuration";

export type OverlayStatus = {
  open: boolean;
  click_through: boolean;
  x?: number | null;
  y?: number | null;
};

type OverlayPrefs = {
  maxRows: number;
  fontSize: number;
  showPercent: boolean;
  opacity: number;
};

const PREFS_KEY = "eql-overlay-prefs";
const OVERLAY_WIDTH = 380;
const MENU_H = 210;
const BANNER_H = 24;

const DEFAULT_PREFS: OverlayPrefs = {
  maxRows: 5,
  fontSize: 13,
  showPercent: false,
  opacity: 88,
};

function loadPrefs(): OverlayPrefs {
  try {
    const raw = localStorage.getItem(PREFS_KEY);
    if (!raw) return DEFAULT_PREFS;
    return { ...DEFAULT_PREFS, ...JSON.parse(raw) };
  } catch {
    return DEFAULT_PREFS;
  }
}

function formatDps(n: number): string {
  return Math.round(n).toLocaleString();
}

function formatNumber(n: number): string {
  return Math.round(n).toLocaleString();
}

function formatPct(n: number): string {
  if (n <= 0) return "0%";
  return `${n.toFixed(1)}%`;
}

function shortName(name: string, max = 20): string {
  if (name.length <= max) return name;
  return `${name.slice(0, max - 1)}…`;
}

function cycleValue<T>(values: T[], current: T): T {
  const index = values.indexOf(current);
  if (index < 0) return values[0];
  return values[(index + 1) % values.length];
}

export default function Overlay() {
  const [meter, setMeter] = useState<MeterState | null>(null);
  const [clickThrough, setClickThrough] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const [prefs, setPrefs] = useState<OverlayPrefs>(() => loadPrefs());
  const menuRef = useRef<HTMLDivElement | null>(null);
  const setupMode = !clickThrough;

  useEffect(() => {
    document.documentElement.dataset.eqlReady = "1";
    invoke<MeterState>("get_meter_state")
      .then(setMeter)
      .catch(() => setMeter(null));

    invoke<OverlayStatus>("get_overlay_status")
      .then((status) => setClickThrough(status.click_through))
      .catch(() => setClickThrough(false));

    const unlistenMeter = listen<MeterState>("meter-update", (event) => {
      setMeter(event.payload);
    });
    const unlistenStatus = listen<OverlayStatus>("overlay-status", (event) => {
      setClickThrough(event.payload.click_through);
    });

    return () => {
      unlistenMeter.then((fn) => fn());
      unlistenStatus.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const timer = window.setInterval(() => {
      void getCurrentWindow()
        .setAlwaysOnTop(true)
        .catch(() => undefined);
    }, 2000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    localStorage.setItem(PREFS_KEY, JSON.stringify(prefs));
  }, [prefs]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 1600);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        void closeOverlay();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    if (!menuOpen) return;
    function onDoc(event: PointerEvent) {
      const target = event.target as Node | null;
      if (!target) return;
      if (menuRef.current?.contains(target)) return;
      if ((event.target as HTMLElement | null)?.closest?.(".overlay-tools")) {
        return;
      }
      setMenuOpen(false);
    }
    const timer = window.setTimeout(() => {
      document.addEventListener("pointerdown", onDoc);
    }, 0);
    return () => {
      window.clearTimeout(timer);
      document.removeEventListener("pointerdown", onDoc);
    };
  }, [menuOpen]);

  function updatePrefs(patch: Partial<OverlayPrefs>) {
    setPrefs((prev) => ({ ...prev, ...patch }));
  }

  async function closeOverlay() {
    try {
      await invoke("hide_overlay");
    } catch (err) {
      setToast(String(err));
    }
  }

  async function setLiveMode(enabled: boolean) {
    try {
      const status = await invoke<OverlayStatus>("set_overlay_click_through", {
        enabled,
      });
      setClickThrough(status.click_through);
      if (enabled) {
        setMenuOpen(false);
        setToast("Locked — clicks go to the game");
      } else {
        setToast("Setup — drag to move, then Lock");
      }
    } catch (err) {
      setToast(String(err));
    }
  }

  // Overlay follows live combat only — don't stick on a finished fight name.
  const fight: FightSummary | null = meter?.active_fight ?? null;
  const character = meter?.character ?? null;
  const liveDuration = useLiveDuration(
    fight?.started_at,
    fight?.duration_secs ?? 0,
    fight?.active ?? false
  );

  const fightIds =
    meter?.active_fights && meter.active_fights.length > 0
      ? meter.active_fights.map((f) => f.id)
      : null;

  async function copyParse() {
    try {
      await invoke<string>("copy_parse", {
        fightId: fight?.id ?? null,
        fightIds: fightIds && fightIds.length > 1 ? fightIds : null,
      });
      setToast("Parse copied");
    } catch (err) {
      setToast(String(err));
    }
    setMenuOpen(false);
  }

  async function clearFights() {
    try {
      const next = await invoke<MeterState>("clear_fights");
      setMeter(next);
      setToast("Cleared");
    } catch (err) {
      setToast(String(err));
    }
    setMenuOpen(false);
  }

  // WebView2 often ignores data-tauri-drag-region on nested text — call startDragging.
  function beginDrag(event: React.PointerEvent) {
    if (event.button !== 0) return;
    if (clickThrough) return;
    const target = event.target as HTMLElement | null;
    if (!target) return;
    if (target.closest("button, a, input, .overlay-tools, .overlay-menu")) {
      return;
    }
    event.preventDefault();
    void getCurrentWindow()
      .startDragging()
      .catch(() => undefined);
  }

  const rows = useMemo(() => {
    if (!fight) return [];
    const total = fight.total_damage;
    return fight.players.slice(0, prefs.maxRows).map((p) => ({
      key: p.name,
      label: p.name,
      isSelf: character != null && p.name === character,
      damage: p.damage,
      dps: fight.active ? liveRate(p.damage, liveDuration) : p.dps,
      secs: liveDuration,
      pct: total > 0 ? (p.damage / total) * 100 : 0,
    }));
  }, [fight, character, liveDuration, prefs.maxRows]);

  const fightDps = fight
    ? fight.active
      ? liveRate(fight.total_damage, liveDuration)
      : fight.total_dps
    : 0;

  const headerH = 28;
  const fightBarH = fight ? 22 : 0;
  const rowH = Math.max(22, prefs.fontSize + 10);
  const toastH = 22;
  const setupBannerH = setupMode ? BANNER_H : 0;
  const bodyRows = fight ? Math.max(rows.length, 1) : 1;

  useEffect(() => {
    const h =
      headerH +
      setupBannerH +
      fightBarH +
      (menuOpen && setupMode ? MENU_H : 0) +
      (toast ? toastH : 0) +
      bodyRows * rowH +
      2;
    void getCurrentWindow()
      .setSize(new LogicalSize(OVERLAY_WIDTH, Math.max(h, headerH + rowH + 2)))
      .catch(() => undefined);
  }, [
    bodyRows,
    toast,
    headerH,
    fightBarH,
    rowH,
    toastH,
    setupBannerH,
    menuOpen,
    setupMode,
  ]);

  const shellStyle = {
    ["--overlay-font" as string]: `${prefs.fontSize}px`,
    ["--overlay-row-h" as string]: `${rowH}px`,
    ["--overlay-header-h" as string]: `${headerH}px`,
    ["--overlay-bg-alpha" as string]: String(prefs.opacity / 100),
  };

  return (
    <div
      className={`overlay-shell ${clickThrough ? "locked" : "setup"} ${
        prefs.showPercent ? "show-pct" : ""
      }`}
      style={shellStyle}
      onPointerDown={beginDrag}
    >
      <header className="overlay-top" title="Drag to move overlay">
        {setupMode ? (
          <div className="overlay-tools">
            <button
              type="button"
              className={`overlay-icon ${menuOpen ? "active" : ""}`}
              title="Setup"
              onClick={() => setMenuOpen((open) => !open)}
              aria-label="Setup"
            >
              <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden>
                <circle
                  cx="8"
                  cy="8"
                  r="2"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.3"
                />
                <path
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.3"
                  d="M8 1.5v2M8 12.5v2M1.5 8h2M12.5 8h2M3.2 3.2l1.4 1.4M11.4 11.4l1.4 1.4M12.8 3.2l-1.4 1.4M4.6 11.4l-1.4 1.4"
                />
              </svg>
            </button>
            <button
              type="button"
              className="overlay-icon"
              title="Copy parse"
              onClick={() => void copyParse()}
              disabled={!fight}
              aria-label="Copy parse"
            >
              <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden>
                <rect
                  x="5.5"
                  y="2.5"
                  width="7"
                  height="9"
                  rx="1"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.3"
                />
                <path
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.3"
                  d="M3.5 5.5h6v8h-6a1 1 0 01-1-1v-6a1 1 0 011-1z"
                />
              </svg>
            </button>
            <button
              type="button"
              className="overlay-lock-btn"
              title="Lock for game — clicks pass through to EverQuest"
              onClick={() => void setLiveMode(true)}
            >
              Lock
            </button>
            <button
              type="button"
              className="overlay-icon close"
              title="Close overlay"
              onClick={() => void closeOverlay()}
              aria-label="Close overlay"
            >
              ✕
            </button>
          </div>
        ) : (
          <span className="overlay-live-label">LIVE</span>
        )}

        <div className="overlay-drag" title="Drag overlay" />

        {prefs.showPercent ? (
          <span className="overlay-col-label">%</span>
        ) : null}
        <span className="overlay-col-label">Damage</span>
        <span className="overlay-col-label">DPS</span>
        <span className="overlay-col-label">Sec</span>
      </header>

      {setupMode ? (
        <p className="overlay-banner setup">
          Setup — drag anywhere, then Lock for game
        </p>
      ) : null}

      {menuOpen && setupMode ? (
        <div className="overlay-menu" ref={menuRef}>
          <p className="overlay-menu-section">Meter</p>
          <button
            type="button"
            onClick={() => void copyParse()}
            disabled={!fight}
          >
            Copy parse for chat
          </button>
          <button type="button" onClick={() => void clearFights()}>
            Clear fight history
          </button>
          <button type="button" onClick={() => void setLiveMode(true)}>
            Lock for game
          </button>

          <p className="overlay-menu-section">Look</p>
          <button
            type="button"
            onClick={() =>
              updatePrefs({
                maxRows: cycleValue([3, 5, 8, 10], prefs.maxRows),
              })
            }
          >
            Players shown: {prefs.maxRows}
          </button>
          <button
            type="button"
            onClick={() =>
              updatePrefs({
                fontSize: cycleValue([12, 13, 14, 16], prefs.fontSize),
              })
            }
          >
            Font size: {prefs.fontSize}pt
          </button>
          <button
            type="button"
            onClick={() =>
              updatePrefs({
                opacity: cycleValue([70, 80, 88, 95], prefs.opacity),
              })
            }
          >
            Opacity: {prefs.opacity}%
          </button>
          <button
            type="button"
            className={prefs.showPercent ? "on" : ""}
            onClick={() => updatePrefs({ showPercent: !prefs.showPercent })}
          >
            {prefs.showPercent ? "✓ " : ""}Show % damage
          </button>

          <p className="overlay-menu-hint">
            Unlock later: Ctrl/Cmd+Shift+U · or Close Overlay in main
          </p>
        </div>
      ) : null}

      {toast ? <p className="overlay-toast">{toast}</p> : null}

      {fight ? (
        <>
          <div className="overlay-rows">
            {rows.length === 0 ? (
              <p className="overlay-empty">No damage yet.</p>
            ) : (
              rows.map((row, index) => {
                const name = `${index + 1}. ${shortName(row.label)}`;

                return (
                  <div
                    key={row.key}
                    className={`overlay-row ${row.isSelf ? "self" : ""} ${
                      index === 0 ? "top" : ""
                    }`}
                  >
                    <span className="overlay-name" title={row.label}>
                      {name}
                    </span>
                    {prefs.showPercent ? (
                      <span className="overlay-num pct">
                        {formatPct(row.pct)}
                      </span>
                    ) : null}
                    <span className="overlay-num">
                      {formatNumber(row.damage)}
                    </span>
                    <span className="overlay-num">{formatDps(row.dps)}</span>
                    <span className="overlay-num">
                      {Math.max(1, Math.floor(row.secs))}
                    </span>
                  </div>
                );
              })
            )}
          </div>
          <div className="overlay-fight">
            <span className="overlay-fight-target" title={fight.target}>
              {shortName(fight.target, 28)}
            </span>
            <span className="overlay-fight-meta">
              {formatDps(fightDps)} DPS ·{" "}
              {Math.max(1, Math.floor(liveDuration))}s
            </span>
          </div>
        </>
      ) : (
        <p className="overlay-empty">Waiting for combat…</p>
      )}
    </div>
  );
}
