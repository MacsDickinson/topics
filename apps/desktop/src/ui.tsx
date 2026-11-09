import {
  cloneElement,
  useId,
  type ComponentProps,
  type ReactElement,
  type ReactNode,
} from "react";
import type { Sensitivity } from "./workspace-types";
import "./ui.css";

const paths = {
  topics: (
    <>
      <rect x="3" y="3" width="7" height="7" rx="2" />
      <rect x="14" y="3" width="7" height="7" rx="2" />
      <rect x="3" y="14" width="7" height="7" rx="2" />
      <rect x="14" y="14" width="7" height="7" rx="2" />
    </>
  ),
  search: (
    <>
      <circle cx="10.5" cy="10.5" r="6.5" />
      <path d="m16 16 4.5 4.5" />
    </>
  ),
  inbox: (
    <>
      <path d="m3 13 3-9h12l3 9v7H3Z" />
      <path d="M3 13h5l2 3h4l2-3h5" />
    </>
  ),
  history: (
    <>
      <path d="M3 11a9 9 0 1 1 2.7 7.3M3 4v7h7" />
      <path d="M12 7v5l3 2" />
    </>
  ),
  file: (
    <>
      <path d="M14 3H5v18h14V8Z" />
      <path d="M14 3v5h5M8 12h8M8 16h6" />
    </>
  ),
  folder: <path d="M3 7V4h6l2 3h10v13H3Z" />,
  plus: <path d="M12 5v14M5 12h14" />,
  analyse: (
    <>
      <path d="M12 3v4M12 17v4M3 12h4M17 12h4M5.6 5.6l2.8 2.8m7.2 7.2 2.8 2.8m0-12.8-2.8 2.8m-7.2 7.2-2.8 2.8" />
      <circle cx="12" cy="12" r="3" />
    </>
  ),
  person: (
    <>
      <circle cx="12" cy="8" r="4" />
      <path d="M4 21v-2a8 8 0 0 1 16 0v2" />
    </>
  ),
  project: (
    <>
      <rect x="3" y="6" width="18" height="15" rx="2" />
      <path d="M8 6V3h8v3M3 12h18M10 12v3h4v-3" />
    </>
  ),
  thread: (
    <>
      <path d="M9 3 7 21M17 3l-2 18M3 9h18M2 15h18" />
    </>
  ),
  lock: (
    <>
      <rect x="5" y="10" width="14" height="11" rx="2" />
      <path d="M8 10V7a4 4 0 0 1 8 0v3M12 14v3" />
    </>
  ),
  check: <path d="m5 12 4 4L19 6" />,
  chevron: <path d="m9 5 7 7-7 7" />,
  quote: (
    <>
      <path d="M10 5H4v7h6v7H4M21 5h-6v7h6v7h-6" />
    </>
  ),
  info: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 11v6M12 7h.01" />
    </>
  ),
  undo: (
    <>
      <path d="M8 4 3 9l5 5M3 9h10a7 7 0 0 1 0 14" />
    </>
  ),
} satisfies Record<string, ReactNode>;

export type IconName = keyof typeof paths;
/** Decorative only: give the containing control a visible label or aria-label. */
export function Icon({
  name,
  className = "",
}: {
  name: IconName;
  className?: string;
}) {
  return (
    <svg
      className={`icon ${className}`}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {paths[name]}
    </svg>
  );
}

export function Button({
  variant = "secondary",
  icon,
  busy = false,
  children,
  className = "",
  disabled,
  ...props
}: ComponentProps<"button"> & {
  variant?: "primary" | "secondary" | "quiet" | "danger";
  icon?: IconName;
  busy?: boolean;
}) {
  return (
    <button
      {...props}
      className={`ui-button ${variant} ${className}`}
      disabled={disabled || busy}
      aria-busy={busy || undefined}
    >
      {icon && <Icon name={icon} />}
      <span>{children}</span>
    </button>
  );
}

export function PageHeader({
  title,
  description,
  actions,
  level = 1,
  className = "",
}: {
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  level?: 1 | 2;
  className?: string;
}) {
  const Heading = level === 1 ? "h1" : "h2";
  return (
    <header className={`page-header ${className}`}>
      <div>
        <Heading tabIndex={-1}>{title}</Heading>
        {description && <p>{description}</p>}
      </div>
      {actions && <div className="header-actions">{actions}</div>}
    </header>
  );
}

export function Badge({
  children,
  tone = "neutral",
}: {
  children: ReactNode;
  tone?: "neutral" | "info" | "warning" | "success" | "danger";
}) {
  return <span className={`badge tone-${tone}`}>{children}</span>;
}

export const privacyLabel = (value: Sensitivity) =>
  value === "local_only"
    ? "Local-only"
    : value === "cloud_allowed"
      ? "Cloud allowed"
      : "Sensitivity unknown";
export function PrivacyBadge({ sensitivity }: { sensitivity: Sensitivity }) {
  return (
    <Badge tone={sensitivity === "local_only" ? "warning" : "neutral"}>
      {sensitivity === "local_only" && <Icon name="lock" />}
      {privacyLabel(sensitivity)}
    </Badge>
  );
}

export function EmptyState({
  title,
  children,
  icon = "topics",
}: {
  title: string;
  children?: ReactNode;
  icon?: IconName;
}) {
  return (
    <div className="empty-state">
      <span className="empty-icon">
        <Icon name={icon} />
      </span>
      <h2>{title}</h2>
      {children && <div>{children}</div>}
    </div>
  );
}

export function Section({
  title,
  description,
  actions,
  children,
  className = "",
}: {
  title: string;
  description?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  const id = useId();
  return (
    <section className={`content-section ${className}`} aria-labelledby={id}>
      <header className="section-heading">
        <div>
          <h2 id={id}>{title}</h2>
          {description && <p className="muted">{description}</p>}
        </div>
        {actions}
      </header>
      {children}
    </section>
  );
}

type FieldControl = ReactElement<{
  id?: string;
  "aria-describedby"?: string;
  "aria-invalid"?: boolean | "true" | "false";
}>;
export function Field({
  id,
  label,
  hint,
  error,
  children,
  className = "",
}: {
  id: string;
  label: string;
  hint?: ReactNode;
  error?: string;
  children: FieldControl;
  className?: string;
}) {
  const description =
    [
      children.props["aria-describedby"],
      hint && `${id}-hint`,
      error && `${id}-error`,
    ]
      .filter(Boolean)
      .join(" ") || undefined;
  return (
    <div className={`field ${className}`}>
      <label htmlFor={id}>{label}</label>
      {cloneElement(children, {
        id,
        "aria-describedby": description,
        "aria-invalid": error ? true : children.props["aria-invalid"],
      })}
      {hint && (
        <p className="field-hint" id={`${id}-hint`}>
          {hint}
        </p>
      )}
      {error && (
        <p className="field-error" id={`${id}-error`}>
          {error}
        </p>
      )}
    </div>
  );
}
