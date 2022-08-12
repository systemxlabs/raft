use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use logging::*;

// 计时器内部线程检查间隔
const THREAD_CHECK_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct Timer {
    name: String,
    alive: Arc<AtomicBool>,
    interval: Arc<Mutex<Duration>>,
    next_tick: Arc<Mutex<Instant>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Timer {
            name: name.to_string(),
            alive: Arc::new(AtomicBool::new(false)),
            interval: Arc::new(Mutex::new(Duration::from_secs(std::u64::MAX))),
            next_tick: Arc::new(Mutex::new(Instant::now())),
            handle: None,
        }
    }

    pub fn schedule<F>(&mut self, interval: Duration, callback: F) where F: 'static + Send + FnMut() -> () {
        info!("{} execute schedule with interval: {:?}", self.name, &interval);

        (*self.interval.lock().unwrap()) = interval;
        (*self.next_tick.lock().unwrap()) = Instant::now() + interval;
        self.alive.store(true, Ordering::SeqCst);

        let interval = self.interval.clone();
        let next_tick = self.next_tick.clone();
        let alive = self.alive.clone();

        self.handle = Some(std::thread::spawn(move || {

            let callback = Arc::new(Mutex::new(callback));
            loop {

                std::thread::sleep(THREAD_CHECK_INTERVAL);

                if !alive.load(Ordering::SeqCst) {
                    break;
                }

                if (*next_tick.lock().unwrap()) <= Instant::now() {

                    // 异步执行回调函数，不阻塞计时器线程
                    let callback = callback.clone();
                    std::thread::spawn(move || {
                        callback.lock().unwrap()();
                    });

                    // 重新计算下一次触发时间
                    (*next_tick.lock().unwrap()) = Instant::now() + (*interval.lock().unwrap());
                }
            }
        }));
    }

    pub fn reset(&mut self, interval: Duration) {
        info!("{} execute reset with interval: {:?}", self.name, &interval);
        (*self.interval.lock().unwrap()) = interval;
        (*self.next_tick.lock().unwrap()) = Instant::now() + interval;
    }

    pub fn stop(&mut self) {
        info!("{} execute stop", self.name);
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