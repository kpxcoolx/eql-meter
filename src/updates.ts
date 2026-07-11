import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";

export type PendingUpdate = {
  version: string;
  notes: string;
};

const RELEASES_PAGE = "https://github.com/kpxcoolx/eql-meter/releases/latest";
const CHECK_TIMEOUT_MS = 12_000;

function withTimeout<T>(promise: Promise<T>, ms: number, label: string): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = window.setTimeout(() => {
      reject(new Error(`${label} timed out. Try Open latest release instead.`));
    }, ms);
    promise.then(
      (value) => {
        window.clearTimeout(timer);
        resolve(value);
      },
      (err) => {
        window.clearTimeout(timer);
        reject(err);
      }
    );
  });
}

export async function checkForAppUpdate(): Promise<PendingUpdate | null> {
  let update: Update | null;
  try {
    update = await withTimeout(
      check({ timeout: CHECK_TIMEOUT_MS }),
      CHECK_TIMEOUT_MS + 1000,
      "Update check"
    );
  } catch (err) {
    const raw = String(err);
    if (
      raw.includes("valid release JSON") ||
      raw.includes("Could not fetch") ||
      raw.includes("404")
    ) {
      throw new Error(
        "Update feed not ready yet (latest.json missing). Use Menu → Latest release… to download the installer."
      );
    }
    throw err;
  }
  if (!update) {
    return null;
  }
  return {
    version: update.version,
    notes: update.body ?? "",
  };
}

export async function installAppUpdate(): Promise<void> {
  const update: Update | null = await withTimeout(
    check({ timeout: CHECK_TIMEOUT_MS }),
    CHECK_TIMEOUT_MS + 1000,
    "Update check"
  );
  if (!update) {
    throw new Error("No update available");
  }
  await update.downloadAndInstall(undefined, { timeout: 120_000 });
  await relaunch();
}

export async function openLatestReleasePage(): Promise<void> {
  await openUrl(RELEASES_PAGE);
}
