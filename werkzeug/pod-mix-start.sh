# Startbefehl fuer den Mix-Lauf (Runde 370) - Vorlage fuer create-pod.
#
# Nach der Auswertung des Vielfalt-Laufs. Der Befehl macht alles in einem Zug:
# Code + Korpus + Config holen, vom 73%-Checkpoint fortsetzen, trainieren,
# und beide Ergebnisdateien direkt zu HuggingFace schieben. Der Upload haengt
# mit `&&` am Training: schlaegt das Training fehl, wird nichts hochgeladen.
#
# Ersetzt den frueheren Watcher, der nie ausloeste, weil `pgrep -f 'shiro train'`
# auch die eigene Kommandozeile traf (Runde 343).

set -e
pip install -q huggingface_hub

python3 -c '
from huggingface_hub import hf_hub_download
import os, shutil

files = {
    "shiro-linux-cuda": "/root/shiro",
    "train500-mix.toml": "/root/train500-mix.toml",
    "mix100k-shard.bin": "/root/mix-train/shard-0000.bin",
    "mix100k-manifest.json": "/root/mix-train/manifest.json",
}
os.makedirs("/root/mix-train", exist_ok=True)
for src, dst in files.items():
    shutil.copy(
        hf_hub_download(repo_id="ChangChi7/shiro-code", filename=src,
                        repo_type="model", token=os.environ["HF_TOKEN"]),
        dst,
    )
os.chmod("/root/shiro", 0o755)
print("READY", flush=True)
'

# Der Resume-State muss da sein, sonst wird nicht trainiert.
ls -l /workspace/500m-10500/ckpt/steps10500-step010500.state

/root/shiro train \
  --config /root/train500-mix.toml \
  --resume /workspace/500m-10500/ckpt/steps10500-step010500.state \
  --backend cuda > /root/train-mix.log 2>&1 \
  && echo TRAIN_OK

tail -5 /root/train-mix.log

# Nur bei erfolgreichem Training: beide Dateien hochladen.
python3 -c '
from huggingface_hub import HfApi
import os, glob
api = HfApi(token=os.environ["HF_TOKEN"])
for f in sorted(glob.glob("/workspace/500m-mix/ckpt/*")):
    api.upload_file(path_or_fileobj=f,
                    path_in_repo="mix500-" + os.path.basename(f),
                    repo_id="ChangChi7/shiro-checkpoints", repo_type="model")
    print("UP", os.path.basename(f), flush=True)
print("BEIDE_FERTIG", flush=True)
'

sleep infinity
