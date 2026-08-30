# 套件與發佈

Omarchy 是 Arch。給使用者的主路徑是 **AUR `-bin`**：預先編好的 x86_64 二進位，用 `yay` 裝。不要走 Flatpak／AppImage 當主格式（桌面裝得起來，但不是 Omarchy 慣例）。官方 `omarchy-pkgs` 是發行版第一方套件庫，第三方 App 進不去，AUR 才是入口。

## 使用者怎麼裝

AUR 上架之後：

```bash
yay -S omarchy-bible-bin
```

或從 GitHub Release 下載 `omarchy-bible-VERSION-x86_64.tar.gz`，把 `omarchy-bible` 放到 `PATH`。

## 發一版

1. 版本號在 workspace `Cargo.toml` 的 `[workspace.package] version`。
2. 打 tag 並推送（會觸發 [Release](../.github/workflows/release.yml)）：

```bash
git tag v0.1.0
git push origin v0.1.0
```

3. Action 會：
   - 在 Ubuntu 22.04 編 `cargo build --release -p omarchy-bible`（glibc 比 Arch 舊，Arch 上跑沒問題）
   - 打包 binary + `.desktop` + `LICENSE` 成 tarball
   - 開 GitHub Release 並附 SHA256
   - 若 repo 有設 AUR secrets，再更新 AUR 的 `omarchy-bible-bin`

手動重跑：Actions → Release → Run workflow。

## AUR 自動上架（可選）

在 GitHub repo Settings → Secrets and variables → Actions 加：

| Secret | 用途 |
|--------|------|
| `AUR_SSH_PRIVATE_KEY` | 能 push 到 `aur@aur.archlinux.org:omarchy-bible-bin.git` 的 SSH 私鑰 |
| `AUR_USERNAME` | AUR 帳號（commit 作者） |
| `AUR_EMAIL` | 同上的 email |

沒有 `AUR_SSH_PRIVATE_KEY` 時，Release 仍會出 GitHub 資產，只是跳過 AUR。第一次上架前要在 AUR 建空的 `omarchy-bible-bin` 或讓 deploy action 建（視該 action 版本而定）；第一次建議本機 `git clone ssh://aur@aur.archlinux.org/omarchy-bible-bin.git` 推一個 PKGBUILD。

本機預覽 PKGBUILD：

```bash
# 先有 dist/omarchy-bible-0.1.0-x86_64.tar.gz 與 SHA
VERSION=0.1.0
SHA=$(sha256sum dist/omarchy-bible-${VERSION}-x86_64.tar.gz | cut -d' ' -f1)
sed -e "s/@PKGVER@/${VERSION}/g" -e "s/@SHA256@/${SHA}/g" \
  packaging/omarchy-bible-bin/PKGBUILD.in
```

## 執行期依賴

`libxkbcommon`、`wayland`、`vulkan-icd-loader`、`fontconfig`。中文字型建議 `noto-fonts-cjk`（optdepends）。
