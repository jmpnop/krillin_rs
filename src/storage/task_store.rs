use crate::types::task::SubtitleTask;
use dashmap::DashMap;

#[derive(Debug, Default)]
pub struct TaskStore {
    tasks: DashMap<String, SubtitleTask>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: DashMap::new(),
        }
    }

    pub fn insert(&self, task: SubtitleTask) {
        self.tasks.insert(task.task_id.clone(), task);
    }

    pub fn get(&self, task_id: &str) -> Option<SubtitleTask> {
        self.tasks.get(task_id).map(|r| r.value().clone())
    }

    pub fn update<F>(&self, task_id: &str, f: F)
    where
        F: FnOnce(&mut SubtitleTask),
    {
        if let Some(mut entry) = self.tasks.get_mut(task_id) {
            f(entry.value_mut());
        }
    }

    pub fn exists(&self, task_id: &str) -> bool {
        self.tasks.contains_key(task_id)
    }
}
