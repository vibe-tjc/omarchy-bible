#!/usr/bin/env bash
# Build NSH's Rust app, then add the app-owned Android Presentation host.
# No installed NSH source, runtime, or prebuilt DEX is modified.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APP="$ROOT/crates/omarchy-bible"
SDK="${ANDROID_HOME:?Set ANDROID_HOME to the Android SDK}"
TOOLS="$SDK/build-tools/35.0.1"
ANDROID="$SDK/platforms/android-35/android.jar"
NSH_BIN="$(python3 -c 'import pathlib,shutil; print(pathlib.Path(shutil.which("nsh")).resolve())')"
NSH_DIST="$(dirname "$(dirname "$NSH_BIN")")"
AAR="${NSH_ANDROID_AAR:-$NSH_DIST/embed/android/nsh.aar}"
[[ -f "$AAR" ]] || { echo 'Set NSH_ANDROID_AAR to nsh.aar from the matching NSH 0.4.0 distribution' >&2; exit 1; }
cd "$APP"
nsh package --target android "$@"
WORK="$APP/.nsh/projection-host"
rm -rf "$WORK"; mkdir -p "$WORK/classes" "$WORK/dex"
unzip -p "$AAR" classes.jar > "$WORK/nsh.jar"
SOURCES=()
while IFS= read -r file; do SOURCES+=("$file"); done < <(find "$ROOT/native/android" -name '*.java')
javac -source 8 -target 8 -encoding UTF-8 -cp "$ANDROID:$WORK/nsh.jar" -d "$WORK/classes" "${SOURCES[@]}"
CLASSES=()
while IFS= read -r file; do CLASSES+=("$file"); done < <(find "$WORK/classes" -name '*.class')
"$TOOLS/d8" --lib "$ANDROID" --classpath "$WORK/nsh.jar" --min-api 28 --output "$WORK/dex" "${CLASSES[@]}"
python3 - "$APP/nsh.json" "$WORK/AndroidManifest.xml" <<'PY'
import json,sys
from xml.sax.saxutils import escape
app=json.load(open(sys.argv[1]))
pkg=escape(app['id']); name=escape(app['name']); version=escape(app['version']); build=int(app['build'])
manifest=f'''<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="{pkg}" android:versionCode="{build}" android:versionName="{version}">
<uses-sdk android:minSdkVersion="28" android:targetSdkVersion="35"/>
<application android:label="{name}" android:extractNativeLibs="false" android:enableOnBackInvokedCallback="true" android:supportsRtl="true" android:theme="@android:style/Theme.Material.NoActionBar">
<meta-data android:name="com.longbridge.nsh.RUST" android:value="true"/>
<activity android:name="com.longbridge.nsh.BibleActivity" android:exported="true" android:windowSoftInputMode="adjustResize" android:configChanges="orientation|screenSize|screenLayout|smallestScreenSize|keyboard|keyboardHidden|navigation|uiMode|density|fontScale|locale|layoutDirection">
<intent-filter><action android:name="android.intent.action.MAIN"/><category android:name="android.intent.category.LAUNCHER"/></intent-filter>
</activity></application></manifest>'''
open(sys.argv[2],'w').write(manifest)
PY
"$TOOLS/aapt2" link -I "$ANDROID" --manifest "$WORK/AndroidManifest.xml" -o "$WORK/resources.apk"
python3 - "$APP/.nsh/out" "$WORK" <<'PY'
import json,sys,zipfile,pathlib
out=pathlib.Path(sys.argv[1]); work=pathlib.Path(sys.argv[2])
name=json.load(open('nsh.json'))['name']
base=out/(name+'.apk')
with zipfile.ZipFile(base) as src, zipfile.ZipFile(work/'resources.apk') as res, zipfile.ZipFile(work/'unsigned.apk','w') as dst:
    for info in src.infolist():
        if info.filename in ('AndroidManifest.xml','resources.arsc') or info.filename.startswith('META-INF/'): continue
        dst.writestr(info,src.read(info.filename))
    for info in res.infolist(): dst.writestr(info,res.read(info.filename))
    dst.write(work/'dex/classes.dex','classes2.dex',compress_type=zipfile.ZIP_DEFLATED)
print('Output uses the debug signing key; not a store release.')
PY
"$TOOLS/zipalign" -P 16 -f 4 "$WORK/unsigned.apk" "$WORK/aligned.apk"
NAME="$(python3 -c 'import json;print(json.load(open("nsh.json"))["name"])')"
"$TOOLS/apksigner" sign --ks "$HOME/.android/debug.keystore" --ks-key-alias androiddebugkey --ks-pass pass:android --key-pass pass:android --out ".nsh/out/$NAME.apk" "$WORK/aligned.apk"
"$TOOLS/apksigner" verify ".nsh/out/$NAME.apk"
echo "Built $APP/.nsh/out/$NAME.apk (with native external-display projection)"
