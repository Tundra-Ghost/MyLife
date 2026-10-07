// Quick capture: type it, press Enter, it's saved to the Inbox.
// Opens from Ctrl+Shift+Space anywhere, or Ctrl+N inside the app.
import { useEffect, useRef, useState } from "react";
import { api, errorText } from "./api";
import { parseCapture } from "./capture/parse";
import { useUi } from "./store";
import { formatDay } from "./time";
import { useRefresh } from "./ui";

export function QuickCapture() {
  const { captureOpen, setCaptureOpen } = useUi();
  const [text, setText] = useState("");
  const [error, setError] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const refresh = useRefresh();

  useEffect(() => {
    if (captureOpen) {
      setText("");
      setError("");
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [captureOpen]);

  if (!captureOpen) return null;
  const preview = text.trim() ? parseCapture(text) : null;

  async function save() {
    if (!preview) return;
    try {
      await api.createTask({
        title: preview.title,
        data: { inbox: true, due_on: preview.dueOn, due_at: preview.dueAt, suggested_module: preview.suggestedModule },
      });
      refresh();
      setCaptureOpen(false);
    } catch (e) {
      setError(errorText(e));
    }
  }

  return (
    <div className="fixed inset-0 z-20 flex items-start justify-center bg-black/60 pt-[20vh]" onClick={() => setCaptureOpen(false)}>
      <div className="w-full max-w-lg space-y-2 rounded-xl bg-slate-900 p-4 shadow-2xl" onClick={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          className="w-full bg-transparent text-lg outline-none"
          placeholder='Try "oil change next Tuesday"'
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") save();
            if (e.key === "Escape") setCaptureOpen(false);
          }}
        />
        <div className="flex gap-3 text-xs text-slate-500">
          {preview?.dueOn && <span>Due {formatDay(preview.dueOn)}</span>}
          {preview?.suggestedModule && <span>Looks like {preview.suggestedModule}</span>}
          <span className="ml-auto">Enter to save. Esc to close.</span>
        </div>
        {error && <p className="text-sm text-accent">{error}</p>}
      </div>
    </div>
  );
}
