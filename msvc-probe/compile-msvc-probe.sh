#!/usr/bin/env bash
# MSVC 试编译 scriptc runtime C 源码 — 收集移植工作量清单
# 用法: bash compile-msvc-probe.sh [文件.c ...]   (相对 src/ 或绝对路径均可)
set -uo pipefail

VSDIR="C:/Program Files/Microsoft Visual Studio/2022/Community"
MSVC="$VSDIR/VC/Tools/MSVC/14.44.35207"
WINSDK="C:/Program Files (x86)/Windows Kits/10"
SDK_VER="10.0.26100.0"
SRC_WIN='E:\AI-workspace\scriptc\vendor\scriptc-runtime\src'
SRC="/e/AI-workspace/scriptc/vendor/scriptc-runtime/src"
OUT_WIN='E:\AI-workspace\scriptc\msvc-probe\out'
LOG="/e/AI-workspace/scriptc/msvc-probe"

mkdir -p "$LOG/out"

# cl.exe 的 include 参数必须用 Windows 形式；MSYS_NO_PATHCONV 只对参数以 / 开头的情况生效，
# include 用 -I"win path" 形式最稳。
COMMON=(
  /nologo /c /W3
  /std:c17 /utf-8
  /D_CRT_SECURE_NO_WARNINGS
  /DNOMINMAX /DWIN32_LEAN_AND_MEAN
  "-I$SRC_WIN"
  "-I$WINSDK/Include/$SDK_VER/ucrt"
  "-I$WINSDK/Include/$SDK_VER/um"
  "-I$WINSDK/Include/$SDK_VER/shared"
  "-I$MSVC/include"
  "-Fo$OUT_WIN\\"
)

FILES=("$@")
if [ ${#FILES[@]} -eq 0 ]; then
  FILES=("$SRC"/*.c)
fi

PASS_LIST=()
FAIL_LIST=()

for f in "${FILES[@]}"; do
  base=$(basename "$f")
  case "$base" in
    scr_loop_epoll.c|scr_loop_kqueue.c|scr_musl.c|scr_fetch_curl.c)
      echo "SKIP  $base (平台专属，不属于 win 构建面)"
      continue
      ;;
  esac
  fwin=$(cygpath -w "$f")
  echo "TRY   $base"
  if "$MSVC/bin/Hostx64/x64/cl.exe" "${COMMON[@]}" "$fwin" \
      > "$LOG/${base%.c}.log" 2>&1; then
    echo "PASS  $base"
    PASS_LIST+=("$base")
  else
    echo "FAIL  $base"
    FAIL_LIST+=("$base")
  fi
done

echo
echo "=========================================="
echo "PASS: ${#PASS_LIST[@]}  FAIL: ${#FAIL_LIST[@]}  (SKIP 4)"
[ ${#FAIL_LIST[@]} -gt 0 ] && { echo "FAILED:"; printf '  %s\n' "${FAIL_LIST[@]}"; }
