export type PilotSource = {
  source_id: string;
  path: string;
  current_hash: string | null;
  analysed_hash: string | null;
  status: "checked" | "pending" | "unavailable" | "skipped" | "partial";
  last_success_at: string | null;
};

const groups = [
  ["unavailable", "Source unavailable"],
  ["partial", "Analysis incomplete"],
  ["pending", "Awaiting batch authorization"],
  ["checked", "Analysed"],
  ["skipped", "Excluded from this briefing"],
] as const;

function LastAnalysis({ source }: { source: PilotSource }) {
  return <>Last successful analysis: {source.last_success_at
    ? <time dateTime={source.last_success_at}>{new Date(source.last_success_at).toLocaleString()}</time>
    : "never"}</>;
}

export function PilotSources({ sources }: { sources: PilotSource[] }) {
  if (!sources.length) {
    return <p>No sources are in this briefing. Choose the scope in folder setup.</p>;
  }
  return (
    <div className="pilot-coverage">
      <p>Source status ({sources.length})</p>
      {groups.map(([status, label]) => {
        const entries = sources.filter((source) => source.status === status);
        if (!entries.length) return null;
        const files = (
          <ul>{entries.map((source) => (
            <li key={source.source_id}>
              <strong>{source.path}</strong> · <LastAnalysis source={source} />
            </li>
          ))}</ul>
        );
        if (sources.length === 1) {
          return <div key={status}><strong>{label}</strong>{files}</div>;
        }
        return (
          <details key={status}>
            <summary>{label} ({entries.length})</summary>
            {files}
          </details>
        );
      })}
    </div>
  );
}
