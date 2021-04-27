use anyhow::{anyhow, Context, Result};
use num_complex::Complex32;
use tap::Pipe;
use uhd::{self, Usrp};

pub fn main() -> Result<()> {
    env_logger::init();

    let mut usrp = Usrp::find("")
        .context("Failed to open device list")?
        .drain(..)
        .next()
        .context("Failed to find a valid USRP to attach to")?
        .pipe(|addr| Usrp::open(&addr))
        .context("Failed to find properly open the USRP")?;

    // Set the stream type to be "fc32" which means "float complex 32"
    // See: https://files.ettus.com/manual/structuhd_1_1stream__args__t.html#a602a64b4937a85dba84e7f724387e252
    let stream = uhd::StreamArgs::<Complex32>::new("fc32");

    Ok(())
}
