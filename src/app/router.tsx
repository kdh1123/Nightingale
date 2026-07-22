import { createBrowserRouter } from "react-router-dom";
import { AppLayout } from "./AppLayout";
import { DashboardPage } from "../features/dashboard/DashboardPage";
import { SystemStatusPage } from "../features/system-status/SystemStatusPage";
import { StatePanel } from "../shared/components/StatePanel";
import { FileMonitoringPage } from "../features/file-monitoring/FileMonitoringPage";
import { SecurityEventsPage } from "../features/security-events/SecurityEventsPage";

const planned = (title: string, text: string) => () => (
  <StatePanel title={title}>{text}</StatePanel>
);
export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: "system", element: <SystemStatusPage /> },
      { path: "monitoring", element: <FileMonitoringPage /> },
      { path: "events", element: <SecurityEventsPage /> },
      {
        path: "policy",
        element: planned(
          "보안 정책",
          "기본 정책 모델은 Rust에 준비되어 있습니다. 사용자 설정 화면은 이후 단계에서 추가됩니다.",
        )(),
      },
    ],
  },
]);
