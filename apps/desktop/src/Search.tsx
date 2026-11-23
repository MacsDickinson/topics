import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmptyState, Field, Icon, PageHeader } from "./ui";

type SearchResult = { title: string; path: string; snippet: string };
type SearchState = {
  status: "idle" | "loading" | "ready" | "error";
  results: SearchResult[];
  error?: string;
};

function Snippet({ text }: { text: string }) {
  // Interpret only FTS's exact highlight markers; all content remains React text.
  let highlighted = false;
  return text.split(/(<mark>|<\/mark>)/).map((part, index) => {
    if (part === "<mark>") {
      highlighted = true;
      return null;
    }
    if (part === "</mark>") {
      highlighted = false;
      return null;
    }
    return highlighted ? <mark key={index}>{part}</mark> : part;
  });
}

export function Search() {
  const input = useRef<HTMLInputElement>(null);
  const request = useRef(0);
  const [query, setQuery] = useState("");
  const [state, setState] = useState<SearchState>({
    status: "idle",
    results: [],
  });

  useEffect(() => {
    const current = ++request.current;
    if (!query.trim()) {
      setState({ status: "idle", results: [] });
      return;
    }
    setState({ status: "loading", results: [] });
    const timer = window.setTimeout(() => {
      invoke<SearchResult[]>("search_notes", { query }).then(
        (results) => {
          if (current === request.current)
            setState({ status: "ready", results });
        },
        (error) => {
          if (current === request.current)
            setState({
              status: "error",
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
      <PageHeader
        title="Search"
        description="Find a passage, retrace a thought, or return to an original note."
      />
      <div className="search-field">
        <Field
          id="search"
          label="Search notes"
          hint="Search titles and content in your selected notes folder."
        >
          <input
            ref={input}
            type="search"
            value={query}
            onChange={(event) => {
              ++request.current;
              setQuery(event.target.value);
              setState({
                status: event.target.value.trim() ? "loading" : "idle",
                results: [],
              });
            }}
            placeholder="Search titles and content"
            autoComplete="off"
            spellCheck={false}
          />
        </Field>
      </div>
      <section
        aria-label="Search results"
        aria-busy={state.status === "loading"}
        className="search-results"
      >
        <p role="status" className="result-status">
          {state.status === "idle"
            ? "Type to search your notes."
            : state.status === "loading"
              ? "Searching…"
              : state.status === "ready"
                ? state.results.length
                  ? `${state.results.length} ${state.results.length === 1 ? "result" : "results"}`
                  : "No matches"
                : ""}
        </p>
        {state.status === "error" && (
          <div className="error">
            <p role="alert">Search failed: {state.error}</p>
            <p>Check the notes folder and try your search again.</p>
          </div>
        )}
        {state.status === "idle" && (
          <EmptyState title="Your notes, within reach" icon="search">
            <p>
              Search for a name, project or phrase. Results include the original
              path and a matching passage.
            </p>
          </EmptyState>
        )}
        {state.status === "ready" && state.results.length === 0 && (
          <EmptyState title="Try another word or phrase" icon="search">
            <p>
              Check the spelling or use fewer words to find a broader set of
              passages.
            </p>
          </EmptyState>
        )}
        <ul className="result-list">
          {state.results.map((result) => (
            <li key={result.path} className="search-result">
              <Icon name="file" />
              <article>
                <h2>{result.title}</h2>
                <p className="path">{result.path}</p>
                <p className="result-excerpt">
                  <Snippet text={result.snippet} />
                </p>
              </article>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
