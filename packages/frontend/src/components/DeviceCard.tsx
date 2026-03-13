import type { DeviceState } from "@/lib/api";
import { getAppDescription } from "@/lib/app-descriptions";

const platformIcons: Record<string, string> = {
  windows: "\u{1F5A5}",  // desktop computer
  android: "\u{1F4F1}",  // mobile phone
};

function timeAgo(isoStr: string): string {
  if (!isoStr) return "unknown";
  const ts = new Date(isoStr).getTime();
  if (isNaN(ts)) return "unknown";
  const diff = Date.now() - ts;
  if (diff < 0) return "just now";
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

export default function DeviceCard({ device }: { device: DeviceState }) {
  const isOnline = device.is_online === 1;
  const icon = platformIcons[device.platform] || "\u{1F4BB}";

  return (
    <div className="card-decorated rounded-md p-4">
      {/* Device header */}
      <div className="flex items-center gap-2 mb-3">
        <span className="text-lg" aria-hidden="true">{icon}</span>
        <span className="font-semibold text-sm">{device.device_name}</span>
        <span className="ml-auto text-xs font-medium" title={isOnline ? "Online" : "Offline"}>
          {isOnline ? "(=^-ω-^=)" : "(-.-)zzZ"}
        </span>
      </div>

      {/* Current activity in VN bubble */}
      <div className="vn-bubble">
        {isOnline ? (
          <p className="text-sm font-medium text-[var(--color-primary)]">
            {getAppDescription(device.app_name)}
          </p>
        ) : (
          <p className="text-sm text-[var(--color-text-muted)] italic">
            不在线喵...
          </p>
        )}
      </div>

      {/* Last seen */}
      <p className="text-xs text-[var(--color-text-muted)] mt-2 text-right">
        {timeAgo(device.last_seen_at)}
      </p>
    </div>
  );
}
