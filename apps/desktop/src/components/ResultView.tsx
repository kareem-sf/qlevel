import type {
  LevelingSessionResultDto,
  WorkbookExportOutcomeDto,
} from "@qlevel/bindings";
import { Badge, Button, SurfaceCard } from "@qlevel/design-system";
import type { Dictionary } from "../i18n";

export interface ResultViewProps {
  dict: Dictionary;
  result: LevelingSessionResultDto;
  exportOutcome: WorkbookExportOutcomeDto | null;
  busy: boolean;
  onSaveWorkbook: (useFallbackIfLocked: boolean) => void;
  onOpenInExcel: (path: string) => void;
  onRevealFolder: (path: string) => void;
  onResetSession: () => void;
}

export function ResultView({
  dict,
  result,
  exportOutcome,
  busy,
  onSaveWorkbook,
  onOpenInExcel,
  onRevealFolder,
  onResetSession,
}: ResultViewProps) {
  const savedPath =
    exportOutcome?.status === "saved" ? exportOutcome.outputPath : null;

  return (
    <div className="space-y-5">
      <SurfaceCard className="border-emerald-200 bg-emerald-50/40">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <Badge tone="success">{dict.resultWinnerLabel}</Badge>
            <h2 className="mt-2 text-xl font-bold text-slate-900">
              {result.trueLowVendorName} -{" "}
              <span className="tabular-nums text-emerald-800">
                {result.trueLowLeveledNet} {result.currency}
              </span>
            </h2>
            <p className="mt-1 text-xs text-slate-600">
              {result.totalBoqItems} BOQ items | VAT {result.vatPct}%
              {result.isSyntheticBoq ? " | Synthetic Unified BOQ" : ""}
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2.5">
            <Button
              variant="primary"
              disabled={busy}
              onClick={() => onSaveWorkbook(false)}
            >
              {dict.saveExcelBtn}
            </Button>
            {savedPath && (
              <>
                <Button
                  variant="secondary"
                  onClick={() => onOpenInExcel(savedPath)}
                >
                  {dict.openExcelBtn}
                </Button>
                <Button
                  variant="ghost"
                  onClick={() => onRevealFolder(savedPath)}
                >
                  {dict.revealFolderBtn}
                </Button>
              </>
            )}
            <Button variant="ghost" onClick={onResetSession}>
              {dict.newComparisonBtn}
            </Button>
          </div>
        </div>

        {exportOutcome?.status === "file_locked" && (
          <div className="mt-4 flex flex-wrap items-center justify-between gap-3 rounded-xl border border-amber-300 bg-amber-50 px-4 py-3">
            <p className="text-xs font-medium text-amber-900">
              {exportOutcome.lockedPath} ({exportOutcome.hint})
            </p>
            <Button
              variant="primary"
              disabled={busy}
              onClick={() => onSaveWorkbook(true)}
            >
              {dict.saveUnlockedFallbackBtn}
            </Button>
          </div>
        )}
      </SurfaceCard>

      <SurfaceCard className="overflow-x-auto">
        <table className="w-full border-collapse text-start text-xs">
          <thead>
            <tr className="border-b border-slate-200 text-slate-600">
              <th className="py-2.5 pe-3 text-start font-semibold">#</th>
              <th className="py-2.5 pe-3 text-start font-semibold">
                {dict.vendorCol}
              </th>
              <th className="py-2.5 pe-3 text-start font-semibold">
                {dict.submittedCol}
              </th>
              <th className="py-2.5 pe-3 text-start font-semibold">
                {dict.plugsCol}
              </th>
              <th className="py-2.5 pe-3 text-start font-semibold">
                {dict.leveledNetCol}
              </th>
              <th className="py-2.5 pe-3 text-start font-semibold">
                {dict.leveledGrossCol}
              </th>
              <th className="py-2.5 text-start font-semibold">
                {dict.trapsCol}
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100">
            {result.vendors.map((v) => (
              <tr
                key={v.vendorId}
                className={v.isTrueLowBidder ? "bg-emerald-50/60 font-semibold" : ""}
              >
                <td className="py-3 pe-3 tabular-nums">{v.rank}</td>
                <td className="py-3 pe-3 text-slate-900">{v.vendorName}</td>
                <td className="py-3 pe-3 tabular-nums">{v.submittedTotal}</td>
                <td className="py-3 pe-3 tabular-nums text-amber-800">
                  {v.totalPlugs}
                </td>
                <td className="py-3 pe-3 tabular-nums font-bold text-slate-900">
                  {v.leveledNetTotal} {result.currency}
                </td>
                <td className="py-3 pe-3 tabular-nums">
                  {v.leveledGrossTotal} {result.currency}
                </td>
                <td className="py-3">
                  <Badge tone={v.trapsCount > 0 ? "danger" : "success"}>
                    {v.trapsCount}
                  </Badge>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </SurfaceCard>

      <SurfaceCard className="space-y-3">
        <h3 className="text-sm font-bold text-slate-900">
          {dict.trapsSectionTitle}
        </h3>
        {result.traps.length === 0 ? (
          <p className="text-xs text-emerald-800">{dict.noTrapsFound}</p>
        ) : (
          <ul className="space-y-2.5">
            {result.traps.map((t, idx) => (
              <li
                key={idx}
                className="rounded-lg border border-red-200 bg-red-50/50 px-3.5 py-2.5 text-xs"
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <span className="font-bold text-red-900">
                    {t.vendorName} - {t.title}
                  </span>
                  <div className="flex items-center gap-2">
                    <Badge tone="danger">{t.category}</Badge>
                    {t.financialImpact && (
                      <span className="tabular-nums font-bold text-red-900">
                        +{t.financialImpact} {result.currency}
                      </span>
                    )}
                  </div>
                </div>
                <p className="mt-1 text-slate-700">{t.detail}</p>
                {t.sourceSnippet && (
                  <blockquote className="mt-1.5 border-s-2 border-red-300 ps-2.5 italic text-slate-600">
                    {t.sourceSnippet}
                  </blockquote>
                )}
              </li>
            ))}
          </ul>
        )}
      </SurfaceCard>
    </div>
  );
}
