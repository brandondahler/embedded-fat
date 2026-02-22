use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

// NOTE: This is only possible because this module is designed to be standalone
#[path = "../src/encoding/unicode/case_folding.rs"]
mod unicode_case_folding;

use unicode_case_folding::fold_codepoint;
use unicode_case_folding::tests::unoptimized_fold_codepoint;

fn criterion_benchmark(c: &mut Criterion) {
    let characters = [
        ('a', "ASCII lowercase, special case hit"),
        ('A', "ASCII uppercase, special case miss"),
        ('1', "ASCII number, special case"),
        ('$', "ASCII symbol, special case"),
        ('À', "Latin supplement uppercase, early range hit"),
        ('à', "Latin supplement lowercase, early range miss"),
        ('ꭰ', "Cherokee supplement, late range hit"),
        ('Ꭰ', "Cherokee, late range miss"),
        ('µ', "Latin supplement, early lookup hit"),
        ('μ', "Greek and coptic, early lookup miss"),
        ('ﬅ', "Alphabetic presentation, late lookup hit"),
        ('ﬆ', "Alphabetic presentation, late lookup miss"),
    ];

    let mut group = c.benchmark_group("Unicode Case Folding");
    for (character, description) in characters {
        let character_code = character as u32;
        let full_description = format!("{description}: {character} (\\u{{{character_code:04X}}})");

        group.bench_with_input(
            BenchmarkId::new("Optimized", &full_description),
            &character_code,
            |b, &input| b.iter(|| fold_codepoint(input)),
        );
        group.bench_with_input(
            BenchmarkId::new("Unoptimized", &full_description),
            &character_code,
            |b, &input| b.iter(|| unoptimized_fold_codepoint(input)),
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
