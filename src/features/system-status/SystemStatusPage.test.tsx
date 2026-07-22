import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SystemStatusPage } from "./SystemStatusPage";

describe("SystemStatusPage", () => {
  it("shows the Phase 1 placeholder", () => {
    render(<SystemStatusPage />);
    expect(screen.getByText("시스템 모니터링은 아직 준비 중입니다")).toBeInTheDocument();
  });
});
