# 經文投影（桌面與 Android）

## 操作

1. 點一下經文指定目前單節，或進入「選擇」模式選取經節。
2. 桌面按工具列「投影」；手機從「⋯ → 投影」進入；選取列也有「投影」。
3. 預設「多節：關閉（單節）」，每張只包含一節；選了數節時以「上一張／下一張」切換。
4. 開啟「多節」後，將選取的經節依閱讀順序排入投影片。長經文或多節超出容量時分頁，不會丟掉內容或無限制縮字。非連續選節會逐一列出節號，避免誤標成連續範圍。
5. 「使用本章」是明確的整章輸入；開始投影不會默默將全章送出。
6. 選擇目標螢幕後按「開始投影」。主畫面仍是控制台，外接端只顯示經文與出處。
7. 「上一節／下一節」直接換到前後一節，跨章、跨卷時會自動翻章，閱讀畫面也會跟著捲到那一節。有選取時，下一節從選取的最後一節往後算，上一節從第一節往前算。換節後回到單節，選取會清除；投影中會直接更新外接畫面。「上一張／下一張」仍是在目前投影片之間切換。
8. 「黑屏／恢復畫面」暫停顯示；「停止投影」關閉輸出。關閉控制面板不會停止正在進行的投影。

多節開關僅在這次執行期間生效；重新開啟 app 一律回到單節，也不會自動恢復先前投影。輸出依目前單語／中英對照模式。選取列決定經節，閱讀模式決定譯本。

## 控制面板與快捷鍵

面板由上到下依操作順序排列：狀態（未投影／投影中／黑屏中）與預覽、上一節／下一節（長經文分頁時中間才出現上一張／下一張）、主要動作（未投影時只有「開始投影」，投影中換成「黑屏」與「停止投影」），最後是較少改的「內容」（單節／多節、目前經文／整章）與「輸出螢幕」。

面板開啟時，鍵盤與簡報遙控器可直接操作：`→`／PageDown 下一張，沒有下一張時換下一節；`←`／PageUp 反向；`B` 黑屏／恢復。Hyprland 有時會把焦點交給投影視窗；這時在投影視窗按同樣的鍵（另含空白鍵、↑↓）也會轉回控制台處理。投影視窗不顯示滑鼠游標。

## 外接螢幕版面

- 依外接端的實際長寬比、解析度及旋轉重新排版，不拉伸手機畫面。
- 預設黑底白字，四周保留 6% 安全邊界，避免投影機 overscan 切掉經文。
- 字級依可用空間調整，中文／英文均為純文字，不能執行 HTML 或載入遠端圖片。
- 已測桌面輸出的 1920×1080、1024×768、2560×1080、1080×1920、640×480 邊界。
- Android 模擬器測過獨立 1920×1080 顯示、切頁、多節、全黑、拔除後不佔主螢幕，以及重新接上 1024×768。

## 桌面（Linux／macOS）

NSH 0.4.0 尚未提供多視窗 API。app 自己附帶 Qt 6 audience helper，不修改 NSH 套件；reader 與 helper 使用私有 stdin pipe 傳純文字 JSON。controller 關閉後 pipe EOF 會結束 helper。

```bash
# Arch 編譯需要 qt6-base（qmake6）；macOS：brew install qt
cargo build -p omarchy-bible
bash packaging/scripts/build-projector.sh debug
cargo run -p omarchy-bible
```

兩個 binary 必須在同一目錄：`omarchy-bible`、`omarchy-bible-projector`。原始碼開發也能用 `OMARCHY_BIBLE_PROJECTOR=/完整路徑/omarchy-bible-projector` 指定。

自動模式選擇非系統主螢幕；只接一台時不會搶佔它。可明確選螢幕 0 預覽，其他編號按 Qt 的 display list 排序，狀態列回報實際螢幕名稱與尺寸。操作台若不在系統主螢幕，請手動選擇正確目標。

Linux 的 Wayland 視窗位置受 compositor 管理，`QWindow::setScreen` 與全螢幕是請求，最終仍依 compositor 政策；必要時手動把投影視窗移至目標螢幕。macOS 的實體外接顯示需另外實機驗證。

拔除目標螢幕後會隱藏／重新選擇仍存在的外接螢幕，不會在自動模式回退主螢幕。重新接上時會等約 0.8 秒、待 compositor 備妥該輸出再放上投影；已用 Hyprland 虛擬螢幕（`hyprctl output create headless`）測過自動選擇、全螢幕、翻頁鍵、黑屏與三次拔插。

## Android（手機／平板）

使用 Android 原生 `DisplayManager` 與 `Presentation`，不是鏡像、瀏覽器或遠端服務。需作業系統提供 presentation display：

- 裝置與轉接器都支援視訊輸出的 USB-C／HDMI。
- 或系統支援、且提供獨立顯示的無線投放。
- 只有鏡像的裝置、或不支援視訊輸出的 USB-C，不會因此變成可用的延伸螢幕。此版本沒有內建 Google Cast／Chromecast 協定。

自動模式選第一個 presentation display；手動編號只列外接端。沒有可用顯示時會在手機顯示提示。投影畫面不會退回手機主畫面。

### 必須使用此打包指令

```bash
# SDK、NDK、Java 設定見 android.md
bash packaging/scripts/build-android.sh --abi x86_64 --abi arm64-v8a
adb -s 手機序號 install -r 'crates/omarchy-bible/.nsh/out/聖經.apk'
```

此指令先由 NSH 編譯 Rust runtime，再編譯 app 自己的 `BibleActivity`，加入第二份 DEX 與 app manifest，重新作 16 KiB native library alignment 並以 debug key 簽署。它不覆寫已安裝的 NSH runtime，也不需要網路或懸浮視窗權限。

**直接執行 `nsh package` 只會產生標準 NSH host，沒有外接投影功能。** 要含投影功能的 APK 必須使用上面的 wrapper。這個 wrapper 目前只做本機 debug-key 測試包，不能直接拿去商店正式上架。

主程式用 NSH storage 保存投影指令；Android host 僅讀取同一個 app 的私有、原子更新檔案。只有內容修改時才重讀；新啟動 session 不會執行上次留下的投影指令。Activity 被銷毀時會清除 audience window。

### 模擬外接顯示測試

```bash
python3 native/android/test_projection.py emulator-5554 /tmp/bible-projection-shots
```

需先安裝上面的 APK。測試嚴格限制 `emulator-*`，拒絕實體手機，並還原模擬器原本的 external-display 設定。測試截圖只包含 app 及合成顯示，不會抓使用者桌面。

## 測試

```bash
cargo test --workspace --locked
bash packaging/scripts/build-projector.sh debug
python3 native/projector/test_projector.py target/debug/omarchy-bible-projector
```

原生畫面測試檢查不同長寬比的文字是否超出安全區、無螢幕時不搶主畫面、手動預覽、黑屏、錯誤螢幕編號，以及 controller 消失後的 helper 清理。模擬器測試不能取代實體投影機的視訊輸出與 overscan 驗證。
