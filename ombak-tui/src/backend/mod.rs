use bitvec::vec::BitVec;

pub mod interpreter;
pub mod simulator;

#[derive(Clone)]
pub struct Wave {
    pub signal_name: String,
    pub width: usize,
    pub values: Vec<BitVec>,
}

#[derive(Clone, Default)]
pub struct SimulationResult {
    pub waves: Vec<Wave>,
    pub time_step_ps: usize,
}
