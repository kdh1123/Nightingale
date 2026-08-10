// The platform snapshot carries no network counter yet, so the network gauge shows a
// CPU-derived approximation. Both the dashboard and the system page must show the same
// number, so the formula lives here instead of at each call site.
const NETWORK_LOAD_CPU_FACTOR = 0.55;
const NETWORK_LOAD_BASELINE_PERCENT = 8;

export function estimateNetworkLoadPercent(cpuPercent: number) {
  return Math.min(100, cpuPercent * NETWORK_LOAD_CPU_FACTOR + NETWORK_LOAD_BASELINE_PERCENT);
}
