# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v0.6.0] - 2023-08-03

- Hide nix from the public api such that it can be updated without resulting in a breaking change.
- Updated nix to version `0.26`; only use the `ioctl` feature.
- Updated `bitflags` to version `2.3`.
- Use `File.read_exact` instead of `File.read` in `LinuxI2CDevice.read` so that the buffer is filled.
- Fix the lifetime parameter on `LinuxI2CMessage` to ensure that it does not outlive the buffer it points to.
- Updated MSRV to 1.60.0.

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

[Unreleased]: https://github.com/rust-embedded/rust-i2cdev/compare/0.6.0...HEAD
[v0.6.0]: https://github.com/rust-embedded/rust-i2cdev/compare/0.5.1...0.6.0
[v0.5.1]: https://github.com/rust-embedded/rust-i2cdev/compare/0.5.0...0.5.1
[v0.5.0]: https://github.com/rust-embedded/rust-i2cdev/compare/0.4.0...0.5.0
