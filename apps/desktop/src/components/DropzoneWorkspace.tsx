import type { DocumentRoleDto, StagedInputDocumentDto } from "@qlevel/bindings";
import { Badge, Button, SurfaceCard } from "@qlevel/design-system";
import type { Dictionary } from "../i18n";

export interface DropzoneWorkspaceProps {
  dict: Dictionary;
  documents: readonly StagedInputDocumentDto[];
  isDragging: boolean;
  busy: boolean;
  onBrowseFiles: () => void;
  onChangeRole: (id: string, role: DocumentRoleDto) => void;
  onChangeVendorName: (id: string, vendorName: string) => void;
  onRemoveDocument: (id: string) => void;
}

const ROLE_ORDER: readonly DocumentRoleDto[] = [
  "master_boq",
  "vendor_quote",
  "spec_or_avl",
];

export function DropzoneWorkspace({
  dict,
  documents,
  isDragging,
  busy,
  onBrowseFiles,
  onChangeRole,
  onChangeVendorName,
  onRemoveDocument,
}: DropzoneWorkspaceProps) {
  const hasMasterBoq = documents.some((d) => d.role === "master_boq");
  const vendorCount = documents.filter((d) => d.role === "vendor_quote").length;

  const roleLabel = (role: DocumentRoleDto): string => {
    switch (role) {
      case "master_boq":
        return dict.roleMasterBoq;
      case "vendor_quote":
        return dict.roleVendorQuote;
      case "spec_or_avl":
        return dict.roleSpecAvl;
    }
  };

  return (
    <SurfaceCard className="space-y-4">
      <div
        className={`flex flex-col items-center justify-center rounded-xl border-2 border-dashed px-6 py-8 text-center transition-colors ${
          isDragging
            ? "border-teal-600 bg-teal-50/60"
            : "border-slate-300 bg-slate-50/50 hover:border-slate-400"
        }`}
      >
        <p className="text-base font-bold text-slate-900">{dict.dropzoneTitle}</p>
        <p className="mt-1 max-w-xl text-xs text-slate-600">{dict.dropzoneSub}</p>
        <div className="mt-4 flex flex-wrap items-center justify-center gap-3">
          <Button variant="primary" disabled={busy} onClick={onBrowseFiles}>
            {dict.browseFilesBtn}
          </Button>
        </div>
        <p className="mt-3 text-xs text-slate-500">{dict.pasteHint}</p>
      </div>

      {!hasMasterBoq && vendorCount >= 1 && (
        <div className="rounded-lg border border-teal-200 bg-teal-50/70 px-4 py-2.5 text-xs font-medium text-teal-900">
          {dict.syntheticBoqNotice}
        </div>
      )}

      {documents.length > 0 && (
        <ul className="divide-y divide-slate-200 rounded-xl border border-slate-200 bg-white">
          {documents.map((doc) => {
            const cycleNextRole = () => {
              const idx = ROLE_ORDER.indexOf(doc.role);
              const next = ROLE_ORDER[(idx + 1) % ROLE_ORDER.length];
              if (next) {
                onChangeRole(doc.id, next);
              }
            };

            return (
              <li
                key={doc.id}
                className="flex flex-wrap items-center justify-between gap-3 px-4 py-3"
              >
                <div className="min-w-[200px] flex-1 space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-sm font-semibold text-slate-900">
                      {doc.fileName}
                    </span>
                    {doc.detectedRowCount !== null && (
                      <Badge tone="neutral">{doc.detectedRowCount} rows</Badge>
                    )}
                    {doc.pageCount !== null && (
                      <Badge tone="neutral">{doc.pageCount} pages</Badge>
                    )}
                    {doc.hiddenRowsCount > 0 && (
                      <Badge tone="warning">
                        {dict.hiddenRowsWarning} ({doc.hiddenRowsCount})
                      </Badge>
                    )}
                  </div>

                  {doc.role === "vendor_quote" && (
                    <input
                      type="text"
                      value={doc.vendorName ?? ""}
                      disabled={busy}
                      onChange={(e) => onChangeVendorName(doc.id, e.target.value)}
                      placeholder={dict.vendorNamePlaceholder}
                      className="w-full max-w-xs rounded-md border border-slate-300 px-2.5 py-1 text-xs text-slate-800 focus:border-teal-700 focus:outline-none"
                    />
                  )}
                </div>

                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    disabled={busy}
                    onClick={cycleNextRole}
                    className={`rounded-lg border px-3 py-1.5 text-xs font-semibold transition-colors ${
                      doc.role === "master_boq"
                        ? "border-teal-300 bg-teal-50 text-teal-900"
                        : doc.role === "vendor_quote"
                          ? "border-sky-300 bg-sky-50 text-sky-900"
                          : "border-amber-300 bg-amber-50 text-amber-900"
                    }`}
                  >
                    {roleLabel(doc.role)}
                  </button>
                  <Button
                    variant="ghost"
                    disabled={busy}
                    onClick={() => onRemoveDocument(doc.id)}
                  >
                    {dict.removeDocBtn}
                  </Button>
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </SurfaceCard>
  );
}
