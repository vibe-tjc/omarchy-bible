# 聖經（omarchy-bible）

Omarchy 上的桌面聖經閱讀器：離線、中英對照、以閱讀為主。

A local-first Bible reader for Omarchy. Traditional Chinese CUV 1919 神版 alongside the King James Version.

## 安裝

Omarchy／Arch 建議裝 AUR 的預編套件（上架後）：

```bash
yay -S omarchy-bible-bin
```

或從 [Releases](https://github.com/vibe-tjc/omarchy-bible/releases) 下載 Linux 的 `omarchy-bible-*-x86_64.tar.gz`，或 macOS（Apple Silicon）的 `omarchy-bible-*-macos-arm64.tar.gz`。

從原始碼跑（Linux 與 macOS）：

```bash
cargo run -p omarchy-bible
```

套件格式與發佈流程見 [docs/packaging.md](docs/packaging.md)。開發細節見 [docs/development.md](docs/development.md)。

## 經文

- 中文：**和合本 1919 神版**（公有領域）
- 英文：**King James Version**（公有領域）
- 全書 66 卷都在本機，開啟後不需要網路
- **沒有**收錄《新標點和合本》（CUNP），該譯本仍受香港聖經公會／聯合聖經公會著作權保護

## 功能

**閱讀**
- 側邊欄依舊約／新約列出全書卷，點選從第 1 章開始
- 章號列跳章；`[` `]` 上一章／下一章，到卷首卷尾會跨卷
- **單語**或**對照**：單語可選和合本或 KJV；對照預設中文在上、英文在下。之後若再加譯本，對照可以一次看多個語系

**搜尋**
- 標題列「搜尋」，或按 `/`
- 預設搜你正在看的譯本；也可以改搜全部譯本
- 中文依原文片段（例如「神愛世人」），英文不區分大小寫
- 點結果會跳到那一章並標出經節

**設定**
- 標題列「設定」
- 主題：深色、淺色、跟隨系統
- 字級：小／中／大，或逐步加減
- `Esc` 先關搜尋，再關設定

**其他**
- `Ctrl+Q` 或 `Cmd+Q` 離開
