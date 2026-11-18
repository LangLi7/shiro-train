import json, sys, os, shutil
# Merge two corpora by copying shard files and rewriting the manifest entries.
# Streaming: only the small manifests are held, shard files are copied byte for byte.
srcs = sys.argv[1:-1]
dst = sys.argv[-1]
os.makedirs(dst, exist_ok=True)
tasks, shards = {}, []
idx = 0
for src in srcs:
    mpath = os.path.join(src, "manifest.json")
    with open(mpath, encoding="utf-8") as f:
        m = json.load(f)
    for k, v in m.get("tasks", {}).items():
        tasks[k] = tasks.get(k, 0) + v
    for sh in m["shards"]:
        name = f"merged-{idx:04d}.bin"
        shutil.copyfile(os.path.join(src, sh["path"]), os.path.join(dst, name))
        e = dict(sh); e["id"] = idx; e["path"] = name
        shards.append(e)
        idx += 1
out = {"format_version": 1, "vocab_size": 256, "byte_order": "little",
       "record_encoding": "raw-bytes-with-special-tokens", "seed": 0,
       "note": f"streaming merge of {len(srcs)} corpora",
       "tasks": tasks, "shards": shards}
with open(os.path.join(dst, "manifest.json"), "w", encoding="utf-8") as f:
    json.dump(out, f, indent=2)
print(f"{len(shards)} Shards, {sum(s['records'] for s in shards)} Records -> {dst}")
