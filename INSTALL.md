# Installatiehandleiding

Deze handleiding beschrijft hoe u **Realisation Manager AI** installeert en in gebruik neemt op Windows.

---

## 1. Systeemvereisten controleren

### Vereist
- Windows 10 64-bit of nieuwer
- WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/
- 4 GB RAM (8 GB aanbevolen)

### Aanbevolen (voor volledige AI-functionaliteit)
- **Tesseract 5.x** voor OCR (merken lezen van naamplaten)
  - Download: https://github.com/UB-Mannheim/tesseract/wiki
  - Installeren in `C:\Program Files\Tesseract-OCR\`
  - Tijdens installatie: vink "Nederlandse taaldata" aan
  - Zorg dat `tesseract.exe` in je PATH staat

- **ffmpeg** voor video-thumbnails en HEIC-ondersteuning
  - Download: https://www.gyan.dev/ffmpeg/builds/ (van "release builds")
  - Pak uit naar bv. `C:\ffmpeg\`
  - Voeg `C:\ffmpeg\bin` toe aan je PATH-omgevingsvariabele

- **NVIDIA GPU + CUDA** (optioneel, voor snelle AI-inferentie)
  - CUDA Toolkit 11.8 of nieuwer
  - NVIDIA-stuurprogramma 522+ 

---

## 2. De applicatie installeren

### Optie A: Kant-en-klare installer (aanbevolen voor eindgebruikers)

1. Download `Realisation-Manager-AI_1.0.0_x64.msi` (of `.exe`) van de releases-pagina
2. Dubbelklik het bestand
3. Volg de wizard
4. De applicatie wordt geinstalleerd in `C:\Program Files\Realisation Manager AI\`
5. Start via Start-menu → "Realisation Manager AI"

### Optie B: Zelf compileren vanaf broncode

Zie [DEVELOPER.md](./DEVELOPER.md) voor gedetailleerde instructies.

Korte versie:
```powershell
# Vereist: Rust 1.75+, Node.js 20+, Git
git clone <repository>
cd realisation-manager-ai
npm install
npm run tauri:build
# Installer verschijnt in src-tauri/target/release/bundle/
```

---

## 3. AI-modellen toevoegen (optioneel maar aanbevolen)

De applicatie werkt zonder AI-modellen (alleen heuristische classificatie + OCR), maar voor de volledige ervaring zijn twee ONNX-modellen nodig.

### Stap 1: Modelmap bepalen
- Open de applicatie
- Ga naar **Instellingen**
- Noteer het pad bij "Modelmap" (meestal `%APPDATA%\nl.marien.realisation-manager-ai\models\`)

### Stap 2: YOLOv8-model downloaden
1. Ga naar https://github.com/onnx/models/tree/main/validated/vision/object_detection_segmentation/yolov8
2. Download `yolov8n.onnx` ( klein en snel) of `yolov8s.onnx` (nauwkeuriger)
3. Hernoem naar `yolov8n.onnx` (als je `yolov8n` hebt gekozen)
4. Plaats in de modelmap

Alternatief: train een eigen YOLOv8-model op sanitair/CV-specifieke objecten met Ultralytics:
```bash
pip install ultralytics
yolo export model=yolov8n.pt format=onnx
```

### Stap 3: CLIP-model downloaden
1. Ga naar https://huggingface.co/openai/clip-vit-base-patch32
2. Exporteer naar ONNX (zie HuggingFace documentatie)
3. Sla op als `clip-vit-base.onnx` in de modelmap

### Stap 4: Herstart de applicatie
- Sluit Realisation Manager AI volledig
- Start opnieuw
- Ga naar **Instellingen** — alle drie de statusindicatoren zouden nu groen moeten zijn

---

## 4. Eerste gebruik

### Stap 1: Mapregels instellen (aanbevolen)
- Ga naar **Privacycentrum**
- Voeg regels toe voor mappen die altijd zakelijk of prive zijn:
  - `C:\Users\Naam\Pictures\Werk` → zakelijk
  - `C:\Users\Naam\Pictures\Familie` → prive
  - `C:\Users\Naam\Pictures\Vakantie` → prive
- Mapregels hebben altijd voorrang op AI

### Stap 2: Media importeren
- Ga naar **Mediabibliotheek**
- Klik op **Map importeren**
- Kies de iCloud-map (meestal `C:\Users\Naam\Pictures\iCloud Photos\`)
- Wacht tot de scan klaar is (duizenden foto's = enkele minuten)

### Stap 3: AI-analyse draaien
- Ga naar **AI-analyse**
- Klik op **Analyseer alles**
- Wacht tot de batch klaar is (per foto ±1 seconde op CPU, ±0.1 seconde op GPU)
- Controleer de classificaties in de Mediabibliotheek

### Stap 4: Projecten detecteren
- Ga naar **AI-analyse** → **Detecteer projecten**
- Of: **Projecten** → **Projecten detecteren**
- Bekijk de gedetecteerde projecten
- Goedkeuren, hernoemen, samenvoegen waar nodig

### Stap 5: SEO-content genereren
- Ga naar een goedgekeurd project
- Klik op **Genereer SEO**
- Bekijk en bewerk in **SEO-manager**
- Exporteer als Markdown wanneer klaar

### Stap 6: Privacymodus (optioneel)
- Klik op het oog-icoon rechtsboven om privacymodus in/uit te schakelen
- In privacymodus worden alle prive-media verborgen in elke weergave, export en AI-suggestie

---

## 5. Backup en gegevensbeheer

### App-gegevens
De volgende bestanden bevatten alle app-state:
- `%APPDATA%\nl.marien.realisation-manager-ai\realisation-manager.db` — SQLite-database
- `%APPDATA%\nl.marien.realisation-manager-ai\thumbnails\` — gegenereerde thumbnails
- `%APPDATA%\nl.marien.realisation-manager-ai\models\` — AI-modellen
- `%APPDATA%\nl.marien.realisation-manager-ai\logs\` — applicatielogs

### Backup maken
Kopieer de volledige map `%APPDATA%\nl.marien.realisation-manager-ai\` naar een veilige locatie.

### Migreren naar een andere pc
1. Installeer de app op de nieuwe pc
2. Start eenmaal en sluit direct weer
3. Kopieer de bestanden uit de backup naar de nieuwe `%APPDATA%` locatie
4. Start de app opnieuw

---

## 6. Probleemoplossing

### App start niet op
- Controleer of WebView2 geinstalleerd is
- Kijk in `%APPDATA%\nl.marien.realisation-manager-ai\logs\` voor foutmeldingen

### Thumbnails worden niet gegenereerd voor video's
- Installeer ffmpeg en voeg toe aan PATH
- Herstart de app

### OCR werkt niet
- Controleer of Tesseract geinstalleerd is: open cmd en typ `tesseract --version`
- Installeer Nederlandse taaldata: `tesseract --list-langs` moet `nld` bevatten

### AI-classificatie is erg traag
- Schakel GPU-versnelling in (Instellingen → AI → "GPU-versnelling gebruiken")
- Verminder batch-grootte als je weinig RAM hebt
- Gebruik `yolov8n.onnx` (n = nano) in plaats van grotere modellen

### Database is vergrendeld
- Sluit alle instanties van de app
- Verwijder `%APPDATA%\nl.marien.realisation-manager-ai\realisation-manager.db-wal` en `-shm`
- Start opnieuw

### Veel foto's worden als "unclassified" gemarkeerd
- Verlaag de confidence-drempel (Instellingen → AI → 0.45 in plaats van 0.55)
- Voeg meer mapregels toe voor specifieke mappen
- Controleer of objectdetectie en CLIP geladen zijn

---

## 7. Ondersteuning

Voor vragen of problemen:
1. Raadpleeg eerst deze INSTALL.md en DEVELOPER.md
2. Bekijk de logs in `%APPDATA%\nl.marien.realisation-manager-ai\logs\`
3. Neem contact op met de ontwikkelaar

---

**Veel plezier met Realisation Manager AI!**
