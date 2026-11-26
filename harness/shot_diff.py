#!/usr/bin/env py
"""控件面板截图逐组比对。

两侧面板同构（同一行顺序、同一 pad / 行带 / label 宽 / 行内 gap），于是第 i 行的 y 区间可直接算：
  pitch = 行高 40 + 行间隙 16 = 56        （CSS px，见 harness/rows.txt 与 panel_shot.rs）
  y0    = panel_pad + i * pitch           （CSS px）
再乘 device_scale 得到设备像素。逐行裁出两个 PNG 的同一段，比较像素差异比例。

默认只裁控件所在的窄格（CELL_X0 起、宽 56、高 40 CSS px）。用整行做分母时控件只占约 9% 像素，
实测 solid 与 filled 错配才 10.6%，阈值放行；同一组窄格像素差异是 2.5%，把 filled 的参照图换成
solid 的才升到 25.8%。而 1px 描边即使占满窄格也只有 6.5%（outlined 与 text 错配），
像素比例对"该有描边却没有"不敏感，所以每条带再比一次"离面板底色最远的众数颜色"
（描边带 + 内部底色带，容差每通道 6）。

antd 参考图由 Qoder 内置浏览器对 harness/antd_reference.html 的 `__panel` 模式整页截图，
再用 harness/crop_panel.py 裁出面板（browser MCP 的 take_screenshot 支持 filePath）；
Rust 侧由 examples/panel_shot.rs + harness/capture_rust.ps1 抓窗口客户端区域产出。

用法：
    py harness/shot_diff.py                 # 逐行比对 harness/baseline 与 harness/antd_shots 下所有图对
    py harness/shot_diff.py solid           # 只比文件名含 solid 的图对
环境变量：MAX_BAD_RATIO 覆盖像素阈值，INCLUDE_LABELS=1 改回整行（含标签列）比对。
"""
import os
import sys
from collections import Counter

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASE = os.path.join(ROOT, "harness", "baseline")
REF = os.path.join(ROOT, "harness", "antd_shots")

