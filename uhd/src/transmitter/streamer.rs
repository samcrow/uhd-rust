use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;

use crate::{
    error::{check_status, Error},
    usrp::Usrp,
    utils::check_equal_buffer_lengths,
    TransmitMetadata,
};

/// A streamer used to transmit samples from a USRP
///
/// The type parameter I is the type of sample that this streamer transmits.
#[derive(Debug)]
pub struct TransmitStreamer<'usrp, I> {
    /// Streamer handle
    handle: uhd_sys::uhd_tx_streamer_handle,

    /// A vector of pointers to buffers (used in transmit() to convert `&mut [&[I]]` to `*mut *const I`
    /// without reallocating memory each time
    ///
    /// Invariant: If this is not empty, its length is equal to the value returned by
    /// self.num_channels().
    buffer_pointers: Vec<*const c_void>,
    /// Link to the USRP that this streamer is associated with
    usrp: PhantomData<&'usrp Usrp>,
    /// Item type phantom data
    item_phantom: PhantomData<I>,
}

impl<I> TransmitStreamer<'_, I> {
    /// Creates a transmit streamer with a null streamer handle (for internal use only)
    ///
    /// After creating a streamer with this function, its streamer handle must be initialized.
    pub(crate) fn new() -> Self {
        TransmitStreamer {
            handle: ptr::null_mut(),
            buffer_pointers: Vec::new(),
            usrp: PhantomData,
            item_phantom: PhantomData,
        }
    }

    /// Returns a reference to the streamer handle
    pub(crate) fn handle_mut(&mut self) -> &mut uhd_sys::uhd_tx_streamer_handle {
        &mut self.handle
    }
    /// Returns the streamer handle
    pub(crate) fn handle(&mut self) -> uhd_sys::uhd_tx_streamer_handle {
        self.handle
    }

    /// Returns the number of channels that this streamer is associated with
    pub fn num_channels(&self) -> usize {
        let mut num_channels = 0usize;
        check_status(unsafe {
            uhd_sys::uhd_tx_streamer_num_channels(
                self.handle,
                &mut num_channels as *mut usize as *mut _,
            )
        })
        .unwrap();
        num_channels
    }

    /// Transmits samples with caller-provided metadata.
    ///
    /// Use this when you need to set `start_of_burst`, `end_of_burst`, or a time spec
    /// on the send (e.x for bursty TX, see [`TransmitMetadata::with_flags`])
    ///
    /// `buffers`: one or more sample buffers (one per channel). Panics if the number
    /// of buffers doesn't match `self.num_channels()` or if buffer lengths differ.
    ///
    /// `metadata`: caller-owned. The `samples` field is updated in place with the
    /// number of samples actually accepted by the streamer.
    ///
    /// `timeout`: the timeout for the send operation, in seconds.
    ///
    /// Returns the number of samples actually accepted by the streamer (may be less
    /// than the buffer length on partial sends).
    pub fn send(
        &mut self,
        buffers: &mut [&[I]],
        metadata: &mut TransmitMetadata,
        timeout: f64,
    ) -> Result<usize, Error> {
        let mut samples_transmitted = 0usize;

        // Initialize buffer_pointers
        if self.buffer_pointers.is_empty() {
            self.buffer_pointers
                .resize(self.num_channels(), ptr::null_mut());
        }
        // Now buffer_pointers.len() is equal to self.num_channels().
        assert_eq!(
            buffers.len(),
            self.buffer_pointers.len(),
            "Number of buffers is not equal to this streamer's number of channels"
        );
        // Check that all buffers have the same length
        let buffer_length = check_equal_buffer_lengths(buffers);

        // Copy buffer pointers into C-compatible form
        for (entry, buffer) in self.buffer_pointers.iter_mut().zip(buffers.iter_mut()) {
            *entry = buffer.as_ptr() as *mut c_void;
        }

        check_status(unsafe {
            uhd_sys::uhd_tx_streamer_send(
                self.handle,
                self.buffer_pointers.as_mut_ptr(),
                buffer_length as _,
                metadata.handle_mut(),
                timeout,
                &mut samples_transmitted as *mut usize as *mut _,
            )
        })?;
        metadata.set_samples(samples_transmitted);

        Ok(samples_transmitted)
    }

    /// Transmits samples from the USRP using default (continuous-stream) metadata —
    /// `start_of_burst = false`, `end_of_burst = false`, no time spec.
    ///
    /// For bursty transmission (discrete packets separated by gaps), use `send` with
    /// `TransmitMetadata::with_flags` instead.
    ///
    /// `buffers`: one or more sample buffers (one per channel). Panics if the number
    /// of buffers doesn't match `self.num_channels()` or if buffer lengths differ.
    ///
    /// `timeout`: the timeout for the transmit operation, in seconds.
    ///
    /// On success, returns the freshly-built `TransmitMetadata` with `samples()`
    /// reflecting the number of samples actually accepted by the streamer.
    pub fn transmit(
        &mut self,
        buffers: &mut [&[I]],
        timeout: f64,
    ) -> Result<TransmitMetadata, Error> {
        let mut metadata = TransmitMetadata::default();
        self.send(buffers, &mut metadata, timeout)?;
        Ok(metadata)
    }

    /// transmits samples on a single channel with a timeout of 0.1 seconds and
    /// one_packet disabled
    pub fn transmit_simple(&mut self, buffer: &mut [I]) -> Result<TransmitMetadata, Error> {
        self.transmit(&mut [buffer], 0.1)
    }
}

impl<I> Drop for TransmitStreamer<'_, I> {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_tx_streamer_free(&mut self.handle) };
    }
}

// Thread safety: see https://files.ettus.com/manual/page_general.html#general_threading
// All functions are thread-safe, except that the uhd_tx_streamer send(), uhd_tx_streamer recv(), and
// uhd_tx_streamer recv_async_msg() functions. The corresponding Rust wrapper functions take &mut
// self, which enforces single-thread access.
unsafe impl<I> Send for TransmitStreamer<'_, I> {}
unsafe impl<I> Sync for TransmitStreamer<'_, I> {}
