import { getAllDeviceStates, getRecentActivities } from "../db";
import type { DeviceState, ActivityRecord } from "../types";
import { visitors } from "../services/visitors";

// Strip window_title from public API responses
function stripWindowTitle<T extends { window_title?: string }>(
  records: T[]
): Omit<T, "window_title">[] {
  return records.map(({ window_title, ...rest }) => rest);
}

export function handleCurrent(clientIp: string): Response {
  visitors.heartbeat(clientIp);

  const devices = getAllDeviceStates.all() as DeviceState[];
  const recentActivities = getRecentActivities.all() as ActivityRecord[];

  return Response.json({
    devices: stripWindowTitle(devices),
    recent_activities: stripWindowTitle(recentActivities),
    server_time: new Date().toISOString(),
    viewer_count: visitors.getCount(),
  });
}
