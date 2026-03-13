import {
  getTimelineByDate,
  getTimelineByDateAndDevice,
} from "../db";
import type { ActivityRecord, TimelineSegment } from "../types";

export function handleTimeline(url: URL): Response {
  const date = url.searchParams.get("date");
  if (!date || !/^\d{4}-\d{2}-\d{2}$/.test(date)) {
    return Response.json(
      { error: "date parameter required (YYYY-MM-DD)" },
      { status: 400 }
    );
  }

  const deviceId = url.searchParams.get("device_id");

  const activities: ActivityRecord[] = deviceId
    ? (getTimelineByDateAndDevice.all(date, deviceId) as ActivityRecord[])
    : (getTimelineByDate.all(date) as ActivityRecord[]);

  // Build timeline segments with duration
  const segments: TimelineSegment[] = [];
  for (let i = 0; i < activities.length; i++) {
    const a = activities[i];
    // Find next activity on same device to compute end time
    let endedAt: string | null = null;
    for (let j = i + 1; j < activities.length; j++) {
      if (activities[j].device_id === a.device_id) {
        endedAt = activities[j].started_at;
        break;
      }
    }

    const startMs = new Date(a.started_at).getTime();
    const endMs = endedAt ? new Date(endedAt).getTime() : startMs;
    const durationMinutes = Math.round((endMs - startMs) / 60000);

    segments.push({
      app_name: a.app_name,
      app_id: a.app_id,
      started_at: a.started_at,
      ended_at: endedAt,
      duration_minutes: durationMinutes,
      device_id: a.device_id,
      device_name: a.device_name,
    });
  }

  // Build summary: total minutes per app per device
  const summaryNested = new Map<string, Map<string, number>>();
  for (const s of segments) {
    let appMap = summaryNested.get(s.device_id);
    if (!appMap) {
      appMap = new Map();
      summaryNested.set(s.device_id, appMap);
    }
    appMap.set(s.app_name, (appMap.get(s.app_name) || 0) + s.duration_minutes);
  }

  const summary: Record<string, Record<string, number>> = {};
  for (const [devId, appMap] of summaryNested) {
    summary[devId] = Object.fromEntries(appMap);
  }

  return Response.json({ date, segments, summary });
}
