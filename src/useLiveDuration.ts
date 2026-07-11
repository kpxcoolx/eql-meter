import { useEffect, useState } from "react";

/** Live-ticking duration for active fights; frozen value for finished ones. */
export function useLiveDuration(
  startedAt: number | null | undefined,
  durationSecs: number,
  active: boolean
): number {
  const [now, setNow] = useState(() => Date.now() / 1000);

  useEffect(() => {
    if (!active || startedAt == null) return;
    setNow(Date.now() / 1000);
    const id = window.setInterval(() => {
      setNow(Date.now() / 1000);
    }, 250);
    return () => window.clearInterval(id);
  }, [active, startedAt]);

  if (!active || startedAt == null) {
    return durationSecs;
  }

  return Math.max(durationSecs, now - startedAt, 1);
}

export function liveRate(total: number, durationSecs: number): number {
  return total / Math.max(durationSecs, 1);
}

export function formatDuration(secs: number): string {
  const total = Math.max(0, Math.floor(secs));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}
