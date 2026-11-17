//! Modellablage auf dem HuggingFace Hub (`shiro hub pull|push`).
//!
//! **Warum ein eigenes, optionales Feature** (`--features hub`): der Trainings-
//! und Messpfad dieses Projekts braucht keinen Netzwerk-Stack. Wer Modelle
//! zwischen fluechtigen Pods sichern oder veroeffentlichen will, baut mit dem
//! Feature; alle anderen tragen weder `reqwest` noch `tokio` mit.
//!
//! **Was hier NICHT passiert:** Trainingsstaende (`.state`) werden nicht
//! automatisch hochgeladen. Dieses Modul ist ein ausdruecklicher Aufruf, kein
//! Hintergrunddienst - ein Push, der waehrend des Trainings laeuft, wuerde bei
//! jedem Zwischenstand die Git-Historie des Repositories aufblaehen
//! (docs/modell-ablage-huggingface.md §5).
//!
//! **Token:** `HF_TOKEN` (ersatzweise `HUGGINGFACE_TOKEN`). Auf RunPod das
//! Secret `HuggingFace` als Umgebungsvariable `HF_TOKEN` mit dem Wert
//! `{{ RUNPOD_SECRET_HuggingFace }}` eintragen; RunPod ersetzt den Platzhalter
//! beim Start des Containers.

use crate::error::{Context, Result, anyhow, bail};
use std::path::PathBuf;

/// Auftrag fuer `shiro hub pull`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequest {
    /// Repository in der Form `owner/name`.
    pub repo: String,
    /// Zielordner; wird angelegt, wenn er fehlt.
    pub dir: PathBuf,
    /// Branch, Tag oder Commit; `None` = Hauptzweig.
    pub revision: Option<String>,
}

/// Auftrag fuer `shiro hub push`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushRequest {
    /// Repository in der Form `owner/name`; wird bei Bedarf angelegt.
    pub repo: String,
    /// Lokaler Ordner, dessen Inhalt hochgeladen wird.
    pub dir: PathBuf,
    /// Zielpfad im Repository; `None` = Wurzel.
    pub path_in_repo: Option<String>,
    /// Commit-Nachricht; `None` = aus Ordner und Datum abgeleitet.
    pub message: Option<String>,
    /// Neues Repository privat anlegen (Vorgabe: `true`).
    pub private: bool,
}

/// Liest den Token aus der Umgebung.
///
/// `hf-hub` liest `HF_TOKEN` selbst; diese Funktion dient nur der klaren
/// Fehlermeldung, bevor ein Upload in einen 401 laeuft.
fn token_from_env() -> Option<String> {
    // Die Pruefung auf leer muss IN den Closure: sonst gilt ein gesetztes, aber
    // leeres HF_TOKEN als Treffer, und HUGGINGFACE_TOKEN wird nie gelesen.
    ["HF_TOKEN", "HUGGINGFACE_TOKEN"]
        .iter()
        .find_map(|key| {
            std::env::var(key)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}

/// Der eine Pfad, unter dem `hf auth login` den Token ablegt.
///
/// Bewusst genau EIN Ort nach dokumentierter Reihenfolge (`HF_TOKEN_PATH`,
/// dann `HF_HOME`, dann der Cache-Standard des Systems). Kein Durchprobieren
/// mehrerer Wurzeln: sonst koennte ein Token aus einem fremden Verzeichnis
/// verwendet werden, das der Aufrufer nicht gemeint hat.
fn token_file_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("HF_TOKEN_PATH") {
        return Some(PathBuf::from(path));
    }
    if let Ok(hf_home) = std::env::var("HF_HOME") {
        return Some(PathBuf::from(hf_home).join("token"));
    }
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return Some(PathBuf::from(xdg).join("huggingface").join("token"));
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return Some(
            PathBuf::from(profile)
                .join(".cache")
                .join("huggingface")
                .join("token"),
        );
    }
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".cache").join("huggingface").join("token"))
}

/// Ermittelt den Token: Umgebung zuerst, dann der gespeicherte Login.
///
/// `hf-hub` liest von sich aus nur `HF_TOKEN`; ein mit `hf auth login`
/// gespeicherter Token wuerde sonst ignoriert und der Upload in einen 401 laufen.
/// Der Wert wird nie protokolliert und nie ausgegeben.
fn resolve_token() -> Option<String> {
    if let Some(token) = token_from_env() {
        return Some(token);
    }
    token_file_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|content| content.trim().to_string())
        .filter(|token| !token.is_empty())
}

/// Steht ueberhaupt ein Token zur Verfuegung (Umgebung oder gespeicherter Login)?
fn token_available() -> bool {
    resolve_token().is_some()
}

