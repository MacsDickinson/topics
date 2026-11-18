import { useEffect, useRef, useState } from "react";
import { DatePicker, Select } from "./boardui/controls";
import { ipc, apiError } from "./ipc";
import { SourceEvidence } from "./Evidence";
import { PilotSources, type PilotSource } from "./PilotSources";
import { Button, Field, PageHeader, Section } from "./ui";
import type { SourceRef } from "./workspace-types";

type Item = {
  item_id: string;
  topic_id: string;
  topic_name: string;
  revision_id: string;
  predicate: string;
  value: string;
  sources: SourceRef[];
  rule_id: string | null;
  original_value: string | null;
};
type Briefing = {
  root_id: string;
  generated_at: string;
  since: string | null;
  history_incomplete: boolean;
  briefing_token: string;
  sources: PilotSource[];
  items: Item[];
  rules: {
    rule_id: string;
    topic_id: string;
    topic_name: string;
    predicate: string;
    match_value: string;
    replacement: string;
  }[];
  occurrences: {
    rule_id: string;
    occurrence_id: string;
    observed_at: string;
    recurred: boolean;
  }[];
};
type Snapshot = { id: string; captured_at: string; briefing: Briefing };
type Classification =
  | "analysed"
  | "pending_unavailable"
  | "first_learned"
  | "unknown";
type Feedback = {
  id: string;
  snapshot_id: string;
  item_id: string | null;
  kind: "wrong" | "omitted" | "missed_commitment";
  text: string;
  evidence: string | null;
  evidence_at_cutoff: Classification;
  recorded_at: string;
  recorded_by: string;
};
type Judgement = {
  id: string;
  description: string;
  outcome: "surfaced_correctly" | "wrong" | "omitted";
  evidence_at_cutoff: Classification;
  feedback_id: string | null;
};
type Week = {
  week: string;
  project: string;
  phase: "baseline" | "pilot";
  snapshot_id: string | null;
  source_count: number | null;
  decision_count: number | null;
  commitment_count: number | null;
  total_minutes: number | null;
  correction_minutes: number | null;
  useful_change: string;
  workload_comparison: "comparable" | "materially_different" | "unknown";
  useful_change_judgement: "observed" | "none" | "unknown";
  manual_commitments: number | null;
  item_adjustments: { item_id: string; wrong: boolean }[];
  material_judgements: Judgement[];
  rule_occurrences: { id: string; rule_id: string; recurred: boolean }[];
  largest_cost_and_next_change: string;
};
type Comparisons = {
  baseline_week: string | null;
  pilot_weeks: string[];
  preparation_time: string;
  useful_change: string;
  omission_trend: string;
  source_coverage: string;
};
type Measures = {
  wrong_items: number;
  reported_misses: number;
  known_omissions: number;
  eligible_changes: number;
  pending_unavailable: number;
  first_learned: number;
  unknown: number;
  missed_commitments: number;
  draft_quality: {
    unchanged: number;
    edited: number;
    wrong: number;
    undecided: number;
  } | null;
  learning: {
    rule_id: string;
    topic_id: string;
    topic_name: string;
    predicate: string;
    match_value: string;
    replacement: string;
    eligible: number;
    recurrences: number;
  }[];
};
const classifications: [Classification, string][] = [
  ["unknown", "Unclassified / unknown"],
  ["analysed", "Evidence analysed before the meeting"],
  ["pending_unavailable", "Evidence pending or unavailable"],
  ["first_learned", "First learned in the meeting"],
];
const blankWeek = (week: string, project: string): Week => ({
  week,
  project,
  phase: "pilot",
  snapshot_id: null,
  source_count: null,
  decision_count: null,
  commitment_count: null,
  total_minutes: null,
  correction_minutes: null,
  useful_change: "",
  workload_comparison: "unknown",
  useful_change_judgement: "unknown",
  manual_commitments: null,
  item_adjustments: [],
  material_judgements: [],
  rule_occurrences: [],
  largest_cost_and_next_change: "",
});

