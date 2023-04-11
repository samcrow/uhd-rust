# Unreleased
* Remove `enumerate_registers` from `Usrp` since the low level register apis have been removed from libuhd 4.0 .

# [0.2.0](https://github.com/samcrow/canadensis/tree/v0.2.0) - 2023-02-09

* Added support for transmitting with TransmitStreamer
* Remove kind() and message() methods on Error. Error is now implemented with the ThisError
  crate internally. Error message strings from UHD can instead be retrieved using
  `uhd::last_error_message()`.
* Bump num-complex crate to v0.4
* The following methods on `Ursp` now require a mutable reference:
  `clear_command_time`
  `get_rx_stream`
  `set_rx_agc_enabled`
  `set_rx_antenna`
  `set_rx_bandwidth`
  `set_rx_dc_offset_enabled`
  `set_rx_frequency`
  `set_rx_gain`
  `set_rx_sample_rate`
  `set_tx_antenna`
* Use `sc16` over-the-wire format in transmit and receive examples

# 0.1.1 - 2021-03-30

* Fixes to compile with the version of UHD in the Raspberry Pi repositories (no public API changes, except panics in some situations)

