import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';

// Only the app-owned root directory is outside the source-note byte guard.
// Symlinks remain forbidden, including at the reserved path.
export function sourceSnapshot(root) {
  function walk(directory, isRoot) {
    return readdirSync(directory, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name))
      .flatMap((entry) => {
        const path = join(directory, entry.name);
        if (entry.isSymbolicLink()) throw new Error('Unexpected symlink in synthetic notes');
        if (isRoot && entry.name === 'topics' && entry.isDirectory()) return [];
        return [[entry.name, entry.isDirectory() ? walk(path, false) : createHash('sha256').update(readFileSync(path)).digest('hex')]];
      });
  }
  return walk(root, true);
}
