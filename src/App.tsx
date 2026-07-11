import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type MouseEvent as ReactMouseEvent,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { formatDuration, liveRate, useLiveDuration } from "./useLiveDuration";
import {
  checkForAppUpdate,
  installAppUpdate,
  openLatestReleasePage,
  type PendingUpdate,
} from "./updates";
import "./App.css";

export type AbilityStat = {
  name: string;
  hits: number;
  damage: number;
  healing: number;
};

export type TimelinePoint = {
  sec: number;
  damage: number;
};

export type TypeStat = {
  name: string;
  damage: number;
  hits: number;
  pct: number;
};

export type PlayerStat = {
  name: string;
  damage: number;
  hits: number;
  crits: number;
  max_hit: number;
  dps: number;
  pct: number;
  attempts: number;
  misses: number;
  accuracy_pct: number;
  healing: number;
  overheal: number;
  hps: number;
  heal_pct: number;
  healing_received: number;
  abilities: AbilityStat[];
  timeline: TimelinePoint[];
  heal_timeline: TimelinePoint[];
};

export type FightSummary = {
  id: number;
  target: string;
  targets: TypeStat[];
  started_at: number;
  ended_at: number | null;
  duration_secs: number;
  total_damage: number;
  total_dps: number;
  peak_dps: number;
  total_hits: number;
  crits: number;
  crit_pct: number;
  max_hit: number;
  max_hit_by: string | null;
  damage_taken: number;
  taken_hits: number;
  dtps: number;
  max_taken_hit: number;
  attempts: number;
  misses: number;
  accuracy_pct: number;
  dodges: number;
  parries: number;
  blocks: number;
  ripostes: number;
  resists: number;
  healing: number;
  overheal: number;
  hps: number;
  overheal_pct: number;
  kills: number;
  active: boolean;
  players: PlayerStat[];
  timeline: TimelinePoint[];
  heal_timeline: TimelinePoint[];
  damage_types: TypeStat[];
  heal_spells: TypeStat[];
  taken_sources: TypeStat[];
};

export type RosterPlayer = {
  name: string;
  class_name: string;
  level: number;
  group: number;
};

export type RaidRoster = {
  captured_at: number;
  players: RosterPlayer[];
};

export type MiscKind = "loot" | "random" | "roll" | "chat";

export type MiscEvent = {
  timestamp: string;
  time_secs: number | null;
  kind: MiscKind;
  summary: string;
  who: string | null;
  detail: string | null;
};

export type MeterState = {
  character: string | null;
  server: string | null;
  stance: string | null;
  log_path: string | null;
  monitoring: boolean;
  focus_primary: boolean;
  min_fight_damage: number;
  active_fights: FightSummary[];
  active_fight: FightSummary | null;
  recent_fights: FightSummary[];
  raid_roster: RaidRoster | null;
  recent_rosters: RaidRoster[];
  misc_log: MiscEvent[];
  spells_count: number;
  spells_path: string | null;
};

export type AppSettings = {
  last_log_path: string | null;
  auto_monitor_on_start: boolean;
  focus_primary: boolean;
  min_fight_damage: number;
  main_window?: {
    x: number;
    y: number;
    width: number;
    height: number;
  } | null;
  overlay_window?: {
    x: number;
    y: number;
    width: number;
    height: number;
  } | null;
  spells_path?: string | null;
};

const emptyState: MeterState = {
  character: null,
  server: null,
  stance: null,
  log_path: null,
  monitoring: false,
  focus_primary: false,
  min_fight_damage: 0,
  active_fights: [],
  active_fight: null,
  recent_fights: [],
  raid_roster: null,
  recent_rosters: [],
  misc_log: [],
  spells_count: 0,
  spells_path: null,
};

/** Default left-rail selection: live combined, else live mob, else newest recent. */
function defaultSelectedFightIds(meter: MeterState): number[] {
  if (meter.active_fights.length > 1) return [0];
  if (meter.active_fights[0]) return [meter.active_fights[0].id];
  if (meter.recent_fights[0]) return [meter.recent_fights[0].id];
  return [];
}

function fightSelectionValid(ids: number[], meter: MeterState): boolean {
  if (ids.length === 0) return false;
  const liveIds = new Set(meter.active_fights.map((f) => f.id));
  const recentIds = new Set(meter.recent_fights.map((f) => f.id));
  return ids.every((id) => {
    if (id === 0) return meter.active_fights.length > 1;
    return liveIds.has(id) || recentIds.has(id);
  });
}

function rosterGroupMap(roster: RaidRoster | null): Map<string, number> {
  const map = new Map<string, number>();
  if (!roster) return map;
  for (const player of roster.players) {
    map.set(player.name.toLowerCase(), player.group);
  }
  return map;
}

type GroupDamageRow = {
  group: number;
  label: string;
  damage: number;
  healing: number;
  players: number;
  pct: number;
};

function groupDamageRows(
  fight: FightSummary,
  roster: RaidRoster | null
): GroupDamageRow[] {
  const groups = rosterGroupMap(roster);
  if (groups.size === 0) return [];

  const totals = new Map<number, { damage: number; healing: number; players: number }>();
  for (const player of fight.players) {
    const group = groups.get(player.name.toLowerCase());
    if (group == null) continue;
    const current = totals.get(group) ?? { damage: 0, healing: 0, players: 0 };
    current.damage += player.damage;
    current.healing += player.healing;
    current.players += 1;
    totals.set(group, current);
  }

  const raidDamage = Math.max(
    [...totals.values()].reduce((sum, row) => sum + row.damage, 0),
    1
  );

  const rows: GroupDamageRow[] = [];
  for (const [group, stats] of totals) {
    let label = "Ungrouped";
    if (group > 0) {
      label = `Group ${group}`;
    }
    rows.push({
      group,
      label,
      damage: stats.damage,
      healing: stats.healing,
      players: stats.players,
      pct: (stats.damage / raidDamage) * 100,
    });
  }
  rows.sort((a, b) => {
    if (b.damage !== a.damage) return b.damage - a.damage;
    return a.group - b.group;
  });
  return rows;
}

function sameFightIds(a: number[], b: number[]): boolean {
  if (a.length !== b.length) return false;
  return a.every((id, i) => id === b[i]);
}

/** Live combined shortcut (rail id 0) or multi-select. */
function isCombinedSelection(ids: number[], meter: MeterState): boolean {
  if (ids.length === 1 && ids[0] === 0 && meter.active_fights.length > 1) {
    return true;
  }
  return ids.length >= 2;
}

function formatNumber(n: number): string {
  return Math.round(n).toLocaleString();
}

function formatDps(n: number): string {
  return Math.round(n).toLocaleString();
}

