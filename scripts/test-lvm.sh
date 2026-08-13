#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
#  lvm 容器内功能测试脚本
#  用法(镜像内):  lvm-test
#  覆盖: 版本 / 预装 / env / hook 自动切换 / LTS 缓存 /
#        校验和 / alias / uninstall(部分版本解析) / 链接清理
# ─────────────────────────────────────────────────────────────
set -u

GREEN=$'\033[0;32m'; RED=$'\033[0;31m'; YEL=$'\033[1;33m'; NC=$'\033[0m'
PASS=0; FAIL=0

pass()   { echo -e "${GREEN}✔ PASS${NC}  $1"; PASS=$((PASS+1)); }
fail()   { echo -e "${RED}✘ FAIL${NC}  $1"; FAIL=$((FAIL+1)); }
section(){ echo; echo -e "${YEL}═══ $1 ═══${NC}"; }

# 加载 lvm 环境与 hook（脚本独立运行，不依赖 .zshrc/.bashrc）
eval "$(lvm env)"
eval "$(lvm hook)"

rm -rf /tmp/lvmtest && mkdir -p /tmp/lvmtest/a /tmp/lvmtest/b /tmp/lvmtest/c

section "1. 版本信息"
if lvm --version 2>&1 | grep -qE '^lvm [0-9]+\.[0-9]+\.[0-9]+$'; then
  pass "lvm --version = $(lvm --version)"
else
  fail "lvm --version 输出异常: $(lvm --version 2>&1)"
fi

section "2. 预装版本 (node 20 / go latest)"
node --version 2>&1 | grep -q '^v20' && pass "node 可执行: $(node --version)" || fail "node 不可执行"
go version 2>&1 | grep -q '^go version go1' && pass "go 可执行: $(go version)" || fail "go 不可执行"
lvm current node 2>&1 | grep -q 'v20' && pass "lvm current node → $(lvm current node)" || fail "lvm current node 异常"
lvm current go 2>&1 | grep -q 'v1' && pass "lvm current go → $(lvm current go)" || fail "lvm current go 异常"

section "3. 环境变量 (lvm env)"
[[ "$LVM_HOME" == "/root/.lvm" ]] && pass "LVM_HOME=$LVM_HOME" || fail "LVM_HOME=$LVM_HOME"
[[ "$GOPATH" == "/root/.lvm/current/go/packages" ]] && pass "GOPATH=$GOPATH" || fail "GOPATH=$GOPATH"
echo "$PATH" | grep -q '/root/.lvm/bin' && pass "PATH 包含 /root/.lvm/bin" || fail "PATH 缺少 /root/.lvm/bin"
echo "$PATH" | grep -q '/root/.lvm/current/node/bin' && pass "PATH 包含 node bin" || fail "PATH 缺少 node bin"

section "4. which / list"
lvm which node 2>&1 | grep -q '/node/v20.*/bin/node' && pass "which node → $(lvm which node)" || fail "which node 异常"
lvm list node 2>&1 | grep -q 'current, default' && pass "list node 标注 current, default" || fail "list node 缺少标记"
lvm list go 2>&1 | grep -q 'current, default' && pass "list go 标注 current, default" || fail "list go 缺少标记"

section "5. hook 自动切换 (未安装→跳过 / 已安装→切换 / .nvmrc LTS)"
echo "node=22" > /tmp/lvmtest/a/.lvmrc           # 未安装 → 应跳过不下载
( cd /tmp/lvmtest/a && __lvm_auto >/dev/null 2>&1 )
lvm current node 2>&1 | grep -q 'v20.20.2' \
  && pass "未安装版本 node22 被跳过, 当前仍为 v20.20.2" || fail "未安装版本处理异常"

echo "node=20.20.2" > /tmp/lvmtest/b/.lvmrc      # 已安装 → 应切换
( cd /tmp/lvmtest/b && __lvm_auto >/dev/null 2>&1 )
lvm current node 2>&1 | grep -q 'v20.20.2' && pass "已安装版本切换正常" || fail "已安装版本切换异常"

