use criterion::{criterion_group, criterion_main, Criterion};
use open_data_structures::data_structure::array_stack::ArrayStack;
use open_data_structures::interface::list::List;

fn array_stack_bench(c: &mut Criterion) {
    c.bench_function("ArrayStack Bench", |b| b.iter(|| {
        let mut array = ArrayStack::new(6);
        for i in 0..100 {
            array.add(i, i.to_string());
        }
        for _i in 0..100 {
            array.remove(0);
        }
    }));
}

criterion_group!(benches, array_stack_bench);
criterion_main!(benches);