# Goose DSP

Goose DSP is a guitar amp simulator built in Rust.

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## TODO

- [x] Select and manage audio devices
- [ ] Proper bit depth and buffer size selection
- [ ] Get rid of audio glitches
- [ ] Linux and macos support
- [ ] FX modules
   - [x] Overdrive
   - [x] EQ
   - [x] Noise gate
   - [ ] Delay
   - [ ] Reverb
- [ ] Impulse responses or proper cab simulation
- [ ] Preset system
- [ ] Settings

## Requirements

- Rust (version 1.56 or later)
- Cargo

## Installation

1. **Clone the repository:**

   ```bash
   git clone https://github.com/obsqrbtz/goose_dsp.git
   cd goose_dsp
   ```

2. **Build the project:**

   ```bash
   cargo build
   ```
> **Notice:** `clang` is required for building ASIO SDK. LLVM should be installed and `CXX` variable should not be pointing to any other compiler. For more information refer to [CPAL docs](https://github.com/RustAudio/cpal/tree/master?tab=readme-ov-file#asio-on-windows).

3. **Run the application:**

   ```bash
   cargo run --bin goose_dsp
   ```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any enhancements or bug fixes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [cpal](https://crates.io/crates/cpal) - Cross-platform audio I/O library
- [hound](https://crates.io/crates/hound) - WAV file reading and writing library
- [eframe](https://crates.io/crates/eframe) - Framework for building native applications with `egui`
