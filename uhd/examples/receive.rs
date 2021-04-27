use std::{env::set_var, time::Duration};

use anyhow::{anyhow, Context, Result};
use num_complex::Complex32;
use tap::Pipe;
use uhd::{self, StreamCommand, StreamCommandType, TuneRequest, Usrp};

const CHANNEL: usize = 0;

pub fn main() -> Result<()> {
    set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    log::info!("Starting receive test");

    let mut usrp = Usrp::find("")
        .context("Failed to open device list")?
        .drain(..)
        .next()
        .context("Failed to find a valid USRP to attach to")?
        .pipe(|addr| Usrp::open(&addr))
        .context("Failed to find properly open the USRP")?;

    usrp.set_rx_sample_rate(1e6, CHANNEL)?;
    usrp.set_rx_antenna("TX/RX", CHANNEL)?;
    usrp.set_rx_frequency(&TuneRequest::with_frequency(2.4e9), CHANNEL)?;

    // std::thread::spawn(move || {
    // usrp.set_rx_sample_rate(2e6 as f64, CHANNEL)?;
    // usrp.set_rx_gain(50, channel, name)

    // dbg!(usrp.get_tx_antennas(CHANNEL)?);
    // dbg!(usrp.get_fe_tx_freq_range(CHANNEL)?);
    // dbg!(usrp.get_normalized_tx_gain(CHANNEL)?);

    // Set the stream type to be "fc32" which means "float complex 32"
    // This gets overridden anyway, because we use the Compelex3D format
    // See: https://files.ettus.com/manual/structuhd_1_1stream__args__t.html#a602a64b4937a85dba84e7f724387e252
    let mut receiver = usrp
        .get_rx_stream(&uhd::StreamArgs::<Complex32>::new("fc32"))
        .unwrap();

    // receiver
    //     .send_command(&StreamCommand {
    //         command_type: StreamCommandType::CountAndDone(10),
    //         time: uhd::StreamTime::Now,
    //     })
    //     .unwrap();
    // let out_buffers = (0..receiver.num_channels())
    //     .map(|_| vec![0; 10000].into_boxed_slice())
    //     .collect::<Vec<_>>()
    //     .as_slice();

    let mut single_chan = vec![Complex32::default(); 1_00].into_boxed_slice();
    let mut bufs = [single_chan.as_mut()];
    let stat = receiver.receive(&mut bufs, 1.0, false).unwrap();

    dbg!(stat);

    // log::info!("Samples received!");
    // log::info!("{:#?}", single_chan);
    // });
    // std::thread::sleep(Duration::from_millis(10000));

    Ok(())
}
