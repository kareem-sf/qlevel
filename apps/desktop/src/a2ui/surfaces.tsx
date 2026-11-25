import {
  A2uiSurface,
  createComponentImplementation,
} from "@a2ui/react";
import {
  Catalog,
  ComponentCommonSchema,
  MessageProcessor,
  RELAXED_VALIDATION,
  type ActionPayload,
  type ComponentApi,
  type SurfaceModel,
} from "@a2ui/web_core/v0_9";
import type {
  AmbiguityPromptPayload,
  AmbiguityResolutionChoice,
  TrapFindingSummaryDto,
} from "@qlevel/bindings";
import { SourceQuoteCard } from "@qlevel/design-system";
import { useEffect, useMemo, useState } from "react";
import { z } from "zod";
import type { Dictionary } from "../i18n";

export const QLEVEL_A2UI_CATALOG_ID = "https://qlevel.app/a2ui/catalog/v1";

const DecisionSurfaceSchema = ComponentCommonSchema.extend({
  headline: z.string(),
  vendorName: z.string(),
  lumpSumLabel: z.string(),
  itemsSummary: z.string(),
  labelAnchorZero: z.string(),
  labelProportional: z.string(),
  labelAlternate: z.string(),
  labelPeerPlug: z.string(),
});

const DecisionSurfaceApi: ComponentApi<typeof DecisionSurfaceSchema> = {
  name: "QlevelMergedDecisionCard",
  schema: DecisionSurfaceSchema,
};

const EvidenceSurfaceSchema = ComponentCommonSchema.extend({
  vendorName: z.string(),
  title: z.string(),
  category: z.string(),
  detail: z.string(),
  sourceSnippet: z.string(),
  impactLabel: z.string(),
});

const EvidenceSurfaceApi: ComponentApi<typeof EvidenceSurfaceSchema> = {
  name: "QlevelEvidenceInspectorCard",
  schema: EvidenceSurfaceSchema,
};

function createQlevelCatalog(
  onChooseResolution?: ((choice: AmbiguityResolutionChoice) => void) | undefined
) {
  const decisionImpl = createComponentImplementation(
    DecisionSurfaceApi,
    ({ props }) => {
      const choices: readonly {
        id: AmbiguityResolutionChoice;
        label: string;
        accent?: boolean;
      }[] = [
        {
          id: "anchor_and_zero_followers",
          label: String(props.labelAnchorZero ?? ""),
          accent: true,
        },
        {
          id: "proportional_split",
          label: String(props.labelProportional ?? ""),
        },
        {
          id: "accepted_alternate",
          label: String(props.labelAlternate ?? ""),
        },
        {
          id: "excluded_with_plug",
          label: String(props.labelPeerPlug ?? ""),
        },
      ];

      const items = String(props.itemsSummary ?? "")
        .split("||")
        .filter((s) => s.length > 0);

      return (
        <div className="space-y-3 rounded-xl border border-amber-500/30 bg-[var(--card)] p-3.5 shadow-2xs">
          <div className="flex items-center justify-between gap-2">
            <span className="inline-flex items-center gap-1.5 text-xs font-semibold text-[var(--foreground)]">
              <span className="h-2 w-2 rounded-full bg-[var(--amber)]" />
              {String(props.headline ?? "")}
            </span>
            <span
              dir="ltr"
              className="font-mono-tabular text-[11px] font-semibold text-[var(--amber-fg)]"
            >
              {String(props.lumpSumLabel ?? "")}
            </span>
          </div>

          <p dir="auto" className="text-[11px] font-medium text-[var(--muted-foreground)]">
            {String(props.vendorName ?? "")}
          </p>

          {items.length > 0 && (
            <ul className="space-y-1 border-s border-[var(--border)] ps-2.5 text-[11px] text-[var(--muted-foreground)]">
              {items.map((line, idx) => (
                <li key={idx} dir="auto">
                  {line}
                </li>
              ))}
            </ul>
          )}

          <div className="grid grid-cols-1 gap-1.5 pt-0.5">
            {choices.map((c) => (
              <button
                key={c.id}
                type="button"
                onClick={() => onChooseResolution?.(c.id)}
                className={
                  c.accent
                    ? "w-full rounded-lg border border-amber-500/40 bg-[var(--amber-subtle)] px-3 py-1.5 text-start text-[11px] font-semibold text-[var(--amber-fg)] transition-colors hover:opacity-90 cursor-pointer"
                    : "w-full rounded-lg border border-[var(--border)] bg-[var(--card)] px-3 py-1.5 text-start text-[11px] font-medium text-[var(--foreground)] transition-colors hover:bg-[var(--secondary)] cursor-pointer"
                }
              >
                {c.label}
              </button>
            ))}
          </div>
        </div>
      );
    }
  );

  const evidenceImpl = createComponentImplementation(
    EvidenceSurfaceApi,
    ({ props }) => (
      <SourceQuoteCard
        vendorName={String(props.vendorName ?? "")}
        title={String(props.title ?? "")}
        category={String(props.category ?? "")}
        detail={String(props.detail ?? "")}
        quoteSnippet={String(props.sourceSnippet ?? "") || null}
        impactLabel={String(props.impactLabel ?? "") || null}
      />
    )
  );

  return new Catalog(QLEVEL_A2UI_CATALOG_ID, "v0.9", [
    decisionImpl,
    evidenceImpl,
  ]);
}

