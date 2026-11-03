import { browser, $ } from '@wdio/globals';
// Embedded WebDriver dispatches option clicks without WebKit's native selection
// default action. Use the DOM selection/change interaction, retaining React and
// every real Rust command; this does not replace transport or service behavior.
export async function chooseOption(selector: string, value: string) {
  await browser.execute(async () => { await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))); });
  await $(selector).waitForDisplayed();
  await $(selector).waitForEnabled();
  await browser.execute(
    (selector, value) => {
      const select = document.querySelector(selector);
      if (!(select instanceof HTMLSelectElement) || select.disabled)
        throw new Error('Select is unavailable');
      if (select.value === value) return;
      select.value = value;
      select.dispatchEvent(new Event('change', { bubbles: true }));
    },
    selector,
    value,
  );
}
