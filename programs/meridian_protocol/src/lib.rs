pub mod states;
pub mod instructions;
pub mod errors;
pub use states::*;

use anchor_lang::prelude::*;

declare_id!("4QTQjYZco26cXUVXeNsTnTvjGc2t58rwFxdQnjgzSwFG");

#[program]
pub mod meridian_protocol {
    use super::*;
}
