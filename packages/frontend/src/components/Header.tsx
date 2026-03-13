export default function Header({ serverTime }: { serverTime?: string }) {
  const timeStr = (() => {
    if (!serverTime) return "--:--";
    const d = new Date(serverTime);
    if (isNaN(d.getTime())) return "--:--";
    return d.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
  })();

  return (
    <header className="flex items-end justify-between pb-4 mb-6 separator-dashed">
      {/* Left: title + chibi cat */}
      <div className="flex items-end gap-3">
        {/* Simple CSS cat face */}
        <div className="relative w-10 h-10 flex-shrink-0" aria-hidden="true">
          <div className="absolute inset-0 bg-[var(--color-accent)] rounded-full opacity-80" />
          <div className="absolute top-[2px] left-[4px] w-0 h-0 border-l-[5px] border-l-transparent border-r-[5px] border-r-transparent border-b-[8px] border-b-[var(--color-accent)]" />
          <div className="absolute top-[2px] right-[4px] w-0 h-0 border-l-[5px] border-l-transparent border-r-[5px] border-r-transparent border-b-[8px] border-b-[var(--color-accent)]" />
          <div className="absolute top-[16px] left-[12px] w-[4px] h-[4px] bg-[var(--color-text)] rounded-full" />
          <div className="absolute top-[16px] right-[12px] w-[4px] h-[4px] bg-[var(--color-text)] rounded-full" />
          <div className="absolute top-[22px] left-1/2 -translate-x-1/2 w-[3px] h-[2px] bg-[var(--color-primary)] rounded-full" />
        </div>
        <div>
          <h1 className="text-xl font-bold font-[var(--font-jp)] text-[var(--color-text)] leading-tight">
            Monika Now
          </h1>
          <p className="text-xs text-[var(--color-text-muted)] mt-0.5">
            what is she doing right now?
          </p>
        </div>
      </div>

      {/* Right: last update time */}
      <div className="text-right">
        <p className="text-xs text-[var(--color-text-muted)]">last update</p>
        <p className="text-sm font-mono font-medium text-[var(--color-secondary)]">
          {timeStr}
        </p>
      </div>
    </header>
  );
}
