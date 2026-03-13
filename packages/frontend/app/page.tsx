"use client";

import { useDashboard } from "@/hooks/useDashboard";
import Header from "@/components/Header";
import DeviceCard from "@/components/DeviceCard";
import DatePicker from "@/components/DatePicker";
import Timeline from "@/components/Timeline";

export default function Home() {
  const { current, timeline, selectedDate, changeDate, loading, error, viewerCount } = useDashboard();

  return (
    <>
      <Header serverTime={current?.server_time} viewerCount={viewerCount} />

      {/* Error banner */}
      {error && (
        <div className="vn-bubble mb-4 border-[var(--color-primary)]">
          <p className="text-sm text-[var(--color-primary)]">
            (&gt;_&lt;) 连接失败了喵...
          </p>
          <p className="text-xs text-[var(--color-text-muted)] mt-1">
            别担心，会自动重试的~
          </p>
        </div>
      )}

      {/* Loading state */}
      {loading && !current && (
        <div className="flex flex-col items-center justify-center py-16 gap-3">
          <p className="text-2xl">(=^-ω-^=)</p>
          <div className="loading-dots">
            <span />
            <span />
            <span />
          </div>
          <p className="text-xs text-[var(--color-text-muted)]">正在加载喵~</p>
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
              <div className="text-center py-6">
                <p className="text-lg mb-1">(´・ω・`)</p>
                <p className="text-sm text-[var(--color-text-muted)] italic">
                  还没有设备连接呢~
                </p>
              </div>
            ) : (
              current.devices.map((d) => (
                <DeviceCard key={d.device_id} device={d} />
              ))
            )}
          </div>

          {/* Right: timeline (wide) */}
          <div className="flex-1 min-w-0">
            {/* Date picker */}
            <div className="flex items-center justify-between mb-4 flex-wrap gap-2">
              <DatePicker selectedDate={selectedDate} onChange={changeDate} />
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
          Monika Now &middot; 每 10 秒自动刷新 &middot; (◕ᴗ◕)
        </p>
      </footer>
    </>
  );
}