function shortPath(path: string | null): string {
  if (!path) return "No log selected";
  if (path === "demo") return "Demo combat";
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

function densifyTimeline(
  points: TimelinePoint[],
  durationSecs: number
): number[] {
  const maxSec = Math.max(
    Math.ceil(durationSecs) - 1,
    points.length ? points[points.length - 1].sec : 0,
    0
  );
  const values = Array.from({ length: maxSec + 1 }, () => 0);
  for (const point of points) {
    if (point.sec >= 0 && point.sec < values.length) {
      values[point.sec] = point.damage;
    }
  }
  return values;
}

function DamageChart({
  fight,
  selectedPlayer,
  durationSecs,
  mode,
}: {
  fight: FightSummary;
  selectedPlayer: string | null;
  durationSecs: number;
  mode: "dps" | "heals";
}) {
  const width = 640;
  const height = 160;
  const padL = 8;
  const padR = 8;
  const padT = 12;
  const padB = 22;
  const svgRef = useRef<SVGSVGElement | null>(null);
  const [hover, setHover] = useState<{
    sec: number;
    group: number;
    player: number | null;
    x: number;
    y: number;
  } | null>(null);

  const isHeals = mode === "heals";
  const groupTimeline = isHeals ? fight.heal_timeline : fight.timeline;
  const totalSeries = densifyTimeline(groupTimeline, durationSecs);
  const playerSeries = useMemo(() => {
    const player = fight.players.find((p) => p.name === selectedPlayer);
    if (!player) return null;
    const points = isHeals ? player.heal_timeline : player.timeline;
    return densifyTimeline(points, durationSecs);
  }, [fight, selectedPlayer, durationSecs, isHeals]);

  const maxVal = Math.max(1, ...totalSeries, ...(playerSeries ?? []));

  const plotW = width - padL - padR;
  const plotH = height - padT - padB;
  const lastSec = Math.max(totalSeries.length - 1, 1);

  function toPoints(series: number[]): string {
    return series
      .map((value, sec) => {
        const x = padL + (sec / lastSec) * plotW;
        const y = padT + plotH - (value / maxVal) * plotH;
        return `${x},${y}`;
      })
      .join(" ");
  }

  function toArea(series: number[]): string {
    if (series.length === 0) return "";
    const line = toPoints(series);
    const x0 = padL;
    const x1 = padL + plotW;
    const yBase = padT + plotH;
    return `${x0},${yBase} ${line} ${x1},${yBase}`;
  }

  function onMove(event: ReactMouseEvent<SVGSVGElement>) {
    const svg = svgRef.current;
    if (!svg) return;
    const rect = svg.getBoundingClientRect();
    const x = ((event.clientX - rect.left) / rect.width) * width;
    const clamped = Math.min(Math.max(x, padL), width - padR);
    const sec = Math.round(((clamped - padL) / plotW) * lastSec);
    const group = totalSeries[sec] ?? 0;
    const player = playerSeries ? (playerSeries[sec] ?? 0) : null;
    const y = padT + plotH - (group / maxVal) * plotH;
    setHover({ sec, group, player, x: clamped, y });
  }

  const midSec = Math.floor(lastSec / 2);
  const fillId = isHeals ? "healFill" : "dmgFill";
  const fillColor = isHeals ? "#5aae8b" : "#e0a84a";

  return (
    <div className="chart-card">
      <div className="chart-header">
        <h3>{isHeals ? "Healing over time" : "Damage over time"}</h3>
        <p>
          Group total
          {selectedPlayer ? ` · ${selectedPlayer} overlay` : ""}
          {hover
            ? ` · ${formatDuration(hover.sec)} · ${formatNumber(hover.group)}`
            : " · hover for second"}
        </p>
      </div>
      <div className="chart-wrap">
        <svg
          ref={svgRef}
          className={`damage-chart ${isHeals ? "heals" : ""}`}
          viewBox={`0 0 ${width} ${height}`}
          role="img"
          aria-label={isHeals ? "Healing over time chart" : "Damage over time chart"}
          onMouseMove={onMove}
          onMouseLeave={() => setHover(null)}
        >
          <defs>
            <linearGradient id={fillId} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={fillColor} stopOpacity="0.45" />
              <stop offset="100%" stopColor={fillColor} stopOpacity="0.02" />
            </linearGradient>
          </defs>
          <line
            className="chart-grid"
            x1={padL}
            x2={width - padR}
            y1={padT + plotH / 2}
            y2={padT + plotH / 2}
          />
          <polygon
            className="chart-area"
            points={toArea(totalSeries)}
            fill={`url(#${fillId})`}
          />
          <polyline className="chart-line total" points={toPoints(totalSeries)} />
          {playerSeries ? (
            <polyline
              className="chart-line player"
              points={toPoints(playerSeries)}
            />
          ) : null}
          {hover ? (
            <>
              <line
                className="chart-hover-line"
                x1={hover.x}
                x2={hover.x}
                y1={padT}
                y2={padT + plotH}
              />
              <circle
                className="chart-hover-dot"
                cx={hover.x}
                cy={hover.y}
                r={3.5}
                style={isHeals ? { fill: fillColor } : undefined}
              />
            </>
          ) : null}
          <text className="chart-axis" x={padL} y={height - 6}>
            0:00
          </text>
          <text
            className="chart-axis"
            x={padL + plotW / 2}
            y={height - 6}
            textAnchor="middle"
          >
            {formatDuration(midSec)}
          </text>
          <text
            className="chart-axis"
            x={width - padR}
            y={height - 6}
            textAnchor="end"
          >
            {formatDuration(lastSec)}
          </text>
          <text
            className="chart-axis"
            x={width - padR}
            y={padT + 4}
            textAnchor="end"
          >
            {formatNumber(maxVal)}/s peak bucket
          </text>
        </svg>
        {hover ? (
          <div
            className="chart-tooltip"
            style={{
              left: `${(hover.x / width) * 100}%`,
            }}
          >
            <strong>{formatDuration(hover.sec)}</strong>
            <span>Group {formatNumber(hover.group)}</span>
            {hover.player != null && selectedPlayer ? (
              <span>
                {selectedPlayer} {formatNumber(hover.player)}
              </span>
            ) : null}
          </div>
        ) : null}
      </div>
    </div>
  );
}

function ComparePulls({
  fights,
  target,
}: {
  fights: FightSummary[];
  target: string;
}) {
  const [count, setCount] = useState(5);
  const baseName = target.toLowerCase();
  const same = useMemo(() => {
    return fights
      .filter((f) => f.target.toLowerCase() === baseName)
      .slice(0, count);
  }, [fights, baseName, count]);

  if (same.length < 2) return null;

  const avgDps =
    same.reduce((sum, f) => sum + f.total_dps, 0) / Math.max(same.length, 1);
  const best = same.reduce((a, b) => (a.total_dps > b.total_dps ? a : b));
  const avgDur =
    same.reduce((sum, f) => sum + f.duration_secs, 0) / Math.max(same.length, 1);

  return (
    <div className="compare-card">
      <div className="chart-header">
        <h3>Compare pulls</h3>
        <p>
          Last {same.length} · {target}
        </p>
      </div>
      <div className="compare-controls">
        {[3, 5, 10].map((n) => (
          <button
            key={n}
            type="button"
            className={count === n ? "active" : ""}
            onClick={() => setCount(n)}
          >
            {n}
          </button>
        ))}
      </div>
      <div className="compare-stats">
        <div>
          <span>Avg DPS</span>
          <strong>{formatDps(avgDps)}</strong>
        </div>
        <div>
          <span>Best DPS</span>
          <strong>{formatDps(best.total_dps)}</strong>
        </div>
        <div>
          <span>Avg time</span>
          <strong>{formatDuration(avgDur)}</strong>
        </div>
      </div>
      <div className="compare-list">
        {same.map((fight, index) => {
          const widthPct = Math.max(
            4,
            (fight.total_dps / Math.max(best.total_dps, 1)) * 100
          );
          return (
            <div key={fight.id} className="compare-row">
              <span className="compare-rank">#{index + 1}</span>
              <div className="compare-bar-wrap">
                <div className="compare-bar" style={{ width: `${widthPct}%` }} />
              </div>
              <span className="compare-meta">
                {formatDps(fight.total_dps)} · {formatDuration(fight.duration_secs)}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function App() {
  const [meter, setMeter] = useState<MeterState>(emptyState);
  const [selectedFightIds, setSelectedFightIds] = useState<number[]>([]);
  const [combinedFight, setCombinedFight] = useState<FightSummary | null>(null);
  const [selectedPlayer, setSelectedPlayer] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [overlayOpen, setOverlayOpen] = useState(false);
  const [overlayClickThrough, setOverlayClickThrough] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const [appVersion, setAppVersion] = useState<string>("");
  const [pendingUpdate, setPendingUpdate] = useState<PendingUpdate | null>(
    null
  );
  const [updateBusy, setUpdateBusy] = useState(false);
  const [fightContextMenu, setFightContextMenu] = useState<{
    x: number;
    y: number;
    fightId: number;
  } | null>(null);
  const [isMac, setIsMac] = useState(false);
  const [meterTab, setMeterTab] = useState<"dps" | "heals" | "raid" | "misc">(
    "dps"
  );
  const [miscFilter, setMiscFilter] = useState<"all" | MiscKind>("all");
  const menuRef = useRef<HTMLDivElement | null>(null);
  const prevLiveCount = useRef(0);
  // When false, rail follows live combat (Combined on multi-mob pulls).
  const selectionPinned = useRef(false);

  const startPath = useCallback(async (path: string, fromStart: boolean) => {
    setError(null);
    setBusy(true);
    try {
      const next = await invoke<MeterState>("start_monitoring", {
        path,
        fromStart,
      });
      setMeter(next);
      selectionPinned.current = false;
      prevLiveCount.current = next.active_fights.length;
      setSelectedFightIds(defaultSelectedFightIds(next));
      setCombinedFight(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    invoke<string>("host_os")
      .then((os) => setIsMac(os === "macos"))
      .catch(() => setIsMac(false));

    invoke<MeterState>("get_meter_state")
      .then((next) => {
        setMeter(next);
        selectionPinned.current = false;
        prevLiveCount.current = next.active_fights.length;
        setSelectedFightIds(defaultSelectedFightIds(next));
      })
      .catch((err) => setError(String(err)));

    invoke<{ open: boolean; click_through: boolean }>("get_overlay_status")
      .then((status) => {
        setOverlayOpen(status.open);
        setOverlayClickThrough(status.click_through);
      })
      .catch(() => undefined);

    invoke<AppSettings>("get_settings")
      .then(async (settings) => {
        if (
          settings.auto_monitor_on_start &&
          settings.last_log_path
        ) {
          try {
            const next = await invoke<MeterState>("start_monitoring", {
              path: settings.last_log_path,
              fromStart: false,
            });
            setMeter(next);
            selectionPinned.current = false;
            prevLiveCount.current = next.active_fights.length;
            setSelectedFightIds(defaultSelectedFightIds(next));
          } catch {
            // Last path may be missing on this machine; stay idle.
          }
        }
      })
      .catch(() => undefined);

    const unlisten = listen<MeterState>("meter-update", (event) => {
      setMeter(event.payload);
    });
    const unlistenOverlay = listen<{ open: boolean; click_through: boolean }>(
      "overlay-status",
      (event) => {
        setOverlayOpen(event.payload.open);
        setOverlayClickThrough(event.payload.click_through);
      }
    );
    const unlistenToast = listen<string>("hotkey-toast", (event) => {
      setToast(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenOverlay.then((fn) => fn());
      unlistenToast.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    getVersion()
      .then(setAppVersion)
      .catch(() => setAppVersion(""));

    // Quiet startup check — only surface a banner when something is available.
    checkForAppUpdate()
      .then((update) => {
        if (update) {
          setPendingUpdate(update);
        }
      })
      .catch(() => undefined);
  }, []);

  const runUpdateCheck = useCallback(async () => {
    setUpdateBusy(true);
    setError(null);
    try {
      const update = await checkForAppUpdate();
      if (!update) {
        setPendingUpdate(null);
        setToast(
          appVersion
            ? `You're on the latest version (${appVersion})`
            : "You're on the latest version"
        );
        return;
      }
      setPendingUpdate(update);
      setToast(`Update ${update.version} is available`);
    } catch (err) {
      setError(
        `Could not check for updates. Open the latest release page instead. (${String(err)})`
      );
    } finally {
      setUpdateBusy(false);
    }
  }, [appVersion]);

  const runInstallUpdate = useCallback(async () => {
    setUpdateBusy(true);
    setError(null);
    setToast("Downloading update…");
    try {
      await installAppUpdate();
    } catch (err) {
      setError(
        `Update install failed. Try the release page instead. (${String(err)})`
      );
      setUpdateBusy(false);
    }
  }, []);

  useEffect(() => {
    const liveCount = meter.active_fights.length;

    // New pull, or grew into a multi-mob pull — follow Combined / live default.
    if (liveCount > 0 && prevLiveCount.current === 0) {
      selectionPinned.current = false;
    }
    if (liveCount > 1 && prevLiveCount.current <= 1) {
      selectionPinned.current = false;
    }
    if (liveCount === 0) {
      selectionPinned.current = false;
    }
    prevLiveCount.current = liveCount;

    if (
      selectionPinned.current &&
      fightSelectionValid(selectedFightIds, meter)
    ) {
      return;
    }

    const next = defaultSelectedFightIds(meter);
    if (sameFightIds(next, selectedFightIds)) return;
    setSelectedFightIds(next);
  }, [meter, selectedFightIds]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 2500);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    let cancelled = false;
    async function loadCombined() {
      const ids = selectedFightIds.filter((id) => id !== 0);
      if (selectedFightIds.length === 1 && selectedFightIds[0] === 0) {
        setCombinedFight(null);
        return;
      }
      if (ids.length < 2) {
        setCombinedFight(null);
        return;
      }
      try {
        const combined = await invoke<FightSummary>("combine_fights", {
          fightIds: ids,
        });
        if (!cancelled) setCombinedFight(combined);
      } catch {
        if (!cancelled) setCombinedFight(null);
      }
    }
    void loadCombined();
    return () => {
      cancelled = true;
    };
  }, [selectedFightIds, meter.active_fights, meter.recent_fights]);

  const displayedFight = useMemo(() => {
    // Live Combined (N) row — always use backend combined active fight.
    if (
      selectedFightIds.length === 1 &&
      selectedFightIds[0] === 0 &&
      meter.active_fight &&
      meter.active_fights.length > 1
    ) {
      return meter.active_fight;
    }
    if (combinedFight && selectedFightIds.length >= 2) {
      return combinedFight;
    }
    if (selectedFightIds.length === 1) {
      const id = selectedFightIds[0];
      const live = meter.active_fights.find((f) => f.id === id);
      if (live) return live;
      const found = meter.recent_fights.find((f) => f.id === id);
      if (found) return found;
    }
    if (meter.active_fight) return meter.active_fight;
    return meter.recent_fights[0] ?? null;
  }, [meter, selectedFightIds, combinedFight]);

  const showingCombined = isCombinedSelection(selectedFightIds, meter);

  function isFightHighlighted(fightId: number): boolean {
    if (selectedFightIds.length > 1) {
      return selectedFightIds.includes(fightId);
    }
    if (selectedFightIds.length === 1) {
      return selectedFightIds[0] === fightId;
    }
    return false;
  }

  function selectFight(
    id: number,
    event: ReactMouseEvent<HTMLButtonElement>
  ) {
    selectionPinned.current = true;
    const additive = event.metaKey || event.ctrlKey;
    setSelectedFightIds((prev) => {
      if (!additive) {
        return [id];
      }
      // Don't mix the live Combined (0) shortcut with manual multi-select.
      const base = prev.filter((x) => x !== 0);
      if (id === 0) {
        return [0];
      }
      if (base.includes(id)) {
        const next = base.filter((x) => x !== id);
        return next.length > 0 ? next : [id];
      }
      return [...base, id];
    });
  }

  function selectAllVisible() {
    const ids = [
      ...meter.active_fights.map((f) => f.id),
      ...meter.recent_fights.map((f) => f.id),
    ];
    if (ids.length === 0) return;
    selectionPinned.current = true;
    setSelectedFightIds(ids);
  }

  function clearFightSelection() {
    selectionPinned.current = false;
    setSelectedFightIds(defaultSelectedFightIds(meter));
  }

  const liveDuration = useLiveDuration(
    displayedFight?.started_at,
    displayedFight?.duration_secs ?? 0,
    displayedFight?.active ?? false
  );

  const activeLiveDuration = useLiveDuration(
    meter.active_fight?.started_at,
    meter.active_fight?.duration_secs ?? 0,
    Boolean(meter.active_fight?.active)
  );

  const liveGroupDps = displayedFight
    ? liveRate(displayedFight.total_damage, liveDuration)
    : 0;
  const liveActiveDps = meter.active_fight
    ? liveRate(meter.active_fight.total_damage, activeLiveDuration)
    : 0;
  const liveHps = displayedFight
    ? liveRate(displayedFight.healing, liveDuration)
    : 0;
  const liveDtps = displayedFight
    ? liveRate(displayedFight.damage_taken, liveDuration)
    : 0;

  useEffect(() => {
    if (!displayedFight) {
      setSelectedPlayer(null);
      return;
    }
    if (
      selectedPlayer &&
      displayedFight.players.some((p) => p.name === selectedPlayer)
    ) {
      return;
    }
    const selfName = meter.character;
    if (selfName && displayedFight.players.some((p) => p.name === selfName)) {
      setSelectedPlayer(selfName);
      return;
    }
    setSelectedPlayer(displayedFight.players[0]?.name ?? null);
  }, [displayedFight, meter.character, selectedPlayer]);

  const selectedPlayerStat = useMemo(() => {
    if (!displayedFight || !selectedPlayer) return null;
    return displayedFight.players.find((p) => p.name === selectedPlayer) ?? null;
  }, [displayedFight, selectedPlayer]);
  const selectedPetDamage = useMemo(() => {
    if (!selectedPlayerStat) return 0;
    return selectedPlayerStat.abilities
      .filter((a) => a.name.startsWith("Pet ("))
      .reduce((sum, a) => sum + a.damage, 0);
  }, [selectedPlayerStat]);

  const openLog = useCallback(async () => {
    setError(null);
    setBusy(true);
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "EQ Log", extensions: ["txt"] }],
      });
      if (!selected || Array.isArray(selected)) {
        setBusy(false);
        return;
      }
      await startPath(selected, false);
    } catch (err) {
      setError(String(err));
      setBusy(false);
    }
  }, [startPath]);

  const autoDetect = useCallback(async () => {
    setError(null);
    setBusy(true);
    try {
      const found = await invoke<{ path: string; character: string | null }>(
        "auto_detect_log"
      );
      await startPath(found.path, false);
    } catch (err) {
      setError(String(err));
      setBusy(false);
    }
  }, [startPath]);

  const autoDetectParallels = useCallback(async () => {
    setError(null);
    setBusy(true);
    try {
      const found = await invoke<{
        path: string;
        character: string | null;
        source: string;
      }>("auto_detect_parallels_log");
      await startPath(found.path, false);
      setToast(
        `Live monitoring Parallels log${
          found.character ? ` · ${found.character}` : ""
        }`
      );
    } catch (err) {
      setError(String(err));
      setBusy(false);
    }
  }, [startPath]);

  const stop = useCallback(async () => {
    setBusy(true);
    try {
      const next = await invoke<MeterState>("stop_monitoring");
      setMeter(next);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }, []);

  const clearFights = useCallback(async () => {
    const next = await invoke<MeterState>("clear_fights");
    setMeter(next);
    selectionPinned.current = false;
    prevLiveCount.current = next.active_fights.length;
    setSelectedFightIds(defaultSelectedFightIds(next));
    setCombinedFight(null);
    setSelectedPlayer(null);
  }, []);

  const removeFight = useCallback(async (fightId: number) => {
    setFightContextMenu(null);
    const next = await invoke<MeterState>("remove_fight", { fightId });
    setMeter(next);
    prevLiveCount.current = next.active_fights.length;
    setSelectedFightIds((prev) => {
      const remaining = prev.filter((id) => id !== fightId && id !== 0);
      if (remaining.length > 0) {
        return remaining;
      }
      selectionPinned.current = false;
      return defaultSelectedFightIds(next);
    });
    setCombinedFight(null);
  }, []);

  const toggleOverlay = useCallback(async () => {
    try {
      const status = await invoke<{ open: boolean; click_through: boolean }>(
        "toggle_overlay"
      );
      setOverlayOpen(status.open);
      setOverlayClickThrough(status.click_through);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const setClickThrough = useCallback(async (enabled: boolean) => {
    try {
      const status = await invoke<{ open: boolean; click_through: boolean }>(
        "set_overlay_click_through",
        { enabled }
      );
      setOverlayOpen(status.open);
      setOverlayClickThrough(status.click_through);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const copyParse = useCallback(async () => {
    try {
      const ids = selectedFightIds.filter((id) => id !== 0);
      const text = await invoke<string>("copy_parse", {
        fightId:
          selectedFightIds.length === 1 ? selectedFightIds[0] : null,
        fightIds: ids.length > 1 ? ids : null,
      });
      setToast(`Copied: ${text}`);
    } catch (err) {
      setError(String(err));
    }
  }, [selectedFightIds]);

  useEffect(() => {
    if (!menuOpen) return;

    function onPointerDown(event: MouseEvent) {
      if (!menuRef.current) return;
      if (!menuRef.current.contains(event.target as Node)) {
        setMenuOpen(false);
      }
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setMenuOpen(false);
      }
    }

    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [menuOpen]);

  useEffect(() => {
    if (!fightContextMenu) return;

    function onPointerDown() {
      setFightContextMenu(null);
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setFightContextMenu(null);
      }
    }

    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [fightContextMenu]);

  function openFightContextMenu(
    fightId: number,
    event: ReactMouseEvent<HTMLButtonElement>
  ) {
    event.preventDefault();
    event.stopPropagation();
    setMenuOpen(false);
    setFightContextMenu({
      x: event.clientX,
      y: event.clientY,
      fightId,
    });
  }

  function runMenuAction(action: () => void | Promise<void>) {
    setMenuOpen(false);
    void action();
  }

  const maxDamage = displayedFight?.players[0]?.damage ?? 1;
  const maxType = displayedFight?.damage_types[0]?.damage ?? 1;
  const healPlayers = useMemo(() => {
    if (!displayedFight) return [];
    return [...displayedFight.players]
      .filter((p) => p.healing > 0)
      .sort((a, b) => {
        if (b.healing !== a.healing) return b.healing - a.healing;
        return a.name.localeCompare(b.name);
      });
  }, [displayedFight]);
  const recvPlayers = useMemo(() => {
    if (!displayedFight) return [];
    return [...displayedFight.players]
      .filter((p) => p.healing_received > 0)
      .sort((a, b) => {
        if (b.healing_received !== a.healing_received) {
          return b.healing_received - a.healing_received;
        }
        return a.name.localeCompare(b.name);
      });
  }, [displayedFight]);
  const groupRows = useMemo(
    () =>
      displayedFight
        ? groupDamageRows(displayedFight, meter.raid_roster)
        : [],
    [displayedFight, meter.raid_roster]
  );
  const groupLookup = useMemo(
    () => rosterGroupMap(meter.raid_roster),
    [meter.raid_roster]
  );
  const maxHeal = healPlayers[0]?.healing ?? 1;
  const maxRecv = recvPlayers[0]?.healing_received ?? 1;
  const maxGroupDamage = groupRows[0]?.damage ?? 1;
  const maxHealSpell = displayedFight?.heal_spells[0]?.damage ?? 1;
  const isHealsTab = meterTab === "heals";
  const isRaidTab = meterTab === "raid";
  const isMiscTab = meterTab === "misc";
  const isCombatTab = meterTab === "dps" || meterTab === "heals";
  const showMeterShell =
    Boolean(displayedFight) || isRaidTab || isMiscTab || meter.monitoring;
  const filteredMisc = meter.misc_log.filter((entry) => {
    if (miscFilter === "all") return true;
    return entry.kind === miscFilter;
  });

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand-block">
          <p className="brand">EQL Meter</p>
          <p className="tagline">Live combat for EverQuest Legends</p>
        </div>

        <div className="status-block">
          <div
            className={`pulse ${meter.monitoring ? "on" : "off"}`}
            aria-hidden
          />
          <div>
            <p className="status-label">
              {meter.monitoring ? "Monitoring" : "Idle"}
              {meter.character ? ` · ${meter.character}` : ""}
              {meter.server
                ? ` · ${meter.server.charAt(0).toUpperCase()}${meter.server.slice(1)}`
                : ""}
            </p>
            <p className="status-path" title={meter.log_path ?? undefined}>
              {shortPath(meter.log_path)}
            </p>
          </div>
        </div>

        <div className="actions">
          {isMac ? (
            <button
              type="button"
              className="btn primary"
              disabled={busy}
              onClick={autoDetectParallels}
              title="Monitor the live eqlog from your Parallels Windows VM"
            >
              Live Parallels
            </button>
          ) : (
            <button
              type="button"
              className="btn primary"
              disabled={busy}
              onClick={autoDetect}
              title="Find eqlog_*.txt under the EverQuest Legends Logs folder"
            >
              Auto-detect
            </button>
          )}
          <button
            type="button"
            className="btn"
            disabled={!displayedFight}
            onClick={copyParse}
            title="Copy a compact parse to the clipboard for chat"
          >
            Copy Parse
          </button>
          <button
            type="button"
            className={`btn ${overlayOpen ? "primary" : ""}`}
            onClick={toggleOverlay}
          >
            {overlayOpen ? "Close Overlay" : "Overlay"}
          </button>

          <div className="menu" ref={menuRef}>
            <button
              type="button"
              className={`btn menu-trigger ${menuOpen ? "open" : ""}`}
              aria-haspopup="menu"
              aria-expanded={menuOpen}
              onClick={() => setMenuOpen((open) => !open)}
            >
              Menu
              <span className="menu-caret" aria-hidden>
                ▾
              </span>
            </button>

            {menuOpen ? (
              <div className="menu-panel" role="menu">
                <button
                  type="button"
                  role="menuitem"
                  disabled={busy}
                  onClick={() => runMenuAction(() => openLog())}
                >
                  Choose log…
                </button>
                <button
                  type="button"
                  role="menuitem"
                  disabled={!meter.monitoring || busy}
                  onClick={() => runMenuAction(stop)}
                >
                  Stop monitoring
                </button>
                <button
                  type="button"
                  role="menuitem"
                  disabled={!overlayOpen}
                  onClick={() =>
                    runMenuAction(() => setClickThrough(!overlayClickThrough))
                  }
                  title="Ctrl/Cmd+Shift+U clickable · L click-through"
                >
                  {overlayClickThrough ? "Make overlay clickable" : "Click-through to game"}
                </button>
                <div className="menu-divider" />
                <button
                  type="button"
                  role="menuitem"
                  disabled={updateBusy}
                  onClick={() => runMenuAction(runUpdateCheck)}
                >
                  {updateBusy ? "Checking…" : "Check for updates"}
                </button>
                {pendingUpdate ? (
                  <button
                    type="button"
                    role="menuitem"
                    className="menu-item-accent"
                    disabled={updateBusy}
                    onClick={() => runMenuAction(runInstallUpdate)}
                  >
                    Install {pendingUpdate.version}
                  </button>
                ) : null}
                {appVersion ? (
                  <p className="menu-footer">Version {appVersion}</p>
                ) : null}
              </div>
            ) : null}
          </div>
        </div>
      </header>

      {error ? <div className="error-banner">{error}</div> : null}
      {pendingUpdate ? (
        <div className="update-banner">
          <span>
            Update <strong>{pendingUpdate.version}</strong> is available
            {appVersion ? ` (you have ${appVersion})` : ""}.
          </span>
          <button
            type="button"
            className="btn primary"
            disabled={updateBusy}
            onClick={() => void runInstallUpdate()}
          >
            {updateBusy ? "Updating…" : "Install update"}
          </button>
          <button
            type="button"
            className="btn"
            disabled={updateBusy}
            onClick={() => void openLatestReleasePage()}
          >
            Release notes
          </button>
        </div>
      ) : null}
      {toast ? <div className="toast-banner">{toast}</div> : null}

      <div className="layout">
        <aside className="fight-rail">
          <div className="fight-rail-head">
            <h2>Fights</h2>
            <div className="fight-rail-actions">
              <button
                type="button"
                className="linkish"
                onClick={selectAllVisible}
                disabled={
                  meter.active_fights.length + meter.recent_fights.length < 2
                }
              >
                Select all
              </button>
              {selectedFightIds.length > 1 ? (
                <button
                  type="button"
                  className="linkish"
                  onClick={clearFightSelection}
                >
                  Deselect
                </button>
              ) : null}
              <button
                type="button"
                className="linkish"
                onClick={() => void clearFights()}
                disabled={
                  meter.active_fights.length + meter.recent_fights.length === 0
                }
              >
                Clear all
              </button>
            </div>
          </div>
          <p className="fight-rail-hint">
            {showingCombined
              ? selectedFightIds.length > 1
                ? `${selectedFightIds.length} selected · combined summary`
                : `Live combined · ${meter.active_fights.length} mobs`
              : "Cmd/Ctrl+click to combine · right-click to delete"}
          </p>
          <div className="fight-list">
            {meter.active_fights.length > 1 && meter.active_fight ? (
              <button
                type="button"
                className={`fight-item active-live ${
                  isFightHighlighted(0) ? "selected" : ""
                }`}
                onClick={(event) => selectFight(0, event)}
              >
                <span className="fight-target">{meter.active_fight.target}</span>
                <span className="fight-meta">
                  LIVE · {formatDuration(activeLiveDuration)} ·{" "}
                  {formatDps(liveActiveDps)} DPS
                </span>
              </button>
            ) : null}

            {meter.active_fights.map((fight) => (
              <button
                key={fight.id}
                type="button"
                className={`fight-item active-live ${
                  isFightHighlighted(fight.id) ? "selected" : ""
                }`}
                onClick={(event) => selectFight(fight.id, event)}
                onContextMenu={(event) => openFightContextMenu(fight.id, event)}
              >
                <span className="fight-target">{fight.target}</span>
                <span className="fight-meta">
                  LIVE · {formatDuration(fight.duration_secs)} ·{" "}
                  {formatDps(fight.total_dps)} DPS
                </span>
              </button>
            ))}

            {meter.recent_fights.map((fight) => (
              <button
                key={fight.id}
                type="button"
                className={`fight-item ${
                  isFightHighlighted(fight.id) ? "selected" : ""
                }`}
                onClick={(event) => selectFight(fight.id, event)}
                onContextMenu={(event) => openFightContextMenu(fight.id, event)}
              >
                <span className="fight-target">{fight.target}</span>
                <span className="fight-meta">
                  {formatDuration(fight.duration_secs)} ·{" "}
                  {formatDps(fight.total_dps)} DPS
                </span>
              </button>
            ))}

            {meter.active_fights.length === 0 &&
            meter.recent_fights.length === 0 ? (
              <p className="empty">
                Open your character log to watch DPS update in real time.
              </p>
            ) : null}
          </div>
          {fightContextMenu ? (
            <div
              className="fight-context-menu"
              style={{
                left: fightContextMenu.x,
                top: fightContextMenu.y,
              }}
              onMouseDown={(event) => event.stopPropagation()}
            >
              <button
                type="button"
                onClick={() => void removeFight(fightContextMenu.fightId)}
              >
                Delete fight
              </button>
            </div>
          ) : null}
        </aside>

        <main className="meter-panel">
          {showMeterShell ? (
            <>
              {displayedFight && !isRaidTab && !isMiscTab ? (
                <div className="fight-header">
                  <div>
                    <p className="eyebrow">
                      {showingCombined
                        ? `Combined · ${
                            selectedFightIds.length > 1
                              ? selectedFightIds.length
                              : meter.active_fights.length
                          } fights · ${formatDuration(liveDuration)}`
                        : displayedFight.active
                          ? `Active combat · ${formatDuration(liveDuration)}`
                          : "Combat"}
                    </p>
                    <h1>{displayedFight.target}</h1>
                    {displayedFight.kills > 0 ? (
                      <p className="fight-kills">
                        {displayedFight.kills} kill
                        {displayedFight.kills === 1 ? "" : "s"}
                      </p>
                    ) : null}
                  </div>
                  <div className="fight-stats">
                    {isHealsTab ? (
                      <>
                        <div>
                          <span>Duration</span>
                          <strong>{formatDuration(liveDuration)}</strong>
                        </div>
                        <div>
                          <span>Healed</span>
                          <strong>{formatNumber(displayedFight.healing)}</strong>
                        </div>
                        <div>
                          <span>Group HPS</span>
                          <strong>{formatDps(liveHps)}</strong>
                        </div>
                        <div>
                          <span>Overheal</span>
                          <strong>
                            {formatNumber(displayedFight.overheal)} ·{" "}
                            {displayedFight.overheal_pct.toFixed(0)}%
                          </strong>
                        </div>
                      </>
                    ) : (
                      <>
                        <div>
                          <span>Duration</span>
                          <strong>{formatDuration(liveDuration)}</strong>
                        </div>
                        <div>
                          <span>Total</span>
                          <strong>
                            {formatNumber(displayedFight.total_damage)}
                          </strong>
                        </div>
                        <div>
                          <span>Group DPS</span>
                          <strong>{formatDps(liveGroupDps)}</strong>
                        </div>
                        <div>
                          <span>Peak DPS</span>
                          <strong>{formatDps(displayedFight.peak_dps)}</strong>
                        </div>
                      </>
                    )}
                  </div>
                </div>
              ) : null}

              <div className="meter-tabs">
                <button
                  type="button"
                  className={meterTab === "dps" ? "active" : ""}
                  onClick={() => setMeterTab("dps")}
                >
                  DPS
                </button>
                <button
                  type="button"
                  className={meterTab === "heals" ? "active" : ""}
                  onClick={() => setMeterTab("heals")}
                >
                  Heals
                </button>
                <button
                  type="button"
                  className={meterTab === "raid" ? "active" : ""}
                  onClick={() => setMeterTab("raid")}
                >
                  Raid
                </button>
                <button
                  type="button"
                  className={meterTab === "misc" ? "active" : ""}
                  onClick={() => setMeterTab("misc")}
                >
                  Misc
                </button>
              </div>

              {isRaidTab ? (
                <div className="raid-panel">
                  <p className="eyebrow">
                    From /who all raid · type it in-game to refresh
                  </p>
                  {meter.raid_roster && meter.raid_roster.players.length > 0 ? (
                    <>
                      <h3>
                        {meter.raid_roster.players.length} players ·{" "}
                        {
                          new Set(
                            meter.raid_roster.players
                              .map((p) => p.group)
                              .filter((g) => g > 0)
                          ).size
                        }{" "}
                        groups
                      </h3>
                      <div className="raid-groups">
                        {Array.from(
                          new Set(
                            meter.raid_roster.players.map((p) => p.group)
                          )
                        )
                          .sort((a, b) => a - b)
                          .map((group) => {
                            const members = meter.raid_roster!.players.filter(
                              (p) => p.group === group
                            );
                            return (
                              <div key={group} className="raid-group">
                                <h4>
                                  {group === 0 ? "Ungrouped" : `Group ${group}`}
                                </h4>
                                <ul>
                                  {members.map((p) => (
                                    <li key={p.name}>
                                      <span>{p.name}</span>
                                      <span className="raid-meta">
                                        {p.level} {p.class_name}
                                      </span>
                                    </li>
                                  ))}
                                </ul>
                              </div>
                            );
                          })}
                      </div>
                    </>
                  ) : (
                    <p className="empty">
                      No raid roster yet. In EQ, run{" "}
                      <code>/who all raid</code> while logging is on.
                    </p>
                  )}
                </div>
              ) : null}

              {isMiscTab ? (
                <div className="misc-panel">
                  <p className="eyebrow">Loot · randoms · chat from the log</p>
                  <div className="misc-filters">
                    {(
                      [
                        ["all", "All"],
                        ["loot", "Loot"],
                        ["random", "Random"],
                        ["roll", "Rolls"],
                        ["chat", "Chat"],
                      ] as const
                    ).map(([value, label]) => (
                      <button
                        key={value}
                        type="button"
                        className={miscFilter === value ? "active" : ""}
                        onClick={() => setMiscFilter(value)}
                      >
                        {label}
                      </button>
                    ))}
                  </div>
                  {filteredMisc.length > 0 ? (
                    <ul className="misc-list">
                      {filteredMisc.map((entry, index) => (
                        <li key={`${entry.timestamp}-${entry.kind}-${index}`}>
                          <span className={`misc-kind ${entry.kind}`}>
                            {entry.kind}
                          </span>
                          <div className="misc-body">
                            <span className="misc-summary">{entry.summary}</span>
                            <span className="misc-time">{entry.timestamp}</span>
                          </div>
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p className="empty">
                      No loot, rolls, or chat captured yet. Keep monitoring
                      while the raid runs.
                    </p>
                  )}
                </div>
              ) : null}

              {isCombatTab && displayedFight ? (
                <>
              {isHealsTab ? (
                <div className="stat-strip">
                  <div className="stat-chip">
                    <span>Effective</span>
                    <strong>{formatNumber(displayedFight.healing)}</strong>
                  </div>
                  <div className="stat-chip">
                    <span>Overheal</span>
                    <strong>
                      {formatNumber(displayedFight.overheal)} (
                      {displayedFight.overheal_pct.toFixed(0)}%)
                    </strong>
                  </div>
                  <div className="stat-chip">
                    <span>HPS</span>
                    <strong>{formatDps(liveHps)}</strong>
                  </div>
                  <div className="stat-chip">
                    <span>Healers</span>
                    <strong>
                      {
                        displayedFight.players.filter((p) => p.healing > 0)
                          .length
                      }
                    </strong>
                  </div>
                </div>
              ) : (
              <div className="stat-strip">
                <div className="stat-chip">
                  <span>Hits</span>
                  <strong>{formatNumber(displayedFight.total_hits)}</strong>
                </div>
                <div className="stat-chip">
                  <span>Accuracy</span>
                  <strong>
                    {displayedFight.attempts > 0
                      ? `${displayedFight.accuracy_pct.toFixed(0)}%`
                      : "—"}
                  </strong>
                </div>
                <div className="stat-chip">
                  <span>Crits</span>
                  <strong>
                    {formatNumber(displayedFight.crits)} (
                    {displayedFight.crit_pct.toFixed(1)}%)
                  </strong>
                </div>
                <div className="stat-chip">
                  <span>Max hit</span>
                  <strong>
                    {formatNumber(displayedFight.max_hit)}
                    {displayedFight.max_hit_by
                      ? ` · ${displayedFight.max_hit_by}`
                      : ""}
                  </strong>
                </div>
                <div className="stat-chip">
                  <span>Taken</span>
                  <strong>
                    {formatNumber(displayedFight.damage_taken)} ·{" "}
                    {formatDps(liveDtps)} DTPS
                  </strong>
                </div>
                <div className="stat-chip">
                  <span>Defense</span>
                  <strong>
                    {displayedFight.dodges +
                      displayedFight.parries +
                      displayedFight.blocks +
                      displayedFight.ripostes >
                    0
                      ? `${displayedFight.dodges}d ${displayedFight.parries}p ${displayedFight.blocks}b ${displayedFight.ripostes}r`
                      : "—"}
                  </strong>
                </div>
                <div className="stat-chip">
                  <span>Resists</span>
                  <strong>{formatNumber(displayedFight.resists)}</strong>
                </div>
              </div>
              )}

              <DamageChart
                fight={displayedFight}
                selectedPlayer={selectedPlayer}
                durationSecs={liveDuration}
                mode={isHealsTab ? "heals" : "dps"}
              />

              {!isHealsTab ? (
                <ComparePulls
                  fights={meter.recent_fights}
                  target={displayedFight.target}
                />
              ) : null}

              {isHealsTab ? (
                <>
                  {displayedFight.heal_spells.length > 0 ? (
                    <div className="type-mix">
                      <h3>Heal mix</h3>
                      <div className="type-list">
                        {displayedFight.heal_spells.map((spell) => {
                          const width = Math.max(
                            4,
                            (spell.damage / Math.max(maxHealSpell, 1)) * 100
                          );
                          return (
                            <div key={spell.name} className="type-row">
                              <div className="type-top">
                                <span>{spell.name}</span>
                                <span>
                                  {formatNumber(spell.damage)} ·{" "}
                                  {spell.pct.toFixed(1)}% · {spell.hits} casts
                                </span>
                              </div>
                              <div className="bar-track thin">
                                <div
                                  className="bar-fill type-heal"
                                  style={{ width: `${width}%` }}
                                />
                              </div>
                            </div>
                          );
                        })}
                      </div>
                    </div>
                  ) : (
                    <p className="empty">
                      No healing logged for this combat yet.
                    </p>
                  )}

                  <div className="player-bars">
                    <h3>Healers</h3>
                    {healPlayers.map((player, index) => {
                      const width = Math.max(
                        4,
                        (player.healing / Math.max(maxHeal, 1)) * 100
                      );
                      const isSelf =
                        meter.character != null &&
                        player.name === meter.character;
                      const isSelected = player.name === selectedPlayer;
                      const playerHps = displayedFight.active
                        ? liveRate(player.healing, liveDuration)
                        : player.hps;
                      const group = groupLookup.get(player.name.toLowerCase());

                      return (
                        <button
                          key={player.name}
                          type="button"
                          className={`player-row ${isSelf ? "self" : ""} ${
                            isSelected ? "selected" : ""
                          }`}
                          onClick={() => setSelectedPlayer(player.name)}
                        >
                          <div className="player-rank">{index + 1}</div>
                          <div className="player-main">
                            <div className="player-top">
                              <span className="player-name">
                                {group != null && group > 0
                                  ? `G${group} · `
                                  : ""}
                                {player.name}
                                {isSelf ? " (you)" : ""}
                              </span>
                              <span className="player-nums">
                                {formatNumber(player.healing)} ·{" "}
                                {formatDps(playerHps)} HPS ·{" "}
                                {player.heal_pct.toFixed(1)}%
                                {player.overheal > 0
                                  ? ` · ${formatNumber(player.overheal)} OH`
                                  : ""}
                              </span>
                            </div>
                            <div className="bar-track">
                              <div
                                className="bar-fill heal"
                                style={{ width: `${width}%` }}
                              />
                            </div>
                          </div>
                        </button>
                      );
                    })}
                  </div>

                  {recvPlayers.length > 0 ? (
                    <div className="player-bars">
                      <h3>Healing received</h3>
                      <p className="eyebrow">
                        Effective heals landed on each player this fight
                      </p>
                      {recvPlayers.map((player, index) => {
                        const width = Math.max(
                          4,
                          (player.healing_received / Math.max(maxRecv, 1)) * 100
                        );
                        const isSelf =
                          meter.character != null &&
                          player.name === meter.character;
                        const isSelected = player.name === selectedPlayer;
                        const group = groupLookup.get(
                          player.name.toLowerCase()
                        );

                        return (
                          <button
                            key={`recv-${player.name}`}
                            type="button"
                            className={`player-row ${isSelf ? "self" : ""} ${
                              isSelected ? "selected" : ""
                            }`}
                            onClick={() => setSelectedPlayer(player.name)}
                          >
                            <div className="player-rank">{index + 1}</div>
                            <div className="player-main">
                              <div className="player-top">
                                <span className="player-name">
                                  {group != null && group > 0
                                    ? `G${group} · `
                                    : ""}
                                  {player.name}
                                  {isSelf ? " (you)" : ""}
                                </span>
                                <span className="player-nums">
                                  {formatNumber(player.healing_received)}
                                </span>
                              </div>
                              <div className="bar-track">
                                <div
                                  className="bar-fill heal"
                                  style={{ width: `${width}%` }}
                                />
                              </div>
                            </div>
                          </button>
                        );
                      })}
                    </div>
                  ) : null}
                </>
              ) : (
                <>
              {displayedFight.targets.length > 0 ? (
                <div className="type-mix">
                  <h3>Mobs</h3>
                  <div className="type-list">
                    {displayedFight.targets.map((mob) => {
                      const width = Math.max(
                        4,
                        (mob.damage /
                          Math.max(displayedFight.total_damage, 1)) *
                          100
                      );
                      return (
                        <div key={mob.name} className="type-row">
                          <div className="type-top">
                            <span>{mob.name}</span>
                            <span>
                              {formatNumber(mob.damage)} · {mob.pct.toFixed(1)}%
                              {mob.hits > 0 ? ` · ${mob.hits} hits` : ""}
                            </span>
                          </div>
                          <div className="bar-track thin">
                            <div
                              className="bar-fill type-melee"
                              style={{ width: `${width}%` }}
                            />
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              ) : null}

              {displayedFight.damage_types.length > 0 ? (
                <div className="type-mix">
                  <h3>Damage mix</h3>
                  <div className="type-list">
                    {displayedFight.damage_types.map((type) => {
                      const width = Math.max(
                        4,
                        (type.damage / maxType) * 100
                      );
                      return (
                        <div key={type.name} className="type-row">
                          <div className="type-top">
                            <span>{type.name}</span>
                            <span>
                              {formatNumber(type.damage)} · {type.pct.toFixed(1)}%
                            </span>
                          </div>
                          <div className="bar-track thin">
                            <div
                              className={`bar-fill type-${type.name.toLowerCase()}`}
                              style={{ width: `${width}%` }}
                            />
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              ) : null}

              {displayedFight.taken_sources.length > 0 ? (
                <div className="type-mix">
                  <h3>Damage taken</h3>
                  <p className="taken-summary">
                    {formatNumber(displayedFight.damage_taken)} total ·{" "}
                    {displayedFight.taken_hits} hits · max{" "}
                    {formatNumber(displayedFight.max_taken_hit)}
                  </p>
                  <div className="type-list">
                    {displayedFight.taken_sources.map((source) => {
                      const width = Math.max(
                        4,
                        (source.damage /
                          Math.max(displayedFight.damage_taken, 1)) *
                          100
                      );
                      return (
                        <div key={source.name} className="type-row">
                          <div className="type-top">
                            <span>{source.name}</span>
                            <span>
                              {formatNumber(source.damage)} · {source.hits} hits
                            </span>
                          </div>
                          <div className="bar-track thin">
                            <div
                              className="bar-fill type-taken"
                              style={{ width: `${width}%` }}
                            />
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              ) : null}

              <div className="player-bars">
                <h3>Contributors</h3>
                {groupRows.length > 0 ? (
                  <div className="group-dps">
                    <h4>By raid group</h4>
                    <p className="eyebrow">
                      From latest /who all raid · unmatched players omitted
                    </p>
                    {groupRows.map((row) => {
                      const width = Math.max(
                        4,
                        (row.damage / Math.max(maxGroupDamage, 1)) * 100
                      );
                      const groupDps = displayedFight.active
                        ? liveRate(row.damage, liveDuration)
                        : row.damage / Math.max(liveDuration, 1);
                      return (
                        <div key={row.group} className="type-row">
                          <div className="type-top">
                            <span>
                              {row.label} · {row.players} players
                            </span>
                            <span>
                              {formatNumber(row.damage)} · {formatDps(groupDps)}{" "}
                              DPS · {row.pct.toFixed(1)}%
                            </span>
                          </div>
                          <div className="bar-track thin">
                            <div
                              className="bar-fill type-melee"
                              style={{ width: `${width}%` }}
                            />
                          </div>
                        </div>
                      );
                    })}
                  </div>
                ) : null}
                {displayedFight.players.map((player, index) => {
                  const width = Math.max(
                    4,
                    (player.damage / Math.max(maxDamage, 1)) * 100
                  );
                  const isSelf =
                    meter.character != null &&
                    player.name === meter.character;
                  const isSelected = player.name === selectedPlayer;
                  const playerDps = displayedFight.active
                    ? liveRate(player.damage, liveDuration)
                    : player.dps;
                  const group = groupLookup.get(player.name.toLowerCase());

                  return (
                    <button
                      key={player.name}
                      type="button"
                      className={`player-row ${isSelf ? "self" : ""} ${
                        isSelected ? "selected" : ""
                      }`}
                      onClick={() => setSelectedPlayer(player.name)}
                    >
                      <div className="player-rank">{index + 1}</div>
                      <div className="player-main">
                        <div className="player-top">
                          <span className="player-name">
                            {group != null && group > 0 ? `G${group} · ` : ""}
                            {player.name}
                            {isSelf ? " (you)" : ""}
                          </span>
                          <span className="player-nums">
                            {formatNumber(player.damage)} ·{" "}
                            {formatDps(playerDps)} DPS · {player.pct.toFixed(1)}%
                            {player.attempts > 0
                              ? ` · ${player.accuracy_pct.toFixed(0)}% acc`
                              : ""}
                            {player.healing > 0
                              ? ` · ${formatNumber(player.healing)} heal`
                              : ""}
                          </span>
                        </div>
                        <div className="bar-track">
                          <div
                            className="bar-fill"
                            style={{ width: `${width}%` }}
                          />
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>
                </>
              )}
                </>
              ) : isCombatTab ? (
                <p className="empty">
                  Waiting for combat. Raid and Misc tabs work anytime while
                  monitoring.
                </p>
              ) : null}
            </>
          ) : (
            <div className="hero-empty">
              <p className="brand-large">EQL Meter</p>
              <p>
                {isMac
                  ? "Keep your Parallels Windows VM running so its C: drive is mounted, then click Live Parallels to follow eqlog_*.txt under EverQuest Legends\\Logs in real time."
                  : "Click Auto-detect to find your character log under EverQuest Legends\\Logs, or choose the file manually. Combat updates live while you play."}
              </p>
              <div className="hero-actions">
                {isMac ? (
                  <button
                    type="button"
                    className="btn primary"
                    disabled={busy}
                    onClick={autoDetectParallels}
                  >
                    Live Parallels Log
                  </button>
                ) : (
                  <button
                    type="button"
                    className="btn primary"
                    disabled={busy}
                    onClick={autoDetect}
                  >
                    Auto-detect Log
                  </button>
                )}
                <button
                  type="button"
                  className="btn"
                  disabled={busy}
                  onClick={() => openLog()}
                >
                  Choose Log File
                </button>
              </div>
            </div>
          )}
        </main>

        <aside className="breakdown-panel">
          <h2>Breakdown</h2>
          {selectedPlayerStat ? (
            <>
              <p className="breakdown-name">{selectedPlayerStat.name}</p>
              <p className="breakdown-summary">
                {isHealsTab ? (
                  <>
                    {formatNumber(selectedPlayerStat.healing)} healed ·{" "}
                    {formatDps(
                      displayedFight?.active
                        ? liveRate(selectedPlayerStat.healing, liveDuration)
                        : selectedPlayerStat.hps
                    )}{" "}
                    HPS · {selectedPlayerStat.heal_pct.toFixed(1)}%
                  </>
                ) : (
                  <>
                    {formatNumber(selectedPlayerStat.damage)} damage ·{" "}
                    {selectedPlayerStat.hits} hits ·{" "}
                    {formatDps(
                      displayedFight?.active
                        ? liveRate(selectedPlayerStat.damage, liveDuration)
                        : selectedPlayerStat.dps
                    )}{" "}
                    DPS
                  </>
                )}
              </p>
              <p className="breakdown-summary secondary">
                {isHealsTab ? (
                  <>
                    Overheal {formatNumber(selectedPlayerStat.overheal)}
                    {selectedPlayerStat.damage > 0
                      ? ` · Also ${formatNumber(selectedPlayerStat.damage)} dmg`
                      : ""}
                  </>
                ) : (
                  <>
                    Crits {selectedPlayerStat.crits} · Max hit{" "}
                    {formatNumber(selectedPlayerStat.max_hit)}
                    {selectedPlayerStat.attempts > 0
                      ? ` · Acc ${selectedPlayerStat.accuracy_pct.toFixed(0)}%`
                      : ""}
                    {selectedPetDamage > 0
                      ? ` · Pet ${formatNumber(selectedPetDamage)}`
                      : ""}
                    {selectedPlayerStat.healing > 0
                      ? ` · Heal ${formatNumber(selectedPlayerStat.healing)}`
                      : ""}
                  </>
                )}
              </p>
              <div className="ability-list">
                {selectedPlayerStat.abilities
                  .filter((ability) =>
                    isHealsTab ? ability.healing > 0 : ability.damage > 0
                  )
                      .map((ability) => {
                    const amount = isHealsTab
                      ? ability.healing
                      : ability.damage;
                    const total = isHealsTab
                      ? Math.max(selectedPlayerStat.healing, 1)
                      : Math.max(selectedPlayerStat.damage, 1);
                    const width = Math.max(4, (amount / total) * 100);
                    const isPet = ability.name.startsWith("Pet (");
                    let fillClass = "ability";
                    if (isHealsTab) {
                      fillClass = "heal";
                    } else if (isPet) {
                      fillClass = "pet";
                    }
                    return (
                      <div
                        key={ability.name}
                        className={`ability-row ${isPet ? "pet" : ""}`}
                      >
                        <div className="ability-top">
                          <span>{ability.name}</span>
                          <span>
                            {formatNumber(amount)} · {ability.hits}{" "}
                            {isHealsTab ? "casts" : "hits"}
                          </span>
                        </div>
                        <div className="bar-track thin">
                          <div
                            className={`bar-fill ${fillClass}`}
                            style={{ width: `${width}%` }}
                          />
                        </div>
                      </div>
                    );
                  })}
              </div>
            </>
          ) : (
            <p className="empty">
              Select a player to see ability {isHealsTab ? "heals" : "damage"}.
            </p>
          )}
        </aside>
      </div>
    </div>
  );
}

export default App;
