import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SecurityManagementPage } from "./SecurityManagementPage";

vi.mock("./api", () => ({
  getApplicationSettings: vi.fn(() =>
    Promise.resolve({
      monitoringEnabled: true,
      threatDetectionEnabled: true,
      autoBaselineRefresh: false,
      securityScoreEnabled: true,
      logRetentionDays: 90,
      uiTheme: "system",
    }),
  ),
  updateApplicationSettings: vi.fn(),
  getSecurityReport: vi.fn(() =>
    Promise.resolve({
      generatedAt: "2026-07-22 12:00:00",
      securityScore: { score: 88, openIncidentCount: 1, criticalIncidentCount: 0 },
      totalIncidents: 3,
      severityCounts: { info: 1, low: 2, medium: 0, high: 1, critical: 0 },
      monitoredFolderCount: 2,
      fileEventCount: 17,
      recentDetections: [],
      recentRiskEvents: [],
    }),
  ),
  getDetectionPolicy: vi.fn(() =>
    Promise.resolve({
      version: 1,
      sensitivity: "medium",
      features: {
        massFileChanges: true,
        massExtensionChanges: true,
        suspiciousFileActivity: true,
        integrityChanges: false,
        phishingUrlAnalysis: false,
        systemAnomalyDetection: false,
      },
      monitoredPaths: [],
      excludedPaths: [],
    }),
  ),
  updateDetectionPolicy: vi.fn(),
  listAllowlistEntries: vi.fn(() =>
    Promise.resolve([
      {
        id: 5,
        entryType: "path",
        value: "/tmp/trusted",
        expiresAt: null,
        createdAt: "2026-07-22 12:00:00",
      },
    ]),
  ),
  addAllowlistEntry: vi.fn(),
  removeAllowlistEntry: vi.fn(),
  listAllowlistAudit: vi.fn(() => Promise.resolve([])),
  listNotifications: vi.fn(() => Promise.resolve([])),
  markNotificationRead: vi.fn(),
  cleanupSecurityLogs: vi.fn(),
}));

function renderPage() {
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SecurityManagementPage />
    </QueryClientProvider>,
  );
}

describe("SecurityManagementPage", () => {
  it("shows the report summary and the placeholder for a category without settings", async () => {
    renderPage();
    expect(await screen.findByText("88")).toBeInTheDocument();
    expect(screen.getByText("17")).toBeInTheDocument();
    expect(
      screen.getByText(
        "General preferences will appear here as this local security profile evolves.",
      ),
    ).toBeInTheDocument();
  });

  it("renders the settings that belong to the selected category", async () => {
    renderPage();
    fireEvent.click(await screen.findByRole("button", { name: "Monitoring" }));
    expect(screen.getByText("Real-time monitoring")).toBeInTheDocument();
    expect(screen.getByText("Automatic baseline refresh")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Detection" }));
    expect(screen.getByText("Detection sensitivity")).toBeInTheDocument();
    expect(screen.getByText("Baseline integrity changes")).toBeInTheDocument();
    expect(screen.queryByText("Real-time monitoring")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Database" }));
    expect(screen.getByRole("button", { name: "Clean old logs" })).toBeInTheDocument();
  });

  it("keeps a half-typed trusted item while another category is visited", async () => {
    renderPage();
    fireEvent.click(await screen.findByRole("button", { name: "Allowlist" }));
    expect(await screen.findByText("/tmp/trusted")).toBeInTheDocument();

    const input = screen.getByPlaceholderText("/path/to/trusted-folder");
    fireEvent.change(input, { target: { value: "/tmp/draft" } });
    fireEvent.click(screen.getByRole("button", { name: "General" }));
    fireEvent.click(screen.getByRole("button", { name: "Allowlist" }));

    expect(screen.getByPlaceholderText("/path/to/trusted-folder")).toHaveValue("/tmp/draft");
  });
});
