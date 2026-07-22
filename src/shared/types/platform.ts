export type CapabilityStatus =
  | "supported"
  | "partially_supported"
  | "permission_required"
  | "unsupported"
  | "temporarily_unavailable";

export interface PlatformCapability {
  key: string;
  status: CapabilityStatus;
  detail: string;
}

export interface AppStatus {
  appVersion: string;
  operatingSystem: string;
  capabilities: PlatformCapability[];
}
