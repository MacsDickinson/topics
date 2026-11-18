import { openDatePicker } from "./interaction";
import { $, expect } from '@wdio/globals';

describe('Friday meeting history after restarting Topics', () => {
  it('opens the most recent briefing and retrieves the saved feedback', async () => {
    await expect($('h1=Today')).toBeDisplayed();
    const latest = await $('button*=Open briefing');
    await latest.waitForDisplayed();
    await latest.click();
    await expect($('p=The second meeting moved to Wednesday.')).toBeDisplayed();
  });

  it('saves and reopens a partial weekly scorecard through real IPC', async () => {
    await $('summary=Weekly review').click();
    await openDatePicker('Week: Select date');
    await $('.board-date-chip').setValue('2026-11-20');
    await $('button=Apply').click();
    await $('#pilot-project').setValue('Fictional ledger');
    await $('button=Open weekly review').click();
    await $('#pilot-total_minutes').setValue('12');
    await $('#pilot-correction_minutes').setValue('0');
    await $('button=Save weekly review').click();
    await expect($('p=Weekly review saved locally.')).toBeDisplayed();
    await $('#pilot-project').setValue('Another project');
    await $('button=Open weekly review').click();
    await expect($('#pilot-total_minutes')).toHaveValue('');
    await $('#pilot-project').setValue('FICTIONAL LEDGER');
    await $('button=Open weekly review').click();
    await expect($('#pilot-total_minutes')).toHaveValue('12');
    await expect($('#pilot-correction_minutes')).toHaveValue('0');
    await expect($('#pilot-decision_count')).toHaveValue('');
  });
});
