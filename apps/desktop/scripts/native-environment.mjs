import { existsSync, mkdirSync, statSync } from 'node:fs';
import { isAbsolute, join } from 'node:path';

export function prepareNativeEnvironment(sandbox) {
  if (!isAbsolute(sandbox) || !statSync(sandbox).isDirectory()) throw new Error('Native sandbox must be an existing absolute temporary directory');
  const home = join(sandbox, 'session-home');
  const xdgData = join(home, 'xdg');
  // A GUI process needs a real, writable synthetic HOME, not a nonexistent path.
  // HOME alone does not isolate Cocoa/WebKit; the native overlay also uses a
  // nonpersistent WebKit store. None of this changes the production window.
  mkdirSync(join(home, 'Library', 'Application Support'), { recursive: true });
  mkdirSync(xdgData, { recursive: true });
  return { HOME: home, XDG_DATA_HOME: xdgData };
}

export function assertNativeEnvironment(environment, sandbox) {
  if (!sandbox || !isAbsolute(sandbox)) throw new Error('Native sandbox must be absolute');
  const expected = { HOME: join(sandbox, 'session-home'), XDG_DATA_HOME: join(sandbox, 'session-home', 'xdg') };
  for (const key of ['HOME', 'XDG_DATA_HOME']) {
    const path = environment[key];
    if (path !== expected[key] || !statSync(path).isDirectory()) throw new Error(`Native ${key} must be a pre-created synthetic directory`);
  }
}

export function assertNoTopicsFallback(sandbox) {
  const home = join(sandbox, 'session-home');
  const forbidden = [
    join(home, 'Library', 'Application Support', 'com.macsdickinson.topics'),
    join(home, 'xdg', 'com.macsdickinson.topics'),
    join(sandbox, 'invalid-relative-app-data'),
  ];
  for (const path of forbidden) {
    if (existsSync(path)) throw new Error(`Topics created unexpected fallback app data in the synthetic sandbox: ${path}`);
  }
}
