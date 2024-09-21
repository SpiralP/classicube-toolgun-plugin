// pub mod local_blocks;
pub mod other_blocks;

pub fn initialize() {
    // local_blocks::initialize();
    other_blocks::initialize();
}

pub fn free() {
    other_blocks::free();
    // local_blocks::free();
}
