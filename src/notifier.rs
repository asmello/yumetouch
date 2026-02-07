use std::process::{Child, Command};

pub trait Notifier {
    fn notify_touch_needed(&mut self);
    fn dismiss(&mut self);
}

// --- Notification Center Banner ---

pub struct NotificationCenterNotifier {
    sound: String,
}

impl NotificationCenterNotifier {
    pub fn new(sound: &str) -> Self {
        Self {
            sound: sound.to_string(),
        }
    }
}

impl Notifier for NotificationCenterNotifier {
    fn notify_touch_needed(&mut self) {
        log::debug!("sending notification center banner");
        let result = mac_notification_sys::Notification::default()
            .title("YubiKey")
            .message("Touch your YubiKey")
            .sound(&self.sound)
            .send();

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
    dialog_child: Option<Child>,
    sound_child: Option<Child>,
}

impl DialogNotifier {
    pub fn new(sound: &str) -> Self {
        Self {
            sound: sound.to_string(),
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

        let script = r#"display dialog "Touch your YubiKey" with title "YubiKey" with icon caution buttons {"OK"} giving up after 30"#;

        match Command::new("osascript")
            .args(["-e", script])
            .spawn()
        {
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
