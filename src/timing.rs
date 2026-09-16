use std::cell::RefCell;
use std::rc::Rc;



pub struct TurnOrderRegister {
    pub breath: i32,
    pub owner: TurnTaker
}

pub enum TurnTaker {
    Actor(String),
    Timer(Rc<RefCell<TimingHandle>>)
}

pub struct TimingHandle {
    pub event: String,
    pub fired: bool,
    pub is_valid: bool
}
