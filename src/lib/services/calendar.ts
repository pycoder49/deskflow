import { invoke } from '@tauri-apps/api/core';

export interface CalendarEvent {
  start_date: string;
  start_time: string;
  end_date: string;
  end_time: string;
  title: string;
  calendar: string;
  all_day: boolean;
}

// Calendar-source → palette color. Per-event colors aren't exposed via gcalcli
// TSV, so we color by source. Holidays are intentionally muted.
export const CALENDAR_COLORS: Record<string, string> = {
  'rn.ahuja04@gmail.com': '#7aa2f7', // Personal — soft blue
  'Family': '#e0af68',                // warm tangerine
  'Accelerate 2025-2026 Live Session & Events': '#9ece6a', // sage
};

const DEFAULT_COLOR = '#7aa2f7';
const MUTED_COLOR = '#6b7280';

export function eventColor(calendar: string): string {
  if (calendar.toLowerCase().includes('holiday')) return MUTED_COLOR;
  return CALENDAR_COLORS[calendar] ?? DEFAULT_COLOR;
}

// `start` and `end` are date strings — either YYYY-MM-DD or any phrase
// gcalcli accepts (e.g. "today", "in 7 days").
export const getCalendarEvents = (start: string, end: string) =>
  invoke<CalendarEvent[]>('get_calendar_events', { start, end });
