export type UiLanguage = "ar" | "en";

export interface Dictionary {
  appName: string;
  appTagline: string;
  connectAiBtn: string;
  connectedBadge: string;
  notConnectedBadge: string;
  dropzoneTitle: string;
  dropzoneSub: string;
  browseFilesBtn: string;
  pasteHint: string;
  roleMasterBoq: string;
  roleVendorQuote: string;
  roleSpecAvl: string;
  syntheticBoqNotice: string;
  vendorNamePlaceholder: string;
  hiddenRowsWarning: string;
  removeDocBtn: string;
  advancedTitle: string;
  providerLabel: string;
  providerAuto: string;
  modelLabel: string;
  modelAuto: string;
  refreshModelsBtn: string;
  effortLabel: string;
  effortAuto: string;
  effortLow: string;
  effortMedium: string;
  effortHigh: string;
  layoutLabel: string;
  layoutSingle: string;
  layoutDual: string;
  mergedModeLabel: string;
  mergedInteractive: string;
  mergedYellow: string;
  plugBasisLabel: string;
  plugLowest: string;
  plugAverage: string;
  plugHighest: string;
  plugPte: string;
  currencyLabel: string;
  vatLabel: string;
  sheetDirLabel: string;
  avlLabel: string;
  avlPlaceholder: string;
  customRulesLabel: string;
  customRulesPlaceholder: string;
  loadRulesFileBtn: string;
  compareAndExportBtn: string;
  cancelRunBtn: string;
  authModalTitle: string;
  authModalSub: string;
  signInChatGptBtn: string;
  apiKeyPlaceholder: string;
  saveApiKeyBtn: string;
  disconnectBtn: string;
  geminiPrivacyLabel: string;
  closeBtn: string;
  ambiguityTitle: string;
  ambiguitySub: string;
  choiceAnchorZero: string;
  choiceProportional: string;
  choiceAlternate: string;
  choiceExcludePlug: string;
  resultWinnerLabel: string;
  saveExcelBtn: string;
  openExcelBtn: string;
  revealFolderBtn: string;
  saveUnlockedFallbackBtn: string;
  newComparisonBtn: string;
  vendorCol: string;
  submittedCol: string;
  plugsCol: string;
  leveledNetCol: string;
  leveledGrossCol: string;
  trapsCol: string;
  trapsSectionTitle: string;
  noTrapsFound: string;
}

