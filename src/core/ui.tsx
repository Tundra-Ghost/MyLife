// Shared small UI pieces.
import { useEffect, useState, type ReactNode } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { Energy } from "./api";

/** Spec: no more than 7 items visible in any list by default. */
export const LIST_LIMIT = 7;

/** Refetch everything after a change. Data is local, so this is cheap. */
export function useRefresh() {
  const qc = useQueryClient();
  return () => qc.invalidateQueries();
}

export function Button({
  children,
  onClick,
  kind = "plain",
  type = "button",
  disabled,
  title,
}: {
  children: ReactNode;
  onClick?: () => void;
  kind?: "plain" | "primary" | "action";
  type?: "button" | "submit";
  disabled?: boolean;
  title?: string;
}) {
  const styles = {
    plain: "bg-slate-800 hover:bg-slate-700 text-slate-100",
    primary: "bg-brand hover:brightness-110 text-slate-950 font-semibold",
    // The single "needs action" accent.
    action: "bg-accent hover:brightness-110 text-slate-950 font-semibold",
  }[kind];
  return (
    <button
      type={type}
      title={title}
      disabled={disabled}
      onClick={onClick}
      className={`min-h-10 rounded-lg px-4 py-2 text-sm disabled:opacity-50 ${styles}`}
    >
      {children}
    </button>
  );
}

/** Renders up to 7 items, with a "Show more" link for the rest. */
export function ShortList<T>({
  items,
  render,
  empty,
  extra = 0,
}: {
  items: T[];
  render: (item: T) => ReactNode;
  empty: ReactNode;
  /** Items the backend already left out (it also caps at 7). */
  extra?: number;
}) {
  const [all, setAll] = useState(false);
  if (items.length === 0) return <p className="text-sm text-slate-500">{empty}</p>;
  const shown = all ? items : items.slice(0, LIST_LIMIT);
  const hidden = items.length - shown.length;
  return (
    <div className="space-y-2">
      {shown.map(render)}
      {hidden > 0 && (
        <button className="text-sm text-slate-400 hover:text-slate-200" onClick={() => setAll(true)}>
          Show {hidden} more
        </button>
      )}
      {extra > 0 && <p className="text-sm text-slate-500">And {extra} more.</p>}
    </div>
  );
}

export function EnergyTag({ energy }: { energy: Energy | null }) {
  if (!energy) return null;
  const label = { low: "Low energy", medium: "Medium energy", high: "High energy" }[energy];
  return <span className="rounded bg-slate-800 px-2 py-0.5 text-xs text-slate-300">{label}</span>;
}

export function Section({ title, children, right }: { title: string; children: ReactNode; right?: ReactNode }) {
  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-400">{title}</h2>
        {right}
      </div>
      {children}
    </section>
  );
}

/** Calls `onClose` when Esc is pressed. For modals and side panels. */
export function useEscape(onClose: () => void) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);
}
