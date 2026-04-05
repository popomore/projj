# Projj v3 改造设计文档

## 定位

Git 仓库入库工具。通过目录约定（URL → 路径）管理仓库的存放位置，整合 zoxide/fzf 处理导航和选择。

核心价值：**约定 + 入库 + 批量操作**。不做导航工具，不重复造轮子。

## 技术选型

从 Node.js 迁移到 Rust。

理由：
- CLI 工具启动速度敏感，Rust 编译为原生二进制，冷启动 ~5ms vs Node.js ~100ms+
- 单文件分发，不依赖 Node.js 运行时
- 文件系统操作密集（扫描目录、读写索引），Rust 性能优势明显
- 通过 cargo install 或 homebrew 分发，比 npm 全局安装更干净

## 目录约定

```
$BASE/
├── github.com/
│   └── popomore/
│       └── projj/
└── gitlab.com/
    └── popomore/
        └── projj/
```

URL `git@github.com:popomore/projj.git` → 路径 `$BASE/github.com/popomore/projj`

这是整个工具的核心，不变。

## 命令设计

### projj init

初始化配置 + 存量数据同步到 zoxide。

```bash
projj init
```

流程：
1. 创建 `~/.projj/config.toml`（交互式设置 base、platform）
2. 扫描 base 目录验证可访问
3. 如果检测到 zoxide，扫描所有仓库并灌入 zoxide
4. 如果开启了索引，构建初始索引
5. 输出 shell integration 提示（可选）

非 init 命令执行时缺少配置 → 直接报错 `请先运行 projj init`，不弹交互。

### projj add \<repo\>

按约定 clone 仓库。

```bash
projj add git@github.com:popomore/projj.git
projj add https://github.com/popomore/projj.git
projj add popomore/projj                         # 短写法
projj add ./some/local/repo                       # 本地目录移入
```

流程：
1. 解析 repo 地址（短写法使用 config 中的 platform 补全）
2. 计算目标路径 `$BASE/host/owner/repo`
3. 多 base 目录 → 交互选择（使用 fzf 或内置列表）
4. 目标已存在 → 打印路径，结束
5. 执行 preadd hook
6. 远程地址 → `git clone`；本地路径 → 移动目录
7. 更新索引（增量）
8. 执行 postadd hook
9. `zoxide add <path>`（如果 zoxide 可用）
10. 打印目标路径

短写法规则：`popomore/projj` → `git@{platform}:popomore/projj.git`，platform 默认 `github.com`。

### projj find [keyword]

定位仓库。整合 fzf 做选择。

```bash
projj find              # 列出所有仓库，fzf 选择
projj find projj        # 带关键词过滤
projj find eggjs/egg    # owner/repo 匹配
```

流程：
1. 读取索引
2. 有 keyword → 过滤匹配项；无 keyword → 全部
3. 有 fzf → pipe 给 fzf 选择；无 fzf → 内置列表选择
4. 输出选中路径到 stdout

配合 shell integration 可直接 cd：

```bash
# ~/.zshrc
p() {
  local dir
  dir=$(projj find "$@")
  [ -n "$dir" ] && cd "$dir"
}
```

### projj remove [keyword]

删除仓库。

```bash
projj remove projj
```

流程：
1. 搜索匹配（逻辑同 find）
2. 多个匹配 → fzf 选择
3. 显示完整路径，要求输入 `owner/repo` 确认
4. 删除目录
5. 索引在下次使用时自动更新

### projj run \<script\> [--all]

在仓库中执行脚本。

```bash
projj run "npm install"          # 当前目录
projj run "npm install" --all    # 所有仓库
projj run postadd                # 执行预定义 hook
projj run postadd --all
```

参数如果匹配 config 中的 hook 名则执行对应命令，否则直接当 shell 命令执行。

`--all` 时并发执行，显示进度。

### projj list

输出仓库路径列表，纯文本，可 pipe。

```bash
projj list                  # 列出所有仓库（扫描文件系统）
projj list --rebuild        # 强制重建索引（仅开启索引时有意义）
projj list | fzf            # 配合 fzf 使用
projj list | xargs -I{} git -C {} pull   # 配合其他工具
```

## 配置文件

`~/.projj/config.toml`

```toml
base = ["/Users/x/projj"]
platform = "github.com"

[hooks]
postadd = "npm install --no-save"
preadd = "echo cloning..."

# hook 的额外配置通过 PROJJ_HOOK_CONFIG 环境变量传递
[hooks_config.postadd]
"github.com" = { name = "popomore", email = "sakura9515@gmail.com" }
"gitlab.com" = { name = "贯高", email = "guangao@example.com" }
```

从 JSON 改为 TOML：更适合配置文件，支持注释，手写友好。

字段说明：
- `base` — 仓库根目录，支持多个
- `platform` — 短写法的默认平台，默认 `github.com`
- `hooks` — hook 名到命令的映射
- `hooks_config` — hook 的额外配置，执行时通过环境变量 `PROJJ_HOOK_CONFIG` 传入

