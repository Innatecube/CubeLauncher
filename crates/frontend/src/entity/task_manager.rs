use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use bridge::{instance::InstanceID, modal_action::ModalAction};
use gpui::{Context, SharedString};
use schema::unique_bytes::UniqueBytes;

#[derive(Clone, Debug)]
pub struct TaskSnapshot {
    pub key: usize,
    pub id: Option<InstanceID>,
    pub name: SharedString,
    pub icon: Option<UniqueBytes>,
    pub stage: Arc<str>,
    pub count: usize,
    pub total: usize,
    pub speed_mb_s: f64,
    pub eta_secs: Option<u64>,
    pub is_finished: bool,
    pub is_error: bool,
    pub is_modpack: bool,
    pub modal_action: ModalAction,
}

#[derive(Clone, Debug)]
pub struct CompletedTaskSnapshot {
    pub name: SharedString,
    pub icon: Option<UniqueBytes>,
    pub finished_at: Instant,
    pub is_error: bool,
}

struct TaskInternal {
    key: usize,
    id: Option<InstanceID>,
    name: SharedString,
    icon: Option<UniqueBytes>,
    modal_action: ModalAction,
    last_count: usize,
    last_instant: Instant,
    speed_bytes_s: f64,
    is_modpack: bool,
}

pub struct TaskManager {
    next_key: usize,
    tasks: Vec<TaskInternal>,
    pub completed: Vec<CompletedTaskSnapshot>,
    pub is_flyout_open: bool,
    pub last_closed_at: Option<Instant>,
    pending_creations: std::collections::HashMap<String, (Option<UniqueBytes>, ModalAction, bool)>,
}

