# 開發說明

給要編譯、改程式或更新經文資料的人。使用方式見 [README](../README.md)。

## 執行與測試

```bash
cd /path/to/omarchy-bible
cargo run -p omarchy-bible
cargo test
cargo build -p omarchy-bible
```

介面用 [Native Shell（NSH）](https://nsh.dev) 0.4 寫成：Linux 用 GTK 4 原生元件，macOS 用 AppKit。

Linux 編譯需要 GTK 4.14 以上、GStreamer 與 clang（Arch：`gtk4 gstreamer gst-plugins-base-libs gst-plugins-bad-libs clang`；Ubuntu 24.04：`libgtk-4-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev clang libclang-dev`）。Omarchy 桌面多半已具備。

macOS 需要 14 以上與 Xcode command line tools，`cargo run -p omarchy-bible` 即可。CI 的 `macos-latest` job 會跑 `cargo test --workspace --locked` 與 `cargo build --release -p omarchy-bible --locked`。

`cargo test` 會在真的 NSH 引擎上無頭（headless）跑使用流程（`crates/omarchy-bible/tests/reader.rs`）：換章、跨卷、選書、跳轉、搜尋、Esc、單語／對照、選取複製、設定存檔、希伯來文頁、長章節的列回收。測試各用自己的暫存設定檔，不會動到你的設定。

除錯版可以用 `OMARCHY_BIBLE_SCREEN` 直接開到某個畫面，方便截圖檢查外觀：`books`、`chapters`、`search`、`settings`、`select`、`hebrew`，或一段經文如 `約3:16`。

```bash
OMARCHY_BIBLE_SCREEN=hebrew cargo run -p omarchy-bible
```

## 專案結構

```
crates/bible-core     資料模型、66 卷目錄、章節載入、經文搜尋、測試（不依賴 UI）
crates/omarchy-bible  NSH 桌面程式
  src/settings.rs     設定檔、主題色盤、Omarchy colors.toml 偵測
  src/store.rs        閱讀狀態（signals）與唯一的寫入方法
  src/ui/             主題 token 與 app 自己的控制元件（chip、按鈕、欄位、面板）
  src/pages/          閱讀區、書卷／章節選擇、搜尋、設定、希伯來文頁
  tests/reader.rs     無頭使用流程測試
  nsh.json            `nsh run` / `nsh package` 用的設定
data/cuv-kjv.json     對齊後的全書（神版 + KJV，編譯時嵌入）
data/genesis-1.json   創世記 1 快照
data/he/              MorphHB 樣本（Gen 1）
resources/            Linux `.desktop` 等包裝檔（macOS 不使用）
```

對照閱讀用 translation lane（`TranslationId` 加經文），不是寫死的中英兩個欄位。目前內建兩個 lane：`Cuv1919`、`Kjv`。再加譯本時擴充 lane 與資料即可。

## 設定檔

- Linux：`~/.config/omarchy-bible/settings.json`（若設定了 `XDG_CONFIG_HOME` 則用該目錄）
- macOS：`~/Library/Application Support/omarchy-bible/settings.json`

欄位：`theme`（dark / light / system）、`font_size`、`view_mode`。

跟隨系統時：

- **Linux**：先看 Omarchy 主題目錄 `~/.local/state/omarchy/current/theme`（或 `XDG_STATE_HOME`）：有 `light.mode` 視為淺色；否則讀 `colors.toml` 的 `mode`，顏色也取自 `colors.toml`。每 1.5 秒檢查一次，切換 Omarchy 主題時會即時跟著變。沒有 Omarchy 就用系統外觀（深／淺）。
- **macOS**：只用系統外觀，**不會**讀 Omarchy 路徑。

## 快捷鍵

| 按鍵 | 作用 |
|------|------|
| `/` | 開啟搜尋（搜尋開著時可正常輸入斜線） |
| `Esc` | 依序：離開希伯來文頁、離開選擇模式、關閉開著的面板（搜尋、設定、選書、選章） |
| `[` `]` | 上一章／下一章（搜尋開著時停用） |
| `Ctrl+Q` / `Cmd+Q` | 結束 |

## 經文資料

執行時以 `include_str!` 嵌入，不需網路。細節也見 [data/README.md](../data/README.md)。

- `data/cuv-kjv.json` — 66 卷對齊：和合本 1919 **神版**（匯入時「上帝」換成「神」）+ KJV
- `data/genesis-1.json` — 創世記第 1 章快照
- 來源（建置時只 curl 單檔，不要 clone 整個 bible-data）：
  - https://raw.githubusercontent.com/midvash/bible-data/main/versions/zh/cuv/cuv.json
  - https://raw.githubusercontent.com/midvash/bible-data/main/versions/en/kjv/kjv.json
- 未轉換的原始 dump 放 `data/raw/`（已 gitignore）

**不要**把《新標點和合本》（CUNP）經文放進這個 repository。

## 授權

| 內容 | 授權 |
|------|------|
| 本專案程式碼 | [Apache-2.0](../LICENSE) |
| 和合本 1919（CUV）經文 | 公有領域 |
| King James Version（KJV）經文 | 公有領域 |
| 新標點和合本（CUNP） | 未收錄（HKBS／UBS） |

## 之後再說

閱讀歷史、WEB 或其他公有領域譯本、經授權後的 CUNP 經文包。

## 套件

Omarchy／Arch 與 macOS Release 資產見 [packaging.md](packaging.md)。
