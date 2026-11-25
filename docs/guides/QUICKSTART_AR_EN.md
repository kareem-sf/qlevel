# دليل التشغيل السريع لمهندس المشتريات والمكتب الفني (Quickstart Guide - AR / EN)

## أولا: الدليل السريع بالعربية (الوضع الأساسي Basic)

صمم `qlevel` ليعمل مع مهندس المشتريات والحصر في 3 خطوات مباشرة بدون أي تعقيد تقني:

1. **ربط حساب الذكاء الاصطناعي لمرة واحدة**:
   - افتح نافذة **الحسابات (AI Connection)** في أعلى التطبيق.
   - اختر إما تسجيل الدخول بضغطة زر عبر **Sign in with ChatGPT**، أو لصق مفتاح API مباشر من **OpenAI** أو **Google Gemini** أو **Anthropic Claude**.
   - تحفظ المفاتيح مشفرة محليا على جهازك فقط ولا تغادره إلى أي خادم وسيط.
2. **إلقاء الملفات أو اللصق المباشر (`Ctrl+V`)**:
   - اسحب ملف **المقايسة الأساسية (Master BOQ)** وملفات **عروض الموردين** (Excel أو PDF رقمي أو ممسوح ضوئيا) وأفلتها داخل مساحة العمل.
   - يمكنك أيضا نسخ جدول عرض سعر من بريد إلكتروني أو لقطة شاشة من واتساب والضغط على `Ctrl+V` مباشرة داخل البرنامج.
   - يصنف البرنامج كل ملف تلقائيا (`مقايسة أساسية` / `عرض مورد` / `مواصفات وقائمة موردين معتمدين`) ويمكنك تعديل التصنيف أو اسم المورد بنقرة واحدة. إذا لم تتوفر مقايسة أساسية، يبني البرنامج مقايسة موحدة تلقائيا من عروض الموردين.
3. **المقارنة وتصدير شيت الإكسيل الحي**:
   - اضغط **قارن وصدر شيت الإكسيل**.
   - إذا دمج أحد الموردين عدة بنود في مبلغ مقطوع واحد، تظهر لك بطاقة حسم سريعة واضحة (أو يمكنك اختيار التظليل الأصفر التلقائي من الخيارات المتقدمة).
   - اضغط **فتح في Excel** لمراجعة الشيت بالمعادلات الحية (`SUM` / `MIN` / `IF`) وجدول الفخاخ والاستثناءات الفنية. إذا كان الملف مفتوحا مسبقا في Excel، يعرض البرنامج حفظ نسخة محدثة `_v2.xlsx` بضغطة واحدة.

### متى أفتح لوحة الخيارات المتقدمة (`Advanced Options`)؟
اتركها مطوية في الاستخدام اليومي (الوضع التلقائي `Auto` يختار الموديل الأنسب لكل صفحة)، وافتحها فقط عندما تريد:
- اختيار موديل Multimodal بعينه أو تحديد مستوى التفكير (`Low` / `Medium` / `High`).
- التبديل بين **شيت واحد شامل (`Single Sheet`)** أو **شيتين منفصلين (`CBE + TBE`)**.
- التبديل بين **بطاقة الحسم السريعة** أو **التظليل الأصفر المباشر** للبنود المدمجة.
- تغيير العملة، أو نسبة ضريبة القيمة المضافة (`15%` السعودية، `14%` مصر، `5%` الإمارات)، أو إضافة **قواعد فحص خاصة بمشروعك**.

## Second: English Quickstart Guide

1. **Connect Your Provider Once**: Click **AI Connection** in the header and sign in via **OpenAI Sign in with ChatGPT** or paste a direct API key for **OpenAI**, **Google Gemini**, or **Anthropic Claude**. Credentials are envelope-encrypted locally (`32-byte` OS Keyring DEK + `AES-256-GCM`).
2. **Drop Files or Paste (`Ctrl+V`)**: Drag Excel/PDF tender documents into the workspace or press `Ctrl+V` to paste spreadsheet ranges or quote images. Verify or toggle each card's role (`Master BOQ`, `Vendor Quote`, `Specs / AVL`). If you only have vendor quotes without a Master BOQ, `qlevel` synthesizes a unified baseline automatically.
3. **Compare & Open Live Excel**: Click **Compare & Export Excel**. Resolve any merged lump-sum cards (or choose `Yellow Highlight` auto-export in `Advanced Options`) and open the live-formula `.xlsx` workbook directly in Microsoft Excel.
