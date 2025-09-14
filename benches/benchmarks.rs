use criterion::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rs_merkle_tree::node::Node;
use rs_merkle_tree::store::MemoryStore;
use rs_merkle_tree::{hasher::Keccak256Hasher, store::SledStore, tree::GenericMerkleTree};

fn bench_insertions(c: &mut Criterion) {
    std::fs::remove_dir_all("sled.db").ok();
    let mut group = c.benchmark_group("sled_store");
    let total_inserts = 5_000;
    let batch_size = 1_000;

    group.throughput(Throughput::Elements(total_inserts));
    group.bench_function(
        BenchmarkId::new("sled_store_test", "add_leaves_keccak_depth_32"),
        |b| {
            b.iter(|| {
                let mut tree: GenericMerkleTree<Keccak256Hasher, SledStore, 32> =
                    GenericMerkleTree::new(Keccak256Hasher, SledStore::new("sled.db", false));

                let num_batches = total_inserts / batch_size;

                for _ in 0..num_batches {
                    let leaves: Vec<Node> =
                        (0..batch_size).map(|_| black_box(Node::random())).collect();
                    tree.add_leaves(&leaves).unwrap();
                }
            });
        },
    );
    group.finish();

    let mut group = c.benchmark_group("memory_store");
    group.throughput(Throughput::Elements(total_inserts));
    group.bench_function(
        BenchmarkId::new("memory_store", "add_leaves_keccak_depth_32"),
        |b| {
            b.iter(|| {
                let mut tree: GenericMerkleTree<Keccak256Hasher, MemoryStore, 32> =
                    GenericMerkleTree::new(Keccak256Hasher, MemoryStore::new());

                let num_batches = total_inserts / batch_size;

                for _ in 0..num_batches {
                    let leaves: Vec<Node> =
                        (0..batch_size).map(|_| black_box(Node::random())).collect();
                    tree.add_leaves(&leaves).unwrap();
                }
            });
        },
    );

    group.finish();
}

criterion_group!(benches, bench_insertions);
criterion_main!(benches);
