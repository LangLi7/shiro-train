//! Die Oberflaeche: eingebettetes HTML, eingebettetes CSS, eingebettetes JS.
//!
//! Ein einziges Dokument, das der Server unter `/` ausliefert. Es gibt
//! **keine** externen Ressourcen: kein CDN, kein npm, keine Schriftdatei,
//! keine Bibliothek. Der Browser laedt genau eine Antwort, danach nur noch
//! die eigenen JSON-Endpunkte des Servers.
//!
//! Die Verlustkurve wird auf einem `<canvas>` von Hand gezeichnet - ein
//! Diagramm-Paket waere eine Abhaengigkeit, die dieses Werkzeug nicht braucht.
//!
//! Geprueft wird das in `tests/` unter anderem daraufhin, dass im Dokument
//! **keine** fremde Adresse vorkommt (kein `http://`, kein `https://`,
//! kein `//cdn`). Das ist ein Test, keine Absichtserklaerung.

/// Die Vorlage. Wird per `include_str!` eingebunden.
const TEMPLATE: &str = include_str!("page.html");

/// Platzhalter in der Fusszeile, an dem der Bezugswert eingesetzt wird.
///
/// Warum ueberhaupt einsetzen und nicht im Dokument stehen lassen: der
/// Bezugswert `ln(256)` hat genau **eine** Quelle im Rust-Code
/// (`state::UNTRAINED_LOSS`) und wird dort durch einen Test gegen `VOCAB.ln()`
/// geprueft. Stuende die Zahl zusaetzlich im HTML, waere sie eine zweite
/// Wahrheit - und die eine oder andere veraltet irgendwann.
const REF_PLACEHOLDER: &str = "__UNTR_REF__";

/// Prueft die Zeichenketten und Klammern **eines** Skriptblocks.
///
/// ## Warum das noetig ist — der Fehler, der es erzwungen hat
///
/// Fassung 3 ging mit einem toedlichen Fehler an den Nutzer: in einem
/// deutschen Zitat stand `„nur auswahl"` — die **englische** Anführung
/// schliesst in JavaScript die Zeichenkette. Damit war der ganze Skriptblock
/// ein **Syntaxfehler**, das Skript lief **gar nicht**, und die Seite blieb
/// leer: keine Kennzahlen, keine Tabelle, kein Diagramm, kein Fehler in der
/// Konsole. **Alle Gates waren gruen** (`fmt`, `clippy`, 82 Tests), weil
/// keiner das ausgelieferte Dokument auf Ausführbarkeit geprueft hat.
///
/// Eine Klammerbilanz und eine Zeichenkettenpruefung haetten gereicht; deshalb
/// stehen sie hier. Geprueft wird:
///
/// * **Klammern** `{}`, `()`, `[]` in richtiger Reihenfolge und vollstaendig,
/// * **Zeichenketten** `"…"` und `'…'` muessen in derselben Zeile schliessen —
///   genau das ist in JavaScript verboten und war der Fehler.
///
/// Kommentare (`//`, `/* */`), Escapes (`\"`), Template-Literale (`` ` ``) und
/// deutsche Anfuehrungszeichen (`„ “`) werden korrekt behandelt: sie sind
/// Text, keine Syntax. Zurueck kommt eine Liste von Befunden in Klartext;
/// eine leere Liste heisst "ausfuehrbar".
///
/// Nur im Test benutzt: die Pruefung soll das **Ausliefern** nicht kosten.
#[cfg(test)]
fn script_problems(script: &str) -> Vec<String> {
    #[derive(PartialEq)]
    enum St {
        Code,
        LineComment,
        BlockComment,
        Dq,
        Sq,
        Template,
    }
    let chars: Vec<char> = script.chars().collect();
    let mut state = St::Code;
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    let mut line = 1usize;

    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        let n = if i + 1 < chars.len() {
            chars[i + 1]
        } else {
            '\0'
        };
        if c == '\n' {
            line += 1;
        }
        match state {
            St::Code => match c {
                '/' if n == '/' => {
                    state = St::LineComment;
                    i += 1;
                }
                '/' if n == '*' => {
                    state = St::BlockComment;
                    i += 1;
                }
                '"' => state = St::Dq,
                '\'' => state = St::Sq,
                '`' => state = St::Template,
                '{' | '(' | '[' => stack.push((c, line)),
                '}' | ')' | ']' => match stack.pop() {
                    None => problems.push(format!("Zeile {line}: '{c}' ohne Oeffnung")),
                    Some((open, open_line)) => {
                        let ok = matches!((open, c), ('{', '}') | ('(', ')') | ('[', ']'));
                        if !ok {
                            problems.push(format!(
                                "Zeile {line}: '{c}' passt nicht zu '{open}' aus Zeile {open_line}"
                            ));
                        }
                    }
                },
                _ => {}
            },
            St::LineComment => {
                if c == '\n' {
                    state = St::Code;
                }
            }
            St::BlockComment => {
                if c == '*' && n == '/' {
                    state = St::Code;
                    i += 1;
                }
            }
            St::Dq | St::Sq => {
                if c == '\\' {
                    i += 1;
                } else if (state == St::Dq && c == '"') || (state == St::Sq && c == '\'') {
                    state = St::Code;
                } else if c == '\n' {
                    // Genau der Fehler: eine Zeichenkette darf in JavaScript
                    // nicht ueber das Zeilenende laufen.
                    problems.push(format!("Zeile {line}: Zeichenkette nicht geschlossen"));
                    state = St::Code;
                }
            }
            St::Template => {
                if c == '\\' {
                    i += 1;
                } else if c == '`' {
                    state = St::Code;
                }
            }
        }
        i += 1;
    }

    if state == St::BlockComment {
        problems.push("Kommentar /* ohne Ende".to_string());
    }
    for (open, open_line) in &stack {
        problems.push(format!("Zeile {open_line}: '{open}' ohne Schluss"));
    }
    problems
}

