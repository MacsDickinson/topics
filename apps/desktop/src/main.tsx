import { WindowStatus } from "./WindowStatus";
import { Appearance } from "./Appearance";
import { Pilot } from "./Pilot";
import { ContextReview } from "./ContextReview";
import { FolderSetup } from "./FolderSetup";
import {
  Badge,
  Button,
  EmptyState,
  Field,
  Icon,
  PageHeader,
  PrivacyBadge,
  Section,
} from "./ui";
import { createRoot } from "react-dom/client";
import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Search } from "./Search";
import { Sources } from "./Sources";
import { ipc, apiError } from "./ipc";
import { ProposalEditor } from "./ProposalEditor";
import { TopicDetail } from "./TopicDetail";
import {
  emptySpec,
  emptyDraft,
  parseScope,
  hasPaths,
  ScopeFields,
  ScopeReview,
  type ScopeDraft,
} from "./Scope";
import type {
  ApiError,
  Candidate,
  Operation,
  Proposal,
  RootRef,
  RuntimeStatus,
  ScopePreview,
  Sensitivity,
  Topic,
  TopicView,
  WorkspaceRunEvent,
} from "./workspace-types";
import "./style.css";
import "./window-chrome.css";
if (import.meta.env.VITE_NATIVE_TEST === "1")
  await import("@wdio/tauri-plugin");
type Screen =
  | "today"
  | "search"
  | "topics"
  | "inbox"
  | "history"
  | "analyse"
  | "sources"
  | "setup";
