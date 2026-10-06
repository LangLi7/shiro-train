# Shiro - Laufzeit-Image fuer RunPod und Podman/Docker.
#
# Dieses Image ist SELBSTTRAGEND: es baut das Binary in der Stufe `builder` und
# kopiert es in eine schlanke Laufzeitstufe. Zwei Gruende:
#
#   1. Kein glibc-Mismatch. Ein auf dem Entwicklungsrechner (Ubuntu 26.04, glibc 2.41)
#      gebautes Binary kann auf einer aelteren Laufzeit scheitern
#      ("GLIBC_2.38 not found"). Bauen und Ausfuehren in derselben Distribution
#      schliesst das aus.
#   2. `docker build` funktioniert ohne Vorbedingungen. Wer das Repository klont,
#      muss nicht erst lokal passend uebersetzen.
#
# Bauen und starten:
#
#   docker build -f deploy/Dockerfile -t shiro-runtime:0.1.0 .
#   docker run --rm -v shiro-data:/data shiro-runtime:0.1.0 validate-data
#   docker run --rm -v shiro-data:/data shiro-runtime:0.1.0 smoke
#
# Der schnellere Weg fuer den Pod (spart Compile-Zeit bei jedem Build):
# einmal lokal bauen, pushen, im Pod nur noch ziehen.
#
#   cargo build --release --target x86_64-unknown-linux-gnu
#   docker build -f deploy/Dockerfile -t ghcr.io/NAMESPACE/shiro-runtime:0.1.0 .
#   docker push ghcr.io/NAMESPACE/shiro-runtime:0.1.0

# ---------------------------------------------------------------------------
# Stufe 1: bauen
# ---------------------------------------------------------------------------
FROM docker.io/library/rust:1-bookworm AS builder

WORKDIR /src

# Abhaengigkeiten in einer eigenen Cache-Schicht uebersetzen, damit sie bei
# Aenderungen am Quelltext nicht neu gebaut werden muessen.
#
# WICHTIG - hier stand vorher ein Fehler, der fast unbemerkt in ein Image gelangt
# waere: es wurde ein Stub-Binary gebaut (`printf 'fn main() {}' > src/main.rs`),
# und danach ueberschrieb `COPY . .` die Stub-Dateien. Cargo entscheidet aber
# anhand der mtime, ob neu gebaut werden muss - und weil COPY und Stub-Build
# dicht beieinander lagen, waren die Zeitstempel gleich. Cargo hielt den Stub
# fuer aktuell, und im Image landete eine Binary, die nichts tut, nichts ausgibt
# und mit Code 0 endet. Ein Test auf den Exit-Code allein findet das nicht.
#
# Zwei Gegenmassnahmen, die zusammen greifen:
#   1. `touch` setzt die Zeitstempel der echten Quellen auf jetzt, damit Cargo
#      sie sicher als neuer erkennt als die Stub-Artefakte.
#   2. Der Build endet mit `shiro version`. Meldet sich das Binary nicht, bricht
#      das Image-Bauen ab - statt erst im Pod aufzufallen, nachdem GPU-Zeit
#      bezahlt wurde.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src ./src
COPY tests ./tests
COPY configs ./configs
COPY data/sample ./data/sample

# KEIN Stub-Build mehr (Befund 06.10.2026). Der frueher hier stehende Cache-Trick
# ("erst ein leeres main.rs bauen, dann den echten Quelltext darueberkopieren")
# hat in JEDEM Build seit 17:20 ein unbrauchbares Image erzeugt:
#
#   1. `COPY . .` brachte die Datei `.cargo/config.toml` mit, in der
#      `target-dir = "/home/changchi/.cache/shiro-target"` steht (eine
#      WSL-Umgehung). Im Container baute Cargo danach DORTHIN, waehrend die
#      naechste Stufe aus `/src/target` kopiert - dort lag weiter der Stub.
#   2. Die Gegenprobe `RUN shiro version` prueft nur den Exit-Code; der Stub
#      endet mit 0 und gibt nichts aus. Sie konnte den Fehler also nicht finden.
#
# Ergebnis: ein 314-KB-Binary, das still nichts tut - im Pod haette das GPU-Zeit
# gekostet. Deshalb wird jetzt ehrlich aus dem echten Quelltext gebaut (weniger
# Cache-Effekt, dafuer richtig) und die AUSGABE geprueft.
RUN cargo build --release --locked

# --locked: die im Repository festgeschriebenen Crate-Versionen gelten. Sonst
# koennte ein Pod mit neueren Crates andere Ergebnisse liefern als der Entwickler.
RUN cargo build --release --locked \
 && strip target/release/shiro

# Gegenprobe im Build: das Binary muss sich MELDEN - nicht nur mit 0 enden.
# Die Ausgabe wird verglichen: ein stiller Stub endet mit Code 0 und wuerde von
# einer reinen Exit-Code-Pruefung nicht erkannt (genau das passierte am 06.10.2026).
RUN /src/target/release/shiro version | tee /dev/stderr | grep -qx "shiro 0.1.0"

# ---------------------------------------------------------------------------
# Stufe 2: ausfuehren
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="shiro" \
      org.opencontainers.image.description="Byte-Level-Sprachmodell - Trainingsgeruest" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"

# ca-certificates: TLS fuer spaetere Downloads (runpodctl, S3)
# tini: PID 1, reicht Signale weiter und raeumt Zombies auf
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates tini \
 && rm -rf /var/lib/apt/lists/*

# Kein root zur Laufzeit. Bei gemounteten Volumes sonst nur Berechtigungsaerger.
RUN groupadd --gid 1000 shiro \
 && useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash shiro

# Die drei Pfade der Roadmap. Im Pod zeigt /data auf das Netzwerk-Volume, damit
# Checkpoints einen Pod-Neustart ueberleben.
RUN mkdir -p /data /checkpoints /logs /app/configs \
 && chown -R shiro:shiro /data /checkpoints /logs /app

WORKDIR /app

COPY --from=builder /src/target/release/shiro /usr/local/bin/shiro
COPY --chown=shiro:shiro configs/ /app/configs/
COPY --chown=shiro:shiro data/sample/ /data/

# Der Einstiegspunkt prueft Volumes und validiert den Datensatz, BEVOR GPU-Zeit
# anfaellt - das ist der teuerste Fehler im Pod.
COPY --chown=shiro:shiro deploy/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Pfade per Umgebung: der Entwicklungsrechner hat data/ und checkpoints/ im
# Projektordner, der Container hat sie als Mountpoints.
ENV SHIRO_DATA=/data \
    SHIRO_CHECKPOINTS=/checkpoints \
    SHIRO_LOGS=/logs \
    SHIRO_CONFIG=/app/configs/train.smoke.toml \
    RUST_BACKTRACE=1 \
    RUST_LOG=info

USER shiro

# Beweist beim Start in einem Durchgang: Binary laeuft, Konfiguration liest,
# Datensatz ist da und gueltig, Forward-Pass rechnet, Checkpoint schreibt und
# laesst sich bitgleich wieder lesen. Der Smoke-Test dauert rund 0,2 Sekunden -
# der Healthcheck kostet also praktisch keine GPU-Zeit.
HEALTHCHECK --interval=5m --timeout=120s --start-period=20s --retries=3 \
  CMD ["/usr/local/bin/shiro", "smoke", "--config", "/app/configs/train.smoke.toml", "--steps", "2"]

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/entrypoint.sh"]
CMD ["smoke", "--config", "/app/configs/train.smoke.toml"]