# 与 harness/antd_reference.html 的 PANEL / examples/panel_shot.rs 保持一致
PANEL_PAD = 16
ROW_PITCH = 56  # 行高 40 + 行间隙 16
ROW_HEIGHT = 40
PANEL_WIDTH = 420
LABEL_WIDTH = 140
CELL_GAP = 8
DEVICE_SCALE = 2
TOLERANCE = 15  # 每通道
# 控件盒最宽的一档（禁用态 49 CSS px）留一点余量给投影
CELL_WIDTH = 56
# 控件左上角在面板里的位置：面板内边距 + 标签列 + 行内间隙（实测 antd 参照图 x=328 设备像素 = 164 CSS）
CELL_X0 = PANEL_PAD + LABEL_WIDTH + CELL_GAP
# 各控件在面板里的单元格宽度（CSS px）：按钮 56 够包住禁用态投影，输入框占满剩余宽度
CELL_SPECS = {
    "button": (CELL_X0, 56),
    "input": (CELL_X0, 240),
    "textarea": (CELL_X0, 240),
    "taginput": (CELL_X0, 240),
    "mentions": (CELL_X0, 240),
    "autocomplete": (CELL_X0, 240),
    "hotkey": (CELL_X0, 240),
    "tag": (CELL_X0, 120),
    "select": (CELL_X0, 240),
    "multi": (CELL_X0, 240),
    "menu": (CELL_X0, 240),
    "tooltip": (CELL_X0, 240),
    "popover": (CELL_X0, 240),
    "pagination": (CELL_X0, 240),
    "switch": (CELL_X0, 240),
    "checkbox": (CELL_X0, 240),
    "radio": (CELL_X0, 240),
    "progress": (CELL_X0, 240),
    "avatar": (CELL_X0, 240),
    "badge": (CELL_X0, 240),
    "breadcrumb": (CELL_X0, 240),
    "skeleton": (CELL_X0, 240),
    "alert": (CELL_X0, 240),
    "card": (CELL_X0, 240),
    "steps": (CELL_X0, 240),
    "timeline": (CELL_X0, 240),
    "segmented": (CELL_X0, 240),
    "tabs": (CELL_X0, 240),
    "slider": (CELL_X0, 240),
    "link": (CELL_X0, 240),
    "statistic": (CELL_X0, 240),
    "typography": (CELL_X0, 240),
    "descriptions": (CELL_X0, 240),
    "divider": (CELL_X0, 240),
    "space": (CELL_X0, 240),
    "result": (CELL_X0, 240),
    "tree": (CELL_X0, 240),
    "table": (CELL_X0, 240),
    "upload": (CELL_X0, 240),
    "transfer": (16, 388),
    "modal": (16, 388),
    "drawer": (16, 388),
    "carousel": (CELL_X0, 240),
    "rating": (CELL_X0, 240),
    "spin": (CELL_X0, 240),
    "empty": (CELL_X0, 240),
    "empty2": (CELL_X0, 240),
    "colorpicker": (CELL_X0, 240),
    "colorpicker2": (CELL_X0, 240),
    "datepicker": (CELL_X0, 240),
    "datepicker2": (CELL_X0, 300),
    "timepicker": (CELL_X0, 240),
    "timepicker2": (CELL_X0, 240),
    "rangepicker": (CELL_X0, 240),
    "rangepicker2": (CELL_X0, 576),
    "calendar": (16, 760),
    "calendar2": (16, 760),
    "notification": (16, 384),
    "message": (CELL_X0, 240),
    "tour": (16, 528),
    "collapse": (CELL_X0, 240),
    "anchor": (CELL_X0, 240),
    "form": (CELL_X0, 240),
    "cascader": (CELL_X0, 184),
    "cascader2": (CELL_X0, 336),
    "treeselect": (CELL_X0, 240),
    "treeselect2": (CELL_X0, 240),
    "floatbutton": (CELL_X0, 240),
    "splitter": (CELL_X0, 240),
    "layout": (CELL_X0, 240),
    "grid": (CELL_X0, 240),
    "navmenu": (CELL_X0, 240),
    "countdown": (CELL_X0, 240),
    "kbd": (CELL_X0, 240),
    # 滚动条只比 8px 槽条（内容行文字/行高不参与）
    "scrollbar": (CELL_X0 + 232, 8),
}


def spec_kind(key):
    parts = key.split("-")
    return parts[1] if len(parts) > 1 and parts[0] in ("rust", "antd") else parts[0]


def cell_spec(key):
    """按键名里的控件名取单元格几何：rust-input-outlined -> input。"""
    return CELL_SPECS.get(spec_kind(key), (CELL_X0, CELL_WIDTH))


# 少数控件行高不是 40：值是 (pitch, 行高)，单位 CSS px
ROW_SPECS = {
    "card": (216, 200),
    "steps": (136, 120),
    "timeline": (136, 120),
    "tabs": (80, 64),
    "slider": (80, 64),
    "statistic": (112, 96),
    "typography": (88, 72),
    "descriptions": (232, 216),
    "divider": (76, 60),
    "space": (136, 120),
    "result": (256, 240),
    "tree": (136, 120),
    "table": (248, 232),
    "upload": (176, 160),
    "transfer": (216, 200),
    "modal": (216, 200),
    "drawer": (216, 200),
    "carousel": (176, 160),
    "spin": (80, 64),
    "empty": (196, 180),
    "empty2": (196, 180),
    "colorpicker2": (196, 180),
    "datepicker2": (412, 396),
    "timepicker2": (336, 320),
    "rangepicker2": (412, 396),
    "calendar": (806, 790),
    "calendar2": (552, 536),
    "popconfirm": (176, 160),
    "notification": (176, 160),
    "tour": (196, 180),
    "collapse": (156, 140),
    "anchor": (166, 150),
    "form": (116, 100),
    "cascader2": (256, 240),
    "treeselect2": (196, 180),
    "floatbutton": (112, 96),
    "splitter": (136, 120),
    "layout": (216, 200),
    "navmenu": (160, 144),
    "countdown": (88, 72),
    "scrollbar": (136, 120),
}


def row_spec(key):
    return ROW_SPECS.get(spec_kind(key), (ROW_PITCH, ROW_HEIGHT))