type ActiveRun = { root: string; id: string; seq: number };
function validEvent(value: unknown): value is WorkspaceRunEvent {
  if (!value || typeof value !== "object") return false;
  const e = value as WorkspaceRunEvent;
  if (
    typeof e.root_id !== "string" ||
    typeof e.run_id !== "string" ||
    !Number.isSafeInteger(e.seq) ||
    e.seq < 0 ||
    !e.event ||
    typeof e.event !== "object"
  )
    return false;
  switch (e.event.kind) {
    case "text_delta":
      return typeof e.event.text === "string";
    case "proposal":
      return typeof e.event.proposal_id === "string";
    case "error":
      return (
        !!e.event.error &&
        typeof e.event.error.message === "string" &&
        typeof e.event.error.code === "string"
      );
    case "completed":
    case "cancelled":
      return true;
    default:
      return false;
  }
}
function App() {
  const [root, setRoot] = useState<RootRef | null>();
  const rootRef = useRef<RootRef | null>(null);
  const formTrigger = useRef<HTMLButtonElement | null>(null);
  const selectedTopicId = useRef<string | null>(null);
  const navigationSeq = useRef(0);
  const epoch = useRef(0),
    detailSeq = useRef(0),
    refreshSeq = useRef(0),
    scopeSeq = useRef(0),
    chatScopeSeq = useRef(0);
  const [screen, setScreen] = useState<Screen>("today");
  const [pilotCorrection, setPilotCorrection] = useState<{topicId: string; revisionId: string; statement: string} | null>(null);
  const [autoSetup, setAutoSetup] = useState(false);
  const [settings, setSettings] = useState(false),
    [path, setPath] = useState(""),
    [create, setCreate] = useState(false);
  const [topics, setTopics] = useState<Topic[]>([]),
    [proposals, setProposals] = useState<Proposal[]>([]),
    [operations, setOperations] = useState<Operation[]>([]);
  const [topic, setTopic] = useState<TopicView | null>(null),
    [topicLoading, setTopicLoading] = useState(false);
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null),
    [error, setError] = useState<ApiError | null>(null),
    [bootError, setBootError] = useState<ApiError | null>(null);
  const [busy, setBusy] = useState(false),
    [notice, setNotice] = useState("");
  const [spec, setSpec] = useState<ScopeDraft>(emptyDraft),
    [preview, setPreview] = useState<ScopePreview | null>(null),
    [confirmed, setConfirmed] = useState(false);
  const [chatPreview, setChatPreview] = useState<ScopePreview | null>(null),
    [chatConfirmed, setChatConfirmed] = useState(false);
  const [message, setMessage] = useState(""),
    [manual, setManual] = useState(false),
    [name, setName] = useState(""),
    [kind, setKind] = useState<Topic["topic_kind"]>("thread"),
    [manualSensitivity, setManualSensitivity] =
      useState<Sensitivity>("unknown");
  const [run, setRun] = useState<ActiveRun | null>(null),
    [stream, setStream] = useState("");
  const runRef = useRef<ActiveRun | null>(null),
    pendingStart = useRef(false),
    earlyEvents = useRef<WorkspaceRunEvent[]>([]),
    eventReady = useRef<Promise<void>>(Promise.resolve());
  const current = (generation: number, id?: string) =>
    epoch.current === generation && (!id || rootRef.current?.root_id === id);
  const report = (e: unknown) => setError(apiError(e));
  const invalidateAnalysisScope = () => {
    ++scopeSeq.current;
    setPreview(null);
    setConfirmed(false);
  };
  const invalidateChatScope = () => {
    ++chatScopeSeq.current;
    setChatPreview(null);
    setChatConfirmed(false);
  };
  const invalidateScope = () => {
    invalidateAnalysisScope();
    invalidateChatScope();
  };
  const clearTopic = () => {
    selectedTopicId.current = null;
    ++detailSeq.current;
    setTopic(null);
    setTopicLoading(false);
    setMessage("");
    invalidateScope();
  };
  async function refresh(selected: RootRef, generation = epoch.current) {
    const sequence = ++refreshSeq.current;
    const results = await Promise.allSettled([
      ipc<{ topics: Topic[] }>("workspace_topics_list", {
        root_id: selected.root_id,
      }),
      ipc<{ proposals: Proposal[] }>("workspace_proposals_list", {
        root_id: selected.root_id,
      }),
      ipc<{ operations: Operation[] }>("workspace_history_list", {
        root_id: selected.root_id,
      }),
    ]);
    if (
      !current(generation, selected.root_id) ||
      sequence !== refreshSeq.current
    )
      return;
    if (results[0].status === "fulfilled") setTopics(results[0].value.topics);
    if (results[1].status === "fulfilled")
      setProposals(
        results[1].value.proposals.filter(
          (p) => p.root_id === selected.root_id,
        ),
      );
    if (results[2].status === "fulfilled")
      setOperations(results[2].value.operations);
    const failure = results.find((r) => r.status === "rejected");
    if (failure?.status === "rejected") report(failure.reason);
    return results[1].status === "fulfilled"
      ? { proposals: results[1].value.proposals }
      : undefined;
  }
  async function openTopic(id: string) {
    const topicChanged = selectedTopicId.current !== id;
    selectedTopicId.current = id;
    const selected = rootRef.current;
    if (!selected) return;
    window.scrollTo(0, 0);
    const generation = epoch.current,
      sequence = ++detailSeq.current;
    if (topicChanged) {
      setTopic(null);
      setTopicLoading(true);
      setMessage("");
      invalidateChatScope();
    }
    setError(null);
    try {
      const result = await ipc<TopicView>("workspace_topic_get", {
        root_id: selected.root_id,
        topic_id: id,
      });
      if (
        current(generation, selected.root_id) &&
        sequence === detailSeq.current
      )
        setTopic(result);
    } catch (e) {
      if (
        current(generation, selected.root_id) &&
        sequence === detailSeq.current
      )
        report(e);
    } finally {
      if (
        current(generation, selected.root_id) &&
        sequence === detailSeq.current
      )
        setTopicLoading(false);
    }
  }
  function receive(value: unknown) {
    if (!validEvent(value) || value.root_id !== rootRef.current?.root_id)
      return;
    const active = runRef.current;
    if (!active) {
      if (pendingStart.current && earlyEvents.current.length < 512)
        earlyEvents.current.push(value);
      return;
    }
    if (
      active.root !== value.root_id ||
      active.id !== value.run_id ||
      value.seq <= active.seq
    )
      return;
    active.seq = value.seq;
    if (value.event.kind === "text_delta") {
      const delta = value.event.text;
      setStream((text) => (text + delta).slice(-100000));
    } else if (["completed", "cancelled", "error"].includes(value.event.kind)) {
      runRef.current = null;
      setRun(null);
      pendingStart.current = false;
      earlyEvents.current = [];
      if (value.event.kind === "error") {
        report(value.event.error);
        setNotice("Run failed. No model changes were accepted.");
      } else if (value.event.kind === "cancelled")
        setNotice("Run stopped. No model changes were accepted.");
      else {
        setNotice("Analysis complete. Review proposed changes in Inbox.");
        const selected = rootRef.current;
        if (selected) {
          const generation = epoch.current;
          void refresh(selected).then((result) => {
            if (
              current(generation, selected.root_id) &&
              !runRef.current &&
              result?.proposals.some(
                (p) =>
                  p.status === "accepted" &&
                  p.candidates.length === 0 &&
                  p.origin.kind === "analysis" &&
                  p.origin.run_id === value.run_id,
              )
            )
              setNotice(
                "No new changes. Your accepted understanding is already up to date for this selection.",
              );
          });
        }
      }
    }
    // Proposal events never refresh data until the successful terminal event.
  }
  const receiveRef = useRef(receive);
  receiveRef.current = receive;
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    eventReady.current = listen<WorkspaceRunEvent>(
      "workspace_run_event",
      (event) => receiveRef.current(event.payload),
    ).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    void eventReady.current.catch(() => undefined);
    const generation = epoch.current;
    void ipc<{ root: RootRef | null }>("workspace_root_get")
      .then((result) => {
        if (disposed || !current(generation)) return;
        rootRef.current = result.root;
        setRoot(result.root);
        setPath(result.root?.path || "");
        if (result.root) {
          const selected = result.root;
          void refresh(selected, generation);

        }
      })
      .catch((e) => {
        if (!disposed && current(generation)) {
          setBootError(apiError(e));
          setRoot(null);
        }
      });
    void checkRuntime();
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  useEffect(() => {
    const shortcut = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        if (!rootRef.current) return;
        event.preventDefault();
        setScreen("search");
        setManual(false);
        setSettings(false);
        requestAnimationFrame(() => {
          const input = document.querySelector<HTMLInputElement>("#search");
          input?.focus();
          input?.select();
        });
      }
    };
    window.addEventListener("keydown", shortcut);
    return () => window.removeEventListener("keydown", shortcut);
  }, []);
  async function checkRuntime() {
    try {
      setRuntime(await ipc<RuntimeStatus>("workspace_runtime_status"));
    } catch (e) {
      setRuntime({
        available: false,
        version: null,
        message: apiError(e).message,
      });
    }
  }
  async function stop() {
    const active = runRef.current || run;
    if (!active) return;
    const generation = epoch.current;
    // Close events immediately, but keep a retry action until IPC confirms stop.
    runRef.current = null;
    pendingStart.current = false;
    earlyEvents.current = [];
    setError(null);
    setNotice("Stopping run…");
    try {
      const result = await ipc<{ cancelled: boolean }>("workspace_run_cancel", {
        root_id: active.root,
        run_id: active.id,
      });
      if (!current(generation, active.root)) return;
      setRun(null);
      setNotice(
        result.cancelled
          ? "Run stopped. No model changes were accepted."
          : "Run is no longer active. Review Inbox for any completed proposals.",
      );
      if (!result.cancelled && rootRef.current)
        await refresh(rootRef.current, generation);
    } catch (e) {
      if (current(generation, active.root)) {
        setNotice(
          "Cancellation was not confirmed. Retry Stop run or change notes folder.",
        );
        report(e);
      }
    }
  }
  async function chooseRoot() {
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      const chosen = await ipc<string | null>("workspace_choose_root");
      if (chosen !== null) {
        setPath(chosen);
        setCreate(false);
      }
    } catch (e) {
      report(e);
    } finally {
      setBusy(false);
    }
  }
  async function changeRoot() {
    if (!path.trim() || busy) return;
    setBusy(true);
    setError(null);
    setBootError(null);
    const generation = ++epoch.current;
    const active = runRef.current || run;
    runRef.current = null;
    pendingStart.current = false;
    earlyEvents.current = [];
    setRun(null);
    setStream("");
    setNotice("");
    rootRef.current = null;
    setRoot(null);
    setTopics([]);
    setSpec(emptyDraft);
    setName("");
    setKind("thread");
    setManualSensitivity("unknown");
    setProposals([]);
    setOperations([]);
    clearTopic();
    setPilotCorrection(null);
    setManual(false);
    try {
      if (active)
        await ipc("workspace_run_cancel", {
          root_id: active.root,
          run_id: active.id,
        });
      const result = await ipc<{
        root: RootRef;
        index: { document_count: number; warnings: string[] };
      }>("workspace_root_set", { path: path.trim(), create });
      if (!current(generation)) return;
      rootRef.current = result.root;
      setRoot(result.root);
      setPath(result.root.path);
      setSettings(false);
      setAutoSetup(true);
      setScreen("setup");
      setCreate(false);
      if (result.index.warnings?.length)
        setNotice(result.index.warnings.join("\n"));
      await refresh(result.root, generation);
    } catch (e) {
      if (current(generation)) report(e);
    } finally {
      if (current(generation)) setBusy(false);
    }
  }
  async function getPreview(chat = false) {
    const selected = rootRef.current;
    if (!selected) return;
    setError(null);
    setBusy(true);
    const generation = epoch.current,
      scopeRequest = chat ? chatScopeSeq : scopeSeq,
      sequence = ++scopeRequest.current;
    const updatePreview = chat ? setChatPreview : setPreview;
    (chat ? setChatConfirmed : setConfirmed)(false);
    updatePreview(null);
    const chatFiles = topic
      ? [
          ...new Set(
            [...topic.current_claims, ...topic.history]
              .flatMap((c) =>
                c.sources.flatMap((s) =>
                  s.kind === "passage" ? [s.source.path] : [],
                ),
              )
              .concat(
                topic.connections.flatMap((c) => c.sources.map((s) => s.path)),
              ),
          ),
        ]
      : [];
    try {
      const result = await ipc<ScopePreview>("workspace_scope_preview", {
        root_id: selected.root_id,
        spec: chat ? { ...emptySpec, files: chatFiles } : parseScope(spec),
      });
      if (
        current(generation, selected.root_id) &&
        sequence === scopeRequest.current
      )
        updatePreview(result);
    } catch (e) {
      if (
        current(generation, selected.root_id) &&
        sequence === scopeRequest.current
      )
        report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  const scopeReady = (scope: ScopePreview | null, consent: boolean) =>
    !!scope &&
    scope.root_id === root?.root_id &&
    scope.sources.every((s) => s.sensitivity !== "local_only") &&
    (!scope.sources.some((s) => s.sensitivity === "unknown") || consent);
  const canSend = scopeReady(preview, confirmed),
    canSendChat = scopeReady(chatPreview, chatConfirmed);
  async function start(chat = false) {
    const selected = rootRef.current;
    const selectedPreview = chat ? chatPreview : preview,
      consent = chat ? chatConfirmed : confirmed;
    if (
      !selected ||
      !selectedPreview ||
      !(chat ? canSendChat : canSend) ||
      busy ||
      runRef.current ||
      !runtime?.available
    )
      return;
    const generation = epoch.current;
    setBusy(true);
    setError(null);
    setNotice("");
    setStream("");
    pendingStart.current = true;
    earlyEvents.current = [];
    try {
      await eventReady.current;
      if (!current(generation, selected.root_id)) return;
      const request = {
        root_id: selected.root_id,
        scope_id: selectedPreview.scope_id,
        confirm_unknown: consent,
      };
      const result = await ipc<{ run_id: string }>(
        chat ? "workspace_chat_start" : "workspace_analysis_start",
        chat ? { ...request, topic_id: topic!.topic.id, message } : request,
      );
      if (!current(generation, selected.root_id)) {
        void ipc("workspace_run_cancel", {
          root_id: selected.root_id,
          run_id: result.run_id,
        }).catch(() => undefined);
        return;
      }
      const active = { root: selected.root_id, id: result.run_id, seq: -1 };
      runRef.current = active;
      setRun(active);
      pendingStart.current = false;
      for (const event of earlyEvents.current.splice(0)) receive(event);
      (chat ? invalidateChatScope : invalidateAnalysisScope)();
      if (chat) setMessage("");
    } catch (e) {
      if (current(generation, selected.root_id)) {
        pendingStart.current = false;
        earlyEvents.current = [];
        report(e);
        if (apiError(e).code === "stale_scope")
          (chat ? invalidateChatScope : invalidateAnalysisScope)();
      }
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  async function decide(proposal: Proposal, candidates: Candidate[] | null) {
    const selected = rootRef.current;
    if (!selected || busy) return;
    const generation = epoch.current;
    const detailId = selectedTopicId.current;
    setBusy(true);
    setError(null);
    try {
      await ipc("workspace_proposal_decide", {
        root_id: selected.root_id,
        proposal_id: proposal.id,
        expected_fingerprint: proposal.fingerprint,
        decision:
          candidates === null
            ? { kind: "reject" }
            : { kind: "approve", candidates },
      });
      if (current(generation, selected.root_id)) {
        setNotice(
          candidates === null
            ? "Proposal rejected."
            : "Changes accepted. Your source notes were not rewritten.",
        );
        await refresh(selected, generation);
        if (
          detailId &&
          selectedTopicId.current === detailId &&
          current(generation, selected.root_id)
        )
          await openTopic(detailId);
        // The decided proposal is replaced; keep keyboard focus in the result.
        requestAnimationFrame(() => {
          document.querySelector<HTMLElement>(".inbox h1")?.focus();
        });
      }
    } catch (e) {
      if (current(generation, selected.root_id)) report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  async function manualPropose() {
    const selected = rootRef.current;
    if (!selected || !name.trim() || busy) return;
    const generation = epoch.current;
    setBusy(true);
    setError(null);
    try {
      const candidates: Candidate[] = [
        {
          kind: "topic",
          topic: {
            id: crypto.randomUUID(),
            topic_kind: kind,
            name: name.trim(),
            aliases: [],
            sensitivity: manualSensitivity,
          },
        },
      ];
      await ipc("workspace_manual_propose", {
        root_id: selected.root_id,
        candidates,
      });
      if (current(generation, selected.root_id)) {
        setManual(false);
        setName("");
        setScreen("inbox");
        await refresh(selected, generation);
        requestAnimationFrame(() => {
          document.querySelector<HTMLElement>(".inbox h1")?.focus();
        });
      }
    } catch (e) {
      if (current(generation, selected.root_id)) report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  async function undo(id: string) {
    const selected = rootRef.current;
    if (!selected || busy) return;
    const generation = epoch.current;
    setBusy(true);
    setError(null);
    try {
      await ipc("workspace_undo", {
        root_id: selected.root_id,
        operation_id: id,
      });
      if (current(generation, selected.root_id)) {
        setNotice("Undo recorded in history.");
        clearTopic();
        await refresh(selected, generation);
      }
    } catch (e) {
      if (current(generation, selected.root_id)) report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  async function sensitivity(value: Sensitivity) {
    const selected = rootRef.current;
    if (!selected || !topic || busy) return;
    const generation = epoch.current,
      id = topic.topic.id;
    setBusy(true);
    setError(null);
    invalidateScope();
    try {
      await ipc("workspace_sensitivity_set", {
        root_id: selected.root_id,
        target: { kind: "topic", id },
        sensitivity: value,
      });
      if (current(generation, selected.root_id)) {
        await refresh(selected, generation);
        if (selectedTopicId.current === id) await openTopic(id);
      }
    } catch (e) {
      if (current(generation, selected.root_id)) report(e);
    } finally {
      if (current(generation, selected.root_id)) setBusy(false);
    }
  }
  const appDataBootError = !!bootError?.message.includes("TOPICS_APP_DATA_DIR");
  const pendingCount = proposals.filter((p) => p.status === "pending").length;
  const navigate = (next: Screen) => {
    navigationSeq.current++;
    window.scrollTo(0, 0);
    setScreen(next);
    setManual(false);
    setSettings(false);
    setError(null);
  };
  const openManual = (trigger: HTMLButtonElement) => {
    formTrigger.current = trigger;
    setManual(true);
    setSettings(false);
    setError(null);
  };
  const closeForm = () => {
    setManual(false);
    setSettings(false);
    requestAnimationFrame(() => {
      const trigger = formTrigger.current;
      (trigger?.isConnected
        ? trigger
        : document.getElementById(trigger?.id || "")
      )?.focus();
    });
  };
  const quickSearch = () => {
    navigate("search");
    requestAnimationFrame(() => {
      const input = document.querySelector<HTMLInputElement>("#search");
      input?.focus();
      input?.select();
    });
  };
  return (
    <div className="workspace">
      <a className="skip-link" href="#workspace-content">
        Skip to content
      </a>
      <WindowStatus
        runtime={runtime}
        error={error}
        notice={notice}
        bootError={bootError}
        opening={root === undefined}
        running={!!run}
        checkRuntime={checkRuntime}
        dismissError={() => setError(null)}
        dismissNotice={() => setNotice("")}
      />
      <aside className="sidebar" aria-label="Topics workspace">
        <div className="brand">
          <span className="brand-mark">
            <Icon name="topics" />
          </span>
          Topics
        </div>
        <button
          className="quick-search"
          aria-label="Search"
          aria-current={
            screen === "search" && !manual && !settings && root
              ? "page"
              : undefined
          }
          disabled={!root}
          onClick={quickSearch}
        >
          <Icon name="search" />
          <span>Search notes</span>
          <kbd>⌘K</kbd>
        </button>
        <nav aria-label="Workspace">
          <div className="nav-group">
            <p className="nav-label">Library</p>
            <button
              aria-label="Topics"
              aria-current={
                screen === "topics" && !manual && !settings ? "page" : undefined
              }
              disabled={!root}
              onClick={() => navigate("topics")}
            >
              <Icon name="topics" />
              <span>Topics</span>
              <span className="nav-count">{topics.length || ""}</span>
            </button>
            <button
              aria-current={
                screen === "sources" && !manual && !settings
                  ? "page"
                  : undefined
              }
              disabled={!root}
              onClick={() => navigate("sources")}
            >
              <Icon name="file" />
              <span>Sources</span>
            </button>
          </div>
          <div className="nav-group">
            <p className="nav-label">Review</p>
            <button aria-current={screen === "today" && !manual && !settings ? "page" : undefined} disabled={!root} onClick={() => navigate("today")}><Icon name="inbox" /><span>Today</span></button>
            <button
              aria-current={
                screen === "inbox" && !manual && !settings ? "page" : undefined
              }
              disabled={!root}
              onClick={() => navigate("inbox")}
            >
              <Icon name="inbox" />
              <span>Inbox</span>
              {pendingCount > 0 && (
                <span className="nav-count">{pendingCount}</span>
              )}
            </button>
            <button
              aria-current={
                screen === "history" && !manual && !settings
                  ? "page"
                  : undefined
              }
              disabled={!root}
              onClick={() => navigate("history")}
            >
              <Icon name="history" />
              <span>History</span>
            </button>
          </div>
        </nav>
        <div className="sidebar-actions">
          <p className="nav-label">Workspace</p>
          <Button
            icon="folder"
            disabled={!root}
            aria-current={
              screen === "setup" && !manual && !settings ? "page" : undefined
            }
            onClick={() => {
              setAutoSetup(false);
              navigate("setup");
            }}
          >
            Set up folder
          </Button>
          <Button
            icon="analyse"
            disabled={!root}
            aria-current={
              screen === "analyse" && !manual && !settings ? "page" : undefined
            }
            onClick={() => navigate("analyse")}
          >
            Analyse notes
          </Button>
          <Button
            icon="plus"
            id="sidebar-new-topic"
            disabled={!root}
            aria-current={manual ? "page" : undefined}
            onClick={(e) => openManual(e.currentTarget)}
          >
            New topic
          </Button>
        </div>
        <div className="sidebar-bottom">
          <Appearance />
          <div className="folder-context">
            <Icon name="folder" />
            <div>
              <p className="context-label">Notes folder</p>
              <p className="root-path">
                {root?.path || "No notes folder selected"}
              </p>
            </div>
          </div>
          <Button
            variant="quiet"
            id="change-notes-folder"
            disabled={appDataBootError}
            onClick={(e) => {
              formTrigger.current = e.currentTarget;
              setSettings(true);
              setManual(false);
              setError(null);
            }}
          >
            Change notes folder
          </Button>
        </div>
      </aside>
      <main className="workspace-main" id="workspace-content" tabIndex={-1}>
        {!appDataBootError && (settings || root === null) && (
          <section className="setup focused-form">
            <span className="form-icon">
              <Icon name="folder" />
            </span>
            <PageHeader
              title={root ? "Choose notes folder" : "Start with your notes"}
              description="Connect a folder of Markdown notes. Read and search locally, then choose exactly what to analyse."
            />
            <form
              onSubmit={(e) => {
                e.preventDefault();
                void changeRoot();
              }}
            >
              <div className="folder-picker">
                <Button
                  type="button"
                  icon="folder"
                  disabled={busy}
                  onClick={() => void chooseRoot()}
                >
                  Choose folder…
                </Button>
                <p className="muted">
                  Choose an existing folder or enter its full path below.
                </p>
              </div>
              <Field
                id="root-path"
                label="Notes folder path"
                hint="Your original notes stay yours. Connecting a folder does not send its contents."
              >
                <input
                  placeholder="/absolute/path/to/notes"
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  required
                />
              </Field>
              <label className="check">
                <input
                  type="checkbox"
                  checked={create}
                  onChange={(e) => setCreate(e.target.checked)}
                />
                Create a new notes folder
              </label>
              <div className="review-footer">
                <Button
                  type="submit"
                  variant="primary"
                  disabled={busy || !path.trim()}
                >
                  Use notes folder
                </Button>
                {root && (
                  <Button type="button" onClick={closeForm}>
                    Cancel
                  </Button>
                )}
              </div>
            </form>
          </section>
        )}
        {bootError && (
          <section
            role="alert"
            className="error boot-error"
            aria-label="Workspace unavailable"
          >
            <h2>Workspace unavailable</h2>
            <p>{bootError.message}</p>
            <p>
              {appDataBootError
                ? "Set TOPICS_APP_DATA_DIR to a non-empty absolute path, then restart Topics."
                : "Choose an available notes folder above to recover this workspace."}
            </p>
          </section>
        )}
        {root &&
          !settings &&
          !bootError &&
          (manual ? (
            <section className="manual-form focused-form">
              <span className="form-icon">
                <Icon name="plus" />
              </span>
              <PageHeader
                title="New topic"
                description="Capture a person, project or thread. You’ll review the proposal before it becomes accepted knowledge."
              />
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  void manualPropose();
                }}
              >
                <Field id="topic-name" label="Topic name">
                  <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    autoFocus
                    required
                  />
                </Field>
                <div className="form-grid">
                  <Field id="topic-kind" label="Kind">
                    <select
                      value={kind}
                      onChange={(e) =>
                        setKind(e.target.value as Topic["topic_kind"])
                      }
                    >
                      <option value="thread">Thread</option>
                      <option value="person">Person</option>
                      <option value="project">Project</option>
                    </select>
                  </Field>
                  <Field id="manual-sensitivity" label="Sensitivity">
                    <select
                      value={manualSensitivity}
                      onChange={(e) =>
                        setManualSensitivity(e.target.value as Sensitivity)
                      }
                    >
                      <option value="unknown">Unknown</option>
                      <option value="local_only">Local-only</option>
                      <option value="cloud_allowed">Cloud allowed</option>
                    </select>
                  </Field>
                </div>
                <p className="muted">
                  Local-only topics cannot be sent for analysis. Unknown
                  sensitivity requires an explicit decision before chat.
                </p>
                <div className="review-footer">
                  <Button
                    type="submit"
                    variant="primary"
                    disabled={busy || !name.trim()}
                  >
                    Create proposal
                  </Button>
                  <Button type="button" onClick={closeForm}>
                    Cancel
                  </Button>
                </div>
              </form>
            </section>
          ) : (
            <>
              {screen === "today" && <Pilot key={root.root_id} rootId={root.root_id} openSetup={() => {setAutoSetup(false); navigate("setup");}} openCorrection={(item, feedback) => {if (!topics.some(t => t.id === item.topic_id)) return false; setPilotCorrection({topicId: item.topic_id, revisionId: item.revision_id, statement: feedback.text}); navigate("topics"); void openTopic(item.topic_id); return true;}} />}
              {screen === "search" && <Search key={root.root_id} />}
              {screen === "topics" && (
                <section className="topics-view">
                  <PageHeader
                    title="Topics"
                    description="People, projects and the threads that connect your notes."
                    actions={
                      <Button
                        id="new-topic-action"
                        icon="plus"
                        onClick={(e) => openManual(e.currentTarget)}
                      >
                        New topic
                      </Button>
                    }
                  />
                  <div className="topic-workspace">
                    <section className="topic-list" aria-label="Topic library">
                      <div className="list-heading">
                        <h2>Your topics</h2>
                        <span className="muted">{topics.length}</span>
                      </div>
                      {topics.length === 0 ? (
                        <EmptyState title="No accepted topics yet">
                          <p>
                            Add Markdown notes, then analyse a selection. You
                            can also create a topic proposal by hand.
                          </p>
                        </EmptyState>
                      ) : (
                        <div className="topic-rows">
                          {topics.map((t) => (
                            <button
                              className="library-row"
                              key={t.id}
                              aria-label={t.name}
                              aria-pressed={selectedTopicId.current === t.id}
                              onClick={() => void openTopic(t.id)}
                            >
                              <span className="topic-row-icon">
                                <Icon name={t.topic_kind} />
                              </span>
                              <span className="row-content">
                                <strong>{t.name}</strong>
                                <span className="row-meta">
                                  {t.topic_kind === "person"
                                    ? "Person"
                                    : t.topic_kind === "project"
                                      ? "Project"
                                      : "Thread"}
                                </span>
                                <PrivacyBadge sensitivity={t.sensitivity} />
                              </span>
                              {selectedTopicId.current === t.id && (
                                <Icon name="chevron" />
                              )}
                            </button>
                          ))}
                        </div>
                      )}
                    </section>
                    <section
                      className="topic-detail"
                      aria-label="Topic detail"
                      aria-busy={topicLoading}
                    >
                      {topicLoading ? (
                        <p role="status">Opening topic…</p>
                      ) : topic ? (
                        <>
                          <TopicDetail
                            rootId={root.root_id}
                            view={topic}
                            topics={topics}
                            busy={busy}
                            sensitivity={(s) => void sensitivity(s)}
                          >
                            <ContextReview
                              key={`${root.root_id}-${topic.topic.id}`}
                              rootId={root.root_id}
                              view={topic}
                              topics={topics}
                              onHistory={() => navigate("history")}
                              initialCorrection={pilotCorrection?.topicId === topic.topic.id ? pilotCorrection : null}
                              onSaved={() => {
                                setPilotCorrection(null);
                                void refresh(root);
                                void openTopic(topic.topic.id);
                              }}
                            />
                          </TopicDetail>
                          <Section
                            title="Ask or correct"
                            description="Explain what should change. Generated changes return to Inbox for your review."
                            className="chat"
                          >
                            <Field
                              id="topic-message"
                              label="Message about this topic"
                            >
                              <textarea
                                rows={3}
                                value={message}
                                onChange={(e) => setMessage(e.target.value)}
                                placeholder="What should I know about this topic?"
                              />
                            </Field>
                            <Button
                              disabled={
                                busy ||
                                !!run ||
                                topic.topic.sensitivity !== "cloud_allowed"
                              }
                              onClick={() => void getPreview(true)}
                            >
                              Preview chat sources
                            </Button>
                            {chatPreview && (
                              <ScopeReview
                                preview={chatPreview}
                                confirmed={chatConfirmed}
                                setConfirmed={setChatConfirmed}
                              />
                            )}
                            <div className="actions">
                              <Button
                                variant="primary"
                                disabled={
                                  !message.trim() ||
                                  !canSendChat ||
                                  busy ||
                                  !!run ||
                                  !runtime?.available ||
                                  topic.topic.sensitivity !== "cloud_allowed"
                                }
                                onClick={() => void start(true)}
                              >
                                Send message
                              </Button>
                            </div>
                            {topic.topic.sensitivity !== "cloud_allowed" && (
                              <p className="notice">
                                {topic.topic.sensitivity === "local_only"
                                  ? "This topic is local-only. Its context cannot be sent to Codex."
                                  : "Set this topic to Cloud allowed before chatting. Local-only sources remain excluded."}
                              </p>
                            )}
                          </Section>
                        </>
                      ) : (
                        <EmptyState
                          title={
                            topics.length
                              ? "Choose a topic"
                              : "Your understanding starts here"
                          }
                        >
                          <p>
                            Read accepted statements, explore evidence and
                            follow earlier revisions.
                          </p>
                        </EmptyState>
                      )}
                    </section>
                  </div>
                </section>
              )}
              {screen === "setup" && (
                <FolderSetup
                  key={root.root_id}
                  rootId={root.root_id}
                  runtime={
                    runtime?.available ? `Codex ${runtime.version || ""}` : null
                  }
                  onReturning={() => {
                    if (autoSetup) setScreen("topics");
                  }}
                  onSaved={() => void refresh(root)}
                />
              )}
              {screen === "analyse" && (
                <section className="analysis-view">
                  <PageHeader
                    title="Analyse notes"
                    description="Choose the context first. Every proposed change comes back to you for review."
                  />
                  <ol className="flow-steps" aria-label="Analysis steps">
                    <li aria-current={!preview ? "step" : undefined}>
                      <span>1</span>Select notes
                    </li>
                    <li aria-current={preview ? "step" : undefined}>
                      <span>2</span>Review and authorize
                    </li>
                    <li>
                      <span>3</span>Review proposals in Inbox
                    </li>
                  </ol>
                  <div className="form-section">
                    <ScopeFields
                      spec={spec}
                      disabled={busy || !!run}
                      onChange={(value) => {
                        setSpec(value);
                        invalidateAnalysisScope();
                      }}
                    />
                    <Button
                      disabled={busy || !!run || !hasPaths(spec)}
                      onClick={() => void getPreview()}
                    >
                      Preview selection
                    </Button>
                  </div>
                  {preview && (
                    <ScopeReview
                      preview={preview}
                      confirmed={confirmed}
                      setConfirmed={setConfirmed}
                    />
                  )}
                  <div className="review-footer">
                    <Button
                      variant="primary"
                      disabled={
                        !canSend ||
                        !preview?.sources.length ||
                        busy ||
                        !!run ||
                        !runtime?.available
                      }
                      onClick={() => void start()}
                    >
                      Start analysis
                    </Button>
                    <p className="muted">
                      Only the reviewed selection may be sent.
                    </p>
                  </div>
                  {!runtime?.available && (
                    <p className="notice">
                      Check the runtime status in the title bar before starting.
                      Selection previews remain available.
                    </p>
                  )}
                </section>
              )}
              {screen === "inbox" && (
                <section className="inbox">
                  <PageHeader
                    title="Inbox"
                    description="Review proposed understanding. Nothing becomes accepted knowledge until you approve it."
                    actions={
                      <Badge tone={pendingCount ? "warning" : "neutral"}>
                        {pendingCount} pending
                      </Badge>
                    }
                  />
                  {proposals.length === 0 && (
                    <EmptyState title="Nothing to review" icon="inbox">
                      <p>
                        Analyse a selection of notes or create a topic proposal.
                      </p>
                    </EmptyState>
                  )}
                  {proposals.map((p) => (
                    <ProposalEditor
                      key={`${p.id}-${p.status}-${p.fingerprint}`}
                      proposal={p}
                      topics={topics}
                      busy={busy}
                      decide={(c) => decide(p, c)}
                    />
                  ))}
                </section>
              )}
              {screen === "history" && (
                <section className="history">
                  <PageHeader
                    title="History"
                    description="A record of accepted changes and undo operations. Earlier knowledge stays in the record."
                  />
                  {operations.length === 0 && (
                    <EmptyState title="No recorded changes yet" icon="history">
                      <p>
                        Accepted proposals and privacy decisions will appear
                        here.
                      </p>
                    </EmptyState>
                  )}
                  <div className="operation-list">
                    {operations.map((o) => (
                      <article className="operation" key={o.id}>
                        <span className="operation-icon">
                          <Icon name="history" />
                        </span>
                        <div className="operation-body">
                          <h2>{o.description}</h2>
                          <div className="operation-meta">
                            <time dateTime={o.created_at}>
                              {new Date(o.created_at).toLocaleString()}
                            </time>
                            <Badge
                              tone={
                                o.status === "committed"
                                  ? "success"
                                  : o.status === "conflict"
                                    ? "danger"
                                    : "neutral"
                              }
                            >
                              {o.status}
                            </Badge>
                          </div>
                          <details>
                            <summary>Change identity</summary>
                            <p className="path">{o.id}</p>
                          </details>
                          {o.status !== "committed" && (
                            <p className="muted">
                              Only committed changes can be undone.
                            </p>
                          )}
                        </div>
                        <Button
                          icon="undo"
                          disabled={busy || o.status !== "committed"}
                          onClick={() => void undo(o.id)}
                        >
                          Undo
                        </Button>
                      </article>
                    ))}
                  </div>
                </section>
              )}
              {screen === "sources" && (
                <Sources
                  key={root.root_id}
                  root={root}
                  disabled={busy || !!run}
                  onError={report}
                  onChanged={invalidateScope}
                />
              )}
            </>
          ))}
        {(run || stream) && (
          <section className="run-panel" aria-label="Analysis progress">
            <div className="run-heading">
              <h2>{run ? "Working on your selection" : "Run response"}</h2>
              {run && <Button onClick={() => void stop()}>Stop run</Button>}
            </div>
            <div role="log" aria-live="polite" className="stream">
              {stream || "Waiting for the runtime…"}
            </div>
          </section>
        )}
      </main>
    </div>
  );
}
createRoot(document.getElementById("root")!).render(<App />);
