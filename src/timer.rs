use logging::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// 计时器内部线程检查间隔
const THREAD_CHECK_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct Timer {
    // 计时器名称
    name: String,
    // 控制计时器是否执行
    alive: Arc<AtomicBool>,
    // 计时器触发间隔
    trigger_interval: Arc<Mutex<Duration>>,
    // 计时器下一次触发时间
    next_trigger: Arc<Mutex<Instant>>,
    // 上一次重置计时器时间
    pub last_reset_at: Option<Instant>,
    // 计时器线程
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Timer {
            name: name.to_string(),
            alive: Arc::new(AtomicBool::new(false)),
            trigger_interval: Arc::new(Mutex::new(Duration::from_secs(std::u64::MAX))),
            next_trigger: Arc::new(Mutex::new(Instant::now())),
            last_reset_at: None,
            handle: None,
        }
    }

    // 启动计时器
    pub fn schedule<F>(&mut self, trigger_interval: Duration, callback: F)
    where
        F: 'static + Send + FnMut() -> (),
    {
        info!(
            "{} start schedule with trigger interval: {}ms",
            self.name, trigger_interval.as_millis()
        );

        (*self.trigger_interval.lock().unwrap()) = trigger_interval;
        (*self.next_trigger.lock().unwrap()) = Instant::now() + trigger_interval;
        self.alive.store(true, Ordering::SeqCst);

        let trigger_interval = self.trigger_interval.clone();
        let next_trigger = self.next_trigger.clone();
        let alive = self.alive.clone();

        self.handle = Some(std::thread::spawn(move || {
            let callback = Arc::new(Mutex::new(callback));
            loop {
                std::thread::sleep(THREAD_CHECK_INTERVAL);

                if !alive.load(Ordering::SeqCst) {
                    break;
                }

                if (*next_trigger.lock().unwrap()) <= Instant::now() {
                    // 异步执行回调函数，不阻塞计时器线程
                    let callback = callback.clone();
                    std::thread::spawn(move || {
                        callback.lock().unwrap()();
                    });

                    // 重新计算下一次触发时间
                    (*next_trigger.lock().unwrap()) = Instant::now() + (*trigger_interval.lock().unwrap());
                }
            }
        }));
    }

    // 重置计时器触发间隔
    pub fn reset(&mut self, trigger_interval: Duration) {
        info!("{} reset with trigger interval: {}ms", self.name, trigger_interval.as_millis());
        self.last_reset_at = Some(Instant::now());
        (*self.trigger_interval.lock().unwrap()) = trigger_interval;
        (*self.next_trigger.lock().unwrap()) = Instant::now() + trigger_interval;
    }

    // 停止计时器
    pub fn stop(&mut self) {
        info!("{} stopping", self.name);
        self.alive.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_timer() {
        let mut timer = super::Timer::new("test_timer");
        timer.schedule(std::time::Duration::from_secs(1), || {
            println!("hello {:?}", std::time::Instant::now());
        });
        std::thread::sleep(std::time::Duration::from_secs(10));

        timer.reset(std::time::Duration::from_secs(2));

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
