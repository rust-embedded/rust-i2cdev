# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [v0.5.1] - 2021-11-22

- Updated nix to version `0.23`.
- Add PEC support for SMBus compatible adapters.
- Add `LinuxI2CDevice::force_new()` to open the device without checking if the address is bound to a driver.

## [v0.5.0] - 2021-09-21

- Updated `nix` to version `0.22`.
- Updated `bitflags` to version `1.3`.
- Updated MSRV to version 1.46.0.

## v0.4.4 - 2020-12-02

The changelog for previous versions was not recorded.

[Unreleased]: https://github.com/rust-embedded/rust-i2cdev/compare/0.5.1...HEAD
[v0.5.1]: https://github.com/rust-embedded/rust-i2cdev/compare/0.5.0...0.5.1
[v0.5.0]: https://github.com/rust-embedded/rust-i2cdev/compare/0.4.0...0.5.0
