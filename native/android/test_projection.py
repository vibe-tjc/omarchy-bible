#!/usr/bin/env python3
"""Real Android projection smoke test, explicitly restricted to an emulator.
The projection-enabled APK must already be installed. Restores the emulator's
previous simulated-display setting after the test; never touches a phone.
Usage: python3 native/android/test_projection.py emulator-5554 /tmp/bible-shots
"""
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

serial=sys.argv[1]
assert serial.startswith("emulator-"), "Use a dedicated emulator, not a physical device"
out=Path(sys.argv[2]); out.mkdir(parents=True,exist_ok=True)
adb=Path(os.environ["ANDROID_HOME"])/"platform-tools/adb"
pkg="io.github.vibe_tjc.OmarchyBible"
def run(*args, raw=False):
    result=subprocess.run([str(adb),"-s",serial,*args],check=True,capture_output=True,timeout=30)
    return result.stdout if raw else result.stdout.decode(errors="replace").strip()
def nodes():
    run("shell","uiautomator","dump","/sdcard/bible-projection-ui.xml")
    return list(ET.fromstring(run("shell","cat","/sdcard/bible-projection-ui.xml")).iter("node"))
def tap(text):
    for n in nodes():
        if text in (n.get("text"),n.get("content-desc")):
            x1,y1,x2,y2=map(int,re.findall(r"\d+",n.get("bounds")))
            run("shell","input","tap",str((x1+x2)//2),str((y1+y2)//2)); time.sleep(.5); return
    raise AssertionError("Control not visible: "+text)
def external_id():
    text=run("shell","dumpsys","SurfaceFlinger")
    m=re.search(r'Display (\d+) \(virtual, "Overlay #1"\)',text)
    assert m,"Simulated display not found"
    return m[1]
def shot(name):
    data=run("exec-out","screencap","-d",external_id(),"-p",raw=True)
    assert data.startswith(b"\x89PNG"), data[:100]
    (out/name).write_bytes(data)
def has_audience():
    text=run("shell","dumpsys","window","windows")
    # Native Presentation has its own TYPE_PRESENTATION window, not the
    # Activity or the controller's dialog.
    return "ty=PRESENTATION" in text

old=run("shell","settings","get","global","overlay_display_devices")
try:
    run("shell","am","force-stop",pkg)
    run("shell","settings","put","global","overlay_display_devices","1920x1080/240")
    time.sleep(2)
    run("shell","am","start","-n",pkg+"/com.longbridge.nsh.BibleActivity"); time.sleep(3)
    assert not has_audience(),"Opening the app must not resume saved projection"
    if not any(n.get("text")=="投影" for n in nodes()): tap("更多")
    tap("投影"); tap("開始投影"); time.sleep(1)
    assert has_audience(); shot("1080p.png")
    tap("黑屏"); shot("black.png")
    tap("恢復畫面"); tap("使用本章"); tap("下一張"); shot("verse-2.png")
    tap("多節：關閉（單節）"); shot("multiple.png")
    # Unplug while active: there is no audience window on the main screen.
    run("shell","settings","delete","global","overlay_display_devices"); time.sleep(2)
    assert not has_audience(),"Disconnected output must not fall back onto the controller"
    # Reconnect with a different aspect ratio: native layout must recalculate.
    run("shell","settings","put","global","overlay_display_devices","1024x768/160"); time.sleep(2)
    assert has_audience(); shot("4x3.png")
    tap("停止投影"); time.sleep(.5)
    assert not has_audience()
    # Force-stop/reopen must not restart it either.
    run("shell","am","force-stop",pkg)
    run("shell","am","start","-n",pkg+"/com.longbridge.nsh.BibleActivity"); time.sleep(2)
    assert not has_audience()
    print("PASS: separate audience surface, pages, multi-verse mode, black screen, disconnect, 4:3 reconnect, Stop, restart")
finally:
    if old in ("null",""):
        run("shell","settings","delete","global","overlay_display_devices")
    else:
        run("shell","settings","put","global","overlay_display_devices",old)
