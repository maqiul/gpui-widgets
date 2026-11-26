"""从 browser-use 的无障碍快照里提取机器生成的 Button 基准行。

用法: py extract_snapshot.py _snap_light.txt button_matrix_light.txt
快照里每行形如: `uid=1_442 StaticText "solid|default|normal|rgb(255, 255, 255)|..."`
"""

import codecs
import re
import sys

ROW = re.compile(r'StaticText "((?:[^"\\]|\\.)*)"')
KEEP = re.compile(r"^(?:GEO\||[a-z]+\|[a-z]+\|)")


def main() -> int:
    src_path, dst_path = sys.argv[1], sys.argv[2]
    with codecs.open(src_path, "r", "utf-8") as f:
        src = f.read()
    lines = []
    for node in ROW.findall(src):
        for line in codecs.decode(node.replace("\\\n", "\\n"), "unicode_escape").split("\n"):
            line = line.strip()
            if KEEP.match(line):
                lines.append(line)
    with codecs.open(dst_path, "w", "utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"{dst_path}: {len(lines)} lines")
    return 0


if __name__ == "__main__":
    sys.exit(main())
