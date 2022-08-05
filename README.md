# ne-g3

_ne-g3_ is a standalone, [G3 PLC](https://www.itu.int/rec/T-REC-G.9903) coordinator and modem implementation
for the Microchip G3 Stack.


## Features
- Runs in coordinator or device mode.
- Linux and MacOS support.


## Usage

Coordinator:
```sh
RUST_LOG=debug cargo run coordinator -d <USI_SERIAL_DEVICE_NAME>
```

Device:
```sh
RUST_LOG=debug cargo run modem -d <USI_SERIAL_DEVICE_NAME>
```

The application creates a [TUN](https://www.kernel.org/doc/html/latest/networking/tuntap.html) device under linux and UTUN under MacOS. It is therefore essential that the user running the application has the proper permissions.

### Configuration
By default, the application uses ne-g3.toml for configuring various aspects for the G3 Stack and IPv6 network, this can be overriden by -c command line option.

The application will configure the interface with two IPv6 addresses. One is the mandatory link local (LL) and Unique Local Address (ULA).
## License

_ne-g3_ is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE and LICENSE-MIT for details.