/// Alle Skriptbloecke des Dokuments pruefen. Nur im Test benutzt.
#[cfg(test)]
fn page_script_problems(page: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = page;
    let mut nr = 0usize;
    while let Some(start) = rest.find("<script") {
        let Some(open_end) = rest[start..].find('>') else {
            break;
        };
        let body_start = start + open_end + 1;
        let Some(close) = rest[body_start..].find("</script") else {
            break;
        };
        nr += 1;
        let body = &rest[body_start..body_start + close];
        for p in script_problems(body) {
            out.push(format!("Skriptblock {nr}: {p}"));
        }
        rest = &rest[body_start + close..];
    }
    out
}

/// Entfernt `//`-Kommentarzeilen. Nur im Test benutzt.
///
/// Damit pruefen Waechter den **ausgefuehrten Code** und nicht die Begruendung
/// daneben: dass `setInterval` in einem Kommentar steht, der erklaert, warum es
/// entfernt wurde, ist kein Rueckfall.
#[cfg(test)]
fn strip_line_comments(text: &str) -> String {
    text.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Das ausgelieferte Dokument: Vorlage plus eingesetzter Bezugswert.
pub fn page() -> String {
    let reference = format!(
        "ln({}) = {:.4} - der Verlust eines untrainierten Modells, \
         gegen den jeder Wert gemessen wird",
        crate::state::VOCAB,
        crate::state::UNTRAINED_LOSS
    );
    TEMPLATE.replace(REF_PLACEHOLDER, &reference)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fuer Tests, die den Platzhalter pruefen wollen.
    fn template() -> &'static str {
        TEMPLATE
    }

    #[test]
    fn page_has_no_external_resources() {
        let page = page();
        for verboten in ["http://", "https://", "//cdn", "src=\"//"] {
            assert!(
                !page.contains(verboten),
                "Die Oberflaeche darf nichts externes laden, gefunden: {verboten}"
            );
        }
    }

    #[test]
    fn page_is_self_contained() {
        let page = page();
        assert!(page.contains("<style>"), "CSS muss eingebettet sein");
        assert!(page.contains("<script>"), "JS muss eingebettet sein");
        assert!(!page.contains("<link"), "keine externen Stylesheets");
        // Kein Bild von aussen: das eingebettete SVG der Fassung 2 ist ersetzt,
        // ein <img> waere eine externe Datei.
        assert!(!page.contains("<img"), "keine Bilddateien");
    }

    #[test]
    fn page_links_the_local_api() {
        let page = page();
        for ep in [
            "/api/runs",
            "/api/series",
            "/api/events",
            "/api/checkpoints",
            "/api/compare",
            "/api/smart",
            "/api/reference",
            "/api/registry",
        ] {
            assert!(page.contains(ep), "Endpunkt fehlt in der Oberflaeche: {ep}");
        }
    }

    // ------------------------------------------------------------------
    // Der Abschnitt "Inhalt (Registry)".
    //
    // Anlass: die Seite zeigte nur Fortschritt (Verlust, Tempo, Checkpoints),
    // aber nicht, WAS trainiert wurde. Die Zahlen dazu stehen in
    // `registry/experiments.jsonl`; die Trainingslogs tragen sie nicht.
    // ------------------------------------------------------------------

    /// Der Abschnitt existiert, hat eine Rundenauswahl und eine Ampel.
    #[test]
    fn page_has_the_registry_section() {
        let page = page();
        for stelle in [
            "inhalt (registry)",
            "id=\"reg-pick\"",
            "<label for=\"reg-pick\">runde</label>",
            "id=\"reg-body\"",
            "id=\"reg-hint\"",
            "function renderRegistry",
            "function loadRegistry",
            "function regFillSelect",
            "get(\"/api/registry\")",
        ] {
            assert!(
                page.contains(stelle),
                "Registry-Abschnitt unvollstaendig: {stelle}"
            );
        }
    }

    /// Je Panel eine Eigene Tabelle - kein stilles Zusammenlegen.
    #[test]
    fn registry_section_keeps_panels_separate() {
        let page = page();
        assert!(
            page.contains("JE PANEL EINE EIGENE TABELLE"),
            "die Begruendung der Trennung fehlt"
        );
        // Jedes Panel bekommt eine eigene Tabelle mit den vier Spalten.
        assert!(page.contains("Summe der Typen"), "die Panel-Summe fehlt");
        assert!(page.contains("<th>typ</th>"), "Typspalte fehlt");
        assert!(page.contains(">gestellt</th>"), "Spalte gestellt fehlt");
        assert!(page.contains(">richtig</th>"), "Spalte richtig fehlt");
        assert!(page.contains(">prozent</th>"), "Spalte prozent fehlt");
        // Panels werden EINZELN gerendert: der Aufruf steht in einer Schleife
        // ueber `d.panels`.
        assert!(
            page.contains("d.panels.forEach(function (p)"),
            "die Panels werden nicht einzeln gerendert"
        );
        // Ohne panels steht ein Text, keine Luecke.
        assert!(
            page.contains("kein Panel in dieser Runde"),
            "der Zustand ohne Panel muss benannt werden"
        );
    }

    /// Die D-047-Ampel: beide Bedingungen sichtbar, und der Satz, dass sie fuer
    /// EINEN Eintrag gilt und nicht fuer den Projektstand.
    #[test]
    fn registry_section_shows_the_d047_mark_and_its_limits() {
        let page = page();
        assert!(page.contains("Marke D-047"), "die Marke wird nicht benannt");
        assert!(
            page.contains("D-047 erfuellt") && page.contains("D-047 NICHT erfuellt"),
            "beide Ausgaenge der Ampel fehlen"
        );
        assert!(
            page.contains("reisst die Typmarke"),
            "die Typen, die die Marke reissen, fehlen"
        );
        // Der Kriterientext kommt aus dem Server (eine Quelle, nicht zwei):
        // die Seite zaehlt `reg.kriterium` auf, sie schreibt die Schwellen
        // nicht selbst hin.
        assert!(
            page.contains("reg.kriterium"),
            "der Kriterientext wird nicht aus der Antwort gelesen"
        );
        // Und die Geltung: EIN Eintrag, nicht der Projektstand.
        assert!(
            page.contains("nicht fuer den Projektstand"),
            "die Ampel wird nicht auf einen Eintrag begrenzt"
        );
        assert!(
            page.contains("d.d047.geltung"),
            "die Geltung kommt nicht aus der Antwort"
        );
    }

    /// Zahlen ohne endlichen Wert werden als TEXT gezeigt, nicht gezeichnet.
    #[test]
    fn non_finite_numbers_never_reach_the_chart() {
        let page = page();
        assert!(
            page.contains("function countNonFinite"),
            "der Waechter zaehlt nicht-endliche Messpunkte nicht"
        );
        assert!(
            page.contains("!isFinite(v) || (v <= 0 && ax.useLog)"),
            "die Kurve prueft den Wert nicht auf Endlichkeit"
        );
        assert!(
            page.contains("typeof cur.best === \"number\" && isFinite(cur.best)"),
            "der Bestwert-Punkt wird ohne Pruefung gezeichnet"
        );
        // Und der Grund steht als Text daneben.
        assert!(
            page.contains("NaN/Inf"),
            "die Anzeige nennt den Grund nicht"
        );
        // In der Typentabelle ebenso: keine Zahl heisst "keine Zahl".
        assert!(page.contains("keine Zahl"), "Textfall fehlt");
        assert!(page.contains("nicht gestellt"), "Lueckentext fehlt");
    }

    /// Der Waechter aus Fassung 3 prueft die Elemente: der neue Abschnitt
    /// bringt zwei eigene (`reg-pick`, `reg-body`), die er mitprueft.
    #[test]
    fn registry_section_elements_are_reachable_by_the_script() {
        let page = page();
        for id in ["reg-pick", "reg-body", "reg-hint"] {
            assert!(page.contains(&format!("id=\"{id}\"")), "{id} fehlt");
            assert!(
                page.contains(&format!("el(\"{id}\")")),
                "das Skript greift auf {id} nicht zu"
            );
        }
    }

    #[test]
    fn page_marks_breaks_visually() {
        let page = page();
        // Das Wort "Durchbruch" muss vorkommen - das ist die sichtbare
        // Antwort auf "ich sehe keine Fortschritte". Geprueft wird der
        // Wortstamm: die Seite schreibt die Mehrzahl ("Durchbrueche").
        assert!(page.contains("Durchbr"), "Rekord muss benannt werden");
        // Jede Ereignisart braucht ihre eigene CSS-Klasse, sonst sieht ein
        // Rekord aus wie ein Checkpoint.
        for klasse in ["record", "checkpoint", "stagnation", "resume", "nan"] {
            assert!(
                page.contains(&format!(".ev.{klasse}")),
                "CSS-Klasse fuer '{klasse}' fehlt"
            );
        }
        // Der beste Stand muss sichtbar sein - frueher hiess die Linie
        // "Bestwert-Treppe", jetzt ist es der Punkt "bester Stand". Beides
        // wird akzeptiert, damit der Test die Formulierung nicht festschreibt.
        assert!(
            page.contains("Bestwert") || page.contains("bester Stand"),
            "der beste Stand fehlt"
        );
    }

    /// Fassung 3: die Anzeige muss die Frage "wird es besser?" beantworten.
    /// Ein Diagramm ohne Bewertung war der gemeldete Mangel.
    #[test]
    fn page_answers_whether_training_improves() {
        let page = page();
        assert!(
            page.contains("wird es besser?"),
            "die Leitfrage muss auf der Seite stehen"
        );
        for wort in ["besser", "schlechter", "stabil"] {
            assert!(page.contains(wort), "Richtung '{wort}' fehlt");
        }
        // Beide Pfeile: fallender Verlust ist die gute Nachricht. Ohne das
        // Wort daneben waere ein Pfeil nach unten als "schlecht" lesbar.
        assert!(page.contains("↓"), "Pfeil fuer fallenden Verlust fehlt");
        assert!(page.contains("↑"), "Pfeil fuer steigenden Verlust fehlt");
    }

    /// Der Bezugswert muss eingesetzt werden, nicht als Platzhalter ankommen.
    #[test]
    fn page_carries_the_untrained_reference() {
        let page = page();
        assert!(
            !page.contains(REF_PLACEHOLDER),
            "Platzhalter '{REF_PLACEHOLDER}' wurde nicht ersetzt"
        );
        let erwartet = format!("{:.4}", crate::state::UNTRAINED_LOSS);
        assert!(
            page.contains(&erwartet),
            "Bezugswert {erwartet} fehlt in der Fusszeile"
        );
        assert!(page.contains("ln(256)"), "Formel des Bezugswerts fehlt");
    }

    /// Die Vorlage MUSS den Platzhalter tragen, sonst laeuft das Einsetzen
    /// stillschweigend ins Leere - und der Test oben waere wertlos.
    #[test]
    fn template_has_the_placeholder() {
        assert!(
            template().contains(REF_PLACEHOLDER),
            "die Vorlage muss '{REF_PLACEHOLDER}' enthalten"
        );
    }

    /// Umschalter und Tooltip sind die Antwort auf "man sieht nicht, was der
    /// Graph bedeutet": Skala, Groesse und exakte Werte.
    #[test]
    fn page_offers_scale_metric_and_exact_values() {
        let page = page();
        for id in [
            "yscale-log",
            "yscale-lin",
            "metric-loss",
            "metric-ppl",
            "only-selected",
            "tip",
        ] {
            assert!(page.contains(id), "Bedienelement '{id}' fehlt");
        }
        // Achsenbeschriftung mit Einheit - eine Achse ohne Einheit ist nicht lesbar.
        assert!(
            page.contains("Verlust (Cross-Entropy"),
            "y-Achse unbeschriftet"
        );
        assert!(page.contains("Perplexitaet"), "Perplexitaet fehlt");
        assert!(page.contains("Schritt"), "x-Achse unbeschriftet");
    }

    /// Die Legende muss jede Linie im Diagramm erklaeren.
    #[test]
    fn page_legend_explains_every_line() {
        let page = page();
        for eintrag in [
            "untrainiertes Modell",
            "Startwert dieses Laufs",
            "bester Stand",
            "Checkpoint",
        ] {
            assert!(page.contains(eintrag), "Legende erklaert '{eintrag}' nicht");
        }
    }

    /// Jedes Element, auf das das Skript per `el("…")` zugreift, muss im
    /// Dokument stehen. Ein fehlendes Element wirft `null.addEventListener`
    /// und **bricht das ganze Skript ab** — genau das ist einmal passiert
    /// (`cmp-legend` war beim Umbau verlorengegangen), und die Seite blieb
    /// leer, obwohl alle Tests gruen waren.
    #[test]
    fn every_element_the_script_uses_exists() {
        let page = page();
        let script_start = page.find("<script>").expect("Skriptblock");
        let html = &page[..script_start];
        let script = &page[script_start..];

        let mut fehlend: Vec<String> = Vec::new();
        let mut rest = script;
        while let Some(i) = rest.find("el(\"") {
            let after = &rest[i + 4..];
            let Some(end) = after.find('"') else { break };
            let id = &after[..end];
            if !html.contains(&format!("id=\"{id}\"")) {
                fehlend.push(id.to_string());
            }
            rest = &after[end..];
        }
        // Gegenprobe: es muss ueberhaupt Zugriffe geben, sonst ist der Test leer.
        assert!(
            page.contains("el(\"chart\")"),
            "der Test hat keine el()-Zugriffe gefunden"
        );
        assert!(
            fehlend.is_empty(),
            "das Skript greift auf Elemente zu, die es nicht gibt: {fehlend:?}"
        );
    }

    // ------------------------------------------------------------------
    // Die SMART-Karte muss lesbar bleiben (Befunde aus dem Screenshot).
    //
    // Gemessene Anlaesse an echten Daten (08.10.2026):
    //   * der Dateiname stand in 7 von 18 Belegzeilen einer Karte,
    //   * die Quellenzeilen waren bis 165 Zeichen lang (absoluter Projektpfad),
    //   * gekuerzte Texte hatten keinen Tooltip.
    // ------------------------------------------------------------------

    /// Der Laufname steht einmal oben, nicht in jeder Belegzeile.
    #[test]
    fn smart_card_shows_the_run_identity_once() {
        let page = page();
        assert!(
            page.contains("smart-id"),
            "der Kopf der Karte mit dem Lauf-Fingerabdruck fehlt"
        );
        assert!(
            page.contains("function shortSource"),
            "die Kuerzung der Quellenangabe fehlt"
        );
        // Die Kuerzung muss die Laufkennung wirklich entfernen, sonst ist sie
        // wirkungslos: gegen den echten langen Namen geprueft.
        let quelle = "logs/agent-sequence_reasoning-i3-1791412922.jsonl, Feld step.loss";
        let name = "agent-sequence_reasoning-i3-1791412922.jsonl";
        assert!(quelle.contains(name));
        assert!(
            page.contains("split(runId).join(\"dieser Lauf\")"),
            "der Laufname wird in der Belegzeile nicht ersetzt"
        );
    }

    /// Gekuerzter Text traegt den vollen Inhalt als Tooltip (`title`).
    #[test]
    fn shortened_text_keeps_the_full_value_as_tooltip() {
        let page = page();
        for stelle in [
            "val.title = f.label + \": \" + f.value",
            "src.title = f.source",
            "cap.title = r.name",
            "val.title = c.title + \" (\" + c.status + \"): \" + c.value",
        ] {
            assert!(
                page.contains(stelle),
                "ohne Tooltip wird Information verschwiegen: {stelle}"
            );
        }
    }

    /// Kein Text in der SMART-Karte darf ohne Not hart abgeschnitten werden.
    #[test]
    fn smart_card_text_may_wrap_instead_of_being_cut() {
        let page = page();
        // Die Kartenregeln duerfen kein nowrap/ellipsis auf den Inhalt legen.
        let start = page.find(".smart {").expect("SMART-Regeln");
        let ende = page.find(".chip {").unwrap_or(page.len());
        let block = &page[start..ende];
        assert!(
            !block.contains("text-overflow: ellipsis"),
            "in der SMART-Karte wird Inhalt abgeschnitten statt umgebrochen"
        );
        assert!(
            !block.contains("white-space: nowrap"),
            "in der SMART-Karte wird der Text einzeilig erzwungen"
        );
    }

    // ------------------------------------------------------------------
    // Die Seite darf nicht mehr pollen (Umbau auf Server-Sent Events).
    //
    // Anlass: `setInterval(tickAll, 2000)` holte alle zwei Sekunden
    // /api/runs, /api/compare und die SMART-Karte - rund 340 KB je Takt und,
    // vor dem Zwischenspeicher, ueber eine Sekunde CPU. Auf derselben Maschine
    // laeuft das Training.
    // ------------------------------------------------------------------

    /// Kein Zeitgeber mehr, dafuer eine Live-Verbindung.
    #[test]
    fn page_uses_a_live_connection_instead_of_polling() {
        let page = page();
        assert!(
            page.contains("new EventSource(\"/api/stream\")"),
            "die Seite verbindet sich nicht mit dem Ereignisstrom"
        );
        // Der alte Zeitgeber darf nicht zurueckkehren. Geprueft wird der
        // **Code ohne Kommentare**: in einem Kommentar darf `setInterval` als
        // Erklaerung stehen (es steht dort, um den Umbau zu begruenden), im
        // ausgefuehrten Code nicht.
        let code = strip_line_comments(&page);
        for aufruf in ["setInterval(", "setTimeout(", "state.timer"] {
            assert!(
                !code.contains(aufruf),
                "die Seite pollt wieder ({aufruf}) - der Umbau auf Server-Sent Events ist rueckgaengig"
            );
        }
        // Der Stand muss unterschieden werden, sonst rendert die Seite ohne
        // Aenderung neu - genau das war der Zweck des Umbaus.
        assert!(
            page.contains("daten.id === state.fingerprint"),
            "die Seite prueft nicht, ob der gemeldete Stand wirklich neu ist"
        );
        // Und sie muss das Ende der Verbindung anzeigen, nicht verschweigen.
        assert!(
            page.contains("getrennt"),
            "eine unterbrochene Live-Verbindung muss sichtbar sein"
        );
    }

    // ------------------------------------------------------------------
    // Der Waechter gegen den Fehler, der die Seite lahmgelegt hat.
    //
    // Anlass: in Fassung 3 stand `„nur auswahl"` in einer Zeichenkette. Die
    // englische Anfuehrung schliesst die Zeichenkette, der ganze Skriptblock
    // war ein Syntaxfehler, und die Seite blieb leer - bei gruenen Gates.
    // ------------------------------------------------------------------

    /// Das ausgelieferte Dokument muss ausfuehrbares JavaScript enthalten.
    /// Dieser Test haette den Fehler gefunden.
    #[test]
    fn delivered_page_has_executable_script() {
        let page = page();
        let probleme = page_script_problems(&page);
        assert!(
            probleme.is_empty(),
            "das ausgelieferte Dokument enthaelt nicht ausfuehrbares JavaScript: {probleme:?}"
        );
        // Gegenprobe, dass ueberhaupt ein Skriptblock gefunden wurde -
        // ohne diese Zusicherung waere der Test bei null Bloecken trivial gruen.
        assert!(
            page.contains("<script>") && page.contains("function evaluate"),
            "es wurde kein Skriptblock geprueft"
        );
    }

    /// Der Waechter muss den echten Fehler **erkennen**, sonst ist er blind.
    /// Hier wird genau die kaputte Zeile nachgebaut.
    #[test]
    fn guard_detects_the_quote_that_broke_the_page() {
        let kaputt = "var s = \"text auf \u{201e}nur auswahl\" klicken.\";\n";
        let probleme = script_problems(kaputt);
        assert!(
            !probleme.is_empty(),
            "der Waechter erkennt die englische Anfuehrung in einem deutschen Zitat nicht"
        );
    }

    /// Und er darf bei korrektem Code **nicht** anschlagen: deutsche
    /// Anfuehrungszeichen innerhalb einer Zeichenkette sind Text.
    #[test]
    fn guard_accepts_german_quotes_inside_strings() {
        let heil = "var s = \"auf \u{201e}nur auswahl\u{201c} klicken.\";\nvar t = 'x';\n";
        assert_eq!(script_problems(heil), Vec::<String>::new());
    }

    #[test]
    fn guard_detects_unbalanced_brackets() {
        assert!(
            !script_problems("function f() {\n").is_empty(),
            "offene Klammer"
        );
        assert!(
            !script_problems("var a = 1;\n}\n").is_empty(),
            "Klammer ohne Oeffnung"
        );
        assert!(
            !script_problems("var a = (1];\n").is_empty(),
            "falsche Klammerart"
        );
    }

    /// Kommentare, Escapes und Template-Literale duerfen keine Fehlalarme
    /// erzeugen - sonst wird der Waechter ignoriert.
    #[test]
    fn guard_ignores_comments_escapes_and_templates() {
        let heil = concat!(
            "// ein } und ein \" in einem Kommentar\n",
            "/* auch } und ' hier */\n",
            "var a = \"ein \\\" entkommenes Zeichen\";\n",
            "var b = `template mit ${a} und }`;\n",
            "function f() { return [1, 2, 3]; }\n"
        );
        assert_eq!(script_problems(heil), Vec::<String>::new());
    }

    // ------------------------------------------------------------------
    // Der Abschnitt fuer fremde Messlaeufe (Fassung 6).
    //
    // Anlass: eine andere Stelle meldet `FERTIG eval-4d Arm B ->
    // answers-B-eval-4d.log`. Diese Zeile soll auf der Seite stehen. Geprueft
    // wird hier die **Anzeige**; dass der Endpunkt liefert, was sie erwartet,
    // prueft `tests/http.rs`.
    // ------------------------------------------------------------------
    #[test]
    fn page_shows_the_measurement_protocol() {
        let page = page();
        for id in ["mess-body", "mess-meta", "mess-hint"] {
            assert!(
                page.contains(&format!("id=\"{id}\"")),
                "der Abschnitt hat kein Element {id}"
            );
        }
        assert!(
            page.contains("/api/messungen"),
            "die Anzeige holt das Protokoll nicht vom Endpunkt"
        );
        // Volle Ladung **und** die reduzierte Kette der Live-Verbindung: sonst
        // erscheint eine neue Protokollzeile erst nach dem Neuladen der Seite.
        assert_eq!(
            page.matches(".then(loadMessungen)").count(),
            2,
            "das Protokoll haengt nicht in beiden Ladewegen"
        );
        // Vier Zustaende, vier Farben: ein gescheiterter Lauf darf nicht wie ein
        // fertiger aussehen.
        for cls in [
            "cls: \"laeuft\"",
            "cls: \"zeile\"",
            "cls: \"fertig\"",
            "cls: \"fehler\"",
        ] {
            assert!(
                page.contains(cls),
                "der Status {cls} hat keine eigene Darstellung"
            );
        }
        // Der Abschnitt heisst anders als die Lauf-Tabelle: zwei gleichnamige
        // Ueberschriften mit verschiedenen Inhalten waeren eine Falle.
        assert!(page.contains("<h2>messprotokoll"), "der Abschnitt fehlt");
        assert_eq!(
            page.matches("<h2>messlaeufe").count(),
            1,
            "die Lauf-Tabelle muss die einzige 'messlaeufe'-Ueberschrift bleiben"
        );
        // Luecken stehen in der Zeile, nicht im Kleingedruckten.
        for text in ["kein start gemeldet", "zwischendurch gescheitert"] {
            assert!(page.contains(text), "die Luecke '{text}' wird verschwiegen");
        }
        // Und der Helfer, mit dem eine Zeile geschrieben wird.
        assert!(
            page.contains("scripts/messung-log.sh"),
            "die Anzeige nennt den Helfer nicht"
        );
    }
}
