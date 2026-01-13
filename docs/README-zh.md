# Osu! Beatmap Downloader

一款开箱即用的桌面应用，用于搜索并下载 osu! 谱面合集。提供简洁的图形界面，可按谱师或自定义关键字筛选，选择游戏模式，并以并发方式下载多个谱面，带有进度指示。

<div align="center">
  <img src="./image.png" alt="alt text" width="400">
</div>

## ✨ 功能
- 按谱师名称和/或自定义关键字搜索
- 选择游戏模式（std、taiko、catch、mania）
- 若检测到本地 osu 的 Songs 文件夹，自动跳过重复谱面

## 🚀 使用
- 输入谱师名称和/或自定义关键字
  - 查询格式与官网搜索一致，参考 [此处](https://osu.ppy.sh/beatmapsets?s=any)
  - 不建议仅用自定义查询下载（例如只用“status=r”），因为匹配与筛选将耗费较长时间
- 选择游戏模式
- 点击“Download”并等待完成
- 下载进度在终端可见
- 下载的文件保存在本地 `./Songs` 目录

## ⚙️ 配置
- 可选：设置 `OSU_PATH` 以便应用定位你的游戏目录，从而跳过已安装的谱面。如果未设置，应用会尝试在多个默认路径中查找。
  - Windows 示例：
    ```bash
    set OSU_PATH=D:\Games\osu\osu.exe
    ```
  - Linux/macOS 示例：
    ```bash
    export OSU_PATH=/path/to/osu
    ```
- 也可以选择把应用放在osu.exe所在目录下

## 🛠️ 构建
面向开发者。

### 🔧 环境要求
- Rust（stable）和 Cargo

### 📦 安装
```bash
rustup install stable
git clone https://github.com/nicelife1450/osu-downloader
cd osu-downloader
cargo build
```

## 🐞 故障排查
- “Not signed in yet”：等待几秒，应用会自动登录。
- “No beatmaps found”：调整谱师/关键字或游戏模式。
- 下载错误：检查网络连接，稍后重试。

## 📝 说明
- 谱面从公共镜像下载，仅供个人非商业使用。
- 请遵守 osu! 社区规则及镜像站服务条款。

## 🙏 致谢
- 感谢 [Sayobot](https://osu.sayobot.cn/home) 和 [rosu-v2](https://github.com/MaxOhn/rosu-v2) 提供优秀的 API。
