// Audience-only desktop output. Receives plain-text JSON on a private pipe
// owned by the reader, never shell commands or rich text. No network access.
#include <QApplication>
#include <QWidget>
#include <QScreen>
#include <QWindow>
#include <QPainter>
#include <QFontMetricsF>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QSocketNotifier>
#include <QTimer>
#include <QKeyEvent>
#include <QCloseEvent>
#include <QFile>
#include <cstdio>
#include <unistd.h>
#include <fcntl.h>

static void status(const QString &s) {
    const auto bytes = s.toUtf8();
    fwrite(bytes.constData(), 1, size_t(bytes.size()), stdout);
    fputc('\n', stdout); fflush(stdout);
}

class Projector final : public QWidget {
    QJsonObject state;
    QString text, reference;
    QScreen *output = nullptr;
    bool blank = true;
public:
    Projector() {
        setWindowTitle(QStringLiteral("聖經 · 投影"));
        setWindowFlag(Qt::FramelessWindowHint);
        setAttribute(Qt::WA_OpaquePaintEvent);
        QObject::connect(qApp, &QGuiApplication::screenAdded, this, [this] { apply(state); });
        QObject::connect(qApp, &QGuiApplication::screenRemoved, this, [this](QScreen *screen) {
            if (screen == output) { output = nullptr; hide(); }
            // Re-evaluate after Qt has updated its screen list.
            QTimer::singleShot(0, this, [this] { apply(state); });
        });
    }
    void apply(const QJsonObject &next) {
        state = next;
        if (!state.value("active").toBool()) { hide(); output=nullptr; return; }
        const auto screens = QGuiApplication::screens();
        int index = state.value("screen").toInt(-1);
        if (index < 0) {
            // Never fall back to fullscreen on the operator's only screen.
            index = -1;
            for (int i=0; i<screens.size(); ++i)
                if (screens[i] != QGuiApplication::primaryScreen()) { index=i; break; }
        }
        if (index < 0 || index >= screens.size()) {
            hide(); output=nullptr;
            status(QStringLiteral("找不到外接螢幕；請連接螢幕，或明確選擇螢幕 0 預覽"));
            return;
        }
        QScreen *screen = screens[index];
        const auto slides = state.value("slides").toArray();
        const int at = state.value("index").toInt();
        if (at<0 || at>=slides.size()) { hide(); status(QStringLiteral("沒有可投影的經文")); return; }
        const auto slide = slides[at].toObject();
        text=slide.value("text").toString(); reference=slide.value("reference").toString();
        if (text.size()>16000 || reference.size()>2048) { hide(); status(QStringLiteral("投影資料過長")); return; }
        blank=state.value("blank").toBool();
        if (screen!=output || !isVisible()) {
            hide(); winId(); windowHandle()->setScreen(screen);
            setGeometry(screen->geometry()); output=screen; showFullScreen();
        }
        update();
        status(QStringLiteral("投影中：螢幕 %1 · %2 · %3×%4%5")
            .arg(index).arg(screen->name()).arg(screen->size().width()).arg(screen->size().height())
            .arg(blank ? QStringLiteral("（黑屏）") : QString()));
    }
protected:
    void paintEvent(QPaintEvent*) override {
        QPainter p(this); p.fillRect(rect(), Qt::black);
        if (blank) return;
        p.setRenderHint(QPainter::TextAntialiasing);
        const qreal margin=std::max(24.0, std::min(width(),height())*0.06);
        const QRectF safe=QRectF(rect()).adjusted(margin,margin,-margin,-margin);
        QFont font(QStringLiteral("Noto Sans CJK TC"));
        const qreal refSize=std::max(16.0,std::min(width(),height())*0.038);
        const QRectF body(safe.left(),safe.top(),safe.width(),safe.height()-refSize*2.4);
        const int flags=Qt::TextWordWrap|Qt::AlignLeft|Qt::AlignVCenter;
        qreal low=12.0, high=std::min(96.0,height()*0.095);
        for (int i=0;i<18;++i) {
            const auto mid=(low+high)/2;
            font.setPixelSize(int(mid));
            const auto bounds=QFontMetricsF(font).boundingRect(QRectF(0,0,body.width(),100000), Qt::TextWordWrap,text);
            if (bounds.height()<=body.height() && bounds.width()<=body.width()+1) low=mid; else high=mid;
        }
        font.setPixelSize(int(low));
        if (qApp->arguments().contains("--render-test")) {
            const auto measured=QFontMetricsF(font).boundingRect(QRectF(0,0,body.width(),100000),Qt::TextWordWrap,text);
            if (measured.height()>body.height()+1 || measured.width()>body.width()+1) {
                status(QStringLiteral("FIT FAILED")); std::exit(3);
            }
            status(QStringLiteral("FIT OK"));
        }
        p.setFont(font); p.setPen(Qt::white);
        p.drawText(body, flags, text);
        font.setPixelSize(int(refSize)); p.setFont(font); p.setPen(QColor(210,210,210));
        p.drawText(QRectF(safe.left(),body.bottom()+refSize*0.6,safe.width(),refSize*1.8),
            Qt::AlignHCenter|Qt::AlignVCenter, reference);
    }
    void keyPressEvent(QKeyEvent *e) override {
        if (e->key()==Qt::Key_Escape) { status(QStringLiteral("投影視窗已關閉")); qApp->quit(); }
        else QWidget::keyPressEvent(e);
    }
    void closeEvent(QCloseEvent *e) override { e->accept(); qApp->quit(); }
};