/// Baut den Client mit dem gefundenen Token und uebersetzt Fehler in Klartext.
fn client() -> Result<hf_hub::HFClientSync> {
    let mut builder = hf_hub::HFClient::builder();
    if let Some(token) = resolve_token() {
        builder = builder.token(token);
    }
    let inner = builder
        .build()
        .map_err(|e| anyhow!("HuggingFace-Client nicht baubar ({e}). Netzwerk und Zertifikate pruefen."))?;
    hf_hub::HFClientSync::from_inner(inner)
        .map_err(|e| anyhow!("HuggingFace-Client nicht startbar ({e})"))
}

/// Teilt `owner/name` und weist auf das erwartete Format hin.
fn split(repo: &str) -> Result<(String, String)> {
    let (owner, name) = hf_hub::split_id(repo);
    if owner.is_empty() || name.is_empty() {
        bail!("Repository `{repo}` ist nicht in der Form `owner/name` angegeben");
    }
    Ok((owner.to_string(), name.to_string()))
}

/// Laedt ein Repository (oder nur die Auswahl) in einen lokalen Ordner.
///
/// Der Ordner wird direkt beschrieben (`local_dir`), nicht ueber den
/// HuggingFace-Cache: im Container soll der Cache nicht das Volume fuellen.
pub fn pull(req: &PullRequest) -> Result<()> {
    let (owner, name) = split(&req.repo)?;
    let repo = client()?.model(owner, name);

    println!("HuggingFace   : {}", req.repo);
    println!("Zielordner    : {}", req.dir.display());
    println!(
        "Revision      : {}",
        req.revision.as_deref().unwrap_or("Hauptzweig")
    );
    println!(
        "Token         : {}",
        if token_available() {
            "gesetzt"
        } else {
            "keiner (nur oeffentliche Repositories)"
        }
    );

    std::fs::create_dir_all(&req.dir).with_context(|| {
        format!(
            "Zielordner {} laesst sich nicht anlegen - auf RunPod liegt er auf dem Volume?",
            req.dir.display()
        )
    })?;

    // Typzustands-Builder: die Revision muss beim Aufruf schon feststehen.
    let downloaded = match &req.revision {
        Some(revision) => repo
            .snapshot_download()
            .local_dir(req.dir.clone())
            .revision(revision.clone())
            .send(),
        None => repo.snapshot_download().local_dir(req.dir.clone()).send(),
    };
    let path = downloaded.map_err(|e| anyhow!("Download fehlgeschlagen: {e}"))?;

    println!("Geladen       : {}", path.display());
    Ok(())
}

/// Laedt einen lokalen Ordner hoch und legt das Repository bei Bedarf an.
///
/// `exist_ok` haelt den Aufruf idempotent: ein zweiter Push auf dasselbe
/// Repository scheitert nicht am Anlegen, sondern committet die Aenderung.
pub fn push(req: &PushRequest) -> Result<()> {
    if !token_available() {
        bail!(
            "Kein HuggingFace-Token gefunden - ohne Write-Token kein Upload.\n\
             Lokal: `hf auth login` (legt ~/.cache/huggingface/token an).\n\
             Auf RunPod: Secret `HuggingFace` anlegen und die Umgebungsvariable\n\
             HF_TOKEN={{{{ RUNPOD_SECRET_HuggingFace }}}} setzen."
        );
    }
    if !req.dir.is_dir() {
        bail!("Ordner {} existiert nicht", req.dir.display());
    }
    let (owner, name) = split(&req.repo)?;
    let api = client()?;

    println!("HuggingFace   : {}", req.repo);
    println!("Quelle        : {}", req.dir.display());
    println!(
        "Zielpfad      : {}",
        req.path_in_repo.as_deref().unwrap_or("(Wurzel)")
    );

    api.create_repository()
        .repo_id(&req.repo)
        .repo_type(hf_hub::RepoTypeModel)
        .private(req.private)
        .exist_ok(true)
        .send()
        .map_err(|e| anyhow!("Repository {} nicht anlegbar: {e}", req.repo))?;

    // Der Builder ist typzustandsbehaftet: jede gesetzte Option aendert den Typ.
    // Deshalb einmal je Kombination aufrufen statt eine Variable umzubauen.
    let repo = api.model(owner, name);
    let upload = repo.upload_folder().folder_path(req.dir.clone());
    let result = match (&req.path_in_repo, &req.message) {
        (Some(path), Some(message)) => upload
            .path_in_repo(path.clone())
            .commit_message(message.clone())
            .send(),
        (Some(path), None) => upload.path_in_repo(path.clone()).send(),
        (None, Some(message)) => upload.commit_message(message.clone()).send(),
        (None, None) => upload.send(),
    };
    result.map_err(|e| anyhow!("Upload fehlgeschlagen: {e}"))?;

    println!("Hochgeladen   : {}", req.repo);
    Ok(())
}
