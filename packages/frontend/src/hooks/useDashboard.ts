"use client";

import { useState, useEffect, useCallback, useReducer } from "react";
import {
  fetchCurrent,
  fetchTimeline,
  type CurrentResponse,
  type TimelineResponse,
} from "@/lib/api";

const POLL_INTERVAL = 10 * 60 * 1000; // 10 minutes

function todayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

export function useDashboard() {
  const [current, setCurrent] = useState<CurrentResponse | null>(null);
  const [timeline, setTimeline] = useState<TimelineResponse | null>(null);
  const [selectedDate, setSelectedDate] = useState(todayStr);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  // Incrementing counter to force effect re-run on manual refresh
  const [refreshKey, forceRefresh] = useReducer((c: number) => c + 1, 0);

  useEffect(() => {
    const controller = new AbortController();
    let requestId = 0;

    const doFetch = async () => {
      const thisRequest = ++requestId;
      try {
        setError(null);
        setLoading(true);
        const [cur, tl] = await Promise.all([
          fetchCurrent(controller.signal),
          fetchTimeline(selectedDate, controller.signal),
        ]);
        // Only apply if this is still the latest request and not aborted
        if (!controller.signal.aborted && thisRequest === requestId) {
          setCurrent(cur);
          setTimeline(tl);
        }
      } catch (e) {
        if (!controller.signal.aborted && thisRequest === requestId) {
          setError(e instanceof Error ? e.message : "Failed to fetch data");
        }
      } finally {
        if (!controller.signal.aborted && thisRequest === requestId) {
          setLoading(false);
        }
      }
    };

    doFetch();
    const pollId = setInterval(doFetch, POLL_INTERVAL);

    return () => {
      controller.abort();
      clearInterval(pollId);
    };
  }, [selectedDate, refreshKey]);

  const changeDate = useCallback((date: string) => {
    setSelectedDate(date);
  }, []);

  return { current, timeline, selectedDate, changeDate, loading, error, refresh: forceRefresh };
}
