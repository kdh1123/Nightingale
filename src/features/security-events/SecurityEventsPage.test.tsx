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
}));

describe("SecurityEventsPage", () => {
  it("shows an event that can be reviewed", async () => {
    render(
      <QueryClientProvider client={new QueryClient()}>
        <SecurityEventsPage />
      </QueryClientProvider>,
    );
    expect(await screen.findByText("파일 활동 감지")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "검토 완료" })).toBeInTheDocument();
  });
});
