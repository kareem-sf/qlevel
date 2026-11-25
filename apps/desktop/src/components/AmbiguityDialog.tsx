import type {
  AmbiguityPromptPayload,
  AmbiguityResolutionChoice,
} from "@qlevel/bindings";
import { Badge, Button } from "@qlevel/design-system";
import type { Dictionary } from "../i18n";

export interface AmbiguityDialogProps {
  prompt: AmbiguityPromptPayload | null;
  dict: Dictionary;
  onResolve: (choice: AmbiguityResolutionChoice) => void;
}

export function AmbiguityDialog({
  prompt,
  dict,
  onResolve,
}: AmbiguityDialogProps) {
  if (!prompt) {
    return null;
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4"
    >
      <div className="w-full max-w-xl space-y-4 rounded-2xl border border-amber-200 bg-white p-6 shadow-xl">
        <div className="flex items-center justify-between gap-2">
          <h2 className="text-base font-bold text-slate-900">
            {dict.ambiguityTitle}
          </h2>
          <Badge tone="warning">
            {prompt.vendorName} ({prompt.pendingCount})
          </Badge>
        </div>

        <p className="text-xs text-slate-600">{dict.ambiguitySub}</p>

        <div className="rounded-xl border border-slate-200 bg-slate-50 p-3.5 text-xs space-y-2">
          <p className="font-semibold text-slate-800">
            {prompt.group.vendorDescriptions.join(" / ")} -{" "}
            <span className="tabular-nums font-bold text-teal-800">
              {prompt.group.quotedLumpSum}
            </span>
          </p>
          <ul className="list-disc ps-5 space-y-1 text-slate-600">
            {prompt.masterDescriptions.map((desc, idx) => (
              <li key={idx}>{desc}</li>
            ))}
          </ul>
        </div>

        <div className="grid grid-cols-1 gap-2">
          <Button
            variant="primary"
            onClick={() => onResolve("anchor_and_zero_followers")}
          >
            {dict.choiceAnchorZero}
          </Button>
          <Button
            variant="secondary"
            onClick={() => onResolve("proportional_split")}
          >
            {dict.choiceProportional}
          </Button>
          <Button
            variant="secondary"
            onClick={() => onResolve("accepted_alternate")}
          >
            {dict.choiceAlternate}
          </Button>
          <Button
            variant="ghost"
            onClick={() => onResolve("excluded_with_plug")}
          >
            {dict.choiceExcludePlug}
          </Button>
        </div>
      </div>
    </div>
  );
}