echo "lts/iron" > /tmp/lvmtest/c/.nvmrc           # LTS 别名
( cd /tmp/lvmtest/c && __lvm_auto >/dev/null 2>&1 ); ret=$?
[ "$ret" -eq 0 ] && pass ".nvmrc lts/iron 解析不报错 (exit=$ret)" || fail ".nvmrc lts/iron 解析失败 (exit=$ret)"

section "6. LTS 磁盘缓存"
if [ -f /root/.lvm/cache/node-lts.json ]; then
  pass "node-lts.json 缓存已落盘 ($(stat -c%s /root/.lvm/cache/node-lts.json) bytes)"
else
  fail "node-lts.json 缓存缺失"
fi

section "7. install node 18 (下载 + 校验和 + 切换)"
lvm install node 18 >/tmp/lvmtest/node.log 2>&1
if lvm list node 2>&1 | grep -qE '\bv18\.'; then
  pass "node 18 安装成功 ($(lvm list node 2>&1 | tr '\n' ' '))"
else
  fail "node 18 安装失败"; tail -5 /tmp/lvmtest/node.log
fi
if grep -q 'Verifying checksum' /tmp/lvmtest/node.log && grep -q 'Checksum verified' /tmp/lvmtest/node.log; then
  pass "node 校验和已验证"
else
  fail "node 未验证校验和 (查看 /tmp/lvmtest/node.log)"
fi
lvm use node 18 >/dev/null 2>&1
node --version 2>&1 | grep -q '^v18' && pass "切换后 node: $(node --version)" || fail "切换后 node 异常"
# 切回预装的 20.20.2，保持干净状态
lvm use node 20.20.2 >/dev/null 2>&1
node --version 2>&1 | grep -q '^v20' && pass "已切回 node: $(node --version)" || fail "切回失败"

section "8. alias 管理"
lvm alias node testalias 20.20.2 >/dev/null 2>&1
lvm alias node testalias 2>&1 | grep -q '20.20.2' && pass "创建 alias testalias→20.20.2" || fail "创建 alias 失败"
lvm unalias node testalias >/dev/null 2>&1
if lvm alias node testalias 2>&1 | grep -q '20.20.2'; then
  fail "删除 alias 失败"
else
  pass "删除 alias 成功"
fi

section "9. uninstall node 18 (部分版本解析 + 当前版本链接清理)"
# 18 非当前版本 → 只删目录
lvm uninstall node 18 >/dev/null 2>&1
if lvm list node 2>&1 | grep -qE '\bv18\.'; then
  fail "uninstall node 18 失败 (部分版本未解析)"
else
  pass "uninstall node 18 成功 (部分版本 18→18.x.x 解析生效)"
fi
# 卸载当前版本 → 应同时清理 bin 链接
lvm use node 18 >/dev/null 2>&1
lvm uninstall node 18 >/dev/null 2>&1
if [ ! -L /root/.lvm/bin/node ]; then
  pass "卸载当前 node 后 bin 链接已清理"
else
  fail "bin 链接残留 (卸载当前版本未清理)"
fi
# 恢复预装版本
lvm use node 20.20.2 >/dev/null 2>&1
node --version 2>&1 | grep -q '^v20' && pass "已恢复 node: $(node --version)" || fail "恢复 node 失败"

section "10. debug"
lvm debug 2>&1 | grep -q "lvm v" && pass "lvm debug 输出正常" || fail "lvm debug 异常"

echo
echo "══════════════════════════════════════════════"
if [ "$FAIL" -eq 0 ]; then
  echo -e "${GREEN}全部通过 ✅  ($PASS passed / 0 failed)${NC}"
else
  echo -e "${RED}存在失败 ❌  ($PASS passed / $FAIL failed)${NC}"
fi
echo "══════════════════════════════════════════════"
[ "$FAIL" -eq 0 ]
