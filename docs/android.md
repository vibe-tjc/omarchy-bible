# Android 開發與測試

使用 Native Shell 0.4.0，Android 9（API 28）以上。手機與平板共用同一個 APK。

## 環境

需要 JDK 21、Android SDK、NDK 與 Rust Android targets。

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk
export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$ANDROID_HOME/cmdline-tools/latest/bin:$JAVA_HOME/bin:$PATH"

# 先自行閱讀並接受 Google 的 SDK 授權。
sdkmanager --licenses
sdkmanager 'platforms;android-35' 'build-tools;35.0.1' 'platform-tools' \
  'ndk;29.0.14206865' 'emulator' 'system-images;android-35;google_apis;x86_64'
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```

## 模擬器

```bash
avdmanager create avd -n bible_phone -k 'system-images;android-35;google_apis;x86_64' -d pixel_8
avdmanager create avd -n bible_tablet -k 'system-images;android-35;google_apis;x86_64' -d pixel_tablet
emulator -avd bible_phone -gpu swiftshader_indirect
# 平板改用 -avd bible_tablet。每次啟動一台可節省記憶體。
adb devices
```

需要可用的 `/dev/kvm` 才有 CPU 硬體加速。若 GPU 的 host renderer 不相容，上述軟體繪圖選項可作為替代。

## 建置、安裝

```bash
# 在 repository 根目錄執行；此 wrapper 才會加入原生外接顯示 host。
bash packaging/scripts/build-android.sh --abi x86_64 --abi arm64-v8a
adb -s 手機序號 install -r 'crates/omarchy-bible/.nsh/out/聖經.apk'
adb -s 手機序號 shell am start -n io.github.vibe_tjc.OmarchyBible/com.longbridge.nsh.BibleActivity
```

APK 包含 x86_64（模擬器）與 ARM64（大部分實體手機）。預設使用 debug key 簽署，適合本機測試，不能當作商店正式發佈；正式發佈需要自己的 release keystore。產物位於 `crates/omarchy-bible/.nsh/out/聖經.apk`，不應提交 `.nsh/` 或 `target/`。

外接顯示投影的操作、打包限制與模擬器測試見 [projection.md](projection.md)。直接 `nsh package` 的標準 host 沒有原生投影功能。

## 手機與平板版面

- 小於 600 pt：兩列精簡標題列，工具收進「⋯」，書卷、章節、搜尋和設定為全螢幕面板。
- 600 pt 以上：保留桌面工具列與浮動面板；平板使用更大的觸控按鈕。
- 視窗寬度決定版面，旋轉與分割畫面會重新排版。
- 希伯來文的詞詳情在手機位於下方，平板位於側邊。
- 內容避開狀態列與導覽列，觸控按鈕至少 44 pt。
- Android 返回鍵可關面板、離開選取模式、返回經文。
- Android 設定透過 NSH storage 存在 app 私有資料中；桌面仍使用原有 `settings.json`，不會搬動使用者設定。

## 測試

```bash
cargo test --workspace --locked
```

`tests/reader.rs` 包含手機（411×914）、平板（1280×800）和旋轉流程；`tests/storage.rs` 測試 app storage 跨引擎重啟保存，以及損壞設定的預設值。Headless 測試不能證明實際繪圖或輸入法正確；仍須在模擬器檢查鍵盤、面板、返回、捲動與重啟後的設定。
