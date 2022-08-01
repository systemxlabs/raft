use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

// 计时器内部线程检查间隔
const THREAD_CHECK_INTERVAL: Duration = Duration::from_millis(10);

pub struct Timer {
    interval: Arc<Mutex<Duration>>,
    next_tick: Arc<Mutex<Instant>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            interval: Arc::new(Mutex::new(Duration::from_secs(std::u64::MAX))),
            next_tick: Arc::new(Mutex::new(Instant::now())),
            handle: None,
        }
    }

    pub fn schedule<F>(&mut self, interval: Duration, callback: F) where F: 'static + Send + FnMut() -> () {
        (*self.interval.lock().unwrap()) = interval;
        (*self.next_tick.lock().unwrap()) = Instant::now() + interval;

        let interval = self.interval.clone();
        let next_tick = self.next_tick.clone();
        self.handle = Some(std::thread::spawn(move || {
            let callback = Arc::new(Mutex::new(callback));
            loop {
                std::thread::sleep(THREAD_CHECK_INTERVAL);
                if (*next_tick.lock().unwrap()) <= Instant::now() {
                    // TODO 异步执行回调函数，不阻塞计时器线程
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
        (*self.interval.lock().unwrap()) = interval;
        (*self.next_tick.lock().unwrap()) = Instant::now() + interval;
    }

}


#[cfg(test)]
mod tests {
    #[test]
    fn test_timer() {
        let mut timer = super::Timer::new();
        timer.schedule(std::time::Duration::from_secs(1), || {
            println!("hello {:?}", std::time::Instant::now());
        });
        std::thread::sleep(std::time::Duration::from_secs(10));

        timer.reset(std::time::Duration::from_secs(2));

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}