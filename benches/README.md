# Benchmark Suite

This directory contains performance benchmarks comparing biSere against other serialization libraries.

## Benchmarks

### `serialization_bench.rs`

Comprehensive benchmark suite comparing biSere against:
- **bincode**: Rust-native binary serialization
- **postcard**: Compact binary format
- **rmp-serde** (MessagePack): Compact binary format
- **serde_json**: JSON format (for reference)

**Benchmark Categories:**
1. **Serialization**: Time to convert data to bytes
2. **Deserialization**: Time to read data from bytes
3. **Round-trip**: Serialize + deserialize cycle
4. **Field Access**: Zero-copy field access vs full deserialization
5. **In-place Modification**: Updating fields without re-serialization

### `varying_sizes_bench.rs`

Tests serialization performance with varying data sizes:
- 1 struct
- 10 structs
- 100 structs
- 1000 structs

## Running Benchmarks

### Run all benchmarks:
```bash
cargo bench
```

### Run specific benchmark:
```bash
cargo bench --bench serialization_bench
cargo bench --bench varying_sizes_bench
```

### Run with specific options:
```bash
# Run with more iterations for better accuracy
cargo bench --bench serialization_bench -- --sample-size 100

# Save results to file
cargo bench --bench serialization_bench -- --output-format html > report.html

# Compare against previous run
cargo bench --bench serialization_bench -- --save-baseline baseline
cargo bench --bench serialization_bench -- --baseline baseline
```

## Interpreting Results

### Expected Performance Characteristics

**biSere should excel at:**
- **Deserialization**: Zero-copy means no allocation or copying
- **Field Access**: Direct pointer access is extremely fast
- **In-place Modification**: No re-serialization needed

**biSere may be slower at:**
- **Serialization**: Manual setup overhead (offset table construction)
- **Initial Setup**: Creating views requires validation

### Key Metrics

- **Mean time**: Average time per operation
- **Throughput**: Operations per second
- **Standard deviation**: Consistency of results
- **Buffer sizes**: Memory efficiency comparison

## Example Output

```
serialize/bisere          time:   [123.45 ns 125.67 ns 128.90 ns]
serialize/bincode         time:   [89.12 ns 91.34 ns 93.56 ns]

deserialize/bisere        time:   [12.34 ns 13.45 ns 14.56 ns]  ← Should be fastest!
deserialize/bincode       time:   [67.89 ns 70.12 ns 72.34 ns]

field_access/bisere       time:   [1.23 ns 1.45 ns 1.67 ns]     ← Zero-copy advantage!
field_access/bincode      time:   [67.89 ns 70.12 ns 72.34 ns]  ← Full deserialize needed
```

## Tips

1. **Run in release mode**: Benchmarks automatically use `--release`
2. **Warm up**: Criterion handles warm-up automatically
3. **Multiple runs**: Results are averaged over many iterations
4. **Compare buffer sizes**: Check memory efficiency
5. **Test different scenarios**: Use varying data sizes and structures

