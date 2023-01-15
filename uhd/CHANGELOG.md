# Unreleased

* Added support for transmitting with TransmitStreamer
* Remove kind() and message() methods on Error. Error is now implemented with the ThisError
  crate internally

# 0.1.1 - 2021-03-30

* Fixes to compile with the version of UHD in the Raspberry Pi repositories (no public API changes, except panics in some situations)
