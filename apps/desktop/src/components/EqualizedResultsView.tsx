import type {
  LevelingSessionResultDto,
  TrapFindingSummaryDto,
  VendorSummaryRowDto,
  WorkbookExportOutcomeDto,
} from "@qlevel/bindings";
import {
  Badge,
  Button,
  QlevelBrandLockup,
  ScrollArea,
  SourceQuoteCard,
  SurfaceCard,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  cn,
} from "@qlevel/design-system";
import {
  createColumnHelper,
  flexRender,
  tableFeatures,
  useTable,
} from "@tanstack/react-table";
import { ArrowUpRight, Check, FolderOpen } from "lucide-react";
import { useMemo, useState } from "react";
import { Group, Panel, Separator } from "react-resizable-panels";
import { A2uiEvidenceHost } from "../a2ui/surfaces";
import type { Dictionary } from "../i18n";

export interface EqualizedResultsViewProps {
  dict: Dictionary;
  result: LevelingSessionResultDto;
  exportOutcome: WorkbookExportOutcomeDto | null;
  busy: boolean;
  onSaveWorkbook: (useFallbackIfLocked: boolean) => void;
  onOpenInExcel: (path: string) => void;
  onRevealFolder: (path: string) => void;
}

const features = tableFeatures({});
const colHelper = createColumnHelper<typeof features, VendorSummaryRowDto>();

