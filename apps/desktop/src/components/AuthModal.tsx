import { useState } from "react";
import type { ProviderAuthStatusDto, ProviderKind } from "@qlevel/bindings";
import { Badge, Button } from "@qlevel/design-system";
import type { Dictionary } from "../i18n";

export interface AuthModalProps {
  open: boolean;
  dict: Dictionary;
  statuses: readonly ProviderAuthStatusDto[];
  busy: boolean;
  errorMessage: string | null;
  onClose: () => void;
  onSignInChatGpt: () => void;
  onSaveApiKey: (
    provider: ProviderKind,
    apiKey: string,
    ackGeminiPrivacy: boolean
  ) => void;
  onDisconnect: (provider: ProviderKind) => void;
}

const PROVIDER_CARDS: readonly {
  kind: ProviderKind;
  title: string;
  subtitle: string;
}[] = [
  {
    kind: "open_ai",
    title: "OpenAI",
    subtitle: "Sign in with ChatGPT (OAuth PKCE) or direct OPENAI_API_KEY",
  },
  {
    kind: "gemini",
    title: "Google Gemini",
    subtitle: "Direct GEMINI_API_KEY (v1beta Multimodal PDF & Vision)",
  },
  {
    kind: "claude",
    title: "Anthropic Claude",
    subtitle: "Direct ANTHROPIC_API_KEY (2026-10-21 Adaptive Reasoning)",
  },
];

export function AuthModal({
  open,
  dict,
  statuses,
  busy,
  errorMessage,
  onClose,
  onSignInChatGpt,
  onSaveApiKey,
  onDisconnect,
}: AuthModalProps) {
  const [draftKeys, setDraftKeys] = useState<Record<ProviderKind, string>>({
    open_ai: "",
    gemini: "",
    claude: "",
  });
  const [geminiAck, setGeminiAck] = useState<boolean>(false);

  if (!open) {
    return null;
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4"
    >
      <div className="max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-2xl border border-slate-200 bg-white p-6 shadow-xl">
        <div className="flex items-start justify-between gap-4 border-b border-slate-200 pb-4">
          <div>
            <h2 className="text-lg font-bold text-slate-900">
              {dict.authModalTitle}
            </h2>
            <p className="mt-1 text-xs text-slate-600">{dict.authModalSub}</p>
          </div>
          <Button variant="ghost" onClick={onClose}>
            {dict.closeBtn}
          </Button>
        </div>

        {errorMessage && (
          <div className="mt-4 rounded-lg border border-red-200 bg-red-50 px-3.5 py-2.5 text-xs font-medium text-red-800">
            {errorMessage}
          </div>
        )}

        <div className="mt-5 space-y-4">
          {PROVIDER_CARDS.map((card) => {
            const st = statuses.find((s) => s.provider === card.kind);
            const isConnected = st?.connected ?? false;
            const keyVal = draftKeys[card.kind];

            return (
              <div
                key={card.kind}
                className="rounded-xl border border-slate-200 bg-slate-50/70 p-4"
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <h3 className="text-sm font-bold text-slate-900">
                      {card.title}
                    </h3>
                    <p className="text-xs text-slate-500">{card.subtitle}</p>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge tone={isConnected ? "success" : "neutral"}>
                      {isConnected
                        ? `${dict.connectedBadge}${st?.accountLabel ? ` (${st.accountLabel})` : ""}`
                        : dict.notConnectedBadge}
                    </Badge>
                    {isConnected && (
                      <Button
                        variant="ghost"
                        disabled={busy}
                        onClick={() => onDisconnect(card.kind)}
                      >
                        {dict.disconnectBtn}
                      </Button>
                    )}
                  </div>
                </div>

                {card.kind === "open_ai" && (
                  <div className="mt-3">
                    <Button
                      variant="primary"
                      disabled={busy}
                      onClick={onSignInChatGpt}
                    >
                      {dict.signInChatGptBtn}
                    </Button>
                  </div>
                )}

                {card.kind === "gemini" && (
                  <label className="mt-3 flex items-start gap-2 text-xs text-amber-900">
                    <input
                      type="checkbox"
                      checked={geminiAck || (st?.geminiPrivacyAcknowledged ?? false)}
                      onChange={(e) => setGeminiAck(e.target.checked)}
                      className="mt-0.5 rounded border-slate-300"
                    />
                    <span>{dict.geminiPrivacyLabel}</span>
                  </label>
                )}

                <div className="mt-3 flex flex-wrap items-center gap-2">
                  <input
                    type="password"
                    value={keyVal}
                    onChange={(e) =>
                      setDraftKeys((prev) => ({
                        ...prev,
                        [card.kind]: e.target.value,
                      }))
                    }
                    placeholder={dict.apiKeyPlaceholder}
                    className="min-w-[220px] flex-1 rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-xs text-slate-900 focus:border-teal-700 focus:outline-none"
                  />
                  <Button
                    variant="secondary"
                    disabled={busy || keyVal.trim().length === 0}
                    onClick={() => {
                      onSaveApiKey(
                        card.kind,
                        keyVal,
                        card.kind === "gemini"
                          ? geminiAck || (st?.geminiPrivacyAcknowledged ?? false)
                          : true
                      );
                      setDraftKeys((prev) => ({ ...prev, [card.kind]: "" }));
                    }}
                  >
                    {dict.saveApiKeyBtn}
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
