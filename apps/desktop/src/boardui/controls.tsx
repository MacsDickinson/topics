// Adapted from BoardUI select.tsx, date-picker.tsx and shared.tsx; see LICENSE.
import { useEffect, useRef, useState } from "react";
import { parseDate, type CalendarDate } from "@internationalized/date";
import { Button, Calendar, CalendarCell, CalendarGrid, CalendarGridBody, CalendarGridHeader, CalendarHeaderCell, Dialog, Heading, ListBox, ListBoxItem, Popover, Select as AriaSelect, SelectValue } from "react-aria-components";
import "./controls.css";

export function Select({ label, value, options, disabled, onChange }: {label: string; value: string; options: string[]; disabled: boolean; onChange: (value: string) => void}) {
  return <AriaSelect aria-label={label} selectedKey={value} isDisabled={disabled} onSelectionChange={(key) => key != null && onChange(String(key))} className="board-select">
    <Button className="board-trigger"><SelectValue /><span aria-hidden>⌄</span></Button>
    <Popover className="board-popover" offset={4}><ListBox className="board-list">{options.map(option => <ListBoxItem key={option} id={option} textValue={option} className="board-option">{option}</ListBoxItem>)}</ListBox></Popover>
  </AriaSelect>;
}

export function DatePicker({ label, value, disabled, onChange }: {label: string; value: string; disabled: boolean; onChange: (value: string) => void}) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [pendingValue, setPendingValue] = useState<CalendarDate | null>(null);
  const [draft, setDraft] = useState(value);
  const [invalid, setInvalid] = useState(false);
  useEffect(() => { if (isOpen) { setPendingValue(value ? parseDate(value) : null); setDraft(value); setInvalid(false); } }, [isOpen]);
  const close = () => { setIsOpen(false); triggerRef.current?.focus(); };
  return <div className="board-date">
    <span className="board-label">{label}</span>
    <button ref={triggerRef} type="button" className="board-trigger" disabled={disabled} aria-label={`${label}: ${value || "Select date"}`} onClick={() => setIsOpen(!isOpen)}>{value || "Select date"}<span aria-hidden>▦</span></button>
    <Popover triggerRef={triggerRef} isOpen={isOpen} onOpenChange={setIsOpen} offset={4} placement="bottom start" className="board-popover board-calendar">
      <Dialog aria-label={label}>
        <Calendar aria-label={label} value={pendingValue} onChange={(date) => { setPendingValue(date); setDraft(date.toString()); setInvalid(false); }}>
          <div className="board-calendar-heading"><Button slot="previous" aria-label="Previous month">‹</Button><Heading /><Button slot="next" aria-label="Next month">›</Button></div>
          <CalendarGrid><CalendarGridHeader>{day => <CalendarHeaderCell>{day}</CalendarHeaderCell>}</CalendarGridHeader><CalendarGridBody>{date => <CalendarCell date={date} />}</CalendarGridBody></CalendarGrid>
          <label className="board-label">Date (YYYY-MM-DD)<input aria-label={`${label} date (YYYY-MM-DD)`} value={draft} aria-invalid={invalid} onChange={(event) => { const text = event.target.value; setDraft(text); try { const date = parseDate(text); if (date.toString() !== text) throw new Error(); setPendingValue(date); setInvalid(false); } catch { setPendingValue(null); setInvalid(!!text); } }} /></label>{invalid && <p role="alert">Enter a valid date in YYYY-MM-DD format.</p>}
          <div className="board-calendar-actions"><Button slot={null} onPress={() => { onChange(""); close(); }}>Clear</Button><Button slot={null} onPress={close}>Cancel</Button><Button slot={null} isDisabled={!pendingValue || invalid} onPress={() => { if (pendingValue) onChange(pendingValue.toString()); close(); }}>Apply</Button></div>
        </Calendar>
      </Dialog>
    </Popover>
  </div>;
}
