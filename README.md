# Realisation Manager AI

**Lokale AI-media-organisatie voor Mariën Sanitair en Centrale Verwarming**

Een desktop-applicatie voor Windows die duizenden iCloud-foto's en -video's automatisch organiseert, classificeert als zakelijk of prive, installatieprojecten detecteert en SEO-klare Nederlandse content genereert.

**Alle AI draait lokaal.** Geen cloud, geen betaalde diensten, geen externe API's. De applicatie werkt volledig offline na de eerste installatie.

---

## Belangrijkste functies

### 1. Media-import
- Ondersteunt JPG, JPEG, PNG, HEIC, HEIF, MOV, MP4, M4V, NEF, CR2, ARW, DNG
- Leest EXIF-metdata (camera, GPS, datum, lens, etc.)
- Genereert thumbnails (400×400 JPEG)
- Detecteert duplicaten via blake3-hashing (first 1 MiB + file size)
- Recente iCloud Photo Library exports worden automatisch herkend

### 2. Lokale AI-pipeline
- **Objectdetectie** (YOLOv8 ONNX) — boilers, heat pumps, radiatoren, sanitair, technical rooms
- **CLIP-embeddings** — scene-herkenning en visuele gelijkenis
- **OCR** (Tesseract) — leest merken van naamplaten
- **Kwaliteitsscore** — scherpte, contrast, belichting (heuristiek)
- **Merkherkenning** — Daikin, Vaillant, Bosch, Viessmann, Remeha, Buderus, Panasonic, Mitsubishi, Geberit, Grohe, Hansgrohe, Intergas, Atag, Nefit

### 3. Zakelijk/prive-classificatie
- Combineert objectdetectie, OCR, scene-tags, GPS, datum/tijd en bronmap
- Confidence-score per media
- Mapregels (gebruiker-gedefinieerd) overschrijven AI altijd
- Privacy Locked: handmatig vergrendelde media wordt nooit automatisch gewijzigd
- Privacymodus: verbergt alle prive-media in elke weergave

### 4. Projectdetectie
- Groepeert zakelijke media op datum (±3 dagen), GPS (< 1 km), objecten en merken
- Confidence-score per gedetecteerd project
- Goedkeuren, hernoemen, splitsen, samenvoegen, verwijderen
- Automatische projectnaam in het Nederlands (bv. "Badamer renovatie 2026-03 - Klant")

### 5. SEO-generator (Nederlands)
- Genereert titel (≤60 tekens), meta-description (≤160 tekens), URL-slug
- 5-10 keywords, OpenGraph-tags, alt-teksten per foto
- Body in Markdown en HTML (~400-600 woorden)
- JSON-LD schema.org markup (LocalBusiness + Service)
- Leestijd en word-count
- Status: draft → ready → published (nooit automatisch gepubliceerd)
- Optioneel: Ollama LLM voor uitgebreidere tekst

### 6. Projectbeheer
- Galerij per project met virtualized grid
- Before/after vergelijking
- Klantgegevens, locatie, tags
- Bulk-toewijzing van media aan projecten

### 7. Zoeken
- Full-text search (SQLite FTS5) op bestandsnamen, notities, projecten
- Multi-facet filtering (classificatie, type, datum, map, sterren, duplicaten)
- Snelkoppelingen in de top-nav

### 8. Dashboard
- Statistieken: totaal, zakelijk, prive, ongeclassificeerd, duplicaten, projecten
- AI-pipeline status
- Snelle acties: importeren, analyseren, SEO genereren
- Recente projecten

---

## Tech stack

| Laag | Technologie |
|------|-------------|
| Desktop framework | Tauri 2 |
| Frontend | React 18 + TypeScript + Vite |
| Styling | Tailwind CSS 3 |
| State | Zustand + React Query 5 |
| Routing | React Router 6 |
| Backend | Rust |
| Database | SQLite (rusqlite + r2d2 + refinery) |
| AI | ONNX Runtime (ort), Tesseract OCR |
| Media | image crate, kamadak-exif, ffmpeg (thumbnails) |
| Hashing | blake3, sha2 |

---

## Systeemvereisten

### Windows
- Windows 10 64-bit of nieuwer
- WebView2 Runtime (meestal vooraf geïnstalleerd op Windows 11)
- 4 GB RAM minimum (8 GB aanbevolen voor AI)
- 2 GB vrije schijfruimte (zonder mediabestanden)

### Voor AI-versnelling (optioneel)
- NVIDIA GPU met CUDA 11.8+ (sterk aanbevolen voor grote bibliotheken)
- Zonder GPU: CPU-inferentie (werkt, maar trager)

### Voor OCR (optioneel maar aanbevolen)
- Tesseract 5.x geinstalleerd op het systeem
- Download: https://github.com/UB-Mannheim/tesseract/wiki
- Inclusief Nederlandse taaldata (`tesseract-ocr-nld`)

### Voor video-thumbnails (optioneel)
- ffmpeg geinstalleerd op het systeem
- Download: https://ffmpeg.org/download.html
- Voeg toe aan PATH

---

## Zie ook

- **[INSTALL.md](./INSTALL.md)** — Stap-voor-stap installatie
- **[DEVELOPER.md](./DEVELOPER.md)** — Ontwikkelaarsdocumentatie
- **[LICENSE](./LICENSE)** — MIT

---

## Licentie

MIT © 2026 Mariën Sanitair en Centrale Verwarming