impl TaskManager {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(250)).await;
                let res = this.update(cx, |manager, cx| {
                    manager.poll(cx);
                });
                if res.is_err() {
                    break;
                }
            }
        }).detach();

        Self {
            next_key: 1,
            tasks: Vec::new(),
            completed: Vec::new(),
            is_flyout_open: false,
            last_closed_at: None,
            pending_creations: std::collections::HashMap::new(),
        }
    }

    pub fn register_pending_creation(&mut self, name: String, icon: Option<UniqueBytes>, modal_action: ModalAction, is_modpack: bool, cx: &mut Context<Self>) {
        self.pending_creations.insert(name.clone(), (icon.clone(), modal_action.clone(), is_modpack));
        self.add_task(None, name.into(), icon, modal_action, is_modpack, cx);
    }

    pub fn take_pending_creation(&mut self, name: &str, id: InstanceID, icon: Option<UniqueBytes>, cx: &mut Context<Self>) -> Option<ModalAction> {
        self.associate_instance(name, id, icon, cx);
        let key = self.pending_creations.keys().find(|k| *k == name || name.starts_with(k.as_str()) || k.starts_with(name)).cloned();
        if let Some(key) = key {
            if let Some((_, modal_action, is_modpack)) = self.pending_creations.remove(&key) {
                if is_modpack {
                    return None;
                } else {
                    return Some(modal_action);
                }
            }
        }
        None
    }

    pub fn associate_instance(&mut self, name: &str, id: InstanceID, icon: Option<UniqueBytes>, cx: &mut Context<Self>) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id.is_none() && (t.name == name || name.starts_with(&*t.name) || t.name.starts_with(name))) {
            task.id = Some(id);
            if task.icon.is_none() && icon.is_some() {
                task.icon = icon;
            }
            cx.notify();
        }
    }

    pub fn add_task(
        &mut self,
        id: Option<InstanceID>,
        name: SharedString,
        icon: Option<UniqueBytes>,
        modal_action: ModalAction,
        is_modpack: bool,
        cx: &mut Context<Self>,
    ) {
        if let Some(existing) = self.tasks.iter_mut().find(|t| (id.is_some() && t.id == id) || (id.is_none() && t.name == name)) {
            existing.name = name;
            if icon.is_some() {
                existing.icon = icon;
            }
            existing.modal_action = modal_action;
            existing.last_count = 0;
            existing.last_instant = Instant::now();
            existing.speed_bytes_s = 0.0;
            existing.is_modpack = is_modpack;
            if id.is_some() {
                existing.id = id;
            }
        } else {
            let key = self.next_key;
            self.next_key += 1;
            self.tasks.push(TaskInternal {
                key,
                id,
                name,
                icon,
                modal_action,
                last_count: 0,
                last_instant: Instant::now(),
                speed_bytes_s: 0.0,
                is_modpack,
            });
        }
        cx.notify();
    }

    pub fn add_instance_task(
        &mut self,
        id: InstanceID,
        name: SharedString,
        icon: Option<UniqueBytes>,
        modal_action: ModalAction,
        cx: &mut Context<Self>,
    ) {
        self.add_task(Some(id), name, icon, modal_action, false, cx);
    }

    pub fn is_active(&self, id: InstanceID) -> bool {
        self.tasks.iter().any(|t| t.id == Some(id) && t.modal_action.get_finished_at().is_none())
    }

    pub fn active_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.modal_action.get_finished_at().is_none()).count()
    }

    pub fn toggle_flyout(&mut self, cx: &mut Context<Self>) {
        if let Some(closed_at) = self.last_closed_at {
            if closed_at.elapsed() < Duration::from_millis(200) {
                return;
            }
        }
        self.is_flyout_open = !self.is_flyout_open;
        cx.notify();
    }

    pub fn close_flyout(&mut self, cx: &mut Context<Self>) {
        if self.is_flyout_open {
            self.is_flyout_open = false;
            self.last_closed_at = Some(Instant::now());
            cx.notify();
        }
    }

    pub fn clear_completed(&mut self, cx: &mut Context<Self>) {
        self.completed.clear();
        cx.notify();
    }

    pub fn cancel_task_by_key(&mut self, key: usize, cx: &mut Context<Self>) {
        if let Some(task) = self.tasks.iter().find(|t| t.key == key) {
            task.modal_action.request_cancel.cancel();
            cx.notify();
        }
    }

    pub fn cancel_task(&mut self, id: InstanceID, cx: &mut Context<Self>) {
        if let Some(task) = self.tasks.iter().find(|t| t.id == Some(id)) {
            task.modal_action.request_cancel.cancel();
            cx.notify();
        }
    }

    pub fn snapshots(&self) -> Vec<TaskSnapshot> {
        self.tasks.iter().map(|t| {
            let mut count = 0;
            let mut total = 0;
            let mut stage: Arc<str> = Arc::from("Downloading content");
            let mut is_extracting = false;

            t.modal_action.write_trackers(|trackers| {
                for tracker in trackers {
                    let (c, tot) = tracker.get();
                    count += c;
                    total += tot;
                    let title = tracker.get_title();
                    if title.to_ascii_lowercase().contains("extract") {
                        is_extracting = true;
                    }
                    if !t.is_modpack && !title.is_empty() {
                        stage = title;
                    }
                }
            });

            if t.is_modpack {
                if is_extracting {
                    stage = Arc::from("Extracting modpack files");
                } else if t.modal_action.get_finished_at().is_some() {
                    stage = Arc::from("Installed successfully");
                } else {
                    stage = Arc::from("Downloading content");
                }
            } else if stage.is_empty() || stage == Arc::from("Preparing...") {
                stage = Arc::from("Downloading content");
            }

            let is_finished = t.modal_action.get_finished_at().is_some();
            let is_error = t.modal_action.get_error_message().is_some();
            let speed_mb_s = t.speed_bytes_s / 1_048_576.0;

            let eta_secs = if t.speed_bytes_s > 1024.0 && total > count {
                Some(((total - count) as f64 / t.speed_bytes_s) as u64)
            } else {
                None
            };

            TaskSnapshot {
                key: t.key,
                id: t.id,
                name: t.name.clone(),
                icon: t.icon.clone(),
                stage,
                count,
                total,
                speed_mb_s,
                eta_secs,
                is_finished,
                is_error,
                is_modpack: t.is_modpack,
                modal_action: t.modal_action.clone(),
            }
        }).collect()
    }

    fn poll(&mut self, cx: &mut Context<Self>) {
        let now = Instant::now();
        let mut changed = false;

        for task in &mut self.tasks {
            let mut current_count = 0;
            task.modal_action.write_trackers(|trackers| {
                for tracker in trackers {
                    let (c, _) = tracker.get();
                    current_count += c;
                }
            });

            let dt = now.duration_since(task.last_instant).as_secs_f64();
            if dt >= 0.4 {
                let dc = current_count.saturating_sub(task.last_count);
                let raw_speed = dc as f64 / dt;
                task.speed_bytes_s = (task.speed_bytes_s * 0.4) + (raw_speed * 0.6);
                task.last_count = current_count;
                task.last_instant = now;
                changed = true;
            }
        }

        for task in &self.tasks {
            if let Some(finished_at) = task.modal_action.get_finished_at() {
                let is_error = task.modal_action.get_error_message().is_some();
                if !self.completed.iter().any(|c| c.name == task.name) {
                    self.completed.insert(0, CompletedTaskSnapshot {
                        name: task.name.clone(),
                        icon: task.icon.clone(),
                        finished_at,
                        is_error,
                    });
                    if self.completed.len() > 10 {
                        self.completed.truncate(10);
                    }
                    changed = true;
                }
            }
        }

        let prev_len = self.tasks.len();
        self.tasks.retain(|t| {
            if let Some(finished_at) = t.modal_action.get_finished_at() {
                now.duration_since(finished_at) < Duration::from_secs(2)
            } else {
                true
            }
        });

        if prev_len != self.tasks.len() {
            changed = true;
        }

        if changed {
            cx.notify();
        }
    }
}
