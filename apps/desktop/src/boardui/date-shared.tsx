// Adapted from BoardUI components/base/date-picker/shared.tsx, MIT; see LICENSE.
import { useContext, useEffect, useState } from "react";
import { Button as RACButton, CalendarCell, CalendarGrid, CalendarGridBody, CalendarGridHeader, CalendarHeaderCell, CalendarStateContext, RangeCalendarStateContext, useLocale } from "react-aria-components";
import type { CalendarCellRenderProps } from "react-aria-components";
import { endOfWeek, isSameDay, parseDate, startOfWeek, type CalendarDate } from "@internationalized/date";

export function ChevronLeft16() { return <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M9 4L5.70711 7.29289C5.31658 7.68342 5.31658 8.31658 5.70711 8.70711L9 12" stroke="currentColor" strokeWidth="2" strokeLinecap="round" /></svg>; }
export function ChevronRight16() { return <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M7 4L10.2929 7.29289C10.6834 7.68342 10.6834 8.31658 10.2929 8.70711L7 12" stroke="currentColor" strokeWidth="2" strokeLinecap="round" /></svg>; }
export function formatTriggerDate(date: CalendarDate) { return date.toString(); }
export function formatChipDate(date: CalendarDate) { return date.toString(); }
export function parseChipDate(text: string): CalendarDate | null {
  try { const date = parseDate(text); return date.toString() === text ? date : null; } catch { return null; }
}
export function DayCell(props: CalendarCellRenderProps & { isRange: boolean }) {
  const { locale } = useLocale();
  const { date, formattedDate, isSelected, isSelectionStart, isSelectionEnd, isHovered, isFocusVisible, isDisabled, isOutsideMonth, isRange } = props;
  if (isOutsideMonth) return <div className="board-day-outside" />;
  const isWeekStart = isSameDay(date, startOfWeek(date, locale));
  const isWeekEnd = isSameDay(date, endOfWeek(date, locale));
  const isSingleDay = isRange ? isSelectionStart && isSelectionEnd : isSelected;
  const isEdge = isRange ? isSelectionStart || isSelectionEnd : isSelected;
  const extendStart = isRange && isSelected && !isSelectionStart && !isWeekStart;
  const extendEnd = isRange && isSelected && !isSelectionEnd && !isWeekEnd;
  return <div className="board-day">
    <span aria-hidden="true" className="board-day-range" style={{left: isSelectionStart ? "50%" : extendStart ? -6 : 0, right: isSelectionEnd ? "50%" : extendEnd ? -6 : 0, opacity: isSelected && !isSingleDay ? 1 : 0}} />
    <div className={["board-day-inner", isHovered && !isSelected && "hovered", isFocusVisible && "focused", isDisabled && "disabled"].filter(Boolean).join(" ")}>
      <span aria-hidden="true" className="board-day-edge" style={{opacity: isEdge ? 1 : 0}} />
      <span className="board-day-number">{formattedDate}</span>
    </div>
  </div>;
}
export function MonthPanel({ offset, showPrev, showNext, bare = false, hideHeader = false }: {offset: number; showPrev?: boolean; showNext?: boolean; bare?: boolean; hideHeader?: boolean}) {
  const rangeState = useContext(RangeCalendarStateContext);
  const singleState = useContext(CalendarStateContext);
  const state = rangeState ?? singleState;
  const isRange = rangeState != null;
  const panelDate = state ? state.visibleRange.start.add({ months: offset }) : null;
  // Keep civil dates out of JavaScript Date/time-zone conversion.
  const title = panelDate ? `${["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][panelDate.month - 1]} ${panelDate.year}` : "";
  return <div className={`board-month-panel${bare ? " bare" : ""}`}><div className="board-month-content">
    {!hideHeader && <div className="board-month-heading">
      {showPrev ? <RACButton slot="previous" aria-label="Previous month" className="board-month-nav"><ChevronLeft16 /></RACButton> : <span />}
      <span className="board-month-title">{title}</span>
      {showNext ? <RACButton slot="next" aria-label="Next month" className="board-month-nav"><ChevronRight16 /></RACButton> : <span />}
    </div>}
    <CalendarGrid offset={{ months: offset }} weekdayStyle="short" className="board-calendar-grid"><CalendarGridHeader>{(day) => <CalendarHeaderCell>{day.slice(0, 2)}</CalendarHeaderCell>}</CalendarGridHeader><CalendarGridBody>{(date) => <CalendarCell date={date} className="board-calendar-cell">{(cellProps) => <DayCell {...cellProps} isRange={isRange} />}</CalendarCell>}</CalendarGridBody></CalendarGrid>
  </div></div>;
}
export function DateChipInput({ date, label, onCommit, onValidityChange }: {date: CalendarDate | null; label: string; onCommit: (date: CalendarDate | null) => void; onValidityChange: (valid: boolean) => void}) {
  const formatted = date ? formatChipDate(date) : "";
  const [text, setText] = useState(formatted);
  useEffect(() => { setText(formatted); }, [formatted]);
  const change = (next: string) => { setText(next); const parsed = parseChipDate(next); onValidityChange(!!parsed); if (parsed) onCommit(parsed); };
  return <label className="board-chip-label">Date (YYYY-MM-DD)<input className="board-date-chip" type="text" dir="ltr" inputMode="numeric" value={text} aria-label={label} aria-invalid={!!text && !parseChipDate(text)} onChange={(event) => change(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") event.currentTarget.blur(); if (event.key === "Escape") { setText(formatted); onValidityChange(true); } }} />{!!text && !parseChipDate(text) && <span role="alert">Enter a valid date in YYYY-MM-DD format.</span>}</label>;
}
