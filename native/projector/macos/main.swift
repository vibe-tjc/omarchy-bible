// Audience-only desktop output for macOS, drawn with AppKit. Receives
// plain-text JSON lines on a private stdin pipe owned by the reader, never
// shell commands or rich text. No network access. The Linux twin is
// native/projector/gtk; both speak the same protocol and status lines.
import AppKit

let noScreen = "找不到外接螢幕；請連接螢幕，或明確選擇螢幕 0 預覽"

func status(_ line: String) {
    FileHandle.standardOutput.write((line + "\n").data(using: .utf8)!)
}

struct Slide { var text = "", reference = "", blank = false }

enum Plan { case off, fail(String), show(Int, Slide) }

/// Screen 0 is the operator's (the menu-bar screen). Auto (-1) takes the
/// first other screen and never falls back to the only one.
func plan(_ state: [String: Any], screens: Int) -> Plan {
    guard state["active"] as? Bool == true else { return .off }
    var index = (state["screen"] as? NSNumber)?.intValue ?? -1
    if index < 0 { index = screens > 1 ? 1 : -1 }
    guard index >= 0, index < screens else { return .fail(noScreen) }
    let slides = state["slides"] as? [[String: Any]] ?? []
    let at = (state["index"] as? NSNumber)?.intValue ?? 0
    guard at >= 0, at < slides.count else { return .fail("沒有可投影的經文") }
    let text = slides[at]["text"] as? String ?? "", reference = slides[at]["reference"] as? String ?? ""
    guard text.count <= 16000, reference.count <= 2048 else { return .fail("投影資料過長") }
    return .show(index, Slide(text: text, reference: reference, blank: state["blank"] as? Bool ?? false))
}

func font(_ size: CGFloat) -> NSFont {
    NSFont(name: "PingFang TC", size: size) ?? NSFont.systemFont(ofSize: size)
}

func attributed(_ text: String, _ size: CGFloat, _ color: NSColor, _ align: NSTextAlignment) -> NSAttributedString {
    let paragraph = NSMutableParagraphStyle()
    paragraph.lineBreakMode = .byWordWrapping
    paragraph.alignment = align
    return NSAttributedString(string: text, attributes: [.font: font(size), .foregroundColor: color, .paragraphStyle: paragraph])
}

func measure(_ s: NSAttributedString, _ width: CGFloat) -> NSSize {
    s.boundingRect(with: NSSize(width: width, height: 100000), options: [.usesLineFragmentOrigin, .usesFontLeading]).size
}

/// Black background, verses fitted inside a 6% safe area, reference centred
/// below. Draws into the current (flipped) context. Returns whether it fits.
@discardableResult
func paint(_ size: NSSize, _ slide: Slide) -> Bool {
    NSColor.black.setFill()
    NSRect(origin: .zero, size: size).fill()
    if slide.blank { return true }
    let (w, h) = (size.width, size.height)
    let margin = max(24, min(w, h) * 0.06)
    let safe = NSRect(x: margin, y: margin, width: w - 2 * margin, height: h - 2 * margin)
    let refSize = max(16, min(w, h) * 0.038)
    let body = safe.height - refSize * 2.4
    func fits(_ px: CGFloat) -> Bool {
        let m = measure(attributed(slide.text, px.rounded(.down), .white, .left), safe.width)
        return m.height <= body && m.width <= safe.width + 1
    }
    var (low, high): (CGFloat, CGFloat) = (12, min(96, h * 0.095))
    for _ in 0..<18 { let mid = (low + high) / 2; if fits(mid) { low = mid } else { high = mid } }
    let ok = fits(low)
    let verses = attributed(slide.text, low.rounded(.down), .white, .left)
    let vh = measure(verses, safe.width).height
    verses.draw(with: NSRect(x: safe.minX, y: safe.minY + (body - vh) / 2, width: safe.width, height: vh),
                options: [.usesLineFragmentOrigin, .usesFontLeading])
    let grey = NSColor(calibratedRed: 210 / 255, green: 210 / 255, blue: 210 / 255, alpha: 1)
    let reference = attributed(slide.reference, refSize.rounded(.down), grey, .center)
    let rh = measure(reference, safe.width).height
    let band = safe.minY + body + refSize * 0.6
    reference.draw(with: NSRect(x: safe.minX, y: band + (refSize * 1.8 - rh) / 2, width: safe.width, height: rh),
                   options: [.usesLineFragmentOrigin, .usesFontLeading])
    return ok
}

/// Screenshot mode: the same painting path into a bitmap, synthetic slides
/// only, no window or display needed.
func renderTest(_ args: [String]) -> Never {
    let w = args.count > 2 ? Int(args[2]) ?? 1920 : 1920, h = args.count > 3 ? Int(args[3]) ?? 1080 : 1080
    let path = args.count > 4 ? args[4] : "projection.png"
    var slide = Slide(reference: "約翰福音 3:16",
                      text: "16  神愛世人，甚至將他的獨生子賜給他們，叫一切信他的，不致滅亡，反得永生。\n\nFor God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life.",
                      blank: args.contains("--blank"))
    if args.contains("--long") { slide.text = String(repeating: "神愛世人，凡信他的必得永生。", count: 15) }
    let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: w, pixelsHigh: h, bitsPerSample: 8, samplesPerPixel: 4,
                               hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
    let context = NSGraphicsContext(bitmapImageRep: rep)!
    context.cgContext.translateBy(x: 0, y: CGFloat(h))
    context.cgContext.scaleBy(x: 1, y: -1)
    NSGraphicsContext.current = NSGraphicsContext(cgContext: context.cgContext, flipped: true)
    let ok = paint(NSSize(width: w, height: h), slide)
    NSGraphicsContext.current = nil
    try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: path))
    if !slide.blank { status(ok ? "FIT OK" : "FIT FAILED") }
    exit(ok ? 0 : 3)
}

