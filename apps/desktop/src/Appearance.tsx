import { useLayoutEffect, useState } from "react";

export const themes = [
  { id: "light", label: "Light", scheme: "light" },
  { id: "dark", label: "Dark", scheme: "dark" },
] as const;
const key = "topics.appearance";
const system = window.matchMedia("(prefers-color-scheme: dark)");

function readPreference() {
  try {
    const stored = localStorage.getItem(key);
    return themes.find((theme) => theme.id === stored)?.id ?? "system";
  } catch {
    return "system";
  }
}
function apply(preference: string) {
  const theme =
    themes.find((theme) => theme.id === preference) ??
    themes.find((theme) => theme.id === (system.matches ? "dark" : "light"))!;
  document.documentElement.dataset.theme = theme.id;
  document.documentElement.dataset.scheme = theme.scheme;
}
apply(readPreference());

export function Appearance() {
  const [preference, setPreference] = useState(readPreference);
  useLayoutEffect(() => {
    const sync = () => apply(preference);
    sync();
    system.addEventListener("change", sync);
    return () => system.removeEventListener("change", sync);
  }, [preference]);
  return (
    <div className="appearance">
      <label htmlFor="appearance" className="context-label">
        Appearance
      </label>
      <select
        id="appearance"
        value={preference}
        onChange={(event) => {
          const id = event.currentTarget.value;
          const next = themes.find((theme) => theme.id === id)?.id ?? "system";
          setPreference(next);
          try {
            localStorage.setItem(key, next);
          } catch {
            // Storage is optional. The selection still applies to this window.
          }
        }}
      >
        <option value="system">Follow system</option>
        {themes.map((theme) => (
          <option key={theme.id} value={theme.id}>
            {theme.label}
          </option>
        ))}
      </select>
    </div>
  );
}
