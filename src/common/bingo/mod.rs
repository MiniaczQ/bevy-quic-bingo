use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Board {
    pub x_size: u8,
    pub y_size: u8,
    pub prompts: Vec<String>,
    pub activity: Vec<HashSet<u8>>,
}