export const DICTIONARIES: Record<UiLanguage, Dictionary> = {
  ar: {
    appName: "qlevel",
    appTagline: "مقارنة العروض المالية والفنية وتوحيد المقايسات",
    connectAiBtn: "ربط الذكاء الاصطناعي",
    connectedBadge: "متصل",
    notConnectedBadge: "غير متصل",
    dropzoneTitle: "اسحب ملفات المقايسة وعروض الموردين هنا",
    dropzoneSub: "يدعم ملفات Excel و PDF الرقمية والممسوحة ضوئيا، أو الصق جدول عرض سعر مباشرة بـ Ctrl+V",
    browseFilesBtn: "اختيار ملفات",
    pasteHint: "نصيحة: انسخ أي جدول من إيميل أو لقطة واتساب واضغط Ctrl+V للإضافة فورا",
    roleMasterBoq: "مقايسة أساسية",
    roleVendorQuote: "عرض مورد",
    roleSpecAvl: "مواصفات وموردين معتمدين",
    syntheticBoqNotice: "لم يتم رفع مقايسة أساسية - سيبني البرنامج مقايسة موحدة تلقائيا من عروض الموردين دون إسقاط أي بند",
    vendorNamePlaceholder: "اسم المقاول / المورد",
    hiddenRowsWarning: "يحتوي على صفوف مخفية في الإكسيل",
    removeDocBtn: "حذف",
    advancedTitle: "خيارات متقدمة (للمهندس المتخصص)",
    providerLabel: "المزود السحابي",
    providerAuto: "تلقائي (Auto)",
    modelLabel: "موديل الرؤية والتحليل (Multimodal)",
    modelAuto: "توجيه ذكي تلقائي (Auto)",
    refreshModelsBtn: "تحديث القائمة الحية",
    effortLabel: "عمق التدقيق الهندسي",
    effortAuto: "تلقائي",
    effortLow: "سريع",
    effortMedium: "متوازن",
    effortHigh: "عميق",
    layoutLabel: "تخطيط ملف الإكسيل الناتج",
    layoutSingle: "شيت واحد شامل (مالي + فني)",
    layoutDual: "شيتان منفصلان (CBE + TBE)",
    mergedModeLabel: "معالجة البنود المدمجة بمقطوعية",
    mergedInteractive: "بطاقة حسم سريعة أثناء الفحص",
    mergedYellow: "تظليل أصفر تلقائي بدون توقف",
    plugBasisLabel: "تسعير البنود غير المسعرة (Plug)",
    plugLowest: "أقل سعر مطابق",
    plugAverage: "متوسط العروض المطابقة",
    plugHighest: "أعلى سعر متحفظ",
    plugPte: "التكلفة التقديرية للمقايسة",
    currencyLabel: "عملة المقارنة",
    vatLabel: "ضريبة القيمة المضافة %",
    sheetDirLabel: "اتجاه شيت الإكسيل",
    avlLabel: "قائمة الماركات المعتمدة (AVL - مفصولة بفاصلة)",
    avlPlaceholder: "مثال: Trane, Carrier, Daikin, Schneider",
    customRulesLabel: "قواعد فحص إضافية خاصة بالمشروع",
    customRulesPlaceholder: "اكتب كل قاعدة في سطر، مثال: تأكد أن السعر يشمل السقالات واختبارات الطرف الثالث",
    loadRulesFileBtn: "استيراد ملف قواعد",
    compareAndExportBtn: "قارن العروض وجهز شيت الإكسيل",
    cancelRunBtn: "إيقاف العملية",
    authModalTitle: "ربط مزود الذكاء الاصطناعي المباشر",
    authModalSub: "تشفير محلي كامل على جهازك (AES-256-GCM) - لا تغادر ملفات مناقصاتك إلى أي خادم وسيط",
    signInChatGptBtn: "تسجيل الدخول بحساب ChatGPT",
    apiKeyPlaceholder: "الصق مفتاح الـ API المباشر هنا...",
    saveApiKeyBtn: "حفظ وتشفير محليا",
    disconnectBtn: "فصل الحساب",
    geminiPrivacyLabel: "أقر بأن باقة Gemini المجانية قد تستخدم البيانات لتحسين النماذج، أو أنني أستخدم مفتاح فوترة مدفوعا يحافظ على سرية العروض",
    closeBtn: "إغلاق",
    ambiguityTitle: "بند مدمج بمبلغ مقطوع يحتاج قرارك",
    ambiguitySub: "قام المورد بدمج أكثر من بند من المقايسة في رقم واحد. اختر طريقة التوزيع في شيت الإكسيل:",
    choiceAnchorZero: "تثبيت المبلغ على البند الأول وتصفير التوابع مع تظليل أصفر",
    choiceProportional: "توزيع المبلغ تناسبيا حسب أوزان المقايسة",
    choiceAlternate: "اعتماد كبديل فني مطابق",
    choiceExcludePlug: "اعتباره غير مسعر وتعويضه من أسعار المنافسين",
    resultWinnerLabel: "العرض الأقل الحقيقي بعد التوحيد",
    saveExcelBtn: "حفظ ملف الإكسيل الحي (.xlsx)",
    openExcelBtn: "فتح في Excel",
    revealFolderBtn: "عرض في المجلد",
    saveUnlockedFallbackBtn: "الملف مفتوح في Excel - احفظ كنسخة محدثة (_v2.xlsx)",
    newComparisonBtn: "مقارنة جديدة",
    vendorCol: "المقاول / المورد",
    submittedCol: "المقدم الصافي",
    plugsCol: "بدل النواقص والاستثناءات",
    leveledNetCol: "الصافي المقارن العادل",
    leveledGrossCol: "الإجمالي شامل الضريبة",
    trapsCol: "الملاحظات والفخاخ",
    trapsSectionTitle: "الفخاخ الفنية والاستثناءات والتعديلات المكتشفة",
    noTrapsFound: "لم يتم رصد استثناءات فنية أو أخطاء حسابية في العروض",
  },
  en: {
    appName: "qlevel",
    appTagline: "Commercial & Technical Bid Leveling Workbench",
    connectAiBtn: "AI Connection",
    connectedBadge: "Connected",
    notConnectedBadge: "Disconnected",
    dropzoneTitle: "Drop Master BOQ & Vendor Quotes Here",
    dropzoneSub: "Supports digital or scanned Excel & PDF files, or press Ctrl+V to paste tables & screenshots directly",
    browseFilesBtn: "Browse Files",
    pasteHint: "Tip: Copy any quote table from Excel/email or a WhatsApp screenshot and press Ctrl+V",
    roleMasterBoq: "Master BOQ",
    roleVendorQuote: "Vendor Quote",
    roleSpecAvl: "Specs / AVL",
    syntheticBoqNotice: "No Master BOQ uploaded - qlevel will synthesize a unified baseline from vendor quotes with zero dropped rows",
    vendorNamePlaceholder: "Vendor / Subcontractor Name",
    hiddenRowsWarning: "Contains hidden Excel rows",
    removeDocBtn: "Remove",
    advancedTitle: "Advanced Options",
    providerLabel: "Direct Cloud Provider",
    providerAuto: "Auto (Connected Account)",
    modelLabel: "Live Multimodal Model",
    modelAuto: "Smart Auto-Routing",
    refreshModelsBtn: "Refresh Live Models",
    effortLabel: "Audit Reasoning Effort",
    effortAuto: "Auto",
    effortLow: "Low",
    effortMedium: "Medium",
    effortHigh: "High",
    layoutLabel: "Excel Workbook Layout",
    layoutSingle: "Single Master Sheet (CBE + TBE)",
    layoutDual: "Dual Tabs (01_CBE + 02_TBE)",
    mergedModeLabel: "Merged Lump-Sum Items",
    mergedInteractive: "Interactive Quick Decision Card",
    mergedYellow: "Auto Yellow Highlight in Excel",
    plugBasisLabel: "Unquoted Scope Plug Basis",
    plugLowest: "Lowest Compliant Bidder",
    plugAverage: "Average Compliant Bidder",
    plugHighest: "Highest Compliant Bidder",
    plugPte: "Pre-Tender Estimate (PTE)",
    currencyLabel: "Project Currency",
    vatLabel: "Statutory VAT %",
    sheetDirLabel: "Excel Sheet Direction",
    avlLabel: "Approved Vendor List (AVL - comma separated)",
    avlPlaceholder: "e.g. Trane, Carrier, Daikin, Schneider",
    customRulesLabel: "Custom Project Audit Rules",
    customRulesPlaceholder: "One rule per line, e.g. Scaffolding above 6m and third-party T&C must be included",
    loadRulesFileBtn: "Load Rules File",
    compareAndExportBtn: "Compare Bids & Build Excel",
    cancelRunBtn: "Cancel Run",
    authModalTitle: "Connect Direct Cloud AI Provider",
    authModalSub: "Envelope-encrypted locally on your workstation (AES-256-GCM) - zero intermediate servers",
    signInChatGptBtn: "Sign in with ChatGPT",
    apiKeyPlaceholder: "Paste direct API key...",
    saveApiKeyBtn: "Encrypt & Save Locally",
    disconnectBtn: "Disconnect",
    geminiPrivacyLabel: "I acknowledge Google's unpaid Gemini Free Tier may review prompts, or confirm I am using a paid billing API key",
    closeBtn: "Close",
    ambiguityTitle: "Merged Vendor Lump-Sum Item",
    ambiguitySub: "This bidder combined multiple Master BOQ items into one lump sum. Choose how to equalize it:",
    choiceAnchorZero: "Keep lump sum on lead item, zero followers & highlight yellow",
    choiceProportional: "Split proportionally across BOQ weights",
    choiceAlternate: "Accept as compliant technical alternate",
    choiceExcludePlug: "Treat as unquoted & plug from peer bids",
    resultWinnerLabel: "True Lowest Equalized Bidder",
    saveExcelBtn: "Save Live Excel (.xlsx)",
    openExcelBtn: "Open in Excel",
    revealFolderBtn: "Show in Folder",
    saveUnlockedFallbackBtn: "File open in Excel - Save as updated copy (_v2.xlsx)",
    newComparisonBtn: "New Comparison",
    vendorCol: "Vendor",
    submittedCol: "Submitted Net",
    plugsCol: "Equalization Plugs",
    leveledNetCol: "True Leveled Net",
    leveledGrossCol: "Gross (Incl. VAT)",
    trapsCol: "Traps",
    trapsSectionTitle: "Detected Technical Exclusions, AVL & Math Traps",
    noTrapsFound: "No technical exclusions or arithmetic discrepancies detected",
  },
};
