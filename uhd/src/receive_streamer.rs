use std::marker::PhantomData;

use crate::error::{check_status, Error};
use crate::receive_metadata::ReceiveMetadata;
use crate::stream::StreamCommand;
use crate::usrp::Usrp;
use std::os::raw::c_void;

/// A streamer used to receive samples from a USRP
///
/// The type parameter I is the type of sample that this streamer receives.
#[derive(Debug)]
pub struct ReceiveStreamer<'usrp, I> {
    /// Link to the USRP that this streamer is associated with
    pub(crate) usrp: PhantomData<&'usrp Usrp>,
    /// Streamer handle
    pub(crate) handle: uhd_sys::uhd_rx_streamer_handle,
    /// Item type phantom data
    pub(crate) item_phantom: PhantomData<I>,
}

impl<I> ReceiveStreamer<'_, I> {
    pub fn send_command(&self, command: &StreamCommand) -> Result<(), Error> {
        let command_c = command.as_c_command();
        check_status(unsafe { uhd_sys::uhd_rx_streamer_issue_stream_cmd(self.handle, &command_c) })
    }

    pub fn receive(
        &mut self,
        buffer: &mut [I],
        timeout: f64,
        one_packet: bool,
    ) -> Result<(ReceiveMetadata, usize), Error> {
        let mut metadata = ReceiveMetadata::default();
        let mut samples_received = 0usize;

        let mut buffers: [*mut c_void; 1] = [buffer.as_mut_ptr() as *mut c_void];
        check_status(unsafe {
            uhd_sys::uhd_rx_streamer_recv(
                self.handle,
                buffers.as_mut_ptr(),
                buffer.len() as _,
                metadata.handle_mut(),
                timeout,
                one_packet,
                &mut samples_received as *mut usize as *mut _,
            )
        })?;

        Ok((metadata, samples_received))
    }
}

impl<I> Drop for ReceiveStreamer<'_, I> {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_rx_streamer_free(&mut self.handle) };
    }
}

// Thread safety: see https://files.ettus.com/manual/page_general.html#general_threading
// All functions are thread-safe, except that the uhd_tx_streamer send(), uhd_rx_streamer recv(), and
// uhd_rx_streamer recv_async_msg() functions. The corresponding Rust wrapper functions take &mut
// self, which enforces single-thread access.
unsafe impl<I> Send for ReceiveStreamer<'_, I> {}
unsafe impl<I> Sync for ReceiveStreamer<'_, I> {}
