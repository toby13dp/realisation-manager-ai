# Ontwikkelaarsdocumentatie

Technische documentatie voor ontwikkelaars die de applicatie willen uitbreiden, debuggen of aanpassen.

---

## 1. Projectstructuur

```
realisation-manager-ai/
├── package.json                  # Node-afhankelijkheden + scripts
├── tsconfig.json                 # TypeScript-configuratie
├── vite.config.ts                # Vite-configuratie (dev-server, aliases)
├── tailwind.config.js            # Tailwind-theme (kleuren, fonts, animaties)
├── postcss.config.js
├── index.html                    # Vite-entry HTML
├── README.md
├── INSTALL.md
├── DEVELOPER.md                  # dit bestand
├── .gitignore
│
├── src/                          # React-frontend
│   ├── main.tsx                  # App-entry (React Router, QueryClient)
│   ├── App.tsx                   # Routes + Toast-container
│   ├── index.css                 # Tailwind + custom component classes
│   ├── vite-env.d.ts
│   │
│   ├── components/               # Herbruikbare UI-componenten
│   │   ├── Sidebar.tsx           # Navigatie-zijbalk
│   │   ├── TopNav.tsx            # Top-bar met zoek + privacy-toggle
│   │   ├── StatCard.tsx          # Dashboard-statistiek
│   │   ├── MediaCard.tsx         # Enkele media-thumbnail
│   │   ├── MediaGrid.tsx         # Virtualized grid (duizenden items)
│   │   ├── ProjectCard.tsx       # Project-kaart
│   │   ├── ClassificationBadge.tsx
│   │   ├── ConfidenceBadge.tsx
│   │   ├── LoadingSpinner.tsx
│   │   ├── EmptyState.tsx
│   │   ├── Modal.tsx
│   │   └── Toast.tsx
│   │
│   ├── layouts/
│   │   └── MainLayout.tsx        # Sidebar + TopNav + content
│   │
│   ├── pages/                    # Routes
│   │   ├── Dashboard.tsx
│   │   ├── MediaLibrary.tsx
│   │   ├── Projects.tsx
│   │   ├── ProjectDetail.tsx
│   │   ├── AIAnalysis.tsx
│   │   ├── SEOManager.tsx
│   │   ├── PrivacyCenter.tsx
│   │   └── Settings.tsx
│   │
│   ├── hooks/                    # React Query-hooks
│   │   ├── useMedia.ts
│   │   ├── useProjects.ts
│   │   ├── useAIAnalysis.ts
│   │   ├── useSeo.ts
│   │   └── useToast.ts
│   │
│   ├── services/                 # Tauri-command wrappers
│   │   ├── tauri.ts              # invoke() + event-listen helpers
│   │   ├── mediaService.ts
│   │   ├── projectService.ts
│   │   ├── aiService.ts
│   │   ├── seoService.ts
│   │   ├── settingsService.ts
│   │   └── types.ts
│   │
│   ├── store/                    # Zustand-stores
│   │   ├── mediaStore.ts
│   │   ├── projectStore.ts
│   │   ├── settingsStore.ts
│   │   └── toastStore.ts
│   │
│   ├── types/                    # TypeScript-domeinmodellen
│   │   ├── index.ts
│   │   ├── media.ts
│   │   ├── project.ts
│   │   ├── ai.ts
│   │   └── seo.ts
│   │
│   ├── lib/
│   │   ├── utils.ts              # cn() = clsx + tailwind-merge
│   │   ├── format.ts             # Datum-, byte-, percent-formattering (nl-NL)
│   │   └── constants.ts          # App-naam, nav-items, bekende merken
│   │
│   └── utils/
│       ├── fileTypes.ts          # Extensie → MediaType mapping
│       └── validation.ts         # Email, slug, clamp
│
└── src-tauri/                    # Rust-backend
    ├── Cargo.toml
    ├── tauri.conf.json           # Tauri-configuratie (venster, CSP, bundling)
    ├── build.rs                  # tauri_build::build()
    ├── capabilities/
    │   └── default.json          # Tauri 2 permissies
    ├── migrations/
    │   ├── 001_init.sql          # Eerste schema (tabellen, indexes, seeds)
    │   └── 002_add_indexes.sql   # FTS5, triggers, views
    │
    └── src/
        ├── main.rs               # Binary entry (windows_subsystem = windows)
        ├── lib.rs                # App-state, Tauri-builder, command-handler
        ├── db.rs                 # r2d2-pool + refinery migrations
        ├── models.rs             # Domeinmodellen (enum, struct, serde)
        │
        ├── repositories/         # SQL-laag (één repo per tabel)
        │   ├── mod.rs
        │   ├── media_repo.rs
        │   ├── project_repo.rs
        │   ├── ai_repo.rs
        │   ├── seo_repo.rs
        │   ├── settings_repo.rs
        │   ├── job_repo.rs
        │   └── folder_rule_repo.rs
        │
        ├── services/             # Business logic
        │   ├── mod.rs
        │   ├── scanner.rs        # Walkdir + blake3 + parallel insert
        │   ├── exif.rs           # kamadak-exif reader
        │   ├── thumbnails.rs     # image + ffmpeg
        │   ├── ai_pipeline.rs    # ONNX-sessies + OCR + quality
        │   ├── classifier.rs     # Business/prive heuristiek
        │   ├── project_detector.rs # Date/GPS clustering
        │   ├── seo_generator.rs  # Dutch SEO-content
        │   ├── search.rs         # FTS5 wrappers
        │   └── job_registry.rs   # Cancel tokens
        │
        └── commands/             # Tauri-command handlers
            ├── mod.rs
            ├── media.rs
            ├── projects.rs
            ├── scanner.rs
            ├── ai.rs
            ├── seo.rs
            ├── settings.rs
            ├── search.rs
            ├── jobs.rs
            └── stats.rs
```

