"""Sets base_path in Dioxus.toml (used by the Pages workflow). Reads the value from $BASE_PATH.

Replaces an existing base_path line (commented or not); otherwise adds it under [web.app], creating the
section if needed, so it works whatever the file looks like."""
import os, re
base = os.environ["BASE_PATH"]
text = open("Dioxus.toml").read()
line = f'base_path = "{base}"'
# 1. an existing base_path line (commented or not) is replaced
new, count = re.subn(r'(?m)^[ \t]*#?[ \t]*base_path[ \t]*=.*$', line, text)
if count == 0:
    # 2. otherwise add it under [web.app]
    if re.search(r'(?m)^\[web\.app\][ \t]*$', text):
        new = re.sub(r'(?m)^(\[web\.app\])[ \t]*$', lambda m: m.group(1) + "\n" + line, text, count=1)
    else:
        # 3. or create the section
        new = text.rstrip("\n") + f"\n\n[web.app]\n{line}\n"
open("Dioxus.toml", "w").write(new)