final class AudienceView: NSView {
    var slide = Slide()
    override var isFlipped: Bool { true }
    override var acceptsFirstResponder: Bool { true }
    override func draw(_ dirtyRect: NSRect) { paint(bounds.size, slide) }
    // The audience never sees a pointer (AppKit honours this while the
    // pointer is over the window).
    static let blankCursor = NSCursor(image: NSImage(size: NSSize(width: 1, height: 1)), hotSpot: .zero)
    override func resetCursorRects() { addCursorRect(bounds, cursor: Self.blankCursor) }
    override func keyDown(with event: NSEvent) {
        // If the operator clicked into this window, a presentation remote
        // still works: its keys go back to the controller.
        switch event.keyCode {
        case 124, 121, 125, 49: status("@key next")  // → PageDown ↓ space
        case 123, 116, 126: status("@key prev")      // ← PageUp ↑
        case 11, 47: status("@key blank")            // B .
        case 53: status("投影視窗已關閉"); exit(0)    // Esc
        default: super.keyDown(with: event)
        }
    }
}

final class AudienceWindow: NSWindow {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { false }
}

final class Projector {
    let headless: Bool
    var state: [String: Any] = [:]
    var window: AudienceWindow?
    var view: AudienceView?
    var output: NSNumber?
    var placedIndex: Int?

    init(headless: Bool) {
        self.headless = headless
        if headless { return }
        NotificationCenter.default.addObserver(forName: NSApplication.didChangeScreenParametersNotification,
                                               object: nil, queue: .main) { [weak self] _ in
            guard let self else { return }
            if let output = self.output, !NSScreen.screens.contains(where: { Self.id($0) == output }) { self.hide() }
            // A just-connected display settles (arrangement, resolution)
            // after the first notification; place once it has.
            DispatchQueue.main.asyncAfter(deadline: .now() + 1) { self.apply(self.state) }
        }
    }

    static func id(_ screen: NSScreen) -> NSNumber? { screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber }

    var screenCount: Int { headless ? 1 : NSScreen.screens.count }

    func hide() {
        window?.orderOut(nil); window = nil; view = nil; output = nil; placedIndex = nil
    }

    func place(_ screen: NSScreen, _ slide: Slide) {
        hide()
        // A fresh borderless window covering the whole display, above the
        // menu bar and Dock, shown without activating this helper so the
        // operator keeps keyboard focus on the control desk.
        let window = AudienceWindow(contentRect: screen.frame, styleMask: .borderless, backing: .buffered, defer: false, screen: screen)
        window.title = "聖經 · 投影"
        window.isReleasedWhenClosed = false
        window.backgroundColor = .black
        window.level = .screenSaver
        window.collectionBehavior = [.canJoinAllSpaces, .stationary, .ignoresCycle, .fullScreenAuxiliary]
        let view = AudienceView(frame: NSRect(origin: .zero, size: screen.frame.size))
        view.slide = slide
        window.contentView = view
        window.setFrame(screen.frame, display: true)
        window.orderFrontRegardless()
        self.window = window; self.view = view; output = Self.id(screen)
    }

    func apply(_ next: [String: Any]) {
        state = next
        switch plan(state, screens: screenCount) {
        case .off: hide()
        case .fail(let message):
            if message == noScreen { hide() } else { window?.orderOut(nil) }
            status(message)
        case .show(let index, let slide):
            var (name, w, h) = ("headless", 1920, 1080)
            if !headless {
                let screen = NSScreen.screens[index]
                (name, w, h) = (screen.localizedName, Int(screen.frame.width), Int(screen.frame.height))
                if placedIndex == index, output == Self.id(screen), window?.isVisible == true {
                    view?.slide = slide; view?.needsDisplay = true
                } else {
                    place(screen, slide)
                }
            }
            placedIndex = index
            status("投影中：螢幕 \(index) · \(name) · \(w)×\(h)\(slide.blank ? "（黑屏）" : "")")
        }
    }
}

let args = CommandLine.arguments
if args.contains("--render-test") { renderTest(args) }
let headless = ProcessInfo.processInfo.environment["OMARCHY_BIBLE_PROJECTOR_HEADLESS"] != nil
if !headless {
    _ = NSApplication.shared
    // No Dock icon or menu bar; never steals activation from the reader.
    NSApp.setActivationPolicy(.accessory)
}
let projector = Projector(headless: headless)
var input = Data()
FileHandle.standardInput.readabilityHandler = { handle in
    let chunk = handle.availableData
    DispatchQueue.main.async {
        // The controller disappeared: never leave old verses on the screen.
        if chunk.isEmpty { exit(0) }
        input.append(chunk)
        if input.count > 262_144 { status("投影資料超過限制"); exit(0) }
        while let end = input.firstIndex(of: 0x0A) {
            let line = input[input.startIndex..<end]
            input.removeSubrange(input.startIndex...end)
            if let object = try? JSONSerialization.jsonObject(with: line) as? [String: Any] { projector.apply(object) }
        }
    }
}
status("投影視窗已準備；等候外接螢幕")
if headless { dispatchMain() } else { NSApp.run() }
