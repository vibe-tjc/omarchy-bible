package com.longbridge.nsh;

import android.app.Activity;
import android.app.Presentation;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.hardware.display.DisplayManager;
import android.view.Display;
import android.view.View;
import android.view.WindowManager;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.text.StaticLayout;
import android.text.TextPaint;
import android.text.Layout;
import android.widget.Toast;
import org.json.JSONObject;
import org.json.JSONArray;
import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;

/** App-owned host: NSH runs the controller, Android Presentation draws only
 * the verses on a display the OS marks as a presentation display. No cast
 * discovery, network server, overlay permission, or runtime patching. */
public final class BibleActivity extends Activity implements DisplayManager.DisplayListener {
    private NshView view;
    private DisplayManager displays;
    private final Handler handler = new Handler(Looper.getMainLooper());
    private Presentation audience;
    private AudienceText audienceText;
    private int displayId = -1;
    private JSONObject state;
    private String lastPayload = "";
    private String lastStatus = "";
    private File storage;
    private long lastStamp = -1, lastLength = -1;
    private boolean destroyed;

    private void status(String message) {
        if (!message.equals(lastStatus)) {
            lastStatus = message;
            Toast.makeText(this, message, Toast.LENGTH_LONG).show();
        }
    }

    @Override public void onCreate(Bundle saved) {
        super.onCreate(saved);
        DevActivity.edgeToEdge(this);
        displays = (DisplayManager)getSystemService(DISPLAY_SERVICE);
        // NSH and this host read the same private app-storage file. NSH owns
        // writes (atomic); the host only reads bounded projection commands.
        File dir = new File(getFilesDir(), "nsh/" + getPackageName());
        storage = new File(dir, "storage.json");
        // Do not resume a saved projection merely because the app reopened.
        lastPayload = readPayload();
        view = new NshView(this);
        view.useStandaloneLimits();
        view.setStorageDirectory(dir);
        setContentView(view);
        view.runRustApp("nsh");
        displays.registerDisplayListener(this, handler);
        handler.post(poll);
    }

    private String readPayload() {
        try {
            if (!storage.isFile() || storage.length() > 5 * 1024 * 1024) return "";
            long stamp = storage.lastModified(), length = storage.length();
            if (stamp == lastStamp && length == lastLength) return lastPayload;
            lastStamp = stamp; lastLength = length;
            return new JSONObject(new String(Files.readAllBytes(storage.toPath()), StandardCharsets.UTF_8))
                .optString("projection", "");
        } catch (Exception e) { return ""; } // Atomic replacement may race a read.
    }

    private final Runnable poll = new Runnable() {
        @Override public void run() {
            if (destroyed) return;
            String json = readPayload();
            if (!json.isEmpty() && json.length() < 262144 && !json.equals(lastPayload)) {
                lastPayload = json;
                try { state = new JSONObject(json); updateOutput(); }
                catch (Exception e) { status("投影資料無法讀取"); closeOutput(); }
            }
            handler.postDelayed(this, 200);
        }
    };

    private void closeOutput() {
        if (audience != null) { audience.dismiss(); audience = null; }
        audienceText = null; displayId = -1;
    }

