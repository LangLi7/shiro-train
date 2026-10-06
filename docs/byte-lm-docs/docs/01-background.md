# Hintergrund: Modellarten und Byte-Modelle

## Arten von KI-Modellen (grob nach Daten und Aufbau)

| Art | Beispiele | Eingabe | Aufgabe |
|---|---|---|---|
| **LLM** | GPT, Claude, Llama | Text-**Tokens** (Wortstücke) | nächstes Token vorhersagen |
| **Zeitreihen-Modelle** | TimesFM, Chronos, Kronos | Zahlenreihen (Kronos: Kerzen/OHLCV) | zukünftige Werte prognostizieren |
| **Byte-Modelle** | ByT5, MegaByte, Byte Latent Transformer, MambaByte, bGPT | rohe **Bytes** | nächstes Byte vorhersagen |
| **Bild/Video** | Diffusion, ViT | Pixel bzw. Patches | erzeugen, erkennen |
| **Audio/Sprache** | Whisper, TTS | Wellenform bzw. Spektrogramm | erkennen, erzeugen |
| **Multimodal** | verschiedene | mehrere Datenarten | kombiniert |
| **Weitere** | Embedding-Modelle, RL-Agents, GNNs, TabPFN | je nach Typ | je nach Typ |

Die Grenzen sind fließend. **Byte-Modell ist kein eigener Modelltyp, sondern eine Frage der Tokenisierung:** Ein Transformer kann Tokens oder Bytes verarbeiten. Viele Zeitreihenmodelle sind im Kern ebenfalls Transformer, nur mit anderen Daten und anderem Patching.

## Token gegen Byte

| | Token-LLM | Byte-Modell |
|---|---|---|
| Eingabe | Wortstücke, z. B. `["Hall", "o", " Welt"]` als IDs | einzelne Bytes, `"Hallo"` = `72 97 108 108 111` |
| Vokabular | 50.000 bis 200.000 Einträge, feste Tokenizer-Datei | **256** (plus Spezialbytes) |
| Tokenizer nötig | ja | **nein** |
| Sequenzlänge | kurz | ca. **4-5x länger** |
| Buchstaben, Zahlen | Buchstaben im Token nicht direkt sichtbar, Zahlen werden willkürlich zerstückelt (`12345` → `123` + `45`) | jede Ziffer und jeder Buchstabe einzeln sichtbar |
| Seltene Wörter, Tippfehler, andere Sprachen | zerfallen in viele Teile | werden gleich behandelt |
| Dateiformate | brauchen eigene Encoder | alles, was Bytes ist, geht direkt (PDF, Bilder, Audio, Maschinencode) |
| Kosten | Standard, viele vortrainierte Modelle | höher, Attention wächst quadratisch mit der Länge, kein vortrainiertes Modell wiederverwendbar |

Das Modell sieht Text trotzdem nur als Zahlen. Die Rückumwandlung ist verlustfrei (`String::from_utf8_lossy`), die Schnittstelle bleibt normale Sprache. Ungültige UTF-8-Folgen kann das Modell selten erzeugen, `from_utf8_lossy` ersetzt sie.

## Kosten pro Sprache (UTF-8)

| Schrift | Bytes pro Zeichen |
|---|---|
| Lateinisch (Englisch, Deutsch) | 1 (Umlaute 2) |
| Kyrillisch, Arabisch, Hebräisch, Griechisch | 2 |
| Chinesisch, Japanisch, Koreanisch | meist 3 |
| Emojis | 4 |

Es gibt keine harte Sprachgrenze. Ein chinesischer Satz belegt aber ca. dreimal so viel Kontext wie die Zeichenzahl vermuten lässt. Größerer Engpass bei kleinen Modellen ist ohnehin die Datenmenge pro Sprache.

## Bekannte Byte-Ansätze

| Ansatz | Idee |
|---|---|
| **ByT5** | T5-Encoder-Decoder direkt auf Bytes |
| **MegaByte** | Bytes zu Patches zusammenfassen, globales Modell über Patches plus lokales Modell innerhalb des Patches |
| **Byte Latent Transformer (Meta)** | Patches **dynamisch** nach Schwierigkeit (Entropie): einfache Stellen große Patches, schwierige kleine. Hielt bei gleichem Rechenaufwand mit Token-Modellen mit. |
| **MambaByte** | State-Space-Modell statt Attention, wächst linear mit der Länge |
| **bGPT** | Text, Bilder und Audio als reine Bytes gelernt |

## Was ein Byte-Modell ermöglicht

- Beliebige Dateiformate direkt verarbeiten, **Binär- und Malware-Analyse**, Dateiformat-Erkennung
- **Kompression** (Bytes vorhersagen ist im Prinzip Daten komprimieren)
- Robustheit bei Tippfehlern, seltenen Sprachen, Code und Zahlen
- Ein Modell für alle Datenarten

**Impact:** Eher evolutionär als revolutionär. Der Tokenizer entfällt (Fehlerquelle, Sprachbias, Sonderfälle). Der Durchbruch im ganz großen Maßstab fehlt noch. Realistisch der größte Hebel: Binärdaten, Sicherheit, Kompression, Multimodalität.

## Warum Rust statt Python

- Beim **Training** ist Python selten der Flaschenhals, weil es nur GPU-Kernels aufruft. Der Gewinn durch Rust ist dort klein.
- Rust lohnt sich bei **Inferenz** (kein Python-Runtime, kleiner Footprint, einfaches Deployment), der **Datenpipeline** (Generator, Preprocessing) und für **eigene Kernels** oder den Einsatz auf Mac und Embedded.
- Nachteil: weniger fertige Architekturen, weniger Doku, mehr Eigenbau. Du hast dich bewusst dafür entschieden.
