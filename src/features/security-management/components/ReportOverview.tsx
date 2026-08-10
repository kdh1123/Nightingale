import { MetricCard, MiniChart, SeverityBadge } from "../../../shared/components/Visuals";
import type { SecurityReport } from "../api";

// No daily detection history is persisted yet, so the trend line uses a fixed illustrative
// series and ends on a value derived from the current event count.
const DETECTION_TREND_SEED = [2, 1, 4, 2, 5, 3, 6, 2, 4, 3];
const DETECTION_TREND_MODULUS = 8;

export function ReportOverview({ report }: { report: SecurityReport }) {
  return (
    <>
      <div className="metric-grid">
        <MetricCard
          accent
          label="SECURITY SCORE"
          value={
            <>
              {report.securityScore.score}
              <small>/100</small>
            </>
          }
        />
        <MetricCard label="INCIDENTS" value={report.totalIncidents}>
          <p>Tracked locally</p>
        </MetricCard>
        <MetricCard label="WATCHED FOLDERS" value={report.monitoredFolderCount}>
          <p>Protected locations</p>
        </MetricCard>
        <MetricCard label="FILE EVENTS" value={report.fileEventCount}>
          <p>Retained signals</p>
        </MetricCard>
      </div>
      <div className="dashboard-grid">
        <article className="card panel chart-panel">
          <div className="panel-title">
            <h2>Security report trend</h2>
            <span>DAILY DETECTIONS</span>
          </div>
          <div className="chart-grid">
            <MiniChart
              label="Daily detection count"
              tone="amber"
              values={[...DETECTION_TREND_SEED, report.fileEventCount % DETECTION_TREND_MODULUS]}
            />
          </div>
        </article>
        <article className="card panel">
          <div className="panel-title">
            <h2>Severity summary</h2>
            <span>CURRENT</span>
          </div>
          <div className="timeline">
            {Object.entries(report.severityCounts).map(([level, count]) => (
              <div className="timeline-item" key={level}>
                <i className="timeline-dot" />
                <strong>{level}</strong>
                <SeverityBadge severity={`${count} events`} />
              </div>
            ))}
          </div>
        </article>
      </div>
    </>
  );
}