---

## 2. Ontwikkelomgeving opzetten

### Vereisten
- **Rust** 1.75 of nieuwer — https://rustup.rs/
- **Node.js** 20 of nieuwer — https://nodejs.org/
- **Microsoft C++ Build Tools** (voor Tauri) — https://visualstudio.microsoft.com/visual-cpp-build-tools/
- **WebView2** — https://developer.microsoft.com/microsoft-edge/webview2/
- **Git**

### Optioneel (voor volledige AI-pipeline)
- **Tesseract 5** (OCR)
- **ffmpeg** (video-thumbnails, HEIC-decodering)
- **CUDA Toolkit 11.8+** (GPU-versnelling)

### Stap-voor-stap
```powershell
# 1. Clone
git clone <repository>
cd realisation-manager-ai

# 2. Installeer Node-afhankelijkheden
npm install

# 3. Start dev-server (opent Tauri-window met hot-reload)
npm run tauri:dev

# 4. Build productie-installer
npm run tauri:build
# Output: src-tauri/target/release/bundle/msi/*.msi
```

---

## 3. Architectuur

### Frontend ↔ Backend communicatie

Alle communicatie tussen React en Rust verloopt via **Tauri-commands**:

```typescript
// Frontend
import { call } from '@/services/tauri';
const media = await call<Media[]>('list_media', { classification: 'business' });
```

```rust
// Backend
#[tauri::command]
pub async fn list_media(
    state: State<'_, AppState>,
    classification: Option<Classification>,
    // ...
) -> Result<Vec<Media>, String> {
    // ...
}
```

### Event-systeem

De backend emit events tijdens langlopende operaties:

```typescript
import { onEvent } from '@/services/tauri';
onEvent<ScanProgress>('scan://progress', (p) => {
    console.log(`${p.current}/${p.total}`);
});
```

Events gebruikt in deze app:
- `app://ready` — startup voltooid
- `scan://started`, `scan://progress` — scan-voortgang
- `ai://analyzed`, `ai://batch-started`, `ai://batch-progress`, `ai://batch-done`
- `ai://projects-detected`

### State management

- **Server state** (media, projecten, SEO) → React Query
  - Automatische cache, refetch, invalidatie
  - `queryKey` hiërarchie: `['media', 'list', params]`
