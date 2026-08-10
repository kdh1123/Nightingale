import type { ApplicationSettings, DetectionPolicy } from "./api";

export const SETTINGS_CATEGORIES = [
  "General",
  "Monitoring",
  "Security",
  "Detection",
  "Allowlist",
  "Notification",
  "Appearance",
  "Database",
  "Advanced",
] as const;

export type SettingsCategory = (typeof SETTINGS_CATEGORIES)[number];

type BooleanSettingKey = {
  [K in keyof ApplicationSettings]: ApplicationSettings[K] extends boolean ? K : never;
}[keyof ApplicationSettings];

export interface SettingToggle {
  key: BooleanSettingKey;
  label: string;
  description: string;
}

/** Categories not listed here have no plain on/off settings and render their own panel. */
export const SETTING_TOGGLES: Partial<Record<SettingsCategory, readonly SettingToggle[]>> = {
  Monitoring: [
    {
      key: "monitoringEnabled",
      label: "Real-time monitoring",
      description: "Monitor selected locations continuously",
    },
    {
      key: "autoBaselineRefresh",
      label: "Automatic baseline refresh",
      description: "Keep known-good file data current",
    },
  ],
  Security: [
    {
      key: "threatDetectionEnabled",
      label: "Threat detection",
      description: "Create local signals from suspicious changes",
    },
    {
      key: "securityScoreEnabled",
      label: "Security score",
      description: "Calculate the device protection score",
    },
  ],
};

export interface DetectionFeatureToggle {
  key: keyof DetectionPolicy["features"];
  label: string;
  description: string;
}

export const DETECTION_FEATURE_TOGGLES: readonly DetectionFeatureToggle[] = [
  {
    key: "massFileChanges",
    label: "Mass file changes",
    description: "Detect unusual bursts of changes",
  },
  {
    key: "suspiciousFileActivity",
    label: "Suspicious file activity",
    description: "Detect script and executable file activity",
  },
  {
    key: "integrityChanges",
    label: "Baseline integrity changes",
    description: "Detect changes from the known-good baseline",
  },
];