MAX_BAD_RATIO = float(os.environ.get("MAX_BAD_RATIO", "0.12"))
# 标签列由两套文字栅格化各自绘制，逐字差异不代表控件差异：
# 默认只比控件所在的窄格；INCLUDE_LABELS=1 时退回整行（含标签列）
INCLUDE_LABELS = os.environ.get("INCLUDE_LABELS", "") != ""

# 语义探针：窄格的像素比例对 1px 描边不敏感（描边只占格子约 7%），
# 所以在控件盒上再取两条带，比较"最显眼的颜色"。坐标是相对控件盒左上角的 CSS 偏移
# (x, 宽, y, 高)：上边框带避开圆角和字形，内部带落在左内边距里。
BORDER_BAND = (6, 10, 0, 2)
FILL_BAND = (3, 4, 16, 2)
BG_SPAN = (-6, 3, 16, 2)  # 控件盒外的面板底色
# gpui 阴影不被 overflow_hidden 裁剪：Drawer 面板阴影会染灰盒外垫带，
# 底色参照改取盒内遮罩带（两侧遮罩同受阴影影响，众数仍可比）
BG_SPECS = {"drawer": (2, 3, 16, 2)}
PROBE_TOLERANCE = 6
PROBE_DELTA_E = 5.0  # CIE76 ΔE；只吸收抗锯齿/子像素色相抖动，不放过真实色差

