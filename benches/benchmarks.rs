use criterion::black_box;
use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput};
use rs_merkle_tree::store::Store;
use rs_merkle_tree::{hasher::Keccak256Hasher, node::Node, tree::MerkleTree};

#[cfg(feature = "memory_store")]
use rs_merkle_tree::store::MemoryStore;
#[cfg(feature = "rocksdb_store")]
use rs_merkle_tree::store::RocksDbStore;
#[cfg(feature = "sled_store")]
use rs_merkle_tree::store::SledStore;
#[cfg(feature = "sqlite_store")]
use rs_merkle_tree::store::SqliteStore;

// Constants for the benchmarks
const TOTAL_INSERTS: usize = 5_000;
const BATCH_SIZE: usize = 1_000;

// Helper
fn bench_store<S: Store + 'static, const DEPTH: usize, F>(b: &mut Bencher, mut make_store: F)
where
    F: FnMut() -> S,
{
    b.iter(|| {
        let mut tree: MerkleTree<Keccak256Hasher, S, DEPTH> =
            MerkleTree::new(Keccak256Hasher, make_store());

        let num_batches = TOTAL_INSERTS / BATCH_SIZE;
        for _ in 0..num_batches {
            let leaves: Vec<Node> = (0..BATCH_SIZE).map(|_| black_box(Node::random())).collect();
            tree.add_leaves(&leaves).unwrap();
        }
    });
}

fn bench_insertions(c: &mut Criterion) {
    let mut group = c.benchmark_group("inserts");
    group.throughput(Throughput::Elements(TOTAL_INSERTS as u64));
    group.sample_size(10);

    // TODO: Find a clearner way to do this.
    let _ = std::fs::remove_file("sqlite.db");
    let _ = std::fs::remove_dir_all("sled.db");
    let _ = std::fs::remove_file("rocksdb.db");

    // Depth 32 benchmarks
    #[cfg(feature = "sqlite_store")]
    group.bench_function(BenchmarkId::new("sqlite_store", "depth32_keccak256"), |b| {
        let _ = std::fs::remove_file("sqlite.db");
        bench_store::<SqliteStore, 32, _>(b, || SqliteStore::new("sqlite.db"))
    });
    #[cfg(feature = "sled_store")]
    group.bench_function(BenchmarkId::new("sled_store", "depth32_keccak256"), |b| {
        let _ = std::fs::remove_dir_all("sled.db");
        bench_store::<SledStore, 32, _>(b, || SledStore::new("sled.db", false))
    });
    #[cfg(feature = "memory_store")]
    group.bench_function(BenchmarkId::new("memory_store", "depth32_keccak256"), |b| {
        bench_store::<MemoryStore, 32, _>(b, || MemoryStore::new())
    });

    #[cfg(feature = "rocksdb_store")]
    group.bench_function(
        BenchmarkId::new("rocksdb_store", "depth32_keccak256"),
        |b| {
            let _ = std::fs::remove_file("rocksdb.db");
            bench_store::<RocksDbStore, 32, _>(b, || RocksDbStore::new("rocksdb.db"))
        },
    );

    let _ = std::fs::remove_file("sqlite.db");
    let _ = std::fs::remove_dir_all("sled.db");
    let _ = std::fs::remove_file("rocksdb.db");

    group.finish();
}

criterion_group!(benches, bench_insertions);
criterion_main!(benches);
