#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
#  lvm 容器测试一键运行脚本（宿主侧）
#
#  用法:
#    scripts/run-tests.sh              # 用已有 lvm:build 镜像跑容器测试
#    scripts/run-tests.sh --build      # 先重建镜像再跑测试
#    scripts/run-tests.sh --shell zsh  # 用 zsh 执行测试脚本(默认 bash)
#
#  依赖: docker 已启动, lvm:build 镜像存在(或 --build 自动构建)
# ─────────────────────────────────────────────────────────────
set -eu

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
IMAGE="lvm:build"
DOCKERFILE="$ROOT/Dockerfile.build"
SCRIPT_IN_CONTAINER="/test/test-lvm.sh"
SHELL_BIN="bash"

# 参数解析
BUILD=0
while [ $# -gt 0 ]; do
  case "$1" in
    --build) BUILD=1; shift ;;
    --shell) SHELL_BIN="${2:-bash}"; shift 2 ;;
    -h|--help)
      sed -n '2,9p' "$0"
      exit 0
      ;;
    *) echo "未知参数: $1 (用 --help 查看用法)" >&2; exit 1 ;;
  esac
done

# 检查镜像，必要时构建
if [ "$BUILD" -eq 1 ] || ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "==> 构建镜像 $IMAGE ..."
  docker build -f "$DOCKERFILE" -t "$IMAGE" "$ROOT"
fi

echo "==> 启动容器运行测试脚本 ($SHELL_BIN) ..."
echo "    挂载: $HERE -> /test (只读)"
echo
docker run --rm \
  -v "$HERE:/test:ro" \
  "$IMAGE" \
  "$SHELL_BIN" "$SCRIPT_IN_CONTAINER"
