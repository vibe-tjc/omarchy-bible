#!/usr/bin/env python3
"""Exercise the real native helper (GTK 4 on Linux, AppKit on macOS) without
opening windows on the user's desktop. Uses only Python's standard library.
Checks pipe lifecycle and rendered fit on 16:9, 4:3, ultrawide, and portrait
surfaces. PNGs are temporary test artifacts.

`--render-test` paints with the helper's own drawing code into an image, and
OMARCHY_BIBLE_PROJECTOR_HEADLESS runs the real pipe and screen-choice logic
against one simulated display, so neither needs a display server (CI).
"""
import json
import os
from pathlib import Path
import selectors
import struct
import subprocess
import sys
import tempfile
import time

exe = str(Path(sys.argv[1]).resolve())
env = dict(os.environ, OMARCHY_BIBLE_PROJECTOR_HEADLESS="1")
with tempfile.TemporaryDirectory(prefix="bible-projector-test-") as temp:
    for width, height in [(1920,1080),(1024,768),(2560,1080),(1080,1920),(640,480)]:
        path=Path(temp)/f"{width}x{height}.png"
        result=subprocess.run([exe,"--render-test",str(width),str(height),str(path),"--long"],
                              env=env,capture_output=True,timeout=15)
        assert result.returncode==0,(width,height,result.stderr,result.stdout)
        assert b"FIT OK" in result.stdout,(width,height,result.stdout)
        data=path.read_bytes()
        assert data[:8]==b"\x89PNG\r\n\x1a\n"
        assert struct.unpack(">II",data[16:24])==(width,height)
        print(f"render fits: {width}x{height}")

    process=subprocess.Popen([exe],env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL)
    try:
        def command(**kwargs):
            state=dict(active=True,blank=False,screen=-1,index=0,
                       slides=[dict(reference="Test 1:1",text="Plain text <img src='https://invalid'>")])
            state.update(kwargs)
            process.stdin.write((json.dumps(state)+"\n").encode()); process.stdin.flush()
        def expect(fragment):
            # Use an unbuffered descriptor read so Python buffering cannot
            # hide a second status line from select().
            selector=selectors.DefaultSelector(); selector.register(process.stdout,selectors.EVENT_READ)
            received=b""; deadline=time.monotonic()+10
            try:
                while time.monotonic()<deadline:
                    if selector.select(.1):
                        part=os.read(process.stdout.fileno(),8192)
                        if not part: break
                        received+=part
                        if fragment.encode() in received: return
                raise AssertionError(received.decode(errors="replace"))
            finally: selector.close()
        expect("準備")
        command()  # headless has one display: never auto-take the controller.
        expect("找不到外接螢幕")
        command(screen=0)
        expect("投影中")
        command(screen=0,blank=True)
        expect("黑屏")
        command(screen=0,slides=[dict(reference="r",text="x"*16001)])
        expect("投影資料過長")
        command(screen=0,index=5)
        expect("沒有可投影的經文")
        command(screen=999)
        expect("找不到外接螢幕")
        command(active=False)
        process.stdin.close()
        assert process.wait(timeout=5)==0, "helper must quit when its controller disappears"
        print("pipe: explicit preview, no automatic fallback, black screen, limits, invalid display, EOF cleanup")
    finally:
        if process.poll() is None: process.kill(); process.wait()
