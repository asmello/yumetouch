use std::process::{Child, Command};

pub trait Notifier {
    fn notify_touch_needed(&mut self);
    fn dismiss(&mut self);
}

// --- Notification Center Banner ---

pub struct NotificationCenterNotifier {
    sound: String,
    icon_path: Option<String>,
}

impl NotificationCenterNotifier {
    pub fn new(sound: &str, icon_path: Option<String>) -> Self {
        Self {
            sound: sound.to_string(),
            icon_path,
        }
    }
}

impl Notifier for NotificationCenterNotifier {
    fn notify_touch_needed(&mut self) {
        log::debug!("sending notification center banner");
        let mut notification = mac_notification_sys::Notification::default();
        notification
            .title("YubiKey")
            .message("Touch your YubiKey")
            .sound(&self.sound);

        if let Some(ref path) = self.icon_path {
            notification.app_icon(path);
        }

        let result = notification.send();

        if let Err(e) = result {
            log::warn!("failed to send notification: {e}");
        }
    }

    fn dismiss(&mut self) {
        // Notification Center banners auto-dismiss; no API to remove them programmatically
        log::debug!("notification center banner will auto-dismiss");
    }
}

// --- Modal Dialog via osascript ---

pub struct DialogNotifier {
    sound: String,
    icon_path: Option<String>,
    dialog_child: Option<Child>,
    sound_child: Option<Child>,
}

impl DialogNotifier {
    pub fn new(sound: &str, icon_path: Option<String>) -> Self {
        Self {
            sound: sound.to_string(),
            icon_path,
            dialog_child: None,
            sound_child: None,
        }
    }

    fn kill_children(&mut self) {
        if let Some(ref mut child) = self.dialog_child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.dialog_child = None;

        if let Some(ref mut child) = self.sound_child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.sound_child = None;
    }
}

impl Notifier for DialogNotifier {
    fn notify_touch_needed(&mut self) {
        // Dismiss any existing dialog first
        self.kill_children();

        log::debug!("showing modal dialog");

        let mut cmd = Command::new("osascript");

        if let Some(ref path) = self.icon_path {
            let script = format!(
                concat!(
                    "ObjC.import('AppKit');",
                    "var app=$.NSApplication.sharedApplication;",
                    "app.setActivationPolicy(1);",
                    "var img=$.NSImage.alloc.initWithContentsOfFile('{}');",
                    "var W=260,H=170;",
                    "var sf=$.NSScreen.mainScreen.frame;",
                    "var p=$.NSPanel.alloc.initWithContentRectStyleMaskBackingDefer(",
                    "$.NSMakeRect((sf.size.width-W)/2,(sf.size.height-H)/2+100,W,H),32769,2,false);",
                    "p.setTitleVisibility(1);",
                    "p.setTitlebarAppearsTransparent(true);",
                    "p.setMovableByWindowBackground(true);",
                    "p.setLevel(3);",
                    "var cv=p.contentView;",
                    "var iv=$.NSImageView.alloc.initWithFrame($.NSMakeRect((W-64)/2,H-74,64,64));",
                    "iv.setImage(img);iv.setImageScaling(0);cv.addSubview(iv);",
                    "var tf=$.NSTextField.alloc.initWithFrame($.NSMakeRect(0,H-104,W,22));",
                    "tf.setStringValue('YubiKey');",
                    "tf.setFont($.NSFont.boldSystemFontOfSize(15));",
                    "tf.setAlignment(1);tf.setBezeled(false);",
                    "tf.setEditable(false);tf.setSelectable(false);",
                    "tf.setDrawsBackground(false);cv.addSubview(tf);",
                    "var mf=$.NSTextField.alloc.initWithFrame($.NSMakeRect(0,H-126,W,18));",
                    "mf.setStringValue('Touch your YubiKey');",
                    "mf.setFont($.NSFont.systemFontOfSize(13));",
                    "mf.setAlignment(1);mf.setBezeled(false);",
                    "mf.setEditable(false);mf.setSelectable(false);",
                    "mf.setDrawsBackground(false);",
                    "mf.setTextColor($.NSColor.secondaryLabelColor);",
                    "cv.addSubview(mf);",
                    "var btn=$.NSButton.alloc.initWithFrame($.NSMakeRect(20,8,W-40,28));",
                    "btn.setTitle('OK');btn.setBezelStyle(1);",
                    "btn.setKeyEquivalent('\\r');",
                    "btn.setTarget(app);btn.setAction('terminate:');",
                    "cv.addSubview(btn);",
                    "p.makeKeyAndOrderFront(null);",
                    "app.activateIgnoringOtherApps(true);app.run;",
                ),
                path,
            );
            cmd.args(["-l", "JavaScript", "-e", &script]);
        } else {
            cmd.args(["-e", r#"display alert "YubiKey" message "Touch your YubiKey" buttons {"OK"} giving up after 30"#]);
        }

        match cmd.spawn() {
            Ok(child) => self.dialog_child = Some(child),
            Err(e) => log::warn!("failed to spawn osascript dialog: {e}"),
        }

        let sound_path = format!("/System/Library/Sounds/{}.aiff", self.sound);
        match Command::new("afplay").arg(&sound_path).spawn() {
            Ok(child) => self.sound_child = Some(child),
            Err(e) => log::warn!("failed to play sound {sound_path}: {e}"),
        }
    }

    fn dismiss(&mut self) {
        log::debug!("dismissing modal dialog");
        self.kill_children();
    }
}

impl Drop for DialogNotifier {
    fn drop(&mut self) {
        self.kill_children();
    }
}

// --- Composite Notifier (wraps multiple) ---

pub struct CompositeNotifier {
    notifiers: Vec<Box<dyn Notifier>>,
}

impl CompositeNotifier {
    pub fn new(notifiers: Vec<Box<dyn Notifier>>) -> Self {
        Self { notifiers }
    }
}

impl Notifier for CompositeNotifier {
    fn notify_touch_needed(&mut self) {
        for n in &mut self.notifiers {
            n.notify_touch_needed();
        }
    }

    fn dismiss(&mut self) {
        for n in &mut self.notifiers {
            n.dismiss();
        }
    }
}
