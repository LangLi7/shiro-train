# DeepSeek-Richtlinien (Referenzkopien)

Dieser Ordner enthält Referenzkopien von DeepSeek-Richtlinien, die für dieses
Projekt (Nutzung der DeepSeek-API als Teacher/Verifier) relevant sind — jeweils
als englisches Original und als deutsche Übersetzung.

Alle Dateien liegen direkt in `docs/`.

## Inhalt

| Dokument | Englisch | Deutsch |
| --- | --- | --- |
| DeepSeek Open Platform Terms of Service | [`deepseek-open-platform-terms-of-service.en.md`](deepseek-open-platform-terms-of-service.en.md) | [`deepseek-open-platform-terms-of-service.de.md`](deepseek-open-platform-terms-of-service.de.md) |

Die Rohquelle der HTML-Fassung liegt unverändert daneben:
[`deepseek-open-platform-terms-of-service.html`](deepseek-open-platform-terms-of-service.html).

## Stand

| Dokument | Veröffentlichung | Inkrafttreten | Abgerufen | Quelle |
| --- | --- | --- | --- | --- |
| Open Platform Terms of Service | 22.04.2026 | 29.04.2026 | 06.10.2026, 17:43 MESZ | <https://cdn.deepseek.com/policies/en-US/deepseek-open-platform-terms-of-service.html> |

## Wichtige Punkte für dieses Projekt

- **Abschnitt 4.2 (3):** Eingaben und Ausgaben dürfen für ein breites Spektrum von
  Anwendungsfällen genutzt werden, ausdrücklich einschließlich **Training anderer
  Modelle (z. B. Modelldestillation)**. Das ist die vertragliche Grundlage für die
  Teacher/Verifier-Nutzung.
- **Abschnitt 4.2 (1)/(2):** Rechte an Eingaben bleiben beim Nutzer; Rechte an
  Ausgaben werden an den Nutzer übertragen.
- **Abschnitt 11.1:** Geänderte Bedingungen treten sieben (7) Tage nach
  Veröffentlichung in Kraft — der Stand oben ist daher regelmäßig zu prüfen.
- **Abschnitt 8.1:** Ausgaben sind KI-generiert; Endnutzern gegenüber ist das
  offenzulegen, und Ausgaben sind nicht als fachliche Beratung zu verwenden.
- **Abschnitt 7.3:** Haftungsobergrenze = tatsächlich verbrauchte Dienstentgelte
  der letzten zwölf Monate.

## Rechtlicher Hinweis

Rechtsverbindlich ist allein der englische Originaltext. Die deutschen Fassungen
dienen dem Verständnis und sind keine Rechtsberatung.

## Hinweis zur Ablage

Die Projektablage liegt auf einem exFAT-Datenträger. exFAT unterstützt keine
Hardlinks; Werkzeuge, die Dateien atomar über einen Hardlink an ihren Zielort
setzen, schlagen dort mit `EISDIR` fehl. Dateien deshalb direkt schreiben bzw.
kopieren.