# 个别控件的探针带按自身几何调整：值仍是 (描边带, 底色带)，相对控件盒的 CSS 偏移
PROBE_SPECS = {
    # Tabs 的文字落在默认探针带里，字形抗锯齿（Chrome 次像素彩边）会污染众数；
    # 改为「选中项指示条 / 导航右侧分隔线」
    "tabs": ((230, 8, 0, 64), (230, 8, 0, 64)),
    # Slider 轨道在行内 y=14..18（块向外边距 11 + 轨道居中）；左端都是已选段，
    # 右侧 x=150 之后在所有行都落在未选轨道上
    "slider": ((20, 10, 14, 2), (150, 10, 14, 2)),
    # Statistic 只有文字：取右侧空白带，避免字形抗锯齿污染众数
    "statistic": ((210, 10, 10, 20), (210, 10, 50, 20)),
    # Typography 也是纯文字：同样取右侧空白带
    "typography": ((210, 10, 0, 20), (210, 10, 40, 20)),
    # Descriptions：取标题右侧空白，文字行由像素比兜底
    "descriptions": ((210, 10, 0, 20), (210, 10, 24, 20)),
    # Divider：右侧线段带（无文字行 y=24，带文字行 y=16 落在文字带之外）
    # 虚线相位不同导致探针带混入抗锯齿暗色，改取线下方空白带，由像素比兜底
    "divider": ((200, 10, 34, 2), (200, 10, 34, 2)),
    # Space：取右侧空白，布局差异由像素比覆盖
    "space": ((220, 10, 0, 20), (220, 10, 20, 20)),
    # Result：右侧留白带（图标居中，文字由像素比兜底）
    "result": ((224, 8, 8, 20), (224, 8, 208, 20)),
    # Tree：右侧空白带（文字行由像素比兜底）
    "tree": ((224, 8, 0, 20), (224, 8, 80, 20)),
    # Table：表头右侧空白 / 末行下方空白
    "table": ((228, 6, 4, 16), (228, 6, 216, 12)),
    # Upload：左虚线边框带（避开上下边框与相邻行重叠区）/ 区外下方空白
    "upload": ((0, 2, 60, 40), (228, 6, 150, 8)),
    # Transfer：左列表顶边框 / 首条目带（checked 行两侧都是选中底色）
    "transfer": ((60, 10, 0, 2), (100, 10, 44, 10)),
    # Modal：左遮罩带（黑 45% 罩面板底）+  footer 右端 OK 按钮带（主色，右对齐故各行同位）
    "modal": ((4, 20, 10, 20), (290, 20, 125, 10)),
    # Drawer：头部底分隔线（colorSplit）+ 头部右侧空白带（面板底；底色参照取遮罩带）
    "drawer": ((100, 40, 56, 1), (300, 40, 20, 10)),
    # Carousel：slide 纯色块左上 / 右上空白（避开箭头与指示条）
    "carousel": ((12, 10, 20, 10), (200, 10, 20, 20)),
    # Rating：首星左半（半星行也有色）/ 第二星左半带
    "rating": ((2, 8, 8, 4), (30, 8, 8, 4)),
    # Spin：首帧四点旋到四边中点，顶点点落在 y0..2；文案行描述落在 y40..44
    "spin": ((0, 60, 0, 2), (0, 60, 40, 4)),
    # Empty：默认插画投影椭圆带（simple 行落空=None/None）/ 描述文字带
    "empty": ((40, 40, 89, 4), (60, 120, 112, 6)),
    "empty2": ((40, 40, 89, 4), (60, 120, 112, 6)),
    # ColorPicker：触发框顶边框（避开圆角）/ 色块内部带（三档色块都覆盖 x6..18,y6..10）
    "colorpicker": ((4, 24, 0, 1), (6, 12, 6, 4)),
    # 展开行：触发框 active 顶边框 / 选中预设格（第 5 格 x132..156；
    # 默认行浮层顶 y36+padding12=48，small 行 y28+12=40，带 y48..64 两行都落在格内）
    "colorpicker2": ((4, 24, 0, 1), (134, 20, 48, 16)),
    "datepicker": ((4, 24, 0, 1), (12, 60, 10, 12)),
    "datepicker2": ((120, 60, 46, 12), (124, 40, 361, 12)),
    "timepicker": ((4, 24, 0, 1), (12, 60, 10, 12)),
    # TimePicker 展开行：列顶选中格底（row0 01/02/03 各自高亮的单元格）；
    # 旧带 y72..82 在 row1（无选中）会扫到数字字形，取选中格底部 padding 带
    # （y94..97：row0 落在选中底内，row1 落在两行数字之间）
    "timepicker2": ((18, 20, 94, 3), (12, 30, 282, 12)),
    # RangePicker 展开行：区间整格连续底（11 号格）/ 起点主色内格 / 终点左半连续底
    "rangepicker": ((4, 24, 0, 1), (12, 60, 10, 12)),
    "rangepicker2": ((126, 36, 174, 22), (54, 6, 210, 22)),
    # Calendar 月视图：第 3 行（15..21）周一格 = 今日+选中，顶条主色 / 格内选中底；
    # 年视图 3 列×4 行同高几何：第 4 行 Nov 格（col1 x253..507）顶条与选中底。带避开右对齐字形
    "calendar": ((113, 100, 318, 2), (115, 60, 322, 20)),
    "calendar2": ((300, 100, 412, 2), (300, 60, 416, 20)),
    # Popconfirm：警告图标带（三行图标位置不变：浮层顶 48 + padding 12）
    "popconfirm": ((12, 14, 60, 14), (12, 14, 62, 10)),
    # Notification：类型图标带（padding 24/20 起 24px）/ 标题右侧空白底带（避开关闭钮 x338+）
    "notification": ((24, 20, 24, 24), (280, 24, 50, 12)),
    # Message：16px 类型图标带（padding 12 + 居中 3）/ 盒内顶部内边距空白带
    # （盒顶 padding 12 内无内容；旧带 y60 落在盒外投影里，两侧投影衰减不同）
    "message": ((12, 12, 16, 16), (20, 20, 2, 3)),
    # InputNumber：步进列左描边（控件宽 90 − 列宽 22 = x68）/ 列内箭头左侧空白底
    # （箭头字形居中 x75.5..82.5，带 x70..74 避开；disabled 行两侧都落面板底）
    "numberinput": ((68, 1, 4, 8), (70, 4, 4, 8)),
    # Search：顶描边 / 清空实心圆图标带（12px 圆占满带，众数即图标色；无清空行落底）
    "search": ((6, 10, 0, 2), (217, 12, 14, 12)),
    # Tour：遮罩带（盒内左上，卡片外）/ 卡片头内边距带（标题字形之外）
    "tour": ((4, 20, 4, 8), (150, 40, 40, 8)),
    # Collapse：顶描边（避开 r8 圆角）/ 头左内边距底（headerBg，字形 x16 起）
    "collapse": ((12, 10, 0, 1), (3, 4, 12, 4)),
    # Anchor：连续轨带（colorSplit）/ 首行 ink 带（row0 激活 ink，其余行落轨，两侧同位）
    "anchor": ((0, 2, 60, 4), (0, 2, 4, 22)),
    # Form：输入框顶描边（error 行红、small 行顶在 y32 落底）/ 框内左内边距底；
    # 标签/星号/提示色由矩阵测试承载（文字带抗锯齿污染众数）
    "form": ((6, 10, 30, 1), (3, 4, 36, 4)),
    # Cascader 展开行：触发框顶描边（open 未聚焦仍 colorBorder）/ 首列路径项选中底
    # （浮层顶 36 + 列 padding 4 = 40，项高 32；两行首项都在路径上）
    "cascader2": ((6, 10, 0, 1), (10, 20, 44, 12)),
    # TreeSelect 展开行：顶描边 / 选中节点底（浮层顶 52 + padding 8 + 2×28 节点距 = 116..140；
    # 取节点右侧无字区 x150..170、底带 y104..107：row0 落在选中底内，collapsed 行
    # 落在浮层底部内边距（白底），两侧同位；旧带 y104..116 在 collapsed 行落到卡片下方投影）
    "treeselect2": ((6, 10, 0, 1), (150, 20, 104, 3)),
    # FloatButton：(x,w,y,h)；相对盒 240×96 内浮钮落在 (176,8..48)。取圆心左侧内部
    # 底带（避开箭头字形；白底行=无）+ 左缘 1px 描边带（default/square/disabled
    # #d9d9d9，primary 主色）。图标笔画不探：antd 的 SVG 1px 描边缩放后只剩灰阶
    "floatbutton": ((182, 8, 28, 8), (176, 1, 26, 4)),
    # Splitter：中心指示段 colorFill 叠 controlItemBgHover 合成 (208,208,208)；
    # 水平行 knob 2×20 在 (119,50)，垂直行 20×2 在 (110,59)
    "splitter": ((119, 2, 50, 20), (110, 20, 59, 2)),
    # Layout：(x,w,y,h)。row0 深 navy 带；row1 取浅色 sider 文字下方的纯白底带
    # （文字 ink 止于 y≈82，带从 y86 起避开字形）
    "layout": ((10, 20, 10, 10), (10, 20, 86, 8)),
    # Grid：首栏内容带（offset 行落空底）+ 第二栏内容带（三行皆主色）
    "grid": ((10, 20, 10, 20), (100, 20, 10, 20)),
    # NavMenu：row0 选中底带（其余行该位透明落底）/ root 右 1px colorSplit 描边带
    "navmenu": ((100, 40, 10, 20), (239, 1, 10, 40)),
    # Countdown：内容字形带（colorTextHeading；row1 内容 ink 止于 y29，该带落底）/
    # 标题字形核心带（row0 标题 description 色，row1 无标题该位落内容字形核心）
    "countdown": ((4, 30, 34, 12), (2, 20, 7, 8)),
    # Kbd 帽内衬底（249，与白底差 6 → None）+ 首键字形芯（31）；
    # 帽顶相对行顶 antd 2px / gpui 0px，探针避开边框带
    "kbd": ((3, 5, 6, 10), (9, 4, 8, 6)),
    # 滚动条槽：滑块带（127）+ 滑块下方空白（None）
    "scrollbar": ((0, 8, 2, 16), (0, 8, 60, 20)),
}

