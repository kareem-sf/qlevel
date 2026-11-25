export type UiLanguage = "en" | "ar";

export interface Dictionary {
  appName: string;
  intakeHeadline: string;
  intakePlaceholder: string;
  browseBtn: string;
  pasteShortcutHint: string;
  roleMaster: string;
  roleVendor: string;
  roleAvl: string;
  hiddenRowsLabel: string;
  syntheticBaselineNotice: string;
  vendorPlaceholder: string;
  autoMultimodalPill: string;
  singleSheetPill: string;
  dualTabPill: string;
  startLevelingBtn: string;
  navIntake: string;
  navAgentStream: string;
  navResults: string;
  navCommand: string;
  navTheme: string;
  navSettings: string;
  liveTitle: string;
  agentStreamTitle: string;
  stepParseBoq: string;
  stepRasterPdf: string;
  stepExtractQuotes: string;
  stepAuditMath: string;
  checkpointSaved: string;
  stopRunBtn: string;
  commercialEqualizationTitle: string;
  mergedCardHeadline: string;
  choiceAnchorZero: string;
  choiceProportional: string;
  choiceAlternate: string;
  choicePeerPlug: string;
  resultsSubtitle: string;
  tabMatrix: string;
  tabTraps: string;
  tabExcelPreview: string;
  exportXlsxBtn: string;
  openExcelBtn: string;
  revealFolderBtn: string;
  fileLockedNotice: string;
  saveUnlockedCopyBtn: string;
  trueLowestBidder: string;
  savesVsRank2: string;
  colRankVendor: string;
  colSubmitted: string;
  colPlugs: string;
  colLeveledNet: string;
  colLeveledGross: string;
  colTraps: string;
  inspectorTitle: string;
  inspectorEmpty: string;
  noTrapsClean: string;
  settingsTitle: string;
  settingsTabProviders: string;
  settingsTabRules: string;
  settingsTabExcel: string;
  connectedLabel: string;
  disconnectedLabel: string;
  signInChatGpt: string;
  apiKeyPlaceholder: string;
  saveKeyBtn: string;
  disconnectBtn: string;
  geminiPrivacyShort: string;
  geminiPrivacyHint: string;
  effortLabel: string;
  effortAuto: string;
  effortLow: string;
  effortMed: string;
  effortHigh: string;
  plugBasisLabel: string;
  plugLowest: string;
  plugAvg: string;
  plugHigh: string;
  plugPte: string;
  currencyLabel: string;
  vatPctLabel: string;
  avlLabel: string;
  avlPlaceholder: string;
  customRulesLabel: string;
  customRulesPlaceholder: string;
  loadRulesBtn: string;
  mergedBehaviorLabel: string;
  mergedAskInline: string;
  mergedAutoHighlight: string;
  sheetDirectionLabel: string;
  commandPlaceholder: string;
  commandEmpty: string;
}

