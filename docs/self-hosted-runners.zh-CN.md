# InfiniShell-Desktop 自托管 CI Runner

本仓库使用两台长期运行的 GitHub Actions self-hosted runner 做 Linux/Windows
跨平台预检，不依赖 Oz 或 `oz-dev`。工作流位于
`.github/workflows/cross-platform-preflight.yml`。

## 1. Runner 约定

| 平台 | 推荐名称 | 必需标签 |
|---|---|---|
| Linux x64 | `infinishell-linux-x64` | `self-hosted`, `linux`, `x64`, `infinishell-ci` |
| Windows x64 | `infinishell-windows-x64` | `self-hosted`, `windows`, `x64`, `infinishell-ci` |

`self-hosted`、操作系统和架构标签由 GitHub 自动添加；注册时只需添加自定义标签
`infinishell-ci`。如果服务器实际是 ARM64，不要添加 `x64`，并同步调整 workflow。

若 runner 供组织内多个仓库使用，建议创建 `infinishell-preflight` runner group，
并只授权 `Infinimesh-ai/InfiniShell-Desktop`。如果两台机器仅供本仓库使用，直接注册
repository-level runner 即可。

## 2. 安全边界

self-hosted runner 会直接执行仓库代码，因此两台服务器必须是专用构建机：

- 使用独立低权限系统账号，不保存个人 SSH key、云凭据或生产密钥。
- 不挂载开发机目录，不与生产服务共享 Docker socket 或管理员会话。
- workflow 的 `GITHUB_TOKEN` 只授予 `contents: read`，checkout 不保留凭据。
- self-hosted workflow 不监听 `pull_request`；外部 Fork 即使获得 Actions 审批也不会进入
  这两台 runner。
- 建议用 VM 快照定期重置。长期复用机器时，应监控剩余磁盘并及时更新 runner 程序。
- 初始安装依赖可能需要管理员权限；runner 服务账号日常不应拥有免密 root/管理员权限。

## 3. Linux 安装

推荐 Ubuntu 22.04/24.04 x64。创建专用账号并完成一次性依赖安装：

```bash
sudo useradd --create-home --shell /bin/bash infinishell-runner
sudo -iu infinishell-runner
git clone https://github.com/Infinimesh-ai/InfiniShell-Desktop.git
cd InfiniShell-Desktop
```

在允许该账号临时使用 `sudo` 的安装窗口中运行：

```bash
./script/linux/install_build_deps
./script/install_cargo_test_deps
```

安装完成后撤销临时 sudo 权限。随后在 GitHub 仓库页面进入
`Settings → Actions → Runners → New self-hosted runner`，选择 Linux/x64，复制页面当前
版本的下载和校验命令。解压到 `/opt/actions-runner-infinishell` 后注册：

```bash
sudo mkdir -p /opt/actions-runner-infinishell
sudo chown infinishell-runner:infinishell-runner /opt/actions-runner-infinishell
cd /opt/actions-runner-infinishell
```

下载、校验和解压都应以 `infinishell-runner` 身份完成。然后执行：

```bash
./config.sh \
  --url https://github.com/Infinimesh-ai/InfiniShell-Desktop \
  --token '<GitHub 页面生成的短期 token>' \
  --name infinishell-linux-x64 \
  --labels infinishell-ci \
  --work _work \
  --unattended

sudo ./svc.sh install infinishell-runner
sudo ./svc.sh start
sudo ./svc.sh status
```

服务启动后，GitHub Runners 页面应显示 `Idle`，并列出四个必需标签。

## 4. Windows 安装

推荐 Windows Server 2022 或 Windows 11 x64。创建专用本地账号，以该账号登录并在
管理员 PowerShell 中准备仓库：

```powershell
git clone https://github.com/Infinimesh-ai/InfiniShell-Desktop.git
Set-Location InfiniShell-Desktop
$env:WARP_SKIP_GCLOUD_AUTH = '1'
.\script\windows\bootstrap.ps1
```

完成后重启终端，确认 `cargo`、MSVC/Windows SDK、CMake、LLVM、Strawberry Perl 和
Protobuf 25.1 可用。Runner 建议安装到 `C:\actions-runner`。在 GitHub 的
`New self-hosted runner → Windows → x64` 页面复制当前版本的下载命令，然后注册。不要把
账户密码放到命令行；配置程序询问是否作为服务运行时选择 `Y`，并填写上一步的专用
低权限 runner 账户：

