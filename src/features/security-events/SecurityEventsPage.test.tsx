import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SecurityEventsPage } from "./SecurityEventsPage";

vi.mock("./api", () => ({
  listSecurityEvents: vi.fn(() =>
    Promise.resolve([
      {
        id: 1,
        eventType: "modified",
        severity: "low",
        title: "파일 활동 감지",
        description: "선택한 감시 폴더에서 파일 활동이 감지되었습니다.",
        occurredAt: "2026-07-22 12:00:00",
        reviewed: false,
      },
    ]),
  ),
  markSecurityEventReviewed: vi.fn(),
  listIncidents: vi.fn(() =>
    Promise.resolve([
      {
        id: 2,
        severity: "high",
        status: "open",
        title: "대량 파일 변경 감지",
        description: "짧은 시간에 다수의 파일 변경이 감지되었습니다.",
        eventCount: 2,
        firstDetectedAt: "2026-07-22 12:00:00",
        lastDetectedAt: "2026-07-22 12:00:01",
      },
    ]),
  ),
  updateIncidentStatus: vi.fn(),
  getIncidentTimeline: vi.fn(() =>
    Promise.resolve([
      {
        securityEventId: 1,
        eventType: "mass_file_change",
        severity: "high",
        title: "대량 파일 변경 감지",
        description: "짧은 시간에 다수의 파일 변경이 감지되었습니다.",
        occurredAt: "2026-07-22 12:00:00",
        reviewed: false,
        fileEventId: 4,
        fileEventKind: "modified",
        filePath: "/watch/a.txt",
      },
    ]),
  ),
}));

describe("SecurityEventsPage", () => {
  it("shows an event that can be reviewed", async () => {
    render(
      <QueryClientProvider client={new QueryClient()}>
        <SecurityEventsPage />
      </QueryClientProvider>,
    );
    expect(await screen.findByText("파일 활동 감지")).toBeInTheDocument();
    expect(await screen.findByText("대량 파일 변경 감지")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "검토 완료" })).toBeInTheDocument();
    expect(await screen.findByText("탐지 규칙")).toBeInTheDocument();
    expect(screen.getByText("mass_file_change")).toBeInTheDocument();
  });
});