ROW_NAMES_PATH = os.path.join(ROOT, "harness", "rows.txt")


def css_box(row, box, pitch=ROW_PITCH, cell_x0=CELL_X0):
    """CSS 偏移盒 -> 设备像素盒（控件盒左上角 = cell_x0, PANEL_PAD + row * pitch）。"""
    bx, bw, by, bh = box
    x0 = round((cell_x0 + bx) * DEVICE_SCALE)
    y0 = round((PANEL_PAD + row * pitch + by) * DEVICE_SCALE)
    return (x0, y0, x0 + round(bw * DEVICE_SCALE), y0 + round(bh * DEVICE_SCALE))


def band_colors(img, box):
    x0, y0, x1, y1 = box
    px = img.load()
    return [px[x, y] for y in range(y0, y1) for x in range(x0, x1)]


def dominant_color(img, row, box, bg, pitch=ROW_PITCH, cell_x0=CELL_X0):
    """带内离面板底色最远的那簇颜色；整带都是底色时返回 None（即"这里没有描边/填充"）。"""
    pixels = [p for p in band_colors(img, css_box(row, box, pitch, cell_x0))
              if max(abs(p[0] - bg[0]), abs(p[1] - bg[1]), abs(p[2] - bg[2])) > PROBE_TOLERANCE]
    if not pixels:
        return None
    return Counter(pixels).most_common(1)[0][0]


