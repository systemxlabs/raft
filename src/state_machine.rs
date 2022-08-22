use std::fmt::Debug;

pub trait StateMachine: Debug + Send + 'static{
    // 应用日志条目
    fn apply(&mut self, data: &Vec<u8>);
}