    private void updateOutput() {
        if (state == null || !state.optBoolean("active")) { closeOutput(); return; }
        Display[] available = displays.getDisplays(DisplayManager.DISPLAY_CATEGORY_PRESENTATION);
        int index = Math.max(0, state.optInt("screen", -1));
        if (index >= available.length) {
            closeOutput();
            status("找不到外接顯示：請連接支援的 HDMI／USB-C 螢幕，或啟用系統支援的無線延伸顯示");
            return;
        }
        JSONArray slides = state.optJSONArray("slides");
        JSONObject slide = slides == null ? null : slides.optJSONObject(state.optInt("index"));
        if (slide == null) { closeOutput(); status("沒有可投影的經文"); return; }
        String text = slide.optString("text");
        String reference = slide.optString("reference");
        if (text.length() > 16000 || reference.length() > 2048) { closeOutput(); status("投影資料過長"); return; }
        Display target = available[index];
        if (audience == null || displayId != target.getDisplayId()) {
            closeOutput();
            try {
                audience = new Presentation(this, target);
                audienceText = new AudienceText(audience.getContext());
                audience.setContentView(audienceText);
                audience.getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
                audience.getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN, WindowManager.LayoutParams.FLAG_FULLSCREEN);
                audience.show();
                audience.getWindow().setLayout(WindowManager.LayoutParams.MATCH_PARENT, WindowManager.LayoutParams.MATCH_PARENT);
                audience.getWindow().setBackgroundDrawableResource(android.R.color.black);
                displayId = target.getDisplayId();
            } catch (WindowManager.InvalidDisplayException e) {
                closeOutput(); status("外接顯示已中斷"); return;
            }
        }
        audienceText.text = text;
        audienceText.reference = reference;
        audienceText.blank = state.optBoolean("blank");
        audienceText.invalidate();
        status("投影中：" + target.getName() + (audienceText.blank ? "（黑屏）" : ""));
    }

    @Override public void onDisplayAdded(int id) { updateOutput(); }
    @Override public void onDisplayRemoved(int id) {
        if (id == displayId) closeOutput();
        updateOutput();
    }
    @Override public void onDisplayChanged(int id) { updateOutput(); }
    @Override public void onConfigurationChanged(android.content.res.Configuration c) {
        super.onConfigurationChanged(c); DevActivity.systemBars(this);
    }
    @Override @SuppressWarnings("deprecation") public void onBackPressed() {
        if (view == null || !view.handleBack()) super.onBackPressed();
    }
    @Override protected void onDestroy() {
        destroyed = true; handler.removeCallbacks(poll);
        displays.unregisterDisplayListener(this); closeOutput();
        if (view != null) view.destroy();
        super.onDestroy();
    }

    /** Plain text only. The actual external surface size (not the phone's
     * density/rotation) determines the safe rectangle and fitted font. */
    private static final class AudienceText extends View {
        String text = "", reference = "";
        boolean blank;
        final TextPaint paint = new TextPaint(Paint.ANTI_ALIAS_FLAG);
        AudienceText(android.content.Context c) { super(c); setBackgroundColor(Color.BLACK); }
        private StaticLayout layout(String text, int width, float size) {
            paint.setTextSize(size);
            return StaticLayout.Builder.obtain(text, 0, text.length(), paint, Math.max(1, width))
                .setAlignment(Layout.Alignment.ALIGN_NORMAL).setIncludePad(false)
                .setLineSpacing(0, 1.25f).build();
        }
        @Override protected void onDraw(Canvas canvas) {
            canvas.drawColor(Color.BLACK);
            if (blank) return;
            float margin = Math.max(24, Math.min(getWidth(),getHeight()) * .06f);
            int width = Math.max(1, (int)(getWidth() - 2*margin));
            float refSize = Math.max(16, Math.min(getWidth(),getHeight()) * .038f);
            float height = getHeight() - 2*margin - refSize*2.4f;
            float low=12, high=Math.min(96,getHeight()*.095f);
            for (int i=0;i<18;i++) {
                float mid=(low+high)/2;
                if (layout(text,width,mid).getHeight() <= height) low=mid; else high=mid;
            }
            paint.setColor(Color.WHITE);
            StaticLayout body = layout(text,width,low);
            canvas.save(); canvas.translate(margin,margin+Math.max(0,(height-body.getHeight())/2));
            body.draw(canvas); canvas.restore();
            paint.setColor(Color.LTGRAY);
            StaticLayout ref=layout(reference,width,refSize);
            canvas.save(); canvas.translate(margin, getHeight()-margin-ref.getHeight());
            ref.draw(canvas); canvas.restore();
        }
    }
}
