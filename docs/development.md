# 開發說明

給要編譯、改程式或更新經文資料的人。使用方式見 [README](../README.md)。

## 執行與測試

```bash
cd /path/to/omarchy-bible
cargo run -p omarchy-bible
cargo test
cargo build -p omarchy-bible
```

Linux 上 GPUI 通常需要：

- `libxkbcommon`
- `wayland`
- Vulkan loader（`vulkan-icd-loader`）與可用的 ICD

Omarchy 桌面多半已具備。若 `cargo build` 因缺少系統套件失敗，依錯誤訊息用 `pacman -S` 安裝即可。

## 專案結構

```
crates/bible-core     資料模型、66 卷目錄、章節載入、經文搜尋、測試
crates/bible-ui       GPUI：側邊欄、閱讀區、設定、搜尋
crates/omarchy-bible  桌面程式進入點
data/cuv-kjv.json     對齊後的全書（神版 + KJV，編譯時嵌入）
data/genesis-1.json   創世記 1 快照
resources/            .desktop 等包裝檔
```

對照閱讀用 translation lane（`TranslationId` 加經文），不是寫死的中英兩個欄位。目前內建兩個 lane：`Cuv1919`、`Kjv`。再加譯本時擴充 lane 與資料即可。

## 設定檔

使用者設定寫在 `~/.config/omarchy-bible/settings.json`（若設定了 `XDG_CONFIG_HOME` 則用該目錄）。

欄位：`theme`（dark / light / system）、`font_size`、`view_mode`。

跟隨系統時的解析順序：

1. Omarchy 主題目錄 `~/.local/state/omarchy/current/theme`（或 `XDG_STATE_HOME`）：有 `light.mode` 視為淺色；否則讀 `colors.toml` 的 `mode`
2. 否則用 gpui 的視窗外觀（Linux XDG portal）
3. 再不行就回退深色

## 快捷鍵

| 按鍵 | 作用 |
|------|------|
| `/` | 開啟搜尋（搜尋開著時可正常輸入斜線） |
| `Esc` | 先關搜尋，再關設定 |
| `[` `]` | 上一章／下一章（搜尋開著時停用） |
| `Ctrl+Q` | 結束 |

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

Omarchy／Arch 發佈見 [packaging.md](packaging.md)。
