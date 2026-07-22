import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SystemStatusPage } from "./SystemStatusPage";

vi.mock("./api", () => ({
  getSystemSnapshot: vi.fn(() => new Promise(() => {})),
  listProcesses: vi.fn(() => Promise.resolve([])),
}));
describe("SystemStatusPage", () => {
  it("shows a loading state", () => {
    render(
      <QueryClientProvider client={new QueryClient()}>
        <SystemStatusPage />
      </QueryClientProvider>,
    );
    expect(screen.getByText("시스템 상태를 불러오는 중")).toBeInTheDocument();
  });
});
