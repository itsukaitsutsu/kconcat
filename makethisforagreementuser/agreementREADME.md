# Legal Draft Pack — [PRODUCT NAME]

**INTERNAL DRAFT — do not publish until every `[REQUIRED]` field is completed and the documents are reviewed by Indonesian counsel.** These are practical draft templates, not legal advice or a guarantee that any clause will be enforceable.

## Important language note for an Indonesia-facing business

This English set is intended to be easier to review and use alongside the software. It should **not automatically replace the Bahasa Indonesia documents** for agreements with Indonesian customers. Article 31 of Indonesia's Law No. 24 of 2009 requires Bahasa Indonesia for agreements involving Indonesian citizens or Indonesian private institutions. Presidential Regulation No. 63 of 2019 addresses foreign-language versions/counterparts and interpretation of bilingual agreements. Have Indonesian counsel confirm how this applies to your online checkout terms and prepare a matching Bahasa Indonesia version. If you publish bilingual documents, the two versions should be aligned and the final agreement should state language precedence as advised by counsel.

Official references:

- Law No. 24 of 2009: <https://peraturan.bpk.go.id/Download/27970/UU%20Nomor%2024%20Tahun%202009.pdf>
- Presidential Regulation No. 63 of 2019: <https://peraturan.bpk.go.id/Download/112437/Perpres%20Nomor%2063%20Tahun%202019.pdf>

## Assumptions used

- Main market: Indonesia.
- Sales model: one-time purchase for a convenience package—installer/build, selected model packages, documentation, and limited installation assistance.
- Standard assistance does **not** include fixing bugs in Concat or maintaining its code, unless separately agreed.
- The website/store may process account, order, payment, and support data.
- It has not yet been confirmed whether the app sends telemetry/crash reports or uploads user media. The Privacy Policy therefore contains items that must be verified.

## Critical AGPL boundary

This pack is **not a proprietary license for Concat or a modified Concat fork** and does not replace AGPL. If you distribute or provide a modified AGPL binary, provide the *Corresponding Source* for that exact release as AGPL requires. Include the applicable full license text, copyright/attribution notices, change notices, and third-party notices. Do not add terms that restrict recipients' AGPL rights to use, modify, or redistribute the covered software.

- Official AGPL text: <https://www.gnu.org/licenses/agpl-3.0.en.html>
- License in your repository: <https://github.com/itsukaitsutsu/kconcat/blob/main/LICENSE>
- Upstream Concat: <https://github.com/jub0t/Concat>

**Verify the exact license and commit used for each release.** The fork and upstream may identify the license differently. Add the source URL for every release: `[REQUIRED: SOURCE URL FOR THIS BUILD/RELEASE]`. Do not distribute an AGPL binary until its corresponding source is available as required by the license.

## Items to complete before publication

1. Product/brand name that does not use Concat's name or logo as if this were an official release: `[PRODUCT NAME]`.
2. Provider identity: `[INDIVIDUAL/BUSINESS NAME]`, address, service email, privacy email, and any business registration details.
3. Each package's contents: version/commit, supported operating systems, installer, included/downloaded models, source URL, and each third-party component/model license.
4. Scope and duration of installation assistance: `[CHANNEL]`, `[DURATION]`, and a response time you can actually meet.
5. Price, taxes, payment method, delivery process, cancellations/refunds, and complaint channel.
6. Payment, hosting, email, support, and analytics vendors and the countries where they process data.
7. Data actually collected by the website, installer, and app—including IP/log data, cookies, crash reports, telemetry, and whether user media/projects are ever uploaded.
8. Retention periods and deletion procedures for each category of data.
9. Tests performed, known issues, and compatibility that has actually been verified.

## Pre-launch checklist

- [ ] Use your own brand and icon for modified builds; do not present them as official Concat releases.
- [ ] Create a source archive/tag matching every binary and display its link beside the download/purchase access.
- [ ] Include the applicable full AGPL text, `THIRD_PARTY_NOTICES`, and licenses for models/assets you distribute.
- [ ] Confirm that you have rights to commercially distribute every model, font, codec, asset, and packaged file.
- [ ] Test account, payment, data-deletion request, and privacy-incident workflows.
- [ ] Display price, support limits, cancellation/refund policy, and a savable copy of the electronic agreement.
- [ ] Ask Indonesian counsel to review liability, refunds, personal data, and the way you distribute AGPL binaries.

## Draft files

- `01-package-and-limited-support-agreement.md`
- `02-terms-and-conditions.md`
- `03-privacy-policy.md`
