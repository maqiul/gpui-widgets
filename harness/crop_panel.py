#!/usr/bin/env py
"""把浏览器整页截图裁成参照面板，尺寸与 Rust 侧窗口客户端区域一致。

面板在页面里的位置是固定的：`#root` 无内边距时面板左上角 = (16, 16) CSS px，
面板 420×688 CSS px，设备像素比 2（见 harness/antd_reference.html 的 PANEL）。

用法：
    py harness/crop_panel.py <整页截图.png> <输出.png> [x] [y] [w] [h] [scale]
"""
import os
import sys

from PIL import Image

def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    src, dst = sys.argv[1], sys.argv[2]
    x = float(sys.argv[3]) if len(sys.argv) > 3 else 16.0
    y = float(sys.argv[4]) if len(sys.argv) > 4 else 16.0
    w = float(sys.argv[5]) if len(sys.argv) > 5 else 420.0
    h = float(sys.argv[6]) if len(sys.argv) > 6 else 688.0
    scale = float(sys.argv[7]) if len(sys.argv) > 7 else 2.0

    img = Image.open(src).convert("RGB")
    box = tuple(int(round(v * scale)) for v in (x, y, x + w, y + h))
    if box[2] > img.size[0] or box[3] > img.size[1]:
        print(f"FAIL 面板超出整页图：crop={box} image={img.size}")
        return 1
    os.makedirs(os.path.dirname(os.path.abspath(dst)), exist_ok=True)
    crop = img.crop(box)
    crop.save(dst)
    print(f"saved {dst} {crop.size[0]}x{crop.size[1]}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
