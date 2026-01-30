use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

// 2026-01-27:
//   Results showed no meaningful difference among the possible implementations. Review of the
//   generated ASM shows that the only effective difference after optimization is the specific panic
//   messages produced when the input isn't valid.
fn criterion_benchmark(c: &mut Criterion) {
    let test_inputs = [
        (
            [0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF],
            0,
            "Zeros with zero offset",
        ),
        (
            [0x12, 0x34, 0x56, 0x78, 0xFF, 0xFF],
            0,
            "Non-zero with zero offset",
        ),
        (
            [0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00],
            2,
            "Zeros with non-zero offset",
        ),
        (
            [0xFF, 0xFF, 0x12, 0x34, 0x56, 0x78],
            2,
            "Non-zero with non-zero offset",
        ),
    ];

    let mut group = c.benchmark_group("u16");
    for (bytes, offset, description) in test_inputs {
        group.bench_with_input(
            BenchmarkId::new("read_le_u16_slice_copy", description),
            &(bytes, offset),
            |b, &(input_bytes, input_offset)| {
                b.iter(|| read_le_u16_slice_copy(&input_bytes, input_offset))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("read_le_u16_slice_copy_split", description),
            &(bytes, offset),
            |b, &(input_bytes, input_offset)| {
                b.iter(|| read_le_u16_slice_copy_split(&input_bytes, input_offset))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("read_le_u16_try_into", description),
            &(bytes, offset),
            |b, &(input_bytes, input_offset)| {
                b.iter(|| read_le_u16_try_into(&input_bytes, input_offset))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("read_le_u16_try_into_split", description),
            &(bytes, offset),
            |b, &(input_bytes, input_offset)| {
                b.iter(|| read_le_u16_try_into_split(&input_bytes, input_offset))
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("u32");
    for (bytes, offset, description) in test_inputs {
        group.bench_with_input(
            BenchmarkId::new("read_le_u32_slice_copy", description),
            &(bytes, offset),
            |b, (input_bytes, input_offset)| {
                b.iter(|| read_le_u32_slice_copy(input_bytes, *input_offset))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("read_le_u32_slice_copy_split", description),
            &(bytes, offset),
            |b, &(input_bytes, input_offset)| {
                b.iter(|| read_le_u32_slice_copy_split(&input_bytes, input_offset))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("read_le_u32_try_into", description),
            &(bytes, offset),
            |b, &(input_bytes, input_offset)| {
                b.iter(|| read_le_u32_try_into(&input_bytes, input_offset))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("read_le_u32_try_into_split", description),
            &(bytes, offset),
            |b, &(input_bytes, input_offset)| {
                b.iter(|| read_le_u32_try_into_split(&input_bytes, input_offset))
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn read_le_u16_slice_copy(bytes: &[u8], offset: usize) -> u16 {
    let mut value_bytes = [0; 2];
    value_bytes.copy_from_slice(&bytes[offset..offset + 2]);

    u16::from_le_bytes(value_bytes)
}

fn read_le_u16_slice_copy_split(bytes: &[u8], offset: usize) -> u16 {
    let mut value_bytes = [0; 2];
    value_bytes.copy_from_slice(&bytes.split_at(offset).1.split_at(2).0);

    u16::from_le_bytes(value_bytes)
}

fn read_le_u16_try_into(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_le_u16_try_into_split(bytes: &[u8], offset: usize) -> u16 {
    let value_bytes = bytes.split_at(offset).1.split_at(2).0;

    u16::from_le_bytes(value_bytes.try_into().unwrap())
}

fn read_le_u32_slice_copy(bytes: &[u8], offset: usize) -> u32 {
    let mut value_bytes = [0; 4];
    value_bytes.copy_from_slice(&bytes[offset..offset + 4]);

    u32::from_le_bytes(value_bytes)
}

fn read_le_u32_slice_copy_split(bytes: &[u8], offset: usize) -> u32 {
    let mut value_bytes = [0; 4];
    value_bytes.copy_from_slice(&bytes.split_at(offset).1.split_at(4).0);

    u32::from_le_bytes(value_bytes)
}

fn read_le_u32_try_into(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_le_u32_try_into_split(bytes: &[u8], offset: usize) -> u32 {
    let value_bytes = bytes.split_at(offset).1.split_at(4).0;

    u32::from_le_bytes(value_bytes.try_into().unwrap())
}
