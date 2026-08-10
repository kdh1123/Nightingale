import type { AllowlistAuditEntry, AllowlistEntry } from "../api";

const AUDIT_PREVIEW_LIMIT = 5;

export interface AllowlistDraft {
  entryType: AllowlistEntry["entryType"];
  value: string;
  /** Days until the entry expires, as a select value; "0" means the entry never expires. */
  expiry: string;
}

export interface AllowlistSettingsProps {
  entries: AllowlistEntry[] | undefined;
  auditEntries: AllowlistAuditEntry[] | undefined;
  draft: AllowlistDraft;
  disabled: boolean;
  onDraftChange: (patch: Partial<AllowlistDraft>) => void;
  onAdd: () => void;
  onRemove: (id: number) => void;
}

export function AllowlistSettings({
  entries,
  auditEntries,
  draft,
  disabled,
  onDraftChange,
  onAdd,
  onRemove,
}: AllowlistSettingsProps) {
  return (
    <>
      <p className="setting-help">
        Trusted paths and extensions stay in the file log, but do not create security signals or
        Incidents.
      </p>
      <div className="allowlist-form">
        <select
          className="field"
          value={draft.entryType}
          onChange={(e) =>
            onDraftChange({ entryType: e.target.value as AllowlistEntry["entryType"] })
          }
          disabled={disabled}
        >
          <option value="path">Path or folder</option>
          <option value="extension">File extension</option>
        </select>
        <input
          className="field"
          value={draft.value}
          placeholder={draft.entryType === "path" ? "/path/to/trusted-folder" : "log"}
          onChange={(e) => onDraftChange({ value: e.target.value })}
          disabled={disabled}
        />
        <select
          className="field"
          value={draft.expiry}
          onChange={(e) => onDraftChange({ expiry: e.target.value })}
          disabled={disabled}
        >
          <option value="0">No expiry</option>
          <option value="7">7 days</option>
          <option value="30">30 days</option>
          <option value="90">90 days</option>
        </select>
        <button className="btn" disabled={disabled || !draft.value.trim()} onClick={onAdd}>
          Add trusted item
        </button>
      </div>
      <ul className="event-list">
        {entries?.map((entry) => (
          <li key={entry.id} className="allowlist-entry">
            <div>
              <strong>{entry.value}</strong>
              <span>
                {entry.entryType} · {entry.expiresAt ? `Expires ${entry.expiresAt}` : "No expiry"}
              </span>
            </div>
            <button
              className="btn secondary"
              disabled={disabled}
              onClick={() => onRemove(entry.id)}
            >
              Remove
            </button>
          </li>
        )) ?? <li className="empty">No trusted items yet.</li>}
      </ul>
      <h3 className="subsection-title">Recent changes</h3>
      <ul className="event-list compact-list">
        {auditEntries?.slice(0, AUDIT_PREVIEW_LIMIT).map((entry) => (
          <li key={entry.id}>
            <strong>
              {entry.action} · {entry.value}
            </strong>
            <span>
              {entry.entryType} · {entry.occurredAt}
            </span>
          </li>
        )) ?? <li className="empty">No allowlist changes yet.</li>}
      </ul>
    </>
  );
}
