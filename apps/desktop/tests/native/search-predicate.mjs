// Executed inside the WebView: both fields must belong to the same result.
export function hasConvergedResult() {
  return Array.from(document.querySelectorAll('.result-list .search-result')).some((row) =>
    row.querySelector('h2')?.textContent?.trim() === 'Converged ledger' &&
    row.querySelector('.path')?.textContent?.trim() === 'new.md',
  );
}
