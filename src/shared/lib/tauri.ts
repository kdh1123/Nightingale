import { invoke } from "@tauri-apps/api/core";
import type { AppStatus } from "../types/platform";

export async function getAppStatus(): Promise<AppStatus> {
  return invoke<AppStatus>("get_app_status");
}
