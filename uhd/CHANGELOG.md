# Unreleased

* Added support for transmitting with TransmitStreamer
* Remove kind() and message() methods on Error. Error is now implemented with the ThisError
  crate internally
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

# 0.1.1 - 2021-03-30

* Fixes to compile with the version of UHD in the Raspberry Pi repositories (no public API changes, except panics in some situations)
