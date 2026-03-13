"use client";

import { useDashboard } from "@/hooks/useDashboard";
import Header from "@/components/Header";
import DeviceCard from "@/components/DeviceCard";
import DatePicker from "@/components/DatePicker";
import Timeline from "@/components/Timeline";

export default function Home() {
  const { current, timeline, selectedDate, changeDate, loading, error, refresh } = useDashboard();

  return (
    <>
      <Header serverTime={current?.server_time} />

      {/* Error banner */}
      {error && (
        <div className="vn-bubble mb-4 border-[var(--color-primary)]">
          <p className="text-sm text-[var(--color-primary)]">
            {"\u{1F63F}"} Connection failed: {error}
          </p>
          <button className="pill-btn text-xs mt-2" onClick={refresh}>
            retry
          </button>
        </div>
      )}

      {/* Loading state */}
      {loading && !current && (
        <div className="flex justify-center py-16">
          <div className="loading-dots">
            <span />
            <span />
            <span />
          </div>
        </div>
      )}

      {current && (
        <div className="flex flex-col lg:flex-row gap-6">
          {/* Left: device cards (narrow) */}
          <div className="lg:w-64 flex-shrink-0 space-y-3">
            <h2 className="text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider mb-2">
              Devices
            </h2>
            {(!current.devices || current.devices.length === 0) ? (
              <p className="text-sm text-[var(--color-text-muted)] italic">
                No devices registered yet
              </p>
            ) : (
              current.devices.map((d) => (
                <DeviceCard key={d.device_id} device={d} />
              ))
            )}
          </div>

          {/* Right: timeline (wide) */}
          <div className="flex-1 min-w-0">
            {/* Date picker + refresh */}
            <div className="flex items-center justify-between mb-4 flex-wrap gap-2">
              <DatePicker selectedDate={selectedDate} onChange={changeDate} />
              <button className="pill-btn text-xs" onClick={refresh}>
                {loading ? "..." : "\u{21BB} refresh"}
              </button>
            </div>

            <div className="separator-dashed mb-4" />

            {/* Timeline content */}
            {loading && timeline ? (
              <div className="opacity-60">
                <Timeline segments={timeline.segments} summary={timeline.summary} />
              </div>
            ) : timeline ? (
              <Timeline segments={timeline.segments} summary={timeline.summary} />
            ) : null}
          </div>
        </div>
      )}

      {/* Footer */}
      <footer className="mt-12 pt-4 separator-dashed text-center">
        <p className="text-[10px] text-[var(--color-text-muted)]">
          Monika Now &middot; auto-refreshes every 10 min
        </p>
      </footer>
    </>
  );
}
