import type { TimelineSegment } from "@/lib/api";

// Warm color palette for different apps (avoids purple/blue)
const APP_COLORS = [
  "#E8A0BF", // sakura pink
  "#88C9C9", // mint
  "#E8B86D", // amber
  "#C4A882", // warm tan
  "#D4917B", // terracotta
  "#A8C686", // sage green
  "#D4A0A0", // dusty rose
  "#8CB8B0", // teal mist
  "#C9B97A", // gold wheat
  "#B89EC4", // lavender mist (light, not purple)
];

function getAppColor(appName: string, colorMap: Map<string, string>): string {
  const existing = colorMap.get(appName);
  if (existing) return existing;
  const color = APP_COLORS[colorMap.size % APP_COLORS.length] ?? APP_COLORS[0];
  colorMap.set(appName, color);
  return color;
}

function formatTime(isoStr: string): string {
  const d = new Date(isoStr);
  if (isNaN(d.getTime())) return "--:--";
  return d.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
}

interface Props {
  segments: TimelineSegment[];
  summary: Record<string, Record<string, number>>;
}

export default function Timeline({ segments, summary }: Props) {
  const colorMap = new Map<string, string>();

  if (segments.length === 0) {
    return (
      <div className="text-center py-12 text-[var(--color-text-muted)]">
        <p className="text-lg">{"\u{1F4A4}"} No activity data for this day</p>
      </div>
    );
  }

  // Group segments by device
  const byDevice = new Map<string, { name: string; segs: TimelineSegment[] }>();
  for (const seg of segments) {
    let entry = byDevice.get(seg.device_id);
    if (!entry) {
      entry = { name: seg.device_name, segs: [] };
      byDevice.set(seg.device_id, entry);
    }
    entry.segs.push(seg);
  }

  return (
    <div className="space-y-6">
      {Array.from(byDevice.entries()).map(([deviceId, { name, segs }]) => (
        <div key={deviceId}>
          <h3 className="text-sm font-semibold mb-2 text-[var(--color-text-muted)]">
            {name}
          </h3>

          {/* Manga panel timeline bars */}
          <div className="space-y-1">
            {segs.map((seg, i) => {
              const color = getAppColor(seg.app_name, colorMap);
              const duration = seg.duration_minutes;

              return (
                <div key={`${seg.started_at}-${i}`} className="timeline-bar flex items-stretch">
                  {/* Time label */}
                  <div className="flex-shrink-0 w-14 px-2 py-1.5 bg-[var(--color-cream)] flex items-center justify-center border-r-2 border-[var(--color-border)]">
                    <span className="text-[10px] font-mono text-[var(--color-text-muted)]">
                      {formatTime(seg.started_at)}
                    </span>
                  </div>

                  {/* App bar */}
                  <div
                    className="flex-1 px-3 py-1.5 flex items-center gap-2 min-w-0"
                    style={{ backgroundColor: `${color}22` }}
                  >
                    {/* Color dot */}
                    <span
                      className="w-2.5 h-2.5 rounded-full flex-shrink-0"
                      style={{ backgroundColor: color }}
                    />
                    <span className="text-xs font-medium truncate">
                      {seg.app_name}
                    </span>
                    {seg.window_title && (
                      <span className="text-[10px] text-[var(--color-text-muted)] font-mono truncate hidden sm:inline">
                        {seg.window_title}
                      </span>
                    )}
                  </div>

                  {/* Duration */}
                  <div className="flex-shrink-0 w-14 px-2 py-1.5 bg-[var(--color-cream)] flex items-center justify-center border-l-2 border-[var(--color-border)]">
                    <span className="text-[10px] font-mono text-[var(--color-accent)]">
                      {duration > 0 ? `${duration}m` : "<1m"}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Per-device summary */}
          {summary[deviceId] && (
            <div className="mt-3 flex flex-wrap gap-2">
              {Object.entries(summary[deviceId])
                .sort(([, a], [, b]) => b - a)
                .slice(0, 8)
                .map(([app, mins]) => {
                  const color = getAppColor(app, colorMap);
                  return (
                    <span
                      key={app}
                      className="inline-flex items-center gap-1 text-[10px] px-2 py-0.5 rounded-sm border border-[var(--color-border)]"
                    >
                      <span
                        className="w-2 h-2 rounded-full"
                        style={{ backgroundColor: color }}
                      />
                      {app}: {mins}m
                    </span>
                  );
                })}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
