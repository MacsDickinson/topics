import { useEffect, useMemo, useState } from "react";
import { ipc, apiError } from "./ipc";
import { Badge, Button, EmptyState, Icon, PageHeader } from "./ui";
import type { KnowledgeGraph as Graph } from "./generated/KnowledgeGraph";
import type { CheckCounts } from "./generated/CheckCounts";
import type { GraphNode } from "./generated/GraphNode";
import type { PendingRef } from "./generated/PendingRef";
import type { TopicSearchResult } from "./generated/TopicSearchResult";
import "./knowledge.css";

const openChecks = (c: CheckCounts) => c.conflicting + c.uncertain + c.stale;
const count = (n: number, label: string) =>
  n + " " + label + (n === 1 ? "" : "s");
const nodeStatus = (node: GraphNode) =>
  node.status === "pending"
    ? "Pending"
    : node.verified
    ? "Claims confirmed"
    : "Accepted";

// Place each connected component around its most connected topic. The canvas
// scrolls at a readable scale as the workspace grows.
function layout(graph: Graph) {
  const neighbors = new Map(
    graph.nodes.map((n) => [n.topic.id, new Set<string>()]),
  );
  for (const edge of graph.edges) {
    neighbors.get(edge.from)?.add(edge.to);
    neighbors.get(edge.to)?.add(edge.from);
  }
  const remaining = new Set(
    [...graph.nodes]
      .sort(
        (a, b) =>
          neighbors.get(b.topic.id)!.size - neighbors.get(a.topic.id)!.size ||
          a.topic.name.localeCompare(b.topic.name),
      )
      .map((n) => n.topic.id),
  );
  const points = new Map<string, { x: number; y: number }>();
  let width = 680,
    height = 0;
  const isolated: string[] = [];
  for (const hub of remaining) {
    remaining.delete(hub);
    const group = [hub];
    for (const id of group) {
      for (const other of neighbors.get(id)!)
        if (remaining.delete(other)) group.push(other);
    }
    if (group.length === 1) {
      isolated.push(hub);
      continue;
    }
    const leaves = group.length - 1;
    const radius = Math.max(310, leaves * 50);
    const cx = radius + 110,
      cy = height + radius * 0.43 + 62;
    if (leaves === 1) {
      points.set(hub, { x: 150, y: height + 85 });
      points.set(group[1], { x: 530, y: height + 85 });
      height += 175;
    } else {
      points.set(hub, { x: cx, y: cy });
      group.slice(1).forEach((id, i) => {
        const angle = (i * 2 * Math.PI) / leaves - Math.PI / 2;
        points.set(id, {
          x: cx + radius * Math.cos(angle),
          y: cy + radius * 0.43 * Math.sin(angle),
        });
      });
      width = Math.max(width, cx * 2);
      height += radius * 0.86 + 124;
    }
  }
  isolated.forEach((id, i) =>
    points.set(id, {
      x: 140 + (i % 3) * 200,
      y: height + 70 + Math.floor(i / 3) * 110,
    }),
  );
  height += Math.ceil(isolated.length / 3) * 110;
  return { points, width, height: Math.max(170, height) };
}