## 仓库发现

### 默认：文件系统扫描

直接扫描 base 目录，固定深度 3 层（host/owner/repo），检测 `.git` 目录存在即为仓库。

- 不维护额外状态，文件系统就是唯一真相
- 百级仓库扫描耗时 ~5-20ms（Rust + SSD），用户无感
- 所有需要仓库列表的命令（find / list / remove / run --all）都走这个路径

### 可选：SQLite 索引

当仓库量达到千级以上、或 base 目录在网络磁盘上（NFS）导致扫描延迟明显时，可开启索引加速。

配置：

```toml
[index]
enabled = true
```

索引文件：`~/.projj/index.db`（SQLite）

```sql
CREATE TABLE repos (
    path TEXT PRIMARY KEY,
    host TEXT NOT NULL,
    owner TEXT NOT NULL,
    repo TEXT NOT NULL
);
```

索引策略：
- 默认关闭，用户显式开启
- `projj add` / `projj remove` 后增量更新索引
- `projj list --rebuild` 强制从文件系统全量重建
- 索引丢失或损坏 → 自动降级为文件系统扫描，并提示重建
- 索引不是权威数据，文件系统才是；索引只是加速手段

代码层面通过 trait 抽象，扫描和索引实现同一接口：

```rust
trait RepoSource {
    fn list(&self) -> Result<Vec<Repo>>;
    fn find(&self, keyword: &str) -> Result<Vec<Repo>>;
}

struct FsScanner { /* 文件系统扫描 */ }
struct SqliteIndex { /* SQLite 索引 */ }

// 两者实现同一 trait，上层命令不感知差异
```

## 外部工具整合

### zoxide

检测：启动时检查 `zoxide` 是否在 PATH 中。

整合点：
- `projj add` 完成后自动调用 `zoxide add <path>`
- `projj init` 时将所有存量仓库灌入 zoxide
- zoxide 不可用时静默跳过，不影响功能

### fzf

检测：需要交互选择时检查 `fzf` 是否在 PATH 中。

整合点：
- `projj find` 将候选列表 pipe 给 fzf
- `projj remove` 多个匹配时 pipe 给 fzf 选择
- fzf 不可用时降级为内置的简单列表选择（数字编号）

## 输出设计

所有命令的输出分两层：
- **stdout** — 机器可读的结果（路径、列表），可 pipe
- **stderr** — 人类可读的状态信息（进度、提示、错误）

这样 `projj find projj` 的 stdout 只有路径，可以被 `cd $(projj find projj)` 直接消费。日志、进度条等信息走 stderr 不干扰。

## 数据迁移

从 v2（Node.js）迁移到 v3（Rust）：

1. 检测 `~/.projj/config.json` 存在 → 读取并转换为 `config.toml`
2. 检测 `~/.projj/cache.json` 存在 → 如果开启索引则导入到 SQLite
3. 迁移完成后保留旧文件不删除（用户手动清理）
4. `projj init` 时如果发现旧配置，提示是否迁移

## Rust 项目结构

```
projj/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口，CLI 参数解析（clap）
│   ├── config.rs            # 配置加载、迁移
│   ├── repo_source.rs       # RepoSource trait + FsScanner 实现
│   ├── index.rs             # SqliteIndex 实现（可选）
│   ├── git.rs               # git 操作（clone、url 解析）
│   ├── hook.rs              # hook 执行
│   ├── integration.rs       # zoxide/fzf 检测与调用
│   └── cmd/
│       ├── init.rs
│       ├── add.rs
│       ├── find.rs
│       ├── remove.rs
│       ├── run.rs
│       └── list.rs
└── tests/
```

核心依赖：
- `clap` — CLI 参数解析
- `serde` / `toml` / `serde_json` — 配置和索引序列化
- `dialoguer` — 交互式选择（fzf 降级方案）
- `walkdir` — 目录扫描
- `git2` — git 操作（可选，也可以直接调用 git 命令）

## 命令对照（v2 → v3）

| v2 | v3 | 说明 |
|----|-----|------|
| `projj init` | `projj init` | 增加 zoxide 同步 |
| `projj add <repo>` | `projj add <repo>` | 支持短写法、本地路径 |
| `projj find <repo>` | `projj find [keyword]` | 整合 fzf，stdout 输出路径 |
| `projj remove <repo>` | `projj remove [keyword]` | 整合 fzf 选择 |
| `projj import <dir>` | `projj add ./dir` | 合并到 add |
| `projj import --cache` | 删除 | 未来按需考虑 backup/restore |
| `projj sync` | `projj list --rebuild` | 重建索引（开启索引时）或无需对应 |
| `projj run <hook>` | `projj run <script>` | 支持直接执行命令 |
| `projj runall <hook>` | `projj run <script> --all` | 合并到 run |
| 无 | `projj list` | 新增，纯文本输出 |
