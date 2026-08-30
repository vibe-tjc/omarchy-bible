# omarchy-bible（聖經）

本機優先的 GPUI 桌面經文閱讀器。側邊欄可選 66 卷，主畫面顯示**中英對照**、按節對齊。

- 中文：**和合本 1919 神版**（公有領域；`上帝`→`神`）
- 英文：**King James Version**（公有領域）
- 執行時不上網；經文嵌入二進位。
- 側邊欄：舊約／新約全書卷，點選從第 1 章開啟。
- 章節：上一章／下一章、章號列跳轉；快捷鍵 `[` `]`（卷首／卷尾可跨卷）。
- `Ctrl+Q` 結束。

A local-first GPUI desktop reader. Traditional Chinese CUV 1919 神版 over English KJV, with 66-book and chapter navigation. No network at runtime.

## 如何執行 / How to run

需要 Rust（已在 Arch／Omarchy 上以 rustup 安裝即可）：

```bash
cd /home/sy/Projects/omarchy-bible
cargo run -p omarchy-bible
```

測試與編譯：

```bash
cargo test
cargo build -p omarchy-bible
```

Linux 編譯 GPUI 通常需要 Vulkan／Wayland 相關函式庫（多數 Omarchy 桌面已具備），例如：

- `libxkbcommon`
- `wayland`
- Vulkan loader（`vulkan-icd-loader`）與可用的 ICD

若 `cargo build` 因缺少系統套件失敗，用 `pacman -S` 安裝錯誤訊息中的套件即可。

## 授權 / Licenses

| 內容 | 授權 |
|------|------|
| 本專案程式碼 | [Apache-2.0](LICENSE) |
| 和合本 1919（CUV）經文 | 公有領域 public domain |
| King James Version（KJV）經文 | 公有領域 public domain |
| **新標點和合本（CUNP）** | **未收錄**（香港聖經公會／聯合聖經公會著作權，HKBS／UBS） |

Do **not** add CUNP / 新標點和合本 text to this repository.

經文來源（建置時僅 curl 單檔，未 clone 整個 bible-data）：

- https://raw.githubusercontent.com/midvash/bible-data/main/versions/zh/cuv/cuv.json
- https://raw.githubusercontent.com/midvash/bible-data/main/versions/en/kjv/kjv.json

## 專案結構

```
crates/bible-core     資料模型、66 卷目錄、章節載入、測試
crates/bible-ui       GPUI：側邊欄 + 章節視圖
crates/omarchy-bible  桌面程式進入點
data/cuv-kjv.json     對齊後的全書（神版 + KJV，嵌入 bible-core）
data/genesis-1.json   創世記 1 快照
```

## 範圍外（之後再說）

搜尋、歷史、即時 Omarchy 主題、WEB／CUNP 經文包。