export const DICTIONARIES: Record<UiLanguage, Dictionary> = {
  en: {
    appName: "qlevel",
    intakeHeadline: "Level a Tender Package",
    intakePlaceholder:
      "Drop BOQ and vendor quotes, or paste tables and screenshots with Ctrl+V...",
    browseBtn: "Attach files",
    pasteShortcutHint: "Excel, PDF, scanned offers, or Ctrl+V clipboard tables",
    roleMaster: "Master",
    roleVendor: "Vendor",
    roleAvl: "AVL / Spec",
    hiddenRowsLabel: "hidden rows",
    syntheticBaselineNotice:
      "No Master BOQ attached - unified baseline will be synthesized across vendor quotes",
    vendorPlaceholder: "Vendor name",
    autoMultimodalPill: "Auto Multimodal",
    singleSheetPill: "Single Sheet",
    dualTabPill: "Dual Tab (CBE + TBE)",
    startLevelingBtn: "Start Leveling",
    navIntake: "Tender Intake",
    navAgentStream: "Live Agentic Leveling",
    navResults: "Equalized Results",
    navCommand: "Command Menu (Ctrl+K)",
    navTheme: "Toggle Appearance",
    navSettings: "Settings & AI Providers",
    liveTitle: "Live Agentic Leveling",
    agentStreamTitle: "Agent Stream",
    stepParseBoq: "Parsed Master BOQ & worksheet structure",
    stepRasterPdf: "Prepared high-clarity scanned quote pages",
    stepExtractQuotes: "Aligned vendor line items & stripped inclusive VAT",
    stepAuditMath: "Verified extensions, scope omissions & AVL compliance",
    checkpointSaved: "Progress auto-saved locally",
    stopRunBtn: "Stop",
    commercialEqualizationTitle: "Commercial Equalization",
    mergedCardHeadline: "Merged Lump Sum",
    choiceAnchorZero: "Anchor on Lead & Zero Followers",
    choiceProportional: "Split by BOQ Weight",
    choiceAlternate: "Accept as Alternate",
    choicePeerPlug: "Apply Peer Plug",
    resultsSubtitle: "Equalized Results",
    tabMatrix: "Matrix",
    tabTraps: "Traps",
    tabExcelPreview: "Excel Preview",
    exportXlsxBtn: "Export .xlsx",
    openExcelBtn: "Open in Excel",
    revealFolderBtn: "Show in Folder",
    fileLockedNotice: "Workbook is currently open in Excel",
    saveUnlockedCopyBtn: "Save as _v2.xlsx",
    trueLowestBidder: "True Lowest Bidder",
    savesVsRank2: "vs Rank 2 after plugs",
    colRankVendor: "Rank & Vendor",
    colSubmitted: "Submitted",
    colPlugs: "Plugs",
    colLeveledNet: "Leveled Net",
    colLeveledGross: "Gross (Incl. VAT)",
    colTraps: "Traps",
    inspectorTitle: "Source Evidence Inspector",
    inspectorEmpty: "Select any flagged bidder or trap to inspect its verbatim quote clause.",
    noTrapsClean: "Clean compliant offer - zero exclusions or arithmetic discrepancies.",
    settingsTitle: "Settings & AI Providers",
    settingsTabProviders: "AI Providers",
    settingsTabRules: "Tender Rules",
    settingsTabExcel: "Excel Output",
    connectedLabel: "Connected",
    disconnectedLabel: "Not connected",
    signInChatGpt: "Sign in with ChatGPT",
    apiKeyPlaceholder: "Enter direct API key...",
    saveKeyBtn: "Save",
    disconnectBtn: "Disconnect",
    geminiPrivacyShort: "Privacy",
    geminiPrivacyHint:
      "Confirm paid Gemini billing key or acknowledge unpaid tier data policy",
    effortLabel: "Effort",
    effortAuto: "Auto",
    effortLow: "Low",
    effortMed: "Med",
    effortHigh: "High",
    plugBasisLabel: "Plug Basis",
    plugLowest: "Lowest",
    plugAvg: "Avg",
    plugHigh: "High",
    plugPte: "PTE",
    currencyLabel: "Currency",
    vatPctLabel: "VAT %",
    avlLabel: "Approved Manufacturers (AVL)",
    avlPlaceholder: "Trane, Carrier, Daikin, Schneider",
    customRulesLabel: "Project Audit Rules",
    customRulesPlaceholder: "One rule per line, e.g. Scaffolding above 6m included",
    loadRulesBtn: "Import .txt",
    mergedBehaviorLabel: "Merged Lump-Sum Handling",
    mergedAskInline: "Ask Inline in Stream",
    mergedAutoHighlight: "Auto Highlight in Excel",
    sheetDirectionLabel: "Workbook Direction",
    commandPlaceholder: "Jump to screen, switch language, or run action...",
    commandEmpty: "No matching command.",
  },
  ar: {
    appName: "qlevel",
    intakeHeadline: "توحيد ومقارنة عروض المناقصة",
    intakePlaceholder:
      "اسحب المقايسة وعروض الموردين هنا، أو الصق الجداول مباشرة عبر Ctrl+V...",
    browseBtn: "إرفاق ملفات",
    pasteShortcutHint: "يدعم Excel و PDF والعروض الممسوحة ضوئيا ولصق الجداول",
    roleMaster: "مقايسة",
    roleVendor: "مورد",
    roleAvl: "مواصفات / AVL",
    hiddenRowsLabel: "صفوف مخفية",
    syntheticBaselineNotice:
      "لم يتم إرفاق مقايسة أساسية - سيتم بناء مقايسة موحدة تلقائيا من عروض الموردين",
    vendorPlaceholder: "اسم المورد",
    autoMultimodalPill: "توجيه ذكي تلقائي",
    singleSheetPill: "شيت موحد",
    dualTabPill: "شيتان (مالي + فني)",
    startLevelingBtn: "ابدأ المقارنة",
    navIntake: "إدخال المناقصة",
    navAgentStream: "التحليل الحي",
    navResults: "النتائج الموحدة",
    navCommand: "لوحة الأوامر (Ctrl+K)",
    navTheme: "تبديل المظهر",
    navSettings: "الإعدادات ومزودو الذكاء",
    liveTitle: "التحليل والتوحيد الحي",
    agentStreamTitle: "مسار الوكيل",
    stepParseBoq: "قراءة بنود المقايسة وفحص هيكل الجداول",
    stepRasterPdf: "تجهيز الصفحات الممسوحة ضوئيا بدقة عالية",
    stepExtractQuotes: "مطابقة بنود الموردين وتجريد الضريبة الشاملة",
    stepAuditMath: "مراجعة العمليات الحسابية والنواقص وقائمة AVL",
    checkpointSaved: "تم حفظ التقدم تلقائيا",
    stopRunBtn: "إيقاف",
    commercialEqualizationTitle: "التوحيد المالي للعروض",
    mergedCardHeadline: "مبلغ مقطوع مدمج",
    choiceAnchorZero: "تثبيت على البند الأول وتصفير التوابع",
    choiceProportional: "توزيع نسبي بوزن المقايسة",
    choiceAlternate: "اعتماد كبديل مطابق",
    choicePeerPlug: "تعويض من أسعار المنافسين",
    resultsSubtitle: "النتائج الموحدة",
    tabMatrix: "المصفوفة",
    tabTraps: "الملاحظات",
    tabExcelPreview: "معاينة Excel",
    exportXlsxBtn: "تصدير .xlsx",
    openExcelBtn: "فتح في Excel",
    revealFolderBtn: "عرض بالمجلد",
    fileLockedNotice: "الملف مفتوح حاليا في برنامج Excel",
    saveUnlockedCopyBtn: "حفظ كنسخة _v2.xlsx",
    trueLowestBidder: "العرض الأقل الحقيقي",
    savesVsRank2: "وفر مقارنة بالمركز الثاني",
    colRankVendor: "الترتيب والمورد",
    colSubmitted: "المقدم",
    colPlugs: "التعويضات",
    colLeveledNet: "الصافي الموحد",
    colLeveledGross: "شامل الضريبة",
    colTraps: "الملاحظات",
    inspectorTitle: "فاحص الأدلة النصية",
    inspectorEmpty: "اختر أي مورد أو ملاحظة لعرض النص الحرفي من العرض.",
    noTrapsClean: "عرض مطابق بالكامل بدون استثناءات أو فروق حسابية.",
    settingsTitle: "الإعدادات ومزودو الذكاء",
    settingsTabProviders: "مزودو الذكاء",
    settingsTabRules: "قواعد الفحص",
    settingsTabExcel: "مخرجات Excel",
    connectedLabel: "متصل",
    disconnectedLabel: "غير متصل",
    signInChatGpt: "الدخول عبر ChatGPT",
    apiKeyPlaceholder: "أدخل مفتاح API المباشر...",
    saveKeyBtn: "حفظ",
    disconnectBtn: "فصل",
    geminiPrivacyShort: "الخصوصية",
    geminiPrivacyHint:
      "تأكيد استخدام باقة مدفوعة أو الإقرار بسياسة الباقة المجانية",
    effortLabel: "العمق",
    effortAuto: "تلقائي",
    effortLow: "سريع",
    effortMed: "متوازن",
    effortHigh: "عميق",
    plugBasisLabel: "أساس التعويض",
    plugLowest: "الأقل",
    plugAvg: "المتوسط",
    plugHigh: "الأعلى",
    plugPte: "التقديري",
    currencyLabel: "العملة",
    vatPctLabel: "الضريبة %",
    avlLabel: "قائمة المصنعين المعتمدين (AVL)",
    avlPlaceholder: "Trane, Carrier, Daikin, Schneider",
    customRulesLabel: "قواعد فحص المشروع",
    customRulesPlaceholder: "قاعدة في كل سطر، مثال: السعر يشمل السقالات",
    loadRulesBtn: "استيراد ملف",
    mergedBehaviorLabel: "معالجة البنود المدمجة",
    mergedAskInline: "سؤال فوري داخل المسار",
    mergedAutoHighlight: "تظليل أصفر تلقائي",
    sheetDirectionLabel: "اتجاه الشيت",
    commandPlaceholder: "انتقل إلى شاشة أو بدل اللغة أو نفذ أمرا...",
    commandEmpty: "لا توجد نتائج مطابقة.",
  },
};