function FeedbackForm({
  feedback,
  busy,
  reviewRequired,
  onChange,
  onSave,
  onCancel,
}: {
  feedback: Feedback;
  busy: boolean;
  reviewRequired: boolean;
  onChange: (value: Feedback) => void;
  onSave: () => void;
  onCancel: () => void;
}) {
  return (
    <Section
      title={
        feedback.kind === "wrong"
          ? "Correct an item"
          : feedback.kind === "missed_commitment"
            ? "Missed commitment"
            : "Topics missed something"
      }
      description="Save one sentence now; classification can wait. This stays local and does not change accepted knowledge."
    >
      <Field id="pilot-feedback" label="What was wrong or missed?">
        <textarea
          value={feedback.text}
          onChange={(e) => onChange({ ...feedback, text: e.target.value })}
        />
      </Field>
      <Field id="pilot-evidence" label="Evidence (optional)">
        <textarea
          value={feedback.evidence || ""}
          onChange={(e) =>
            onChange({ ...feedback, evidence: e.target.value || null })
          }
        />
      </Field>
      <Field id="pilot-classification" label="Evidence at briefing time">
        <select
          value={feedback.evidence_at_cutoff}
          onChange={(e) =>
            onChange({
              ...feedback,
              evidence_at_cutoff: e.target.value as Classification,
            })
          }
        >
          {classifications.map(([v, l]) => (
            <option key={v} value={v}>
              {l}
            </option>
          ))}
        </select>
      </Field>
      <Button
        variant="primary"
        disabled={busy || reviewRequired || !feedback.text.trim()}
        onClick={onSave}
      >
        Save feedback
      </Button>{" "}
      <Button disabled={busy} onClick={() => onCancel()}>
        Cancel
      </Button>
    </Section>
  );
}

