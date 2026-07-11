import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";

export type PendingUpdate = {
  version: string;
  notes: string;
};

const RELEASES_PAGE = "https://github.com/kpxcoolx/eql-meter/releases/latest";

export async function checkForAppUpdate(): Promise<PendingUpdate | null> {
  const update = await check();
  if (!update) {
    return null;
  }
  return {
    version: update.version,
    notes: update.body ?? "",
  };
}

export async function installAppUpdate(): Promise<void> {
  const update: Update | null = await check();
  if (!update) {
    throw new Error("No update available");
  }
  await update.downloadAndInstall();
  await relaunch();
}

export async function openLatestReleasePage(): Promise<void> {
  await openUrl(RELEASES_PAGE);
}
