pub enum ProcessState {
    Running,
    Sleeping,
    Waiting,
    Dead,
}

#[repr(C)]
pub struct Process {
    frame: TrapFrame,
    stack: *mut u8,
    program_counter: usize,
    pid: u16,
    root: *mut Table,
    state: ProcessState,
}