export function Pilot({
  rootId,
  openSetup,
  openCorrection,
}: {
  rootId: string;
  openSetup: () => void;
  openCorrection: (item: Item, feedback: Feedback) => boolean;
}) {
  const [briefing, setBriefing] = useState<Briefing | null>(null),
    [snapshot, setSnapshot] = useState<Snapshot | null>(null),
    [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [refreshVersion, setRefreshVersion] = useState(0);
  const [loading, setLoading] = useState(true),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [notice, setNotice] = useState("");
  const [numberDrafts, setNumberDrafts] = useState<Record<string, string>>({});
  const [feedbackNeedsReview, setFeedbackNeedsReview] = useState(false);
  const [requiredRefresh, setRequiredRefresh] = useState(0);
  const [unsavedCorrection, setUnsavedCorrection] = useState(false);
  const [weekFeedback, setWeekFeedback] = useState<Feedback[]>([]);
  const [feedback, setFeedback] = useState<Feedback | null>(null),
    [saved, setSaved] = useState<Feedback[]>([]);
  const [weekKey, setWeekKey] = useState(""),
    [project, setProject] = useState(""),
    [week, setWeek] = useState<Week | null>(null),
    [measures, setMeasures] = useState<Measures | null>(null),
    [comparisons, setComparisons] = useState<Comparisons | null>(null);
  const generation = useRef(0);
  useEffect(() => {
    const g = ++generation.current;
    setLoading(true);
    setError("");
    setBriefing(null);
    setSnapshot(null);
    setSnapshots([]);
    setSaved([]);
    setWeek(null);
    setMeasures(null);
    setComparisons(null);
    void ipc<Briefing>("pilot_briefing", { root_id: rootId })
      .then((v) => {
        if (g === generation.current) {
          if (v.root_id === rootId) setBriefing(v);
          else
            setError(
              "Briefing unavailable: workspace changed. Refresh to try again.",
            );
        }
      })
      .catch((e) => {
        if (g === generation.current)
          setError(`Briefing unavailable: ${apiError(e).message}`);
      })
      .finally(() => {
        if (g === generation.current) setLoading(false);
      });
    void ipc<Snapshot[]>("pilot_snapshot_list", { root_id: rootId })
      .then((v) => {
        if (g === generation.current) setSnapshots(v);
      })
      .catch(() => {
        if (g === generation.current) setNotice("Saved briefings unavailable.");
      });
    return () => {
      generation.current++;
    };
  }, [rootId, refreshVersion]);
  useEffect(() => {
    if (!snapshot) {
      setSaved([]);
      return;
    }
    const g = generation.current;
    let active = true;
    void ipc<Feedback[]>("pilot_feedback_list", {
      root_id: rootId,
      snapshot_id: snapshot.id,
    })
      .then((v) => {
        if (active && g === generation.current) setSaved(v);
      })
      .catch((e) => {
        if (active && g === generation.current)
          setError(`Feedback unavailable: ${apiError(e).message}`);
      });
    return () => {
      active = false;
    };
  }, [rootId, snapshot]);
  useEffect(() => {
    if (!week?.snapshot_id) {
      setWeekFeedback([]);
      return;
    }
    let active = true;
    const g = generation.current;
    void ipc<Feedback[]>("pilot_feedback_list", {
      root_id: rootId,
      snapshot_id: week.snapshot_id,
    })
      .then((v) => {
        if (active && g === generation.current) setWeekFeedback(v);
      })
      .catch((e) => {
        if (active && g === generation.current)
          setError(`Weekly feedback unavailable: ${apiError(e).message}`);
      });
    return () => {
      active = false;
    };
  }, [rootId, week?.snapshot_id, saved]);
  async function run<T>(fn: () => Promise<T>, done: (v: T) => void) {
    if (busy) return;
    const g = generation.current;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const v = await fn();
      if (g === generation.current) done(v);
    } catch (e) {
      if (g === generation.current) {
        const failure = apiError(e);
        setError(failure.message);
        if (failure.code === "stale_scope" && feedback && !snapshot) {
          setFeedbackNeedsReview(true);
          setRequiredRefresh(refreshVersion + 1);
        }
      }
    } finally {
      if (g === generation.current) setBusy(false);
    }
  }
  const shown = snapshot?.briefing || briefing;
  const weekSnapshot = snapshots.find((s) => s.id === week?.snapshot_id);
  function startFeedback(item?: Item, kind?: "missed_commitment") {
    setFeedbackNeedsReview(false);
    setFeedback({
      id: crypto.randomUUID(),
      snapshot_id: snapshot?.id || "",
      item_id: item?.item_id || null,
      kind: item ? "wrong" : kind || "omitted",
      text: "",
      evidence: null,
      evidence_at_cutoff: "unknown",
      recorded_at: new Date().toISOString(),
      recorded_by: "user",
    });
  }
  function saveFeedback() {
    if (!feedback || !feedback.text.trim() || feedbackNeedsReview) return;
    void run(
      async () => {
        let current = snapshot;
        if (!current) {
          if (!briefing)
            throw new Error(
              "Refresh and review the briefing before saving feedback.",
            );
          const g = generation.current;
          current = await ipc<Snapshot>("pilot_snapshot_create", {
            root_id: rootId,
            briefing_token: briefing.briefing_token,
          });
          if (g !== generation.current) throw new Error("Workspace changed.");
          setSnapshot(current);
          setSnapshots((old) =>
            old.some((s) => s.id === current!.id) ? old : [current!, ...old],
          );
        }
        const entry = await ipc<Feedback>("pilot_feedback_save", {
          root_id: rootId,
          feedback: { ...feedback, snapshot_id: current.id },
        });
        return { current, entry };
      },
      ({ current, entry }) => {
        setSnapshot(current);
        setSnapshots((old) =>
          old.some((s) => s.id === current.id) ? old : [current, ...old],
        );
        setSaved((old) => [...old.filter((x) => x.id !== entry.id), entry]);
        setFeedback(null);
        setNotice(
          "Feedback saved locally. It does not approve a fact or rule.",
        );
      },
    );
  }
  function loadWeek() {
    if (!weekKey || !project.trim()) return;
    void run(
      () =>
        ipc<{
          week: Week | null;
          measures: Measures;
          comparisons: Comparisons;
        }>("pilot_week_get", {
          root_id: rootId,
          week: weekKey,
          project: project.trim(),
        }),
      (v) => {
        const loaded = v.week || blankWeek(weekKey, project.trim());
        setWeek(loaded);
        setNumberDrafts(
          Object.fromEntries(
            [
              "source_count",
              "decision_count",
              "commitment_count",
              "total_minutes",
              "correction_minutes",
              "manual_commitments",
            ].map((key) => [
              key,
              loaded[key as keyof Week] == null
                ? ""
                : String(loaded[key as keyof Week]),
            ]),
          ),
        );
        setMeasures(v.measures);
        setComparisons(v.comparisons);
      },
    );
  }
  function saveWeek() {
    if (
      !week ||
      week.material_judgements.some((j) => !j.description.trim()) ||
      week.rule_occurrences.some((o) => !o.rule_id.trim())
    ) {
      setError(
        "Complete each material change and correction rule ID before saving.",
      );
      return;
    }
    const numericKeys = [
      "source_count",
      "decision_count",
      "commitment_count",
      "total_minutes",
      "correction_minutes",
      "manual_commitments",
    ] as const;
    if (
      numericKeys.some(
        (key) =>
          numberDrafts[key] &&
          (!/^\d+$/.test(numberDrafts[key]) ||
            !Number.isSafeInteger(Number(numberDrafts[key])) ||
            Number(numberDrafts[key]) > 4294967295),
      )
    ) {
      setError(
        "Enter a nonnegative whole number, or leave the field blank when unknown.",
      );
      return;
    }
    const updated = {
      ...week,
      ...Object.fromEntries(
        numericKeys.map((key) => [
          key,
          numberDrafts[key] ? Number(numberDrafts[key]) : null,
        ]),
      ),
    } as Week;
    void run(
      () =>
        ipc<{
          week: Week | null;
          measures: Measures;
          comparisons: Comparisons;
        }>("pilot_week_save", { root_id: rootId, week: updated }),
      (v) => {
        setWeek(v.week || updated);
        setMeasures(v.measures);
        setComparisons(v.comparisons);
        setNotice("Weekly review saved locally.");
      },
    );
  }
  return (
    <div className="pilot-view">
      <PageHeader
        title="Today"
        description="A local briefing for your Friday sync."
      />
      <Button
        disabled={busy || loading}
        onClick={() => setRefreshVersion((v) => v + 1)}
      >
        Refresh briefing
      </Button>
      {loading && <p role="status">Checking local sources…</p>}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {notice && <p role="status">{notice}</p>}
      {shown && (
        <>
          <Section
            title={snapshot ? "Saved briefing" : "Briefing"}
            description={
              snapshot
                ? `Kept ${new Date(snapshot.captured_at).toLocaleString()}. Later source changes cannot rewrite this record.`
                : `Checked ${new Date(shown.generated_at).toLocaleString()}.`
            }
          >
            <PilotSources sources={shown.sources} />
            {shown.since && <p>Changes since the briefing on {new Date(shown.since).toLocaleString()}.</p>}
            {shown.history_incomplete && <p role="status">Analysis history is incomplete. Some changes may be missing from this briefing.</p>}
            {!shown.items.length && (
              <p>
                {!shown.history_incomplete && shown.sources.length > 0 &&
                shown.sources.every((s) => s.status === "checked")
                  ? "No new changes in analysed sources."
                  : "No topic changes are available in this briefing."}
              </p>
            )}
            {shown.items.map((item) => (
              <article className="candidate" key={item.item_id}>
                <h3>{item.topic_name}</h3>
                <p>
                  <strong>{item.predicate}:</strong> {item.value}
                </p>
                {item.original_value && (
                  <p>Earlier value: {item.original_value}</p>
                )}
                <details>
                  <summary>Sources ({item.sources.length})</summary>
                  {item.sources.map((source, i) => (
                    <SourceEvidence key={i} source={source} />
                  ))}
                </details>
                <Button onClick={() => startFeedback(item)}>
                  Correct {item.topic_name}
                </Button>
              </article>
            ))}
            <Button onClick={() => startFeedback()}>
              Topics missed something
            </Button>{" "}
            <Button
              onClick={() => startFeedback(undefined, "missed_commitment")}
            >
              Add missed commitment
            </Button>{" "}
            {snapshot ? (
              <Button
                onClick={() => {
                  setSnapshot(null);
                  setFeedback(null);
                }}
              >
                Back to Today
              </Button>
            ) : (
              <>
                <Button
                  variant="primary"
                  disabled={busy}
                  onClick={() =>
                    void run(
                      () =>
                        ipc<Snapshot>("pilot_snapshot_create", {
                          root_id: rootId,
                          briefing_token: shown.briefing_token,
                        }),
                      (v) => {
                        setSnapshot(v);
                        setSnapshots((old) =>
                          old.some((s) => s.id === v.id) ? old : [v, ...old],
                        );
                      },
                    )
                  }
                >
                  Keep briefing for meeting
                </Button>{" "}
                <Button onClick={openSetup}>Review batch authorization</Button>
              </>
            )}
          </Section>
        </>
      )}
      {snapshots.length > 0 && (
        <Section title="Saved briefings">
          {snapshots.map((s) => (
            <p key={s.id}>
              <Button
                disabled={busy}
                onClick={() =>
                  void run(
                    () =>
                      ipc<Snapshot>("pilot_snapshot_get", {
                        root_id: rootId,
                        snapshot_id: s.id,
                      }),
                    setSnapshot,
                  )
                }
              >
                Open briefing {new Date(s.captured_at).toLocaleString()}
              </Button>
            </p>
          ))}
        </Section>
      )}
      {unsavedCorrection && (
        <div className="notice" role="status">
          <p>
            This briefing item is not a saved topic yet, or the topic list has
            not loaded. Your feedback is preserved; review and save the topic in
            folder setup, then return here.
          </p>
          <Button onClick={openSetup}>Open folder setup</Button>{" "}
          <Button onClick={() => setUnsavedCorrection(false)}>Dismiss</Button>
        </div>
      )}
      {feedback && feedbackNeedsReview && (
        <div className="notice" role="status">
          <p>
            The briefing changed while you were writing. Your sentence is kept;
            refresh and review the updated evidence before saving.
          </p>
          {briefing &&
            !loading &&
            refreshVersion >= requiredRefresh &&
            (feedback.kind !== "wrong" ||
              briefing.items.some(
                (item) => item.item_id === feedback.item_id,
              )) && (
              <Button onClick={() => setFeedbackNeedsReview(false)}>
                I reviewed the refreshed briefing
              </Button>
            )}
          {briefing &&
            !loading &&
            refreshVersion >= requiredRefresh &&
            feedback.kind === "wrong" &&
            !briefing.items.some(
              (item) => item.item_id === feedback.item_id,
            ) && (
              <p>
                The original item is no longer in this briefing. Keep your
                sentence and report it against an appropriate saved briefing or
                cancel this report.
              </p>
            )}
        </div>
      )}
      {feedback && (
        <FeedbackForm
          feedback={feedback}
          busy={busy}
          reviewRequired={feedbackNeedsReview}
          onChange={setFeedback}
          onSave={saveFeedback}
          onCancel={() => setFeedback(null)}
        />
      )}
      {snapshot &&
        saved.filter((f) => f.snapshot_id === snapshot.id).length > 0 && (
          <Section title="Your feedback">
            {saved
              .filter((f) => f.snapshot_id === snapshot.id)
              .map((f) => {
                const item = shown?.items.find((i) => i.item_id === f.item_id);
                return (
                  <article key={f.id} className="candidate">
                    <p>{f.text}</p>
                    <p>
                      Reported by {f.recorded_by} ·{" "}
                      {new Date(f.recorded_at).toLocaleString()}
                    </p>
                    <p>
                      {
                        classifications.find(
                          ([v]) => v === f.evidence_at_cutoff,
                        )?.[1]
                      }
                    </p>
                    <Button onClick={() => setFeedback(f)}>
                      Edit feedback
                    </Button>
                    {item && (
                      <Button
                        onClick={() => {
                          if (!openCorrection(item, f))
                            setUnsavedCorrection(true);
                        }}
                      >
                        Review scoped correction for {item.topic_name}
                      </Button>
                    )}
                  </article>
                );
              })}
          </Section>
        )}
      <details>
        <summary>Weekly review</summary>
        <Section
          title="Weekly review"
          description="Record the baseline and each comparable pilot week. A zero denominator is inconclusive."
        >
          <div className="form-grid">
            <DatePicker
              label="Week"
              value={weekKey}
              disabled={busy}
              onChange={setWeekKey}
            />
            <Field id="pilot-project" label="Project">
              <input
                value={project}
                onChange={(e) => setProject(e.target.value)}
              />
            </Field>
          </div>
          <Button
            disabled={busy || !weekKey || !project.trim()}
            onClick={loadWeek}
          >
            Open weekly review
          </Button>
          {week && (
            <>
              <Field id="pilot-phase" label="Phase">
                <Select
                  label="Phase"
                  value={week.phase === "baseline" ? "Baseline" : "Pilot"}
                  options={["Baseline", "Pilot"]}
                  disabled={busy}
                  onChange={(value) =>
                    setWeek({
                      ...week,
                      phase: value === "Baseline" ? "baseline" : "pilot",
                    })
                  }
                />
              </Field>
              <Field id="pilot-snapshot" label="Meeting briefing">
                <select
                  value={week.snapshot_id || ""}
                  onChange={(e) =>
                    setWeek({ ...week, snapshot_id: e.target.value || null })
                  }
                >
                  <option value="">None selected</option>
                  {snapshots.map((s) => (
                    <option key={s.id} value={s.id}>
                      {new Date(s.captured_at).toLocaleString()}
                    </option>
                  ))}
                </select>
              </Field>
              <div className="form-grid">
                {(
                  [
                    ["source_count", "Sources"],
                    ["decision_count", "Decisions"],
                    ["commitment_count", "Commitments"],
                    ["total_minutes", "Total preparation minutes"],
                    ["correction_minutes", "Correction minutes"],
                    ["manual_commitments", "Manually added commitments"],
                  ] as const
                ).map(([key, label]) => (
                  <Field key={key} id={`pilot-${key}`} label={label}>
                    <input
                      type="text"
                      inputMode="numeric"
                      value={numberDrafts[key] ?? ""}
                      onChange={(e) =>
                        setNumberDrafts({
                          ...numberDrafts,
                          [key]: e.target.value,
                        })
                      }
                    />
                  </Field>
                ))}
              </div>
              <Field
                id="pilot-comparable"
                label="Workload compared with baseline"
              >
                <select
                  value={week.workload_comparison}
                  onChange={(e) =>
                    setWeek({
                      ...week,
                      workload_comparison: e.target
                        .value as Week["workload_comparison"],
                    })
                  }
                >
                  <option value="unknown">Unknown</option>
                  <option value="comparable">Comparable</option>
                  <option value="materially_different">
                    Materially different
                  </option>
                </select>
              </Field>
              <Field
                id="pilot-useful-judgement"
                label="Useful change observed before the meeting"
              >
                <select
                  value={week.useful_change_judgement}
                  onChange={(e) =>
                    setWeek({
                      ...week,
                      useful_change_judgement: e.target
                        .value as Week["useful_change_judgement"],
                    })
                  }
                >
                  <option value="unknown">Unknown</option>
                  <option value="observed">Observed</option>
                  <option value="none">None</option>
                </select>
              </Field>
              <Field id="pilot-useful" label="Useful change surfaced">
                <textarea
                  value={week.useful_change}
                  onChange={(e) =>
                    setWeek({ ...week, useful_change: e.target.value })
                  }
                />
              </Field>
              <Field id="pilot-cost" label="Largest cost and next change">
                <textarea
                  value={week.largest_cost_and_next_change}
                  onChange={(e) =>
                    setWeek({
                      ...week,
                      largest_cost_and_next_change: e.target.value,
                    })
                  }
                />
              </Field>
              <h3>Material changes</h3>
              <p>
                Record each distinct change once. Edit its classification as
                evidence becomes clear.
              </p>
              {weekSnapshot && (
                <section aria-label="Briefing items to review">
                  <h4>Briefing items</h4>
                  <p>Count only changes that mattered at the meeting.</p>
                  {weekSnapshot.briefing.items.map((item) => (
                    <article className="candidate" key={item.item_id}>
                      <p>
                        {item.topic_name}: {item.predicate} — {item.value}
                      </p>
                      <details>
                        <summary>Sources ({item.sources.length})</summary>
                        {item.sources.map((source, i) => (
                          <SourceEvidence key={i} source={source} />
                        ))}
                      </details>
                      <Button
                        disabled={week.material_judgements.some(
                          (j) => j.id === item.item_id,
                        )}
                        onClick={() =>
                          setWeek({
                            ...week,
                            material_judgements: [
                              ...week.material_judgements,
                              {
                                id: item.item_id,
                                description: `${item.topic_name}: ${item.predicate} — ${item.value}`,
                                outcome: "surfaced_correctly",
                                evidence_at_cutoff:
                                  item.sources.length &&
                                  item.sources.every((source) =>
                                    weekSnapshot.briefing.sources.some(
                                      (s) =>
                                        s.analysed_hash ===
                                          source.document_hash &&
                                        s.path === source.path,
                                    ),
                                  )
                                    ? "analysed"
                                    : "unknown",
                                feedback_id: null,
                              },
                            ],
                          })
                        }
                      >
                        Count {item.topic_name} {item.predicate} as material
                      </Button>
                    </article>
                  ))}
                </section>
              )}
              {weekFeedback.length > 0 && (
                <section aria-label="Meeting feedback">
                  <h4>Meeting feedback</h4>
                  {weekFeedback.map((f) => (
                    <article className="candidate" key={f.id}>
                      <p>
                        {f.text} —{" "}
                        {
                          classifications.find(
                            ([v]) => v === f.evidence_at_cutoff,
                          )?.[1]
                        }
                      </p>
                      {f.kind === "omitted" ? (
                        <p>
                          Reported omissions are counted from this saved
                          feedback.
                        </p>
                      ) : (
                        f.kind === "wrong" && (
                          <Button
                            disabled={week.material_judgements.some(
                              (j) =>
                                j.feedback_id === f.id ||
                                (f.item_id !== null && j.id === f.item_id),
                            )}
                            onClick={() =>
                              setWeek({
                                ...week,
                                material_judgements: [
                                  ...week.material_judgements,
                                  {
                                    id: f.item_id || crypto.randomUUID(),
                                    description: f.text,
                                    outcome: "wrong",
                                    evidence_at_cutoff: f.evidence_at_cutoff,
                                    feedback_id: f.id,
                                  },
                                ],
                              })
                            }
                          >
                            Count this wrong item as material
                          </Button>
                        )
                      )}
                    </article>
                  ))}
                </section>
              )}
              {week.material_judgements.map((j, index) => (
                <article className="candidate" key={j.id}>
                  <Field id={`judgement-${j.id}`} label="Material change">
                    <input
                      value={j.description}
                      onChange={(e) =>
                        setWeek({
                          ...week,
                          material_judgements: week.material_judgements.map(
                            (x, i) =>
                              i === index
                                ? { ...x, description: e.target.value }
                                : x,
                          ),
                        })
                      }
                    />
                  </Field>
                  <Field id={`feedback-link-${j.id}`} label="Related feedback">
                    <select
                      value={j.feedback_id || ""}
                      onChange={(e) =>
                        setWeek({
                          ...week,
                          material_judgements: week.material_judgements.map(
                            (x, i) =>
                              i === index
                                ? { ...x, feedback_id: e.target.value || null }
                                : x,
                          ),
                        })
                      }
                    >
                      <option value="">None</option>
                      {weekFeedback
                        .filter(
                          (f) =>
                            ((j.outcome === "wrong" && f.kind === "wrong") ||
                              (j.outcome === "omitted" &&
                                f.kind === "omitted")) &&
                            !week.material_judgements.some(
                              (other) =>
                                other.id !== j.id &&
                                (other.feedback_id === f.id ||
                                  (f.item_id !== null &&
                                    other.id === f.item_id)),
                            ),
                        )
                        .map((f) => (
                          <option key={f.id} value={f.id}>
                            {f.text}
                          </option>
                        ))}
                    </select>
                  </Field>
                  <Field id={`outcome-${j.id}`} label="Outcome">
                    <select
                      value={j.outcome}
                      onChange={(e) =>
                        setWeek({
                          ...week,
                          material_judgements: week.material_judgements.map(
                            (x, i) =>
                              i === index
                                ? {
                                    ...x,
                                    outcome: e.target
                                      .value as Judgement["outcome"],
                                    feedback_id: null,
                                  }
                                : x,
                          ),
                        })
                      }
                    >
                      <option value="surfaced_correctly">
                        Surfaced correctly
                      </option>
                      <option value="wrong">Wrong</option>
                      <option value="omitted">Omitted</option>
                    </select>
                  </Field>
                  {j.feedback_id ? (
                    <p>
                      Evidence classification:{" "}
                      {classifications.find(
                        ([v]) =>
                          v ===
                          weekFeedback.find((f) => f.id === j.feedback_id)
                            ?.evidence_at_cutoff,
                      )?.[1] || "Feedback unavailable"}
                      . Edit it in the saved briefing.
                    </p>
                  ) : (
                    <Field
                      id={`cutoff-${j.id}`}
                      label="Evidence at briefing time"
                    >
                      <select
                        value={j.evidence_at_cutoff}
                        onChange={(e) =>
                          setWeek({
                            ...week,
                            material_judgements: week.material_judgements.map(
                              (x, i) =>
                                i === index
                                  ? {
                                      ...x,
                                      evidence_at_cutoff: e.target
                                        .value as Classification,
                                    }
                                  : x,
                            ),
                          })
                        }
                      >
                        {classifications.map(([v, l]) => (
                          <option key={v} value={v}>
                            {l}
                          </option>
                        ))}
                      </select>
                    </Field>
                  )}
                  <Button
                    onClick={() =>
                      setWeek({
                        ...week,
                        material_judgements: week.material_judgements.filter(
                          (_, i) => i !== index,
                        ),
                      })
                    }
                  >
                    Remove material change
                  </Button>
                </article>
              ))}
              <Button
                onClick={() =>
                  setWeek({
                    ...week,
                    material_judgements: [
                      ...week.material_judgements,
                      {
                        id: crypto.randomUUID(),
                        description: "",
                        outcome: "surfaced_correctly",
                        evidence_at_cutoff: "unknown",
                        feedback_id: null,
                      },
                    ],
                  })
                }
              >
                Add material change
              </Button>
              <h3>Correct the wrong-item tally</h3>
              <p>
                Use the automatic count, or override it for a specific item.
              </p>
              {weekSnapshot?.briefing.items.map((item) => (
                <Field
                  key={item.item_id}
                  id={`adjust-${item.item_id}`}
                  label={`${item.topic_name}: ${item.predicate} — ${item.value}`}
                >
                  <select
                    value={
                      week.item_adjustments.find(
                        (a) => a.item_id === item.item_id,
                      )?.wrong === true
                        ? "wrong"
                        : week.item_adjustments.find(
                              (a) => a.item_id === item.item_id,
                            )?.wrong === false
                          ? "correct"
                          : "automatic"
                    }
                    onChange={(e) =>
                      setWeek({
                        ...week,
                        item_adjustments: [
                          ...week.item_adjustments.filter(
                            (a) => a.item_id !== item.item_id,
                          ),
                          ...(e.target.value === "automatic"
                            ? []
                            : [
                                {
                                  item_id: item.item_id,
                                  wrong: e.target.value === "wrong",
                                },
                              ]),
                        ],
                      })
                    }
                  >
                    <option value="automatic">Automatic</option>
                    <option value="wrong">Count as wrong</option>
                    <option value="correct">
                      New information or not wrong
                    </option>
                  </select>
                </Field>
              ))}
              <h3>Review later examples</h3>
              {weekSnapshot?.briefing.occurrences.map((o) => {
                const rule = weekSnapshot.briefing.rules.find(
                  (r) => r.rule_id === o.rule_id,
                );
                const override = week.rule_occurrences.find(
                  (x) => x.id === o.occurrence_id,
                );
                return (
                  <Field
                    key={o.occurrence_id}
                    id={`occurrence-${o.occurrence_id}`}
                    label={`${rule?.topic_name || "Correction"}: example ${new Date(o.observed_at).toLocaleString()}`}
                  >
                    <select
                      value={
                        override
                          ? override.recurred
                            ? "recurred"
                            : "held"
                          : "automatic"
                      }
                      onChange={(e) =>
                        setWeek({
                          ...week,
                          rule_occurrences: [
                            ...week.rule_occurrences.filter(
                              (x) => x.id !== o.occurrence_id,
                            ),
                            ...(e.target.value === "automatic"
                              ? []
                              : [
                                  {
                                    id: o.occurrence_id,
                                    rule_id: o.rule_id,
                                    recurred: e.target.value === "recurred",
                                  },
                                ]),
                          ],
                        })
                      }
                    >
                      <option value="automatic">
                        Automatic:{" "}
                        {o.recurred ? "error recurred" : "correction held"}
                      </option>
                      <option value="held">Correction held</option>
                      <option value="recurred">Error recurred</option>
                    </select>
                  </Field>
                );
              })}
              <Button variant="primary" disabled={busy} onClick={saveWeek}>
                Save weekly review
              </Button>
            </>
          )}
          {measures && (
            <div>
              <h3>Measures</h3>
              <p>
                Items that may have needed correction: {measures.wrong_items};
                reported wrong items:{" "}
                {
                  new Set(
                    weekFeedback
                      .filter((f) => f.kind === "wrong" && f.item_id)
                      .map((f) => f.item_id),
                  ).size
                }
                ; reported misses: {measures.reported_misses}; missed
                commitments: {measures.missed_commitments}
              </p>
              <p>
                Known in-scope omissions: {measures.known_omissions} /{" "}
                {measures.eligible_changes}
              </p>
              <p>
                Pending or unavailable: {measures.pending_unavailable}; first
                learned: {measures.first_learned}; unknown: {measures.unknown}
              </p>
              {measures.draft_quality ? (
                <p>
                  Drafts: {measures.draft_quality.unchanged} unchanged,{" "}
                  {measures.draft_quality.edited} edited,{" "}
                  {measures.draft_quality.wrong} wrong,{" "}
                  {measures.draft_quality.undecided} undecided.
                </p>
              ) : (
                <p>Draft quality unavailable.</p>
              )}
              <p>Since correction approval, as of this briefing:</p>
              {measures.learning.length ? (
                measures.learning.map((l) => (
                  <p key={l.rule_id}>
                    {l.topic_name}: {l.predicate}, {l.match_value} →{" "}
                    {l.replacement}:{" "}
                    {l.eligible
                      ? `${l.eligible} later occurrences, ${l.recurrences} recurrences`
                      : "no later example"}
                  </p>
                ))
              ) : (
                <p>No correction rules evaluated.</p>
              )}
            </div>
          )}
          {comparisons && (
            <div>
              <h3>Pilot comparison</h3>
              <p>
                Baseline: {comparisons.baseline_week || "not recorded"}; pilot
                weeks: {comparisons.pilot_weeks.join(", ") || "none recorded"}.
              </p>
              <ul>
                <li>Preparation time: {comparisons.preparation_time}</li>
                <li>Useful change: {comparisons.useful_change}</li>
                <li>Omission trend: {comparisons.omission_trend}</li>
                <li>Source status recorded: {comparisons.source_coverage}</li>
              </ul>
            </div>
          )}
        </Section>
      </details>
    </div>
  );
}