def panel_bg(img, row, pitch=ROW_PITCH, cell_x0=CELL_X0, span=BG_SPAN):
    """控件盒外那条带就是面板底色；参照面板只画浅/深两套，取带内众数即可。"""
    pixels = band_colors(img, css_box(row, span, pitch, cell_x0))
    return Counter(pixels).most_common(1)[0][0]


def delta_e(a, b):
    """sRGB 颜色对的 CIE76 ΔE（D65）。"""
    def to_lab(c):
        def linear(u):
            u /= 255.0
            return ((u + 0.055) / 1.055) ** 2.4 if u > 0.04045 else u / 12.92
        r, g, bl = linear(c[0]), linear(c[1]), linear(c[2])
        x = (r * 0.4124 + g * 0.3576 + bl * 0.1805) / 0.95047
        y = r * 0.2126 + g * 0.7152 + bl * 0.0722
        z = (r * 0.0193 + g * 0.1192 + bl * 0.9505) / 1.08883

        def f(t):
            return t ** (1 / 3) if t > 0.008856 else 7.787 * t + 16 / 116
        fx, fy, fz = f(x), f(y), f(z)
        return (116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz))
    la, lb = to_lab(a), to_lab(b)
    return sum((x - y) ** 2 for x, y in zip(la, lb)) ** 0.5


def probe_match(a, b):
    """两侧同一条带的显眼色是否一致。

    1px 描边在高 DPI 下天然带抗锯齿；gpui-pre 在 Windows 上默认 ClearType
    子像素渲染后，边缘色相的抖动可达每通道 7，逐通道容差会把"同色"误判成错配。
    改用 CIE76 ΔE：实测真实状态错配 ΔE≥16.9（hover 色/错误红 vs 灰），
    抗锯齿抖动 ΔE≤2.4，5 的阈值只滤掉视觉同色。
    """
    if a is None or b is None:
        return a is b
    return delta_e(a, b) <= PROBE_DELTA_E


def row_names(widget):
    """行标签清单：harness/rows.txt 里以 `widget:` 开头的逗号分隔列表。"""
    if not os.path.exists(ROW_NAMES_PATH):
        return None
    for line in open(ROW_NAMES_PATH, encoding="utf-8"):
        line = line.strip()
        if line.startswith(f"{widget}:"):
            return line.split(":", 1)[1].split(",")
    return None


def diff_region(a, b):
    """两张同尺寸 RGB 图块的差异比例。

    ClearType 子像素抗锯齿会在字形边缘引入橙/蓝彩边：单通道差异很大，但覆盖率
    （亮度）不变。只按通道判差会把密集文字行误算成布局差异，因此通道差超阈值时
    再看亮度差——等亮度彩边不计，真实错位/换色仍会改变亮度。
    """
    aw, ah = a.size
    bad = 0
    total = aw * ah
    pa, pb = a.load(), b.load()
    for y in range(ah):
        for x in range(aw):
            ra, ga, ba = pa[x, y][:3]
            rb, gb, bb = pb[x, y][:3]
            if max(abs(ra - rb), abs(ga - gb), abs(ba - bb)) <= TOLERANCE:
                continue
            la = 0.299 * ra + 0.587 * ga + 0.114 * ba
            lb = 0.299 * rb + 0.587 * gb + 0.114 * bb
            if abs(la - lb) > TOLERANCE:
                bad += 1
    return bad / total


