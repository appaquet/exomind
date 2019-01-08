pub trait Persistence {}

pub struct RocksPersistence {}

impl RocksPersistence {
    pub fn new(_bla: usize) -> RocksPersistence {
        RocksPersistence {}
    }
}

impl Persistence for RocksPersistence {}
