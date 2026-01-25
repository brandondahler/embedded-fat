use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

#[path = "../src/encoding/ucs2_character/case_folding.rs"]
mod ucs2_character_case_folding;

use ucs2_character_case_folding::fold_character;
use ucs2_character_case_folding::tests::unoptimized_fold_character;

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

    let mut group = c.benchmark_group("Case Folding");
    for (character, description) in characters {
        let character_code = character as u16;
        let full_description = format!("{description}: {character} (\\u{{{character_code:04X}}})");

        group.bench_with_input(
            BenchmarkId::new("Optimized", &full_description),
            &character_code,
            |b, input| b.iter(|| fold_character(*input)),
        );
        group.bench_with_input(
            BenchmarkId::new("Unoptimized", &full_description),
            &character_code,
            |b, input| b.iter(|| unoptimized_fold_character(*input)),
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