export interface A2uiDecisionHostProps {
  prompt: AmbiguityPromptPayload;
  currency: string;
  dict: Dictionary;
  onResolve: (choice: AmbiguityResolutionChoice) => void;
}

export function A2uiDecisionHost({
  prompt,
  currency,
  dict,
  onResolve,
}: A2uiDecisionHostProps) {
  const [surface, setSurface] = useState<SurfaceModel<any> | undefined>(
    undefined
  );

  const catalog = useMemo(
    () => createQlevelCatalog(onResolve),
    [onResolve]
  );

  useEffect(() => {
    const handleAction = (action: ActionPayload) => {
      const choice = action.context["choice"];
      if (
        choice === "anchor_and_zero_followers" ||
        choice === "proportional_split" ||
        choice === "accepted_alternate" ||
        choice === "excluded_with_plug"
      ) {
        onResolve(choice);
      }
    };

    const processor = new MessageProcessor([catalog], handleAction, {
      version: "v0.9",
      validationConfig: RELAXED_VALIDATION,
      reportErrorsOnly: true,
    });

    const surfaceId = `decision-${prompt.group.groupId}`;
    processor.processMessages([
      {
        version: "v0.9",
        createSurface: {
          surfaceId,
          catalogId: QLEVEL_A2UI_CATALOG_ID,
        },
      },
      {
        version: "v0.9",
        updateComponents: {
          surfaceId,
          components: [
            {
              id: "root",
              component: "QlevelMergedDecisionCard",
              headline: `${dict.mergedCardHeadline}: ${prompt.group.vendorDescriptions.join(" + ")}`,
              vendorName: prompt.vendorName,
              lumpSumLabel: `${prompt.group.quotedLumpSum} ${currency}`,
              itemsSummary: prompt.masterDescriptions.join("||"),
              labelAnchorZero: dict.choiceAnchorZero,
              labelProportional: dict.choiceProportional,
              labelAlternate: dict.choiceAlternate,
              labelPeerPlug: dict.choicePeerPlug,
            },
          ],
        },
      },
    ]);

    setSurface(processor.getSurface(surfaceId));
    return () => {
      processor.dispose();
    };
  }, [catalog, currency, dict, onResolve, prompt]);

  if (!surface) {
    return null;
  }

  return <A2uiSurface surface={surface} />;
}

export interface A2uiEvidenceHostProps {
  trap: TrapFindingSummaryDto;
  currency: string;
}

export function A2uiEvidenceHost({ trap, currency }: A2uiEvidenceHostProps) {
  const [surface, setSurface] = useState<SurfaceModel<any> | undefined>(
    undefined
  );

  const catalog = useMemo(() => createQlevelCatalog(), []);

  useEffect(() => {
    const processor = new MessageProcessor([catalog], undefined, {
      version: "v0.9",
      validationConfig: RELAXED_VALIDATION,
      reportErrorsOnly: true,
    });

    const surfaceId = "evidence-inspector";
    processor.processMessages([
      {
        version: "v0.9",
        createSurface: {
          surfaceId,
          catalogId: QLEVEL_A2UI_CATALOG_ID,
        },
      },
      {
        version: "v0.9",
        updateComponents: {
          surfaceId,
          components: [
            {
              id: "root",
              component: "QlevelEvidenceInspectorCard",
              vendorName: trap.vendorName,
              title: trap.title,
              category: trap.category.replace(/_/g, " "),
              detail: trap.detail,
              sourceSnippet: trap.sourceSnippet,
              impactLabel: trap.financialImpact
                ? `+${trap.financialImpact} ${currency} plug applied`
                : "",
            },
          ],
        },
      },
    ]);

    setSurface(processor.getSurface(surfaceId));
    return () => {
      processor.dispose();
    };
  }, [catalog, currency, trap]);

  if (!surface) {
    return null;
  }

  return <A2uiSurface surface={surface} />;
}
