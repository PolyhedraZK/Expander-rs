# BabyBear Field

## SIMD Support
SIMD support is imported from Plonky3 and requires certain target features for compilation.
### AVX512
Compilation requires `target-feature+=avx512f`, e.g.
```
RUSTFLAGS="-C target-feature=+avx512f" cargo build --release
```
