use self::{abort::Abort, initiate::Initiate};

pub mod abort;
pub mod initiate;



pub trait Trigger: Abort + Initiate {
    
}