```powershell
.\config.cmd --url https://github.com/Infinimesh-ai/InfiniShell-Desktop --token '<GitHub 页面生成的短期 token>' --name infinishell-windows-x64 --labels infinishell-ci --work _work
```

Windows 服务必须使用刚才完成工具链安装的专用账号，否则用户级 Rust、`PROTOC` 等环境
变量不可见。安装完成后在 Services 中确认 GitHub Actions Runner 服务正在运行。

### 4.1 Windows 构建缓存与主机配置

Windows job 使用 checkout 外的固定目录，避免 `actions/checkout` 的 `clean: true` 每轮删除
全部 Rust 编译产物：

```text
C:\infinishell-ci\cargo-target
C:\infinishell-ci\sccache
C:\infinishell-ci\tools\sccache-v0.17.0\sccache.exe
```

从 Mozilla 官方 Release 安装 `sccache 0.17.0`；Windows x64 压缩包的 SHA-256 应为
`e94cfc5b58cbe439302f586c1d1bd7980c2cd371d47bdf385ade657411e6f3ac`。给专用 runner
账户授予 `C:\infinishell-ci` 的继承式 Modify 权限。工作流通过 `CARGO_TARGET_DIR`、
`RUSTC_WRAPPER`、`SCCACHE_DIR` 和 `SCCACHE_CACHE_SIZE` 使用这些目录，并在每轮结束输出
sccache 统计。

在管理员 PowerShell 中启用 Win32 长路径并切换到高性能电源方案。Microsoft Defender
只排除 `cargo-target` 和 `sccache` 两个缓存目录；不要排除源码、Runner 根目录或编译器进程。
缓存目录应定期检查容量，清理前必须确认没有 Cargo、rustc、MSVC 或 Runner job 正在运行。

## 5. 触发验证

工作流尚未合并到默认分支时，推送 `ci/**` 分支即可同时运行 Linux 和 Windows 聚焦
检查。这也用于首次确认 runner 配置：

```bash
git push origin HEAD:refs/heads/ci/responses-api
```

工作流进入默认分支后，建议在 GitHub 的 Actions 页面手动运行
`Cross-platform preflight`。可单独关闭某个平台；`full_workspace_tests` 默认关闭，仅运行：

```text
cargo check -p warp --lib
cargo nextest run --no-fail-fast -p warp --lib -E 'test(responses) | test(ai::agent_providers::) | test(ai::api_error::tests) | test(ai::byop_compaction::) | test(ai::blocklist::controller) | test(ai::agent::conversation::tests) | test(i18n::tests)'
cargo test --manifest-path lib/rust-genai/Cargo.toml --lib
```

需要发布前全量回归时，再开启 `full_workspace_tests`，追加：

```text
cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2
```

Windows 服务会话中的 `warp` 主应用测试由 `.config/nextest.toml` 中的
`windows-warp-tests` 组限制为单并发，避免终端进程清理竞争；其余 crate 使用 nextest
默认并发，不要在工作流命令上增加全局 `--test-threads=1`。该组允许最多重试两次，
用于容忍测试逻辑通过后 Windows 服务会话偶发的进程退出延迟；持续失败仍会阻止发布。

也可以使用 GitHub CLI：

```bash
gh workflow run cross-platform-preflight.yml \
  --ref '<远端分支>' \
  -f run_linux=true \
  -f run_windows=true \
  -f full_workspace_tests=false

gh run list --workflow cross-platform-preflight.yml --limit 5
```

组织成员通过 `ci/**` 分支或手动运行触发验证。Fork PR 永远不会进入 self-hosted runner；
需要验证外部贡献时，应先完成代码审查，再由维护者把明确的提交放入受控分支。

## 6. 日常维护

- 每月检查 runner 版本、操作系统安全更新、Rust toolchain 和剩余磁盘。
- 更新工具链后手动运行一次双平台预检。
- Runner 离线时任务会排队；先从 GitHub Runners 页面检查 `Offline/Busy`，再检查系统服务。
- 失败日志通过 Actions 页面或 `gh run view <run-id> --log-failed` 获取。
- 替换服务器时重新注册并重新添加 `infinishell-ci` 标签；旧 runner 应从 GitHub 删除。
