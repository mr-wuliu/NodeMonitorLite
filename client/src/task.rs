#![allow(unused)]
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering}
};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::Request;
use crate::sampling;
use crate::net;


pub struct Task {
    pub name: String,
    pub enabled: Arc<AtomicBool>, // lite cancel for pause task
    pub meta_data: HashMap<String, String>, // meta data for the task
    pub interval: Option<u64>, // interval for the tasl
    pub task_fn: Arc<
        dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>
            + Send
            + Sync
            + 'static,
    >,
    pub cancel_token: CancellationToken,
}


impl Task {
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let cancel = self.cancel_token.clone();
        let task_fn = self.task_fn.clone();
        let interval = self.interval;
        let enabled = self.enabled.clone();
        tokio::spawn(async move {
            loop {
                // 临时取消，等待100ms后继续, 用于暂停任务
                if !enabled.load(Ordering::Relaxed) {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                if cancel.is_cancelled() {
                    break;
                }
 
                let _ = (task_fn)();
                if let Some(ms) = interval {
                    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                } else {
                    tokio::task::yield_now().await;
                }
            }
        })
    }

    pub fn pause(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn remove(&self) {
        self.cancel_token.cancel();
    }

}


pub struct TaskManager {
    tasks: HashMap<String, Task>,
    handles: HashMap<String, JoinHandle<()>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            handles: HashMap::new(),
        }
    }

    pub fn query_task_status(&self, name: &str) -> Option<bool> {
        if let Some(task) = self.tasks.get(name) {
            return Some(task.enabled.load(Ordering::Relaxed));
        }
        None
    }

    pub fn add_and_start_task(&mut self, task: Task) {
        let name = task.name.clone();
        let handle = task.start();
        self.handles.insert(name.clone(), handle);
        self.tasks.insert(name, task);
    }

    pub fn pause_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.get(name) {
            task.pause();
        }
    }
    
    pub fn resume_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.get(name) {
            task.resume();
        }
    }

    pub async fn remove_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.get(name) {
            task.remove();
        }
        if let Some(handle) = self.handles.remove(name) {
            let _ = handle.await;
        }
        let _ = self.tasks.remove(name);
    }
}

pub async fn machine_register_task(uuid: Option<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let info = sampling::collect_static_info(uuid);
    println!("info: {:?}", info);
    if let Ok(mut client) = net::get_client_clone().await {
        let resp= client.register_machine(Request::new(info)).await;

        return Ok(Some(resp.unwrap().into_inner().uuid));
    }
    Ok(None)
}

pub async fn start_all_tasks(uuid: String, manager: &mut TaskManager) -> Result<(), Box<dyn std::error::Error>> {
    let mut meta = HashMap::new();
    meta.insert("uuid".to_string(), uuid.clone());

    // define dynamic tasks
    let dynamic_cpu_usage_task_fn = Arc::new(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tokio::spawn({
            let uuid = uuid.clone();
            async move {
                let mut sys = System::new_with_specifics(
                    RefreshKind::nothing().with_cpu(CpuRefreshKind::everything())
                );
                loop {
                    tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
                    sys.refresh_cpu_all();
                    let dyn_info = sampling::collect_dynamic_info(&mut sys, &uuid);
                    if let Ok(mut client) = net::get_client_clone().await {
                        let _ = client.report_dynamic_info(
                            Request::new(dyn_info)
                        ).await;
                    }
                }
            }
        });
        Ok(())
    });

    // build task
    let task = Task {
        name: "dynamic_cpu_usage".to_string(),
        enabled: Arc::new(AtomicBool::new(true)),
        meta_data: meta,
        interval: None,
        task_fn: dynamic_cpu_usage_task_fn,
        cancel_token: CancellationToken::new(),
    };


    manager.add_and_start_task(task);

    Ok(())
}