int main(int argc,char **argv) {
    QApplication app(argc,argv);
    app.setQuitOnLastWindowClosed(false);
    app.setDesktopFileName(QStringLiteral("io.github.vibe_tjc.OmarchyBible.Projector"));
    Projector projector;
    // Screenshot mode uses the same painting path, with synthetic slides only.
    if (app.arguments().contains("--render-test")) {
        auto args=app.arguments();
        int w=args.value(2,"1920").toInt(),h=args.value(3,"1080").toInt();
        QJsonObject sample{{"active",true},{"screen",0},{"index",0},{"slides",QJsonArray{QJsonObject{
            {"reference",QStringLiteral("約翰福音 3:16")},
            {"text",QStringLiteral("16  神愛世人，甚至將他的獨生子賜給他們，叫一切信他的，不致滅亡，反得永生。\n\nFor God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life.")}}}}};
        if (args.contains("--long")) {
            auto slide=sample.value("slides").toArray()[0].toObject();
            slide["text"]=QStringLiteral("神愛世人，凡信他的必得永生。").repeated(15);
            sample["slides"]=QJsonArray{slide};
        }
        sample["blank"]=args.contains("--blank");
        projector.apply(sample); projector.showNormal(); projector.resize(w,h);
        QTimer::singleShot(100,&projector,[&] { projector.grab().save(args.value(4,"projection.png")); app.quit(); });
        return app.exec();
    }
    fcntl(STDIN_FILENO,F_SETFL,fcntl(STDIN_FILENO,F_GETFL)|O_NONBLOCK);
    QByteArray input;
    QSocketNotifier notifier(STDIN_FILENO,QSocketNotifier::Read);
    QObject::connect(&notifier,&QSocketNotifier::activated,&app,[&] {
        char buf[8192]; ssize_t n;
        while ((n=::read(STDIN_FILENO,buf,sizeof(buf)))>0) {
            input.append(buf,int(n));
            if (input.size()>262144) { status(QStringLiteral("投影資料超過限制")); app.quit(); return; }
            int end;
            while ((end=input.indexOf('\n'))>=0) {
                const auto line=input.left(end); input.remove(0,end+1);
                QJsonParseError error; const auto document=QJsonDocument::fromJson(line,&error);
                if (error.error==QJsonParseError::NoError && document.isObject()) projector.apply(document.object());
            }
        }
        // The controller disappeared: never leave old verses on the screen.
        if (n==0) app.quit();
    });
    status(QStringLiteral("投影視窗已準備；等候外接螢幕"));
    return app.exec();
}