- **UI state** (filters, selectie, modals) → Zustand
  - `useMediaStore`, `useProjectStore`, `useSettingsStore`, `useToastStore`
- **Persistente settings** → SQLite `settings`-tabel + Zustand-cache

### Database-schema

Zie `src-tauri/migrations/001_init.sql` voor het volledige schema. Belangrijke tabellen:

- `projects` — installatieprojecten
- `media` — alle geimporteerde foto's/video's
- `exif_data` — EXIF-metadata per media
- `ai_analysis` — resultaten van elke AI-stap per media
- `seo` — gegenereerde SEO-content per project
- `settings` — key-value configuratie
- `folder_rules` — gebruiker-gedefinieerde mapregels
- `jobs` — achtergrond-taken

### Concurrency-model

- **r2d2 pool** (8 verbindingen) voor SQLite-reads
- **Single-writer mutex** (`DbPool::write_lock`) om `database is locked` te voorkomen
- **rayon** voor parallelle CPU-workloads (scan, batch-analyse)
- **tokio** voor async Tauri-commands (`spawn_blocking` voor CPU-bound werk)
- **crossbeam-channel** voor progress-forwarding tussen threads

---

## 4. AI-pipeline details

### Pipeline-stappen per media

1. **Objectdetectie** (YOLOv8 ONNX) — 640×640 input, NMS 0.45, threshold 0.45
2. **CLIP-embedding** (CLIP-ViT ONNX) — 224×224 input, 512-dim embedding
3. **Scene-tags** — cosine-similarity vs. vooraf berekende tag-embeddings
4. **OCR** — Tesseract met `nld+eng` talen
5. **Merkherkenning** — string-match van OCR + objectlabels tegen `brands.known` setting
6. **Kwaliteitsscore** — scherpte (gradiënt) + contrast (stdev) + belichting (mean) — gewogen 0.4/0.3/0.3
7. **Classificatie** — combineert alle signalen met gewichten (zie hieronder)
8. **Projectdetectie** — aparte stap, draait op de volledige set zakelijke media

### Classificatie-gewichten

| Signaal | Zakelijk | Prive |
|---------|----------|-------|
| Object `boiler/heat_pump/radiator/...` | +confidence × 1.0 | — |
| Object `person/pet/food/nature` | — | +confidence × 0.7 |
| OCR merk gevonden | +0.9 | — |
| Scene tag `cv-ketel/badkamer/...` | +0.3 | — |
| Scene tag `natuur/eten/...` | — | +0.3 |
| Bronmap bevat `werk/project/klant` | +0.5 | — |
| Bronmap bevat `prive/familie/vakantie` | — | +0.6 |
| Weekdag + daglicht (8-18u) | +0.1 | — |
| Weekend | — | +0.15 |

Eindbeslist: hoogste score wint, mits `score / total ≥ confidence_threshold` (default 0.55).

### Projectdetectie-algoritme

1. Filter alle media met `classification='business'` en `is_private=false`
2. Sorteer op `date_taken`
3. Cluster: media binnen ±3 dagen van elkaar → zelfde cluster
4. Verfijn elk cluster: split als GPS-locaties > 1 km uit elkaar liggen
5. Inferieer project-type uit dominante objectlabel (boiler → `cv_boiler`, etc.)
6. Bereken confidence: cluster-grootte + datumbereik + GPS-consistentie
7. Genereer Nederlandse naam: `<Type> <YYYY-MM> - <laatste mapnaam>`

---

## 5. SEO-generator details

### Templates

De SEO-generator gebruikt string-templates (geen externe afhankelijkheid). Voorbeeld voor titel:

```
{type_label} in {location} ({date_label}) | {brand_name}
```

Truncatie:
- Titel: ≤ 60 tekens
- Meta-description: ≤ 160 tekens (met `...`-suffix indien nodig)
- Slug: `<type>-<location>-<project_id_short>`

### Body-structuur (~400-600 woorden)

