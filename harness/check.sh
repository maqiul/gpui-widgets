#!/usr/bin/env bash
# 控件库改动后的统一验证入口。四步任一失败即退出码 1。
#
#   bash harness/check.sh              # 全量
#   bash harness/check.sh button       # 只跑名字含 button 的测试 + 只比对 button 截图
set -uo pipefail
cd "$(dirname "$0")/.."

FILTER="${1:-}"
FAILED=0

step() { printf '\n=== %s ===\n' "$1"; }
run() { step "$1"; shift; if ! "$@"; then echo "!! 失败: $*"; FAILED=1; fi; }

run "1/4 cargo check --all-targets" cargo check --all-targets --offline

# 步骤 2 的名字过滤是按测试函数名匹配的：控件测试请以控件名开头（如 button_light_matrix），
# 否则 `check.sh button` 会把它们全部过滤掉。
# 全量 `cargo test` 还会编译其它控件的 doctest；src/image.rs 的示例缺 `use gpui::Styled`，
# 属该控件的既有问题，控件迁移到各自步骤时再修。
if [ -n "$FILTER" ]; then
    run "2/4 cargo test $FILTER" cargo test --offline "$FILTER"
else
    run "2/4 cargo test" cargo test --offline
fi

# 3/4 gallery 起 3s 无 panic
step "3/4 gallery 冒烟（3s）"
cargo run --offline --example gallery >/dev/null 2>&1 &
PID=$!
sleep 3
if kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null
    echo "OK：3s 内未退出"
else
    echo "!! gallery 3s 内退出（疑似 panic）"
    FAILED=1
fi
wait "$PID" 2>/dev/null

# 4/4 逐组截图比对（Python + PIL；参照图由 browser MCP 采集，见 .agents/tasks/*/plan.md）
step "4/4 逐组截图比对"
if [ -d harness/baseline ] && [ -d harness/antd_shots ]; then
    if [ -n "$FILTER" ]; then
        py harness/shot_diff.py "$FILTER" || FAILED=1
    else
        py harness/shot_diff.py || FAILED=1
    fi
else
    echo "SKIP：harness/baseline 或 harness/antd_shots 为空（首批截图待 Step 3.2 产出）"
fi

printf '\n=== check.sh 结果 ===\n'
if [ "$FAILED" -eq 0 ]; then echo "PASS"; else echo "FAIL"; fi
exit "$FAILED"
