# Guide: D3 caption translation beyond English (16 Oct 2026)

Status: guide for the owner and for the AI session that builds it. Decision source: roadmap v2, §4-adjacent D3 section (`kconcat-roadmap-v2-execution.md`). If the two disagree, fix the roadmap first.

Discussion only. No repo code was changed by this guide.

---

## 1. Goal (owner, 16 Oct 2026)

Captions can be translated into languages other than English. The app ships with English only. Other languages need a translation model that the user imports. Default is English.

**CapCut Pro item served:** caption translation (Option B D3). **Measurement:** one caption track is translated into the chosen language, and the bilingual output works with the clean-bilingual preset (C2, A9).

---

## 2. Decisions (owner, 16 Oct 2026)

1. **Import, not bundled.** No translation model ships with the app. The user imports a model file, the same way they pick a Whisper model.
2. **Default is English.** English comes from Whisper translate, which is already in the app. No file is needed.
3. **Other languages:** the user imports the model file for that language or set of languages. Storage is used only when the user imports.
4. **Model choice follows the existing pattern:** Settings → Transcriber has a models folder, an installed list, and "Get model". A translation model slot follows the same pattern.

---

## 3. What the user sees

### 3.1 Settings → Transcriber

- A new section: **Translation models**.
- Lists installed translation models, with the language pair or languages each one covers.
- Button: **Import model** (choose a file or folder from disk).
- Each imported model can be removed from the list (the file on disk is not deleted without a confirm step).

### 3.2 Caption dialog

- Today: a toggle **"Translate to English"** (`captions.translateEnglish`), with a hint when the chosen Whisper model cannot translate (turbo fallback, `captions.slint` near line 282).
- New: a **Translate to** dropdown next to the toggle.
  - Lists **English** first, always.
  - Then only languages that have an imported model.
  - If no model is imported, the dropdown shows English only, with a line: "Import a translation model in Settings → Transcriber to add languages."
- The 99-language list in the caption dialog stays for **speech recognition** (source language). It is not the translation list.

### 3.3 Hints and errors

| State | What the user sees |
|---|---|
| No translation model imported | English only. Hint line as in 3.2. |
| Model imported, language chosen | Translation runs after transcription. |
| Chosen Whisper model cannot translate | Existing turbo fallback hint (unchanged). |
| Wrong file imported | Message: "This file is not a translation model the app can read." Nothing is added to the list. |
| Model file removed from disk | Language disappears from the dropdown. The caption text stays as it was. |

---

## 4. Open decisions for the diff (not owner decisions yet)

- **Model format:** which format the engine reads. Opus-MT and NLLB need converting. Support one format first. Decide in the diff; record it in the manual.
- **Import as file or folder:** decide in the diff. The format decides it.
- **Where the translation runs:** same engine as Whisper, or a separate one. Decide in the diff. Storage and speed differ.

---

## 5. Storage (third-party figures, not measured here)

- **Default (English only):** no extra storage.
- **Opus-MT:** about 300 MB per language direction. Example: English to Indonesian is one direction, about 300 MB.
- **NLLB-200 distilled 600M, INT8:** about 1.1 GB, covers many languages in one model.
- The user pays this only for models they import.

---

## 6. Tests

1. **Default:** with no model imported, the dropdown shows English only.
2. **Import:** importing one valid model adds its language(s) to the dropdown.
3. **Wrong file:** importing a file the app cannot read shows the message and adds nothing.
4. **Removal:** removing the model file from disk removes the language from the dropdown; caption text is unchanged.
5. **Translation output:** one caption track translated into the chosen language. Check word count and line breaks in the caption layout.
6. **Turbo hint:** the existing hint still shows for a model that cannot translate.
7. **Bilingual output:** clean-bilingual preset (C2) with the translated line below the original.

---

## 7. Do not

- Do not bundle a translation model in the app.
- Do not change the 99-language speech list.
- Do not change the English translate toggle's behaviour.
- Do not delete a model file from disk without a confirm step.

---

## 8. Done when

- Default is English only, with no extra file.
- A user can import one model and translate one caption track into its language.
- Tests 1 to 7 pass.
