#!/usr/bin/env bash
# gpui-widgets 发布打包：在干净 staging 目录里 cargo package。
# 为什么不在仓库根目录直接打包：
#   1) harness/antd_baseline/node_modules 是指向仓库外的 JUNCTION，cargo 会跟进去；
#   2) cargo 无视 include/exclude 强收任意位置的 LICENSE/README 文件，
#      会把 .ref/node_modules 里的第三方文件带进包。
# staging 只拷白名单文件，两个问题都不存在。
# 用法：bash packaging/package-crate.sh [--verify-tests]
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"
STAGE="$ROOT/target/crate-stage"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
VERIFY_TESTS=0
[ "${1:-}" = "--verify-tests" ] && VERIFY_TESTS=1

rm -rf "$STAGE"
mkdir -p "$STAGE/harness/antd_baseline"
cp -r src tests examples docs "$STAGE/"
cp README.md README.en.md LICENSE Cargo.toml Cargo.lock "$STAGE/"
cp harness/antd_baseline/*.txt "$STAGE/harness/antd_baseline/"

echo "== staging 文件数：$(find "$STAGE" -type f | wc -l) =="
cd "$STAGE"
# 复用主 target 缓存，验证构建不用从头编译 gpui。
export CARGO_TARGET_DIR="$ROOT/target"
cargo package --allow-dirty --offline

CRATE="$ROOT/target/package/gpui-widgets-$VERSION.crate"
echo "== 产物：$CRATE ($(du -h "$CRATE" | cut -f1)) =="
if [ "$VERIFY_TESTS" = 1 ]; then
  EXTRACT="$ROOT/target/package/gpui-widgets-$VERSION"
  echo "== 在解包目录跑测试（验证基准文件随包可用） =="
  (cd "$EXTRACT" && cargo test --offline)
fi