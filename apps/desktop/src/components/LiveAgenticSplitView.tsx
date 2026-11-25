import type {
  AmbiguityPromptPayload,
  AmbiguityResolutionChoice,
  LevelingProgressEvent,
  ProgressStageDto,
  StagedInputDocumentDto,
} from "@qlevel/bindings";
import {
  Artifact,
  ArtifactHeader,
  Badge,
  Button,
  ChainOfThought,
  ChainOfThoughtStep,
  Checkpoint,
  ProgressPill,
  QlevelBrandLockup,
  ScrollArea,
} from "@qlevel/design-system";
import { Square } from "lucide-react";
import { Group, Panel, Separator } from "react-resizable-panels";
import { A2uiDecisionHost } from "../a2ui/surfaces";
import type { Dictionary } from "../i18n";

export interface LiveAgenticSplitViewProps {
  dict: Dictionary;
  currency: string;
  documents: readonly StagedInputDocumentDto[];
  progress: LevelingProgressEvent | null;
  ambiguityPrompt: AmbiguityPromptPayload | null;
  onResolveAmbiguity: (choice: AmbiguityResolutionChoice) => void;
  onCancel: () => void;
}

const STAGE_INDEX: Record<ProgressStageDto, number> = {
  reading_documents: 0,
  rasterizing_scanned_pages: 1,
  extracting_vendor_quotes: 2,
  awaiting_ambiguity_decision: 2,
  auditing_math_and_traps: 3,
  building_workbook: 3,
  completed: 4,
};

export function LiveAgenticSplitView({
  dict,
  currency,
  documents,
  progress,
  ambiguityPrompt,
  onResolveAmbiguity,
  onCancel,
}: LiveAgenticSplitViewProps) {
  const activeIdx = progress ? STAGE_INDEX[progress.stage] : 0;
  const percent = progress?.percent ?? 18;
  const vendors = documents.filter((d) => d.role === "vendor_quote");

  const stepState = (idx: number): "done" | "active" | "pending" => {
    if (activeIdx > idx) return "done";
    if (activeIdx === idx) return "active";
    return "pending";
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden bg-[var(--background)]">
      {/* Top Minimal Header */}
      <header className="flex items-center justify-between border-b border-[var(--border)] bg-[var(--card)] px-5 py-3">
        <QlevelBrandLockup subtitle={dict.liveTitle} />
        <Button variant="secondary" size="sm" onClick={onCancel}>
          <Square className="h-3 w-3 fill-current" />
          <span>{dict.stopRunBtn}</span>
        </Button>
      </header>

      {/* Split-Pane Agentic Workspace */}
      <Group orientation="horizontal" className="flex-1 overflow-hidden">
        {/* Left Agent Stream Pane */}
        <Panel defaultSize="32%" minSize="24%" maxSize="46%">
          <ScrollArea className="h-full border-e border-[var(--border)] bg-[var(--secondary)]/30 p-5">
            <div className="space-y-5">
              <ChainOfThought title={dict.agentStreamTitle} live>
                <ChainOfThoughtStep
                  label={dict.stepParseBoq}
                  status={stepState(0)}
                />
                <ChainOfThoughtStep
                  label={dict.stepRasterPdf}
                  status={stepState(1)}
                />
                <ChainOfThoughtStep
                  label={dict.stepExtractQuotes}
                  detail={progress?.activeVendorName ?? undefined}
                  status={stepState(2)}
                />
                <ChainOfThoughtStep
                  label={dict.stepAuditMath}
                  status={stepState(3)}
                />
              </ChainOfThought>

              {progress?.checkpointHit && (
                <Checkpoint label={dict.checkpointSaved} />
              )}

              {/* Inline Google A2UI Decision Surface (Zero Blocking Popup) */}
              {ambiguityPrompt && (
                <A2uiDecisionHost
                  prompt={ambiguityPrompt}
                  currency={currency}
                  dict={dict}
                  onResolve={onResolveAmbiguity}
                />
              )}
            </div>
          </ScrollArea>
        </Panel>

        <Separator className="w-px bg-[var(--border)] hover:bg-[var(--emerald)] transition-colors" />

        {/* Right Live Commercial Equalization Canvas */}
        <Panel defaultSize="68%" minSize="45%">
          <div className="flex h-full flex-col gap-4 overflow-y-auto p-6">
            <ProgressPill
              percent={percent}
              label={
                progress?.activeVendorName
                  ? `${dict.commercialEqualizationTitle} · ${progress.activeVendorName}`
                  : dict.commercialEqualizationTitle
              }
            />

            <Artifact>
              <ArtifactHeader
                title={dict.commercialEqualizationTitle}
                subtitle={`${vendors.length} vendor packages staged · ${currency}`}
              />
              <div className="overflow-x-auto">
                <table className="w-full border-collapse text-start text-xs">
                  <thead>
                    <tr className="border-b border-[var(--border)] bg-[var(--secondary)]/50 text-[var(--muted-foreground)]">
                      <th className="px-4 py-2.5 text-start font-medium">
                        {dict.roleVendor}
                      </th>
                      <th className="px-4 py-2.5 text-start font-medium">
                        Source
                      </th>
                      <th className="px-4 py-2.5 text-end font-medium">
                        Status
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[var(--border)]">
                    {vendors.map((v) => {
                      const isCurrent =
                        progress?.activeVendorName !== null &&
                        progress?.activeVendorName === v.vendorName;
                      return (
                        <tr key={v.id}>
                          <td
                            dir="auto"
                            className="px-4 py-3 font-medium text-[var(--foreground)]"
                          >
                            {v.vendorName || v.fileName}
                          </td>
                          <td
                            dir="auto"
                            className="px-4 py-3 text-[var(--muted-foreground)]"
                          >
                            {v.fileName}
                          </td>
                          <td className="px-4 py-3 text-end">
                            <Badge tone={isCurrent ? "success" : "outline"}>
                              {isCurrent ? "Streaming" : "Queued"}
                            </Badge>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </Artifact>
          </div>
        </Panel>
      </Group>
    </div>
  );
}