function MapView({
  graph,
  selected,
  onSelect,
}: {
  graph: Graph;
  selected: string | null;
  onSelect: (id: string) => void;
}) {
  const { points, width, height } = useMemo(() => layout(graph), [graph]);
  const [text, setText] = useState(false);
  const names = new Map(graph.nodes.map((n) => [n.topic.id, n.topic.name]));
  useEffect(() => {
    const node = selected && document.getElementById("graph-node-" + selected);
    if (node) node.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [selected]);
  const pairs = new Map<string, number>();
  return (
    <section className="knowledge-map" aria-label="Workspace graph">
      <div className="map-bar">
        <span>
          {count(graph.nodes.length, "topic")} ·{" "}
          {count(graph.edges.length, "relationship")}
        </span>
        <Button
          variant="quiet"
          aria-pressed={text}
          onClick={() => setText(!text)}
        >
          {text ? "Show graph" : "Text view"}
        </Button>
      </div>
      {text ? (
        <div className="graph-text">
          <ul>
            {graph.nodes.map((n) => (
              <li key={n.topic.id}>
                <Button onClick={() => onSelect(n.topic.id)}>
                  {n.topic.name}
                </Button>{" "}
                {nodeStatus(n)}
              </li>
            ))}
          </ul>
          <ul>
            {graph.edges.map((e) => (
              <li key={e.id}>
                {names.get(e.from)} → {e.relation} → {names.get(e.to)} ·{" "}
                {e.status}
                {e.inferred && " · inferred"}
                {e.confirmed && " · confirmed"}
              </li>
            ))}
          </ul>
        </div>
      ) : (
        <div
          className="graph-scroll"
          tabIndex={0}
          aria-label="Scrollable topic map"
        >
          <div className="graph-canvas" style={{ width, height }}>
            <svg aria-hidden="true" width={width} height={height}>
              {graph.edges.map((e) => {
                const a = points.get(e.from)!,
                  b = points.get(e.to)!;
                const key = [e.from, e.to].sort().join("/");
                const i = pairs.get(key) || 0;
                pairs.set(key, i + 1);
                const dy =
                  i === 0 ? 0 : (i % 2 ? 1 : -1) * Math.ceil(i / 2) * 48;
                const self = e.from === e.to;
                const x = (a.x + b.x) / 2,
                  y = (a.y + b.y) / 2 + dy;
                const arrow =
                  Math.abs(b.x - a.x) > Math.abs(b.y - a.y)
                    ? b.x > a.x
                      ? "→"
                      : "←"
                    : b.y > a.y
                    ? "↓"
                    : "↑";
                return (
                  <g
                    key={e.id}
                    className={
                      e.status === "pending" || e.inferred ? "tentative" : ""
                    }
                  >
                    <path
                      d={
                        self
                          ? `M ${a.x - 45} ${
                              a.y
                            } q -75 -100 45 -50 q 100 -10 45 50`
                          : `M ${a.x} ${a.y} Q ${x} ${y + dy} ${b.x} ${b.y}`
                      }
                    />
                    <text x={x} y={self ? y - 49 : y - 5}>
                      {e.relation} {self ? "↺" : arrow}
                      {e.status === "pending"
                        ? " · pending"
                        : e.inferred
                        ? " · inferred"
                        : e.confirmed
                        ? " · confirmed"
                        : ""}
                    </text>
                  </g>
                );
              })}
            </svg>
            {graph.nodes.map((n) => {
              const pos = points.get(n.topic.id)!;
              const checks = openChecks(n.checks);
              return (
                <button
                  key={n.topic.id}
                  id={"graph-node-" + n.topic.id}
                  className={"graph-node " + n.status}
                  style={{ left: pos.x, top: pos.y }}
                  title={n.topic.name}
                  aria-label={n.topic.name}
                  aria-pressed={selected === n.topic.id}
                  onClick={() => onSelect(n.topic.id)}
                >
                  <Icon name={n.topic.topic_kind} />
                  <span>
                    <strong>{n.topic.name}</strong>
                    <small>
                      {nodeStatus(n)}
                      {checks > 0 && " · " + count(checks, "check")}
                    </small>
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      )}
    </section>
  );
}

export function GraphHome({
  rootId,
  graph,
  selected,
  onSelect,
  analyse,
  review,
  validation,
  failed,
  retry,
}: {
  rootId: string;
  graph: Graph | null;
  selected: string | null;
  onSelect: (id: string) => void;
  analyse: () => void;
  review: (ref: PendingRef) => void;
  validation: boolean;
  failed: boolean;
  retry: () => void;
}) {
  const [query, setQuery] = useState("");
  const [result, setResult] = useState<TopicSearchResult | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    let active = true;
    if (!query.trim()) return;
    const timer = setTimeout(() => {
      void ipc<TopicSearchResult>("topic_search", {
        root_id: rootId,
        query,
        limit: 20,
      })
        .then((r) => {
          if (active) setResult(r);
        })
        .catch((e) => {
          if (active) setError(apiError(e).message);
        });
    }, 120);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [query, rootId, graph]);
  const select = (id: string) => {
    setQuery("");
    setResult(null);
    setError("");
    onSelect(id);
  };
  const checks =
    graph?.nodes.filter(
      (n) => openChecks(n.checks) > 0 || n.checks.deferred > 0,
    ) || [];
  return (
    <>
      <PageHeader
        title={validation ? "Needs validation" : "Knowledge graph"}
        description={
          validation
            ? "Review the assertions behind each open question."
            : "Explore what your sources say and where they disagree."
        }
        actions={
          <Button icon="folder" onClick={analyse}>
            Analyse sources
          </Button>
        }
      />
      <label className="graph-search">
        <Icon name="search" />
        <input
          id="topic-search"
          type="search"
          aria-label="Search topics"
          placeholder="Find a topic, alias or claim…"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setResult(null);
            setError("");
          }}
        />
      </label>
      {query.trim() && (
        <div className="graph-hits" aria-live="polite">
          {error ||
            (result ? (
              <>
                <small>{count(result.total, "matching topic")}</small>
                {result.hits.map((h) => (
                  <button key={h.topic.id} onClick={() => select(h.topic.id)}>
                    <strong>{h.topic.name}</strong>
                    <span>
                      {h.snippet || h.topic.topic_kind} · {h.status}
                    </span>
                  </button>
                ))}
              </>
            ) : (
              "Searching…"
            ))}
        </div>
      )}
      {!graph ? (
        <div role="status">
          {failed ? (
            <>
              Graph unavailable. <Button onClick={retry}>Retry</Button>
            </>
          ) : (
            "Loading graph…"
          )}
        </div>
      ) : (
        <>
          {graph.warnings.map((w, i) => (
            <p className="notice" key={i}>
              {w}
            </p>
          ))}
          {!validation &&
            (graph.nodes.length ? (
              <MapView graph={graph} selected={selected} onSelect={select} />
            ) : (
              <EmptyState title="Build your first graph" icon="topics">
                <p>
                  Choose notes to analyse. Discovered topics and relationships
                  appear here for you to explore.
                </p>
              </EmptyState>
            ))}
          {(validation || checks.length > 0) && (
            <section className="graph-checks" aria-label="Validation by topic">
              {!validation && <h2>Needs validation</h2>}
              {!checks.length && <p>No open checks.</p>}
              {checks.map((n) => (
                <button key={n.topic.id} onClick={() => select(n.topic.id)}>
                  <strong>{n.topic.name}</strong>
                  <span>
                    {Object.entries(n.checks)
                      .filter(([k, v]) => k !== "unverified" && v > 0)
                      .map(([k, v]) => v + " " + k)
                      .join(" · ")}
                  </span>
                  <Icon name="chevron" />
                </button>
              ))}
            </section>
          )}
          {graph.pending.length > 0 && (
            <section className="graph-discoveries">
              <h2>Pending discoveries</h2>
              {graph.pending.map((d) => (
                <div key={d.decision.id}>
                  <Badge tone="info">{d.state.replaceAll("_", " ")}</Badge>{" "}
                  {count(d.topics, "topic")} ·{" "}
                  {count(d.assertions, "assertion")} ·{" "}
                  {count(d.relationships, "relationship")}{" "}
                  <Button variant="quiet" onClick={() => review(d.decision)}>
                    Open discovery
                  </Button>
                </div>
              ))}
            </section>
          )}
        </>
      )}
    </>
  );
}
