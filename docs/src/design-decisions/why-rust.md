# Programming Language
The course suggestion is to program in C (or C++) and use [STM32CubeIDE](https://www.st.com/en/development-tools/stm32cubeide.html)
to generate the microcontroller-configuration (setting pins as input/output, setting up I2C, etc.), but this was not a
hard requirement.

I chose Rust for the following reasons:
* It is a systems programming language with similar performance to C/C++
* The language offers many memory-safety related features which C & C++ lack
  * There are many companies and projects which have done research on this and come to the conclusion that this can't be solved
    with C/C++ and are moving towards Rust, e.g.:
    * [Linux Kernel 6.1 with Rust Support for Drivers](https://www.infoq.com/news/2022/12/linux-6-1-rust/)
    * [Android: Memory Safe Languages in Android 13](https://security.googleblog.com/2022/12/memory-safe-languages-in-android-13.html)
    * [Microsoft Security Response Center: Blog Series on Memory Safety](https://msrc-blog.microsoft.com/tag/rust/)
    * [Microsoft Azure CTO Statement](https://www.theregister.com/2022/09/20/rust_microsoft_c/)
    * [AWS loves Rust](https://aws.amazon.com/blogs/opensource/why-aws-loves-rust-and-how-wed-like-to-help/)
    * [Mozilla is oxidating Firefox](https://wiki.mozilla.org/Oxidation)
    * [Chromium supports Rust](https://security.googleblog.com/2023/01/supporting-use-of-rust-in-chromium.html)
    * [Tangram Vision: Why Rust for Robots?](https://www.tangramvision.com/blog/why-rust-for-robots)
    * [tonari: 3K, 60fps, 130ms: achieving it with Rust](https://blog.tonari.no/why-we-love-rust)
* Embedded development is nowadays well-supported in Rust:
  * [Active community around it](https://www.rust-lang.org/what/embedded) (incl. [a working group](https://github.com/rust-embedded/))
  * `thumbv7em-none-eabihf` (for our STM32F4 chip) is a [tier 2 target for the compiler](https://doc.rust-lang.org/rustc/platform-support.html#tier-2)
  * HAL implementations available for all relevant chips
  * Drivers (target-agnostic, based on `embedded-hal` traits) exist for a lot of peripherals
  * Frameworks for many common functionalities exist
  * Various RTOS and RTOS-like frameworks exist
* Due to the [`embedded-hal`](https://crates.io/crates/embedded-hal) abstraction most of the code can be written in a 
  device-agnostic manner, making it more portable.
* The language offers many more modern, concise ways of writing code compared to C/C++
* Personal reasons
  * While I had used Rust before, I had never used it for embedded development and wanted to explore this use-case
  * I wanted to show Rust as an alternative to C/C++ for the FHGR
