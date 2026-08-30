# omarchy-bible（聖經）

Phase 1：本機優先的 GPUI 桌面經文閱讀器。開啟視窗顯示**創世記第 1 章**，中英對照、按節對齊。

- 中文：**和合本 1919**（公有領域）
- 英文：**King James Version**（公有領域）
- 執行時不上網；經文嵌入二進位。
- `Ctrl+Q` 結束。

A local-first GPUI desktop reader. Phase 1 opens Genesis 1, verse-aligned Traditional Chinese (CUV 1919) over English (KJV). No network at runtime.

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

- https://raw.githubusercontent.com/midvash/bible-data/main/versions/zh/cuv/books/Gen.json
- https://raw.githubusercontent.com/midvash/bible-data/main/versions/en/kjv/books/Gen.json

## 專案結構

```
crates/bible-core     資料模型、創世記 1 載入、測試
crates/bible-ui       GPUI 章節視圖
crates/omarchy-bible  桌面程式進入點
data/genesis-1.json   對齊後的創世記 1（嵌入 bible-core）
data/zh/cuv/Gen.json  完整創世記 CUV 1919
data/en/kjv/Gen.json  完整創世記 KJV
```

## 範圍外（之後再說）

全書 66 卷導覽、搜尋、歷史、即時 Omarchy 主題、WEB／CUNP 經文包。
