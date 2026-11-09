// Adapted from BoardUI components/base/select/select.tsx, MIT; see LICENSE.
import { createContext, useContext, useRef, useState } from "react";
import type { ReactNode, Ref } from "react";
import { Button as AriaButton, ListBox as AriaListBox, ListBoxItem as AriaListBoxItem, Popover as AriaPopover, Select as AriaSelect, SelectValue as AriaSelectValue } from "react-aria-components";
import type { ListBoxItemProps as AriaListBoxItemProps, SelectProps as AriaSelectProps, SelectValueRenderProps as AriaSelectValueRenderProps } from "react-aria-components";
import { useDismissOnOutsidePress, useTriggerToggle } from "./use-dismiss-on-outside-press";

type SelectSize = "sm" | "md";
const SelectSizeContext = createContext<SelectSize>("md");
export interface SelectProps<T extends object> extends Omit<AriaSelectProps<T>, "children"> {
  className?: string; triggerClassName?: string; popoverClassName?: string; size?: SelectSize;
  children: ReactNode; items?: Iterable<T>;
  renderValue?: ReactNode | ((values: AriaSelectValueRenderProps<T>) => ReactNode);
  ref?: Ref<HTMLDivElement>;
}
export function Select<T extends object>({ className, triggerClassName, popoverClassName, size = "md", children, items, renderValue, ref, ...props }: SelectProps<T>) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const popoverRef = useRef<HTMLElement>(null);
  const [isOpen, setIsOpen] = useState(false);
  useDismissOnOutsidePress(isOpen, () => setIsOpen(false), [triggerRef, popoverRef]);
  const allowOpenChange = useTriggerToggle(isOpen, triggerRef);
  return <AriaSelect ref={ref} {...props} isOpen={isOpen} onOpenChange={(open) => allowOpenChange(open) && setIsOpen(open)} className={`board-select ${className || ""}`}>
    {({ isOpen }) => <>
      <AriaButton ref={triggerRef} className={`board-trigger board-select-trigger ${triggerClassName || ""}`}>
        <AriaSelectValue className="board-select-value">{renderValue as never}</AriaSelectValue>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true" className={isOpen ? "board-chevron open" : "board-chevron"}><path d="M4 6l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" /></svg>
      </AriaButton>
      <AriaPopover ref={popoverRef} isNonModal offset={4} className={`board-popover board-select-popover ${popoverClassName || ""}`}>
        <AriaListBox items={items} className="board-list"><SelectSizeContext.Provider value={size}>{children}</SelectSizeContext.Provider></AriaListBox>
      </AriaPopover>
    </>}
  </AriaSelect>;
}
export interface SelectItemProps extends Omit<AriaListBoxItemProps, "children"> { children?: ReactNode; }
export function SelectItem({ className, children, ...props }: SelectItemProps) {
  const size = useContext(SelectSizeContext);
  return <AriaListBoxItem {...props} className={(state) => ["board-option", `board-option-${size}`, (state.isFocused || state.isSelected) && "active", state.isDisabled && "disabled", typeof className === "function" ? className(state) : className].filter(Boolean).join(" ")}>{children}</AriaListBoxItem>;
}
