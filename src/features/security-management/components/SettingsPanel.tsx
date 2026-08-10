import type { ApplicationSettings, DetectionPolicy } from "../api";
import {
  DETECTION_FEATURE_TOGGLES,
  SETTING_TOGGLES,
  type SettingsCategory,
} from "../settings-catalog";
import { AllowlistSettings, type AllowlistSettingsProps } from "./AllowlistSettings";

const LOG_RETENTION_MIN_DAYS = 1;
const LOG_RETENTION_MAX_DAYS = 3650;

function SettingToggleRow({
  label,
  description,
  checked,
  disabled,
  onToggle,
}: {
  label: string;
  description: string;
  checked: boolean;
  disabled: boolean;
  onToggle: () => void;
}) {
  return (
    <label className="setting-row">
      <span>
        {label}
        <small>{description}</small>
      </span>
      <input
        className="switch"
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={onToggle}
      />
    </label>
  );
}

function AppearanceSettings({
  settings,
  disabled,
  onSettingsChange,
}: {
  settings: ApplicationSettings;
  disabled: boolean;
  onSettingsChange: (next: ApplicationSettings) => void;
}) {
  return (
    <label className="setting-row">
      <span>
        Application theme<small>Follow the system or choose a fixed appearance</small>
      </span>
      <select
        className="field"
        value={settings.uiTheme}
        disabled={disabled}
        onChange={(e) =>
          onSettingsChange({
            ...settings,
            uiTheme: e.target.value as ApplicationSettings["uiTheme"],
          })
        }
      >
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </label>
  );
}

function DetectionSettings({
  policy,
  disabled,
  onPolicyChange,
}: {
  policy: DetectionPolicy;
  disabled: boolean;
  onPolicyChange: (next: DetectionPolicy) => void;
}) {
  return (
    <>
      <label className="setting-row">
        <span>
          Detection sensitivity
          <small>Safe preset thresholds; higher sensitivity reports mass changes sooner</small>
        </span>
        <select
          className="field"
          value={policy.sensitivity}
          disabled={disabled}
          onChange={(e) =>
            onPolicyChange({
              ...policy,
              sensitivity: e.target.value as DetectionPolicy["sensitivity"],
            })
          }
        >
          <option value="low">Low — 35 changes / minute</option>
          <option value="medium">Medium — 20 changes / minute</option>
          <option value="high">High — 10 changes / minute</option>
        </select>
      </label>
      {DETECTION_FEATURE_TOGGLES.map(({ key, label, description }) => (
        <SettingToggleRow
          key={key}
          label={label}
          description={description}
          checked={policy.features[key]}
          disabled={disabled}
          onToggle={() =>
            onPolicyChange({
              ...policy,
              features: { ...policy.features, [key]: !policy.features[key] },
            })
          }
        />
      ))}
    </>
  );
}

function DatabaseSettings({
  settings,
  disabled,
  onSettingsChange,
  onCleanupLogs,
}: {
  settings: ApplicationSettings;
  disabled: boolean;
  onSettingsChange: (next: ApplicationSettings) => void;
  onCleanupLogs: () => void;
}) {
  return (
    <>
      <label className="setting-row">
        <span>
          Log retention<small>Keep local security logs for this many days</small>
        </span>
        <input
          className="field"
          type="number"
          min={LOG_RETENTION_MIN_DAYS}
          max={LOG_RETENTION_MAX_DAYS}
          value={settings.logRetentionDays}
          disabled={disabled}
          onChange={(e) =>
            onSettingsChange({ ...settings, logRetentionDays: Number(e.target.value) })
          }
        />
      </label>
      <button className="btn secondary" disabled={disabled} onClick={onCleanupLogs}>
        Clean old logs
      </button>
    </>
  );
}

interface SettingsPanelProps {
  category: SettingsCategory;
  settings: ApplicationSettings;
  policy: DetectionPolicy | undefined;
  allowlist: Omit<AllowlistSettingsProps, "disabled">;
  disabled: boolean;
  onSettingsChange: (next: ApplicationSettings) => void;
  onPolicyChange: (next: DetectionPolicy) => void;
  onCleanupLogs: () => void;
}

export function SettingsPanel({
  category,
  settings,
  policy,
  allowlist,
  disabled,
  onSettingsChange,
  onPolicyChange,
  onCleanupLogs,
}: SettingsPanelProps) {
  return (
    <article className="card panel">
      <h2>{category}</h2>
      <SettingsCategoryBody
        category={category}
        settings={settings}
        policy={policy}
        allowlist={allowlist}
        disabled={disabled}
        onSettingsChange={onSettingsChange}
        onPolicyChange={onPolicyChange}
        onCleanupLogs={onCleanupLogs}
      />
    </article>
  );
}

function SettingsCategoryBody({
  category,
  settings,
  policy,
  allowlist,
  disabled,
  onSettingsChange,
  onPolicyChange,
  onCleanupLogs,
}: SettingsPanelProps) {
  const toggles = SETTING_TOGGLES[category];
  if (toggles) {
    return (
      <>
        {toggles.map(({ key, label, description }) => (
          <SettingToggleRow
            key={key}
            label={label}
            description={description}
            checked={settings[key]}
            disabled={disabled}
            onToggle={() => onSettingsChange({ ...settings, [key]: !settings[key] })}
          />
        ))}
      </>
    );
  }

  switch (category) {
    case "Appearance":
      return (
        <AppearanceSettings
          settings={settings}
          disabled={disabled}
          onSettingsChange={onSettingsChange}
        />
      );
    case "Detection":
      return policy ? (
        <DetectionSettings policy={policy} disabled={disabled} onPolicyChange={onPolicyChange} />
      ) : (
        <p className="empty">Loading detection policy…</p>
      );
    case "Allowlist":
      return <AllowlistSettings {...allowlist} disabled={disabled} />;
    case "Database":
      return (
        <DatabaseSettings
          settings={settings}
          disabled={disabled}
          onSettingsChange={onSettingsChange}
          onCleanupLogs={onCleanupLogs}
        />
      );
    default:
      return (
        <p className="empty">
          {category} preferences will appear here as this local security profile evolves.
        </p>
      );
  }
}
