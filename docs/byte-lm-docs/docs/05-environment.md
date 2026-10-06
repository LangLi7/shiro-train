# Schritt 0: Podman-Umgebung

Ziel: Reproduzierbare Umgebung, die lokal und auf RunPod gleich aufgebaut ist.

## Checkliste

- [ ] Podman installieren (rootless), Test mit `podman run --rm hello-world`
- [ ] `Containerfile` mit Rust-Toolchain schreiben (z. B. Basis-Image mit CUDA-Runtime für GPU-Builds, ein zweites CPU-Image für Entwicklung)
- [ ] Projektordner als Volume einbinden, `cargo`-Cache als eigenes Volume (schnellere Builds)
- [ ] `cargo build` und `cargo test` im Container laufen lassen
- [ ] Wenn eine NVIDIA-GPU vorhanden ist: NVIDIA Container Toolkit installieren, CDI-Spec erzeugen und GPU im Container testen
- [ ] Image bauen und in eine Registry pushen (z. B. GHCR oder Docker Hub), damit RunPod es als eigenes Template starten kann

## Beispiel (Entwurf, an dein System anpassen)

```dockerfile
# Containerfile (CPU-Entwicklung)
FROM docker.io/library/rust:1-bookworm
RUN rustup component add clippy rustfmt
WORKDIR /work
```

```bash
podman build -t byte-lm-dev -f Containerfile .
podman run --rm -it \
  -v "$PWD":/work:Z \
  -v byte-lm-cargo:/usr/local/cargo/registry \
  byte-lm-dev cargo test
```

GPU (nur mit NVIDIA-Hardware und Container Toolkit):

```bash
podman run --rm --device nvidia.com/gpu=all byte-lm-gpu nvidia-smi
```

## Hinweise

- Auf **macOS** läuft Podman in einer VM (`podman machine`). Ein Container dort kann die Apple-GPU (Metal) nicht nutzen. Für den **Mac Studio (Hauptfokus)** heißt das: **Training und Inferenz laufen nativ** mit `cargo`, ohne Container. Podman bleibt für Linux, CUDA und RunPod und für reproduzierbare Builds.
- **RunPod** startet selbst Container-Images. Podman im Pod zu betreiben ist in der Regel nicht vorgesehen. Der Weg ist: Image lokal mit Podman bauen, pushen, als Template in RunPod verwenden.
- Rechenzeit sparen: Auf RunPod nur trainieren. Datenerzeugung und Verifier laufen auf CPU, auch lokal.
