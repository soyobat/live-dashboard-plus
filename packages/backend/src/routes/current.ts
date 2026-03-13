import { getAllDeviceStates, getRecentActivities } from "../db";

export function handleCurrent(): Response {
  const devices = getAllDeviceStates.all();
  const recentActivities = getRecentActivities.all();

  return Response.json({
    devices,
    recent_activities: recentActivities,
    server_time: new Date().toISOString(),
  });
}
