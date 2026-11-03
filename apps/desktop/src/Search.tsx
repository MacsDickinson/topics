import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

type SearchResult = { title: string; path: string; snippet: string };
type SearchState = {
  status: 'idle' | 'loading' | 'ready' | 'error';
  results: SearchResult[];
  error?: string;
};

function Snippet({ text }: { text: string }) {
  // Interpret only FTS's exact highlight markers; all content remains React text.
  let highlighted = false;
  return text.split(/(<mark>|<\/mark>)/).map((part, index) => {
    if (part === '<mark>') {
      highlighted = true;
      return null;
    }
    if (part === '</mark>') {
      highlighted = false;
      return null;
    }
    return highlighted ? <mark key={index}>{part}</mark> : part;
  });
}

export function Search() {
  const input = useRef<HTMLInputElement>(null);
  const request = useRef(0);
  const [query, setQuery] = useState('');
  const [state, setState] = useState<SearchState>({
    status: 'idle',
    results: [],
  });

  useEffect(() => {
    const focusSearch = (event: KeyboardEvent) => {
      if (event.metaKey && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        input.current?.focus();
        input.current?.select();
      }
    };
    window.addEventListener('keydown', focusSearch);
    return () => window.removeEventListener('keydown', focusSearch);
  }, []);

  useEffect(() => {
    const current = ++request.current;
    if (!query.trim()) {
      setState({ status: 'idle', results: [] });
      return;
    }
    setState({ status: 'loading', results: [] });
    const timer = window.setTimeout(() => {
      invoke<SearchResult[]>('search_notes', { query }).then(
        (results) => {
          if (current === request.current)
            setState({ status: 'ready', results });
        },
        (error) => {
          if (current === request.current)
            setState({
              status: 'error',
              results: [],
              error: error instanceof Error ? error.message : String(error),
            });
        },
      );
    }, 150);
    return () => {
      window.clearTimeout(timer);
      ++request.current;
    };
  }, [query]);

  return (
    <div className="search-view">
      <header className="page-header">
        <h1>Search</h1>
        <p>Find words in your notes.</p>
      </header>
      <label htmlFor="search">
        Search notes <span>⌘K</span>
      </label>
      <input
        ref={input}
        id="search"
        type="search"
        value={query}
        onChange={(event) => {
          ++request.current;
          setQuery(event.target.value);
          setState({
            status: event.target.value.trim() ? 'loading' : 'idle',
            results: [],
          });
        }}
        placeholder="Search titles and content"
        autoComplete="off"
        spellCheck={false}
      />
      <section
        aria-label="Search results"
        aria-busy={state.status === 'loading'}
      >
        <p role="status">
          {state.status === 'idle'
            ? 'Type to search your notes.'
            : state.status === 'loading'
              ? 'Searching…'
              : state.status === 'ready'
                ? state.results.length
                  ? `${state.results.length} ${state.results.length === 1 ? 'result' : 'results'}`
                  : 'No matches'
                : ''}
        </p>
        {state.status === 'error' && (
          <p role="alert">Search failed: {state.error}</p>
        )}
        <ul>
          {state.results.map((result) => (
            <li key={result.path}>
              <h2>{result.title}</h2>
              <p className="path">{result.path}</p>
              <p>
                <Snippet text={result.snippet} />
              </p>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
