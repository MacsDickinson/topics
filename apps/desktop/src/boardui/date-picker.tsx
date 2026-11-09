// Adapted from BoardUI components/base/date-picker/date-picker.tsx, MIT; see LICENSE.
import { useEffect, useRef, useState } from "react";
import type { RefObject } from "react";
import { Calendar, Dialog, Popover } from "react-aria-components";
import type { CalendarDate } from "@internationalized/date";
import { DateChipInput, MonthPanel, formatTriggerDate } from "./date-shared";
import { useDismissOnOutsidePress } from "./use-dismiss-on-outside-press";

export interface DatePickerProps {
  value?: CalendarDate | null;
  defaultValue?: CalendarDate | null;
  onChange?: (value: CalendarDate | null) => void;
  isDisabled?: boolean;
  className?: string;
  "aria-label"?: string;
  triggerRef?: RefObject<HTMLElement | null>;
  isOpen?: boolean;
  onOpenChange?: (isOpen: boolean) => void;
}
export function DatePicker({ value, defaultValue = null, onChange, isDisabled, className, "aria-label": ariaLabel = "Date", triggerRef: externalTriggerRef, isOpen: controlledIsOpen, onOpenChange: controlledOnOpenChange }: DatePickerProps) {
  const ownTriggerRef = useRef<HTMLButtonElement>(null);
  const triggerRef = externalTriggerRef ?? ownTriggerRef;
  const isExternal = externalTriggerRef !== undefined;
  const popoverRef = useRef<HTMLElement>(null);
  const [internalOpen, setInternalOpen] = useState(false);
  const isOpen = isExternal ? (controlledIsOpen ?? false) : internalOpen;
  const setIsOpen = isExternal ? (controlledOnOpenChange ?? (() => {})) : setInternalOpen;
  const isControlled = value !== undefined;
  const [internalValue, setInternalValue] = useState<CalendarDate | null>(defaultValue);
  const committedValue = isControlled ? (value ?? null) : internalValue;
  const [pendingValue, setPendingValue] = useState<CalendarDate | null>(committedValue);
  const [valid, setValid] = useState(true);
  const [openKey, setOpenKey] = useState(0);
  useEffect(() => { if (isOpen) { setPendingValue(committedValue); setValid(true); } }, [isOpen]);
  const close = () => { setIsOpen(false); triggerRef.current?.focus(); };
  const commit = (next: CalendarDate | null) => { if (!isControlled) setInternalValue(next); onChange?.(next); close(); };
  const openChange = (open: boolean) => { if (open) setOpenKey((key) => key + 1); setIsOpen(open); };
  useDismissOnOutsidePress(isOpen, () => setIsOpen(false), [triggerRef, popoverRef]);
  return <>
    {!isExternal && <button ref={ownTriggerRef} type="button" disabled={isDisabled} aria-label={`${ariaLabel}: ${committedValue ? formatTriggerDate(committedValue) : "Select date"}`} onClick={() => openChange(!isOpen)} className={`board-trigger board-date-trigger ${className || ""}`}>
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true"><rect x="3" y="4.5" width="14" height="12.5" rx="2" stroke="currentColor" strokeWidth="1.5" /><path d="M6.5 2.5v4M13.5 2.5v4M3 8h14" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" /></svg>
      <span>{committedValue ? formatTriggerDate(committedValue) : "Select date"}</span>
    </button>}
    <Popover ref={popoverRef} triggerRef={triggerRef} isOpen={isOpen} onOpenChange={openChange} offset={4} placement="bottom end" isNonModal className="board-popover board-calendar">
      <Dialog aria-label={ariaLabel} className="board-dialog">
        <Calendar key={openKey} aria-label={ariaLabel} value={pendingValue} onChange={(date) => { setPendingValue(date); setValid(true); }}>
          <div className="board-date-picker-content" onKeyDown={(event) => { if (event.key === "Escape") close(); }}>
            <MonthPanel offset={0} showPrev showNext />
            <div className="board-date-footer">
              <DateChipInput date={pendingValue} label={`${ariaLabel} date (YYYY-MM-DD)`} onCommit={setPendingValue} onValidityChange={setValid} />
              <div className="board-calendar-actions">
                <button type="button" onClick={() => commit(null)}>Clear</button>
                <button type="button" onClick={() => { setPendingValue(committedValue); close(); }}>Cancel</button>
                <button type="button" disabled={!pendingValue || !valid} onClick={() => commit(pendingValue)}>Apply</button>
              </div>
            </div>
          </div>
        </Calendar>
      </Dialog>
    </Popover>
  </>;
}