def main():
    widget = (sys.argv[1] if len(sys.argv) > 1 else "").lower()
    if not os.path.isdir(BASE):
        print(f"缺少基线目录 {BASE}，先产出 Rust 面板截图")
        return 0
    if not os.path.isdir(REF):
        print(f"缺少参考目录 {REF}，先用 browser MCP 截 harness/antd_reference.html")
        return 0

    compared = 0
    failures = []
    for name in sorted(os.listdir(BASE)):
        if not name.endswith(".png"):
            continue
        key = name[:-4]
        if widget and widget not in key.lower():
            continue
        ref_name = name.replace("rust", "antd")
        ref_path = os.path.join(REF, ref_name)
        if not os.path.exists(ref_path):
            # 允许同名（非 rust- 前缀）命名
            ref_path = os.path.join(REF, name)
            if not os.path.exists(ref_path):
                print(f"SKIP {name}: 缺参考图 {ref_name}")
                continue

        base = Image.open(os.path.join(BASE, name)).convert("RGB")
        ref = Image.open(ref_path).convert("RGB")
        if base.size != ref.size:
            print(f"FAIL {name}: 面板尺寸不同 rust={base.size} antd={ref.size}")
            failures.append(name)
            continue

        rows = row_names("-".join(key.split("-")[1:])) or row_names(key.split("-")[-1]) or []
        cell_x0, cell_w = cell_spec(key)
        pitch_css, row_height = row_spec(key)
        pitch = round(pitch_css * DEVICE_SCALE)
        panel_pad = PANEL_PAD * DEVICE_SCALE
        n_rows = min(len(rows), (base.size[1] - panel_pad) // pitch)
        if n_rows == 0:
            ratio = diff_region(base, ref)
            ok = ratio <= MAX_BAD_RATIO
            print(f"{'OK  ' if ok else 'FAIL'} {name}: 整图差异 {ratio*100:.2f}%")
            if not ok:
                failures.append(name)
            compared += 1
            continue

        for i in range(n_rows):
            y0 = panel_pad + i * pitch
            if INCLUDE_LABELS:
                box = (panel_pad, y0, base.size[0] - panel_pad, y0 + pitch)
            else:
                x0 = cell_x0 * DEVICE_SCALE
                box = (x0, y0, x0 + cell_w * DEVICE_SCALE, y0 + row_height * DEVICE_SCALE)
            ratio = diff_region(base.crop(box), ref.crop(box))
            bg_span = BG_SPECS.get(spec_kind(key), BG_SPAN)
            bg_r, bg_a = panel_bg(base, i, pitch_css, cell_x0, bg_span), panel_bg(ref, i, pitch_css, cell_x0, bg_span)
            probes = []
            border_band, fill_band = PROBE_SPECS.get(spec_kind(key), (BORDER_BAND, FILL_BAND))
            for tag, band in (("描边", border_band), ("底色", fill_band)):
                pr, pa = dominant_color(base, i, band, bg_r, pitch_css, cell_x0), dominant_color(ref, i, band, bg_a, pitch_css, cell_x0)
                probes.append((tag, pr, pa, probe_match(pr, pa)))
            ok = ratio <= MAX_BAD_RATIO and all(p[3] for p in probes)
            label = f"{name} row{i} [{rows[i]}]"
            detail = " ".join(
                f"{tag}={'无' if pr is None else '#%02x%02x%02x' % pr}"
                f"/{'无' if pa is None else '#%02x%02x%02x' % pa}{'' if match else '≠'}"
                for tag, pr, pa, match in probes
            )
            print(f"{'OK  ' if ok else 'FAIL'} {label}: 差异 {ratio*100:.2f}% {detail}")
            if not ok:
                failures.append(label)
                out = os.path.join(ROOT, "harness", "shots", f"diff-{key}-row{i}.png")
                os.makedirs(os.path.dirname(out), exist_ok=True)
                base.crop(box).save(out)
                ref.crop(box).save(out.replace("diff-", "ref-"))
            compared += 1

    if compared == 0:
        print("没有可比的图对")
        return 0
    if failures:
        print(f"\n{len(failures)} 处超出阈值 {MAX_BAD_RATIO*100:.0f}%：")
        for f in failures:
            print("  -", f)
        return 1
    print(f"\n逐组比对通过（{compared} 组）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