function computeSavingsDelta(result: LevelingSessionResultDto): string | null {
  if (result.vendors.length < 2) return null;
  const sorted = [...result.vendors].sort((a, b) => a.rank - b.rank);
  const v1 = parseFloat(sorted[0]?.leveledNetTotal ?? "0");
  const v2 = parseFloat(sorted[1]?.leveledNetTotal ?? "0");
  if (!Number.isFinite(v1) || !Number.isFinite(v2) || v2 <= v1) return null;
  const diff = v2 - v1;
  return diff.toLocaleString("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  });
}

export function EqualizedResultsView({
  dict,
  result,
  exportOutcome,
  busy,
  onSaveWorkbook,
  onOpenInExcel,
  onRevealFolder,
}: EqualizedResultsViewProps) {
  const [activeTab, setActiveTab] = useState<string>("matrix");
  const [selectedTrapIdx, setSelectedTrapIdx] = useState<number>(0);

  const selectedTrap: TrapFindingSummaryDto | undefined =
    result.traps[selectedTrapIdx] ?? result.traps[0];

  const savedPath =
    exportOutcome?.status === "saved" ? exportOutcome.outputPath : null;
  const savings = useMemo(() => computeSavingsDelta(result), [result]);

  const columns = useMemo(
    () =>
      colHelper.columns([
        colHelper.accessor("vendorName", {
          header: dict.colRankVendor,
          cell: (info) => {
            const row = info.row.original;
            return (
              <div className="flex items-center gap-2">
                <span className="font-mono-tabular text-[var(--muted-foreground)]">
                  {row.rank}.
                </span>
                <span
                  dir="auto"
                  className="font-medium text-[var(--foreground)]"
                >
                  {row.vendorName}
                </span>
              </div>
            );
          },
        }),
        colHelper.accessor("submittedTotal", {
          header: dict.colSubmitted,
          cell: (info) => (
            <span
              dir="ltr"
              className="font-mono-tabular text-[var(--muted-foreground)]"
            >
              {info.getValue()}
            </span>
          ),
        }),
        colHelper.accessor("totalPlugs", {
          header: dict.colPlugs,
          cell: (info) => {
            const val = info.getValue();
            const positive = parseFloat(val) > 0;
            return (
              <span
                dir="ltr"
                className={cn(
                  "font-mono-tabular",
                  positive
                    ? "font-medium text-[var(--amber-fg)]"
                    : "text-[var(--muted-foreground)]"
                )}
              >
                {positive ? `+${val}` : "0"}
              </span>
            );
          },
        }),
        colHelper.accessor("leveledNetTotal", {
          header: dict.colLeveledNet,
          cell: (info) => (
            <span
              dir="ltr"
              className="font-mono-tabular font-semibold text-[var(--foreground)]"
            >
              {info.getValue()} {result.currency}
            </span>
          ),
        }),
        colHelper.accessor("leveledGrossTotal", {
          header: dict.colLeveledGross,
          cell: (info) => (
            <span
              dir="ltr"
              className="font-mono-tabular text-[var(--muted-foreground)]"
            >
              {info.getValue()} {result.currency}
            </span>
          ),
        }),
        colHelper.accessor("trapsCount", {
          header: dict.colTraps,
          cell: (info) => {
            const count = info.getValue();
            const vendorName = info.row.original.vendorName;
            const vendorCategories = Array.from(
              new Set(
                result.traps
                  .filter((t) => t.vendorName === vendorName)
                  .map((t) => t.category.replace(/_/g, " "))
              )
            ).slice(0, 2);

            if (count === 0) {
              return (
                <span className="inline-flex items-center gap-1 text-[var(--emerald)]">
                  <span className="font-mono-tabular text-[var(--muted-foreground)]">
                    0
                  </span>
                  <Check className="h-3.5 w-3.5" />
                </span>
              );
            }

            return (
              <div className="flex flex-wrap items-center gap-1.5">
                <span className="font-mono-tabular font-semibold text-[var(--foreground)]">
                  {count}
                </span>
                {vendorCategories.map((cat) => (
                  <Badge
                    key={cat}
                    tone="neutral"
                    className="px-2 py-0 text-[10px]"
                  >
                    {cat}
                  </Badge>
                ))}
              </div>
            );
          },
        }),
      ]),
    [dict, result]
  );

  const table = useTable({
    features,
    data: result.vendors,
    columns,
  });

  return (
    <Tabs
      value={activeTab}
      onValueChange={setActiveTab}
      className="flex flex-1 flex-col overflow-hidden bg-[var(--background)]"
    >
      {/* Minimal Header */}
      <header className="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border)] bg-[var(--card)] px-5 py-3">
        <QlevelBrandLockup subtitle={dict.resultsSubtitle} />

        <div className="flex flex-wrap items-center gap-2.5">
          <TabsList>
            <TabsTrigger value="matrix">{dict.tabMatrix}</TabsTrigger>
            <TabsTrigger value="traps">
              {dict.tabTraps} ({result.traps.length})
            </TabsTrigger>
            <TabsTrigger value="excel">{dict.tabExcelPreview}</TabsTrigger>
          </TabsList>

          {savedPath && (
            <>
              <Button
                variant="secondary"
                size="md"
                onClick={() => onOpenInExcel(savedPath)}
              >
                <span>{dict.openExcelBtn}</span>
              </Button>
              <Button
                variant="ghost"
                size="icon"
                aria-label={dict.revealFolderBtn}
                onClick={() => onRevealFolder(savedPath)}
              >
                <FolderOpen className="h-4 w-4" />
              </Button>
            </>
          )}

          <Button
            variant="primary"
            size="md"
            disabled={busy}
            onClick={() => onSaveWorkbook(false)}
          >
            <span>{dict.exportXlsxBtn}</span>
            <ArrowUpRight className="h-3.5 w-3.5" />
          </Button>
        </div>
      </header>

      {/* Split Results + Right A2UI Evidence Inspector */}
      <Group orientation="horizontal" className="flex-1 overflow-hidden">
        <Panel defaultSize="72%" minSize="52%">
          <ScrollArea className="h-full p-6">
            <div className="space-y-4">
              {/* Graceful Locked-Excel Fallback Strip */}
              {exportOutcome?.status === "file_locked" && (
                <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-amber-500/30 bg-[var(--amber-subtle)] px-4 py-2.5 text-xs">
                  <span className="font-medium text-[var(--amber-fg)]">
                    {dict.fileLockedNotice} ·{" "}
                    <span dir="ltr" className="font-mono-tabular">
                      {exportOutcome.lockedPath}
                    </span>
                  </span>
                  <Button
                    variant="primary"
                    size="sm"
                    disabled={busy}
                    onClick={() => onSaveWorkbook(true)}
                  >
                    {dict.saveUnlockedCopyBtn}
                  </Button>
                </div>
              )}

              {/* Minimal Emerald True Lowest Bidder Strip */}
              <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-emerald-500/25 bg-[var(--emerald-subtle)]/75 px-4 py-3 text-xs">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="h-2 w-2 rounded-full bg-[var(--emerald)]" />
                  <span className="font-medium text-[var(--emerald-fg)]">
                    {dict.trueLowestBidder} ·
                  </span>
                  <span
                    dir="auto"
                    className="font-semibold text-[var(--foreground)]"
                  >
                    {result.trueLowVendorName}
                  </span>
                  <span>·</span>
                  <span
                    dir="ltr"
                    className="font-mono-tabular font-semibold text-[var(--foreground)]"
                  >
                    {result.trueLowLeveledNet} {result.currency}
                  </span>
                </div>

                {savings && (
                  <Badge tone="neutral">
                    Saves {savings} {result.currency} {dict.savesVsRank2}
                  </Badge>
                )}
              </div>

              <TabsContent value="matrix" className="mt-0">
                <SurfaceCard className="overflow-hidden">
                  <div className="overflow-x-auto">
                    <table className="w-full border-collapse text-start text-xs">
                      <thead>
                        {table.getHeaderGroups().map((hg) => (
                          <tr
                            key={hg.id}
                            className="border-b border-[var(--border)] text-[var(--muted-foreground)]"
                          >
                            {hg.headers.map((header) => (
                              <th
                                key={header.id}
                                className="px-4 py-3 text-start font-medium"
                              >
                                {header.isPlaceholder
                                  ? null
                                  : flexRender(
                                      header.column.columnDef.header,
                                      header.getContext()
                                    )}
                              </th>
                            ))}
                          </tr>
                        ))}
                      </thead>
                      <tbody className="divide-y divide-[var(--border)]">
                        {table.getRowModel().rows.map((row) => {
                          const vName = row.original.vendorName;
                          const firstVendorTrap = result.traps.findIndex(
                            (t) => t.vendorName === vName
                          );
                          return (
                            <tr
                              key={row.id}
                              onClick={() => {
                                if (firstVendorTrap >= 0) {
                                  setSelectedTrapIdx(firstVendorTrap);
                                }
                              }}
                              className={cn(
                                "transition-colors",
                                firstVendorTrap >= 0 &&
                                  "cursor-pointer hover:bg-[var(--secondary)]/55",
                                row.original.isTrueLowBidder &&
                                  "bg-[var(--emerald-subtle)]/25"
                              )}
                            >
                              {row.getAllCells().map((cell) => (
                                <td key={cell.id} className="px-4 py-3.5">
                                  {flexRender(
                                    cell.column.columnDef.cell,
                                    cell.getContext()
                                  )}
                                </td>
                              ))}
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                </SurfaceCard>
              </TabsContent>

              <TabsContent value="traps" className="mt-0">
                {result.traps.length === 0 ? (
                  <SurfaceCard className="p-5 text-xs text-[var(--muted-foreground)]">
                    {dict.noTrapsClean}
                  </SurfaceCard>
                ) : (
                  <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
                    {result.traps.map((t, idx) => (
                      <div
                        key={idx}
                        onClick={() => setSelectedTrapIdx(idx)}
                        className={cn(
                          "cursor-pointer rounded-xl transition-all",
                          selectedTrapIdx === idx &&
                            "ring-1 ring-[var(--foreground)]"
                        )}
                      >
                        <SourceQuoteCard
                          vendorName={t.vendorName}
                          title={t.title}
                          category={t.category.replace(/_/g, " ")}
                          quoteSnippet={t.sourceSnippet}
                          detail={t.detail}
                          impactLabel={
                            t.financialImpact
                              ? `+${t.financialImpact} ${result.currency}`
                              : null
                          }
                        />
                      </div>
                    ))}
                  </div>
                )}
              </TabsContent>

              <TabsContent value="excel" className="mt-0">
                <SurfaceCard dir="ltr" className="overflow-hidden">
                  <div className="flex items-center gap-2 border-b border-[var(--border)] bg-[var(--secondary)]/65 px-3.5 py-2 text-[11px]">
                    <span className="rounded border border-[var(--border)] bg-[var(--card)] px-2 py-0.5 font-mono-tabular font-semibold">
                      D4
                    </span>
                    <span className="font-mono-tabular text-[var(--muted-foreground)]">
                      fx
                    </span>
                    <code className="font-mono-tabular text-[var(--foreground)]">
                      =INDEX(VendorHeaders, 1, MATCH(MIN(LeveledNetRow), LeveledNetRow, 0))
                    </code>
                  </div>
                  <div className="p-4 text-xs space-y-2 text-[var(--muted-foreground)]">
                    <p className="font-medium text-[var(--foreground)]">
                      Live Formula Workbook ({result.totalBoqItems} BOQ items · VAT{" "}
                      {result.vatPct}% ·{" "}
                      {result.layoutMode === "single_master_sheet"
                        ? "01_CBE_TBE_Master"
                        : "01_Commercial_CBE + 02_Technical_TBE"}
                      )
                    </p>
                    <p>
                      Target path:{" "}
                      <span className="font-mono-tabular text-[var(--foreground)]">
                        {savedPath ?? result.suggestedOutputPath}
                      </span>
                    </p>
                  </div>
                </SurfaceCard>
              </TabsContent>
            </div>
          </ScrollArea>
        </Panel>

        <Separator className="w-px bg-[var(--border)] hover:bg-[var(--emerald)] transition-colors" />

        {/* Right Source Evidence Inspector Powered by Google A2UI */}
        <Panel defaultSize="28%" minSize="22%" maxSize="40%">
          <ScrollArea className="h-full border-s border-[var(--border)] bg-[var(--card)] p-5">
            <div className="space-y-3.5">
              <h3 className="text-xs font-semibold text-[var(--foreground)]">
                {dict.inspectorTitle}
              </h3>

              {selectedTrap ? (
                <>
                  <A2uiEvidenceHost
                    trap={selectedTrap}
                    currency={result.currency}
                  />
                  {result.traps.length > 1 && (
                    <div className="flex flex-wrap gap-1.5 pt-1">
                      {result.traps.map((t, i) => (
                        <button
                          key={i}
                          type="button"
                          onClick={() => setSelectedTrapIdx(i)}
                          className={cn(
                            "rounded-md border px-2 py-1 text-[10px] font-medium transition-colors cursor-pointer",
                            selectedTrapIdx === i
                              ? "border-[var(--foreground)] bg-[var(--foreground)] text-[var(--background)]"
                              : "border-[var(--border)] bg-[var(--secondary)] text-[var(--muted-foreground)]"
                          )}
                        >
                          #{i + 1} {t.category.split("_")[0]}
                        </button>
                      ))}
                    </div>
                  )}
                </>
              ) : (
                <p className="text-xs text-[var(--muted-foreground)]">
                  {dict.inspectorEmpty}
                </p>
              )}
            </div>
          </ScrollArea>
        </Panel>
      </Group>
    </Tabs>
  );
}
