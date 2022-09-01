use std::fmt::Debug;

pub trait StateMachine: Debug + Send + 'static{
    // 应用日志条目
    fn apply(&mut self, data: &Vec<u8>);

    // 生成快照
    // TODO Copy-on-write
    fn take_snapshot(&mut self, snapshot_filepath: String);

    // 从快照恢复
    fn restore_snapshot(&mut self, snapshot_filepath: String);
}