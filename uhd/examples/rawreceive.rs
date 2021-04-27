use std::{env::set_var, time::Duration};

use anyhow::{anyhow, Context, Result};
use num_complex::Complex32;
use tap::Pipe;
use uhd::{self, check_status, StreamCommand, StreamCommandType, TuneRequest, Usrp};

const CHANNEL: usize = 0;

fn main() -> Result<()> {
    let mut usrp = Usrp::find("")
        .context("Failed to open device list")?
        .iter()
        .next()
        .context("Failed to find a valid USRP to attach to")?
        .pipe(|addr| Usrp::open(&addr))
        .context("Failed to find properly open the USRP")?;

    let mut receiver = usrp
        .get_rx_stream(&uhd::StreamArgs::<Complex32>::new("fc32"))
        .unwrap();

    let data = receiver.receive_for(std::time::Duration::from_millis(1000));

    Ok(())
}
