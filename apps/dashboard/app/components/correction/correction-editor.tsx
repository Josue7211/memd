import { useState } from "react";
import { useCorrect } from "../../lib/queries";

export function CorrectionEditor({
  itemId,
  currentContent,
  onClose,
}: {
  itemId: string;
  currentContent: string;
  onClose: () => void;
}) {
  const [content, setContent] = useState(currentContent);
  const [reason, setReason] = useState("");
  const correct = useCorrect();

  const canSubmit = content.trim().length > 0 && content !== currentContent;

  return (
    <div className="mt-3 p-4 rounded-lg border border-border-active bg-bg-primary/80 space-y-3">
      <p className="text-xs font-medium text-accent-bright uppercase tracking-wide">
        Correct this memory
      </p>

      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        rows={4}
        className="w-full px-3 py-2 rounded-lg bg-bg-surface border border-border-subtle text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary resize-y"
      />

      <input
        type="text"
        value={reason}
        onChange={(e) => setReason(e.target.value)}
        placeholder="Reason for correction (optional)"
        className="w-full px-3 py-2 rounded-lg bg-bg-surface border border-border-subtle text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary"
      />

      <div className="flex gap-2">
        <button
          disabled={!canSubmit || correct.isPending}
          onClick={() => {
            correct.mutate(
              {
                id: itemId,
                content: content.trim(),
                reason: reason || undefined,
              },
              {
                onSuccess: () => {
                  setContent(currentContent);
                  setReason("");
                  onClose();
                },
              },
            );
          }}
          className="px-4 py-1.5 rounded-lg text-xs font-medium bg-accent-primary/20 text-accent-bright border border-accent-primary/40 hover:bg-accent-primary/30 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {correct.isPending ? "Saving..." : "Save Correction"}
        </button>
        <button
          onClick={onClose}
          className="px-4 py-1.5 rounded-lg text-xs font-medium text-text-tertiary border border-border-subtle hover:border-border-active transition-colors"
        >
          Cancel
        </button>
      </div>

      {correct.isError && (
        <p className="text-xs text-status-expired">
          {(correct.error as Error).message}
        </p>
      )}
    </div>
  );
}
