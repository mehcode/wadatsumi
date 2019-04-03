pub mod instruction;
pub mod state;
pub mod util;

use failure::Error;

fn main() -> Result<(), Error> {
    pretty_env_logger::try_init()?;

    Ok(())
}
