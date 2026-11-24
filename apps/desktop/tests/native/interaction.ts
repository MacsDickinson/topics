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

// Wait for the native scroll position and popover anchor to settle before clicking.
export async function openDatePicker(label: string) {
  await $(`button[aria-label="${label}"]`).waitForExist();
  await browser.execute(async (label) => {
    const trigger = document.querySelector(`button[aria-label="${label}"]`) as HTMLElement;
    trigger.scrollIntoView({block: "center", behavior: "instant"});
    let previous = "", stable = 0;
    await new Promise<void>((resolve, reject) => {
      let frames = 0;
      const check = () => {
        const r = trigger.getBoundingClientRect();
        const current = [r.x, r.y, r.width, r.height, window.scrollX, window.scrollY].map(x => Math.round(x * 10)).join(",");
        stable = current === previous ? stable + 1 : 0;
        previous = current;
        if (stable >= 8) return resolve();
        if (++frames > 180) return reject(new Error("Date trigger did not settle"));
        requestAnimationFrame(check);
      };
      requestAnimationFrame(check);
    });
  }, label);
  await $(`button[aria-label="${label}"]`).click();
  await $(".board-date-chip").waitForDisplayed();
}

export async function openTool(name: string) {
  const menu = await $("nav details");
  if (await menu.getAttribute("open") === null) await menu.$("summary").click();
  await menu.$("button="+name).click();
}
