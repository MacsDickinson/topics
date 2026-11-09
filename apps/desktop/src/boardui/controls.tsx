import { parseDate } from "@internationalized/date";
import { Select as BoardSelect, SelectItem } from "./select";
import { DatePicker as BoardDatePicker } from "./date-picker";
import "./controls.css";

export function Select({ label, value, options, disabled, onChange }: {label: string; value: string; options: string[]; disabled: boolean; onChange: (value: string) => void}) {
  return <BoardSelect aria-label={label} selectedKey={value} isDisabled={disabled} onSelectionChange={(key) => key != null && onChange(String(key))}>{options.map((option) => <SelectItem key={option} id={option} textValue={option}>{option}</SelectItem>)}</BoardSelect>;
}
export function DatePicker({ label, value, disabled, onChange }: {label: string; value: string; disabled: boolean; onChange: (value: string) => void}) {
  return <div className="board-date"><span className="board-label">{label}</span><BoardDatePicker aria-label={label} value={value ? parseDate(value) : null} isDisabled={disabled} onChange={(date) => onChange(date?.toString() || "")} /></div>;
}
