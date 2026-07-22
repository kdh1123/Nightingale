import { createBrowserRouter } from "react-router-dom";
import { AppLayout } from "./AppLayout";
import { DashboardPage } from "../features/dashboard/DashboardPage";
import { SystemStatusPage } from "../features/system-status/SystemStatusPage";
import { FileMonitoringPage } from "../features/file-monitoring/FileMonitoringPage";
import { SecurityEventsPage } from "../features/security-events/SecurityEventsPage";
import { SecurityManagementPage } from "../features/security-management/SecurityManagementPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: "system", element: <SystemStatusPage /> },
      { path: "monitoring", element: <FileMonitoringPage /> },
      { path: "events", element: <SecurityEventsPage /> },
      { path: "policy", element: <SecurityManagementPage /> },
    ],
  },
]);