1. H1 + introductie (1 paragraaf)
2. H2 Projectomschrijving (1 paragraaf)
3. H2 Werkzaamheden (6 bullet points)
4. H2 Materialen en merken (1 paragraaf of lijst)
5. H2 Fotodocumentatie (1 paragraaf)
6. H2 Veelgestelde vragen (3 Q&A's)
7. H2 Contact (1 paragraaf)

### JSON-LD schema.org

Type `Service` met `provider` LocalBusiness, `areaServed`, `serviceType`. Volledig in de `schema_org_json` kolom van de `seo`-tabel.

### Ollama-integratie (optioneel)

Als `ai.enable_ollama=true` wordt de body-tekst via een lokale LLM verrijkt. Endpoint: `http://localhost:11434/api/generate`. De app werkt volledig zonder Ollama.

---

## 6. Code-stijl en conventies

### Rust
- `cargo fmt` + `cargo clippy` voor commit
- Modules: `snake_case` bestandsnamen
- Structs/enums: `PascalCase`
- Publieke functies: volledige documentatie-commentaar
- Errors: `anyhow::Result` in services, `Result<T, String>` in Tauri-commands

### TypeScript
- `eslint` + `tsc --noEmit` voor commit
- Bestanden: `PascalCase` voor componenten, `camelCase` voor services/utils
- Interfaces: `PascalCase`, geen `I`-prefix
- Imports: pad-aliases (`@/`, `@components/`, etc.)
- React-componenten: function components, geen class components

### Database
- Eén migration per feature-toevoeging (volg `00N_naam.sql` patroon)
- Altijd `IF NOT EXISTS` in DDL
- Indexen voor alle vaak-gefilterde kolommen
- Foreign keys met `ON DELETE CASCADE` voor child-tabellen

---

## 7. Debuggen

### Logs
- Rust: `env_logger` — logt naar stderr. Set `RUST_LOG=debug` voor verbose output
- Frontend: browser-devtools (F12 in Tauri-window)

### Database inspecteren
```powershell
# SQLite CLI (indien geinstalleerd)
sqlite3 "%APPDATA%\nl.marien.realisation-manager-ai\realisation-manager.db"

# Handige queries
.tables
.schema media
SELECT * FROM settings ORDER BY category, key;
SELECT classification, COUNT(*) FROM media GROUP BY classification;
SELECT status, COUNT(*) FROM projects GROUP BY status;
```

### Tauri-devtools
- Tijdens `npm run tauri:dev` is de devtools-overlay beschikbaar (right-click → Inspect)
- In productie-builds staan devtools uit

---

## 8. Bijdragen

### Workflow
1. Maak een feature-branch: `git checkout -b feature/mijn-feature`
2. Implementeer + voeg tests toe
3. `cargo fmt && cargo clippy && npm run lint && npm run typecheck`
4. Commit met duidelijke boodschap (Conventional Commits aanbevolen)
5. Open een pull-request

### Tests toevoegen
- **Rust**: `#[cfg(test)]` modules in elk bestand
- **TypeScript**: Jest of Vitest (nog niet geconfigureerd — TODO)

---

## 9. Bekende beperkingen

- HEIC-decodering via `image` crate werkt niet — valt terug op ffmpeg. Voor productie: overweeg `heif` crate.
- RAW-formatten (NEF, CR2, etc.) worden niet gedecodeerd — alleen thumbnails via ffmpeg
- CLIP tag-embeddings zijn dummy-waarden tot een echte text-encoder wordt toegevoegd
- Geen automatische watch-folder polling (gebruiker moet handmatig scan draaien)
- Geen multi-window support (enkel hoofdvenster)
- Geen export naar externe CMS-en (alleen Markdown-export)

---

## 10. Roadmap

- [ ] Echte CLIP text-encoder voor scene-tags
- [ ] Watch-folder polling met `notify` crate
- [ ] Multi-window modus (gelijktijdige media + project weergave)
- [ ] Export naar WordPress / Statamic
- [ ] Reverse geocoding (offline NL dataset)
- [ ] Face-clustering (optioneel, voor betere prive-classificatie)
- [ ] Multi-user support (voor meerdere monteurs)
- [ ] Mobile companion app (foto's direct van telefoon syncen)

---

**Vragen?** Zie README.md voor contactinformatie.
