# Guide: D5 extra editing, music and video (16 Oct 2026)

Status: guide for the owner and for the AI session that builds it. Source: roadmap v2, §6b (Track 6, K1–K5). If the two disagree, fix the roadmap first.

Discussion only. No repo code was changed by this guide.

---

## 1. Goal (owner, 16 Oct 2026)

Editing is not limited to podcasts. Music editing is in scope too. Each task must name its CapCut Pro parity item (Option B D5) or its measurement. All AI runs locally on the owner's machine (standing rule). Model file size is measured in the diff and reported to the owner.

---

## 2. Order

After Track 5 (P6). Within Track 6, the order is K1 (no model) first, then K2 (no model), then K3 to K5 (each needs a local model).

| No | Task | Needs model | Serves |
|---|---|---|---|
| K1 | Beat sync | No | Music cuts on beats |
| K2 | Flicker removal | No | Lamp flicker in footage |
| K3 | Camera tracking | Yes | Tracked crop or graphic |
| K4 | AI colour correction | Yes | Automatic colour fix |
| K5 | AI text remover | Yes | Remove on-screen text |

---

## 3. Each task

### K1 Beat sync
- **What:** detect beats in a music clip, and snap cuts or clip starts to them.
- **Needs:** a local beat detector. No model file needed for a first version.
- **Test:** on a music clip with a known tempo, the detected beats match a hand count within a tolerance set after a baseline.
- **Done when:** the user can snap cuts to beats on one timeline.

### K2 Flicker removal
- **What:** reduce brightness flicker from mains-frequency lighting.
- **Test:** on a flickering test clip, brightness variation drops by a measured amount. The number is set after a baseline.
- **Done when:** the effect is applied to one clip, and the test clip is less flickery.

### K3 Camera tracking
- **What:** follow a moving subject, and drive a crop or a graphic from the track.
- **Needs:** a local tracker. Model size to be measured.
- **Test:** on a clip with a known moving object, the tracked point stays on the object within a tolerance set after a baseline.
- **Done when:** one track drives a crop or a graphic on one clip.

### K4 AI colour correction
- **What:** one button applies an automatic colour correction. The user can undo it and adjust its strength.
- **Note:** existing filters and curves already do manual correction. K4 adds only the automatic part.
- **Needs:** a local model. Model size to be measured.
- **Test:** on a colour-cast test clip, the cast drops by a measured amount. The number is set after a baseline.
- **Done when:** one button applies the correction, and it can be undone.

### K5 AI text remover
- **What:** the user marks an area, and the text in it is removed.
- **Needs:** a local inpainting model. Model size to be measured.
- **Test:** on a clip with a known text area, the text is gone and the surrounding area is not visibly damaged.
- **Done when:** the user can mark an area and remove the text in it.

---

## 4. Rules

- **No new UI** beyond what each task needs: a button, a strength control, or an area marker.
- **Do not change working paths** without need (owner rule).
- **Tests come first.** Write the failing test, then the change.
- **Verify in the app** before calling an item done.

---

## 5. Done

- Each task passes its test, and its result is recorded in the roadmap.
- Model file sizes for K3 to K5 are reported to the owner before any model is bundled. Models are imported by the user, the same way as D3, unless the owner decides otherwise.

---

## 6. Local model list (K3–K5)

Sizes come from the sources named below. "Unverified" means the size or licence was not checked. Nothing is bundled yet. Each model is measured again in the diff before it is accepted.

### K3 Camera tracking
| Option | Size | Note | Source |
|---|---|---|---|
| OpenCV VitTrack (ONNX) | 0.68 MiB (715 kB) | First choice. Gives a tracking score. Needs a box drawn on the first frame. | OpenCV tutorial (29 Aug 2026); HF `opencv/object_tracking_vittrack` |
| OpenCV NanoTrack v2 (two ONNX files) | 1.70 MiB | Backup if VitTrack fails the test. Needs a first-frame box. | OpenCV tutorial; OpenCV TrackerNano docs |
| OpenCV DaSiamRPN | 154.35 MiB | Only if the two small models fail. Large download. | OpenCV tutorial |

Note (assistant proposal): the user draws the box on the first frame, and the tracker follows it. Automatic object detection is not in K3. Detection would add a model and is not proposed.

### K4 AI colour correction
The footage is finished sRGB video. Most published white-balance models work on raw camera data or on single photos, so they do not fit directly. The options below are the closest fits found.

| Option | Size | Note | Source |
|---|---|---|---|
| Learned 3D LUT (image-adaptive) | Under 600K parameters, about 2–3 MB at fp32 (estimate from parameter count) | Predicts a LUT per frame, so it fits the existing LUT path (D2). Public weights unverified. | Zhang et al., PAMI (polyu.edu.hk paper) |
| Perception-inspired WB editing | 6.40 MB | Photo white-balance editing, not video. Public release unverified. | arXiv 2512.09383 |
| Deep White-Balance Editing | About 189 MB package | Includes reference code. Model size alone not separated. Unverified for video. | MATLAB File Exchange (Deep_White_Balance) |
| Camera AWB models (about 500 to 5K parameters) | Under 1 MB | Work on raw camera data. Not suitable for finished video. | arXiv 2507.01342; arXiv 2504.05623 |

Note (assistant proposal): the first thing to check is whether the learned 3D LUT or the perception model has public weights and a licence that allows personal use. If neither does, K4 needs a new decision from the owner.

### K5 AI text remover
Two models are needed: one to find the text (optional at first, since the user marks the area), and one to fill the area.

| Option | Size | Note | Source |
|---|---|---|---|
| MI-GAN (ONNX) | About 29.5 MB | First choice. Fast on CPU. Mobile-oriented. | Card Anvil HF `MI-GAN-ONNX`; MI-GAN paper (ICCV 2023) |
| MI-GAN (LiteRT, fp16) | 16.3 MB | Only if the app moves to LiteRT. Not the current path. | HF `litert-community/MI-GAN-512-Places2-LiteRT` |
| LaMa (ONNX, fp32) | 198–208 MB | Optional "HQ" choice. Slower, large download. | HF `Carve/LaMa-ONNX`; Medium article (LaMa in browser) |
| PP-OCRv5 mobile text detection | 4.7 MB | Only if auto-finding the text is added later. | PaddleX text-detection table |
| PP-OCRv3 text detection (ONNX) | 2.3 MB | Smaller alternative to the v5 mobile model. | HF `monkt/paddleocr-onnx` |
| PP-OCRv5 text detection (ONNX) | 84–88 MB | Server model. Not needed for K5. | HF `monkt/paddleocr-onnx`; HF `bluecopa/paddleocr-v5-onnx` |

Note (assistant proposal): K5 starts with MI-GAN and the user-drawn area. LaMa is an option for quality, and the text detector is a later add-on.

### K1 and K2
No model. K1 uses a local beat detector. K2 uses local image analysis.

---

## 7. Model list summary (first choice)

| Task | First choice | Size | Backup |
|---|---|---|---|
| K3 | OpenCV VitTrack | 0.68 MiB | NanoTrack v2 (1.70 MiB) |
| K4 | Undecided. Check learned 3D LUT or perception model for public weights and licence. | 2–3 MB or 6.40 MB (unverified) | Deep WB (about 189 MB package) |
| K5 | MI-GAN (ONNX) | About 29.5 MB | LaMa (198–208 MB) |
