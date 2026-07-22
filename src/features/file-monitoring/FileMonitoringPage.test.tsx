import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FileMonitoringPage } from "./FileMonitoringPage";

vi.mock("./api", () => ({
  addMonitoredPath: vi.fn(),
  listFileEvents: vi.fn(() => Promise.resolve([])),
  listMonitoredPaths: vi.fn(() =>
    Promise.resolve([
      {
        id: 1,
        path: "/tmp/watch",
        enabled: true,
        monitoringStatus: "running",
        baselineStatus: "complete",
        lastScanAt: null,
        lastEventAt: null,
        lastError: null,
      },
    ]),
  ),
  pauseFileMonitoring: vi.fn(),
  removeMonitoredPath: vi.fn(),
  resumeFileMonitoring: vi.fn(),
  startBaselineScan: vi.fn(),
}));

describe("FileMonitoringPage", () => {
  it("shows controls for a monitored path", async () => {
    render(
      <QueryClientProvider client={new QueryClient()}>
        <FileMonitoringPage />
      </QueryClientProvider>,
    );
    expect(await screen.findByText("/tmp/watch")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "기준선 스캔" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "일시정지" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "삭제" })).toBeInTheDocument();
  });
});
