---
name: Compilation error
about: Report a problem compiling Specsheet
---

If Specsheet fails to compile, or if there is a problem during the build process, then please include the following information in your report:

- The exact Specsheet commit you are building (`git rev-parse --short HEAD`)
- The version of rustc you are compiling it with (`rustc --version`)
- Your operating system and hardware platform
- The Rust build target (the _exact_ output of `rustc --print cfg`)

If you are seeing compilation errors, please include the output of the build process.

---
