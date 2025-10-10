use criterion::black_box;
use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput};
use rs_merkle_tree::hasher::Hasher;
use rs_merkle_tree::stores::{MemoryStore, RocksDbStore, SledStore, SqliteStore};
use rs_merkle_tree::{
    //hasher::{Keccak256Hasher, PoseidonHasher},\
    hasher::Keccak256Hasher,
    node::Node,
    tree::MerkleTree,
    Store,
};

// Constants for the benchmarks
const BATCH_SIZE: u64 = 1000;
const NUM_BATCHES: u64 = 10;
const SAMPLE_SIZE: u64 = 10;

// Helper: accepts closures to create store and hasher so each iteration starts clean
fn bench_store<H, S, const DEPTH: usize, F1, F2>(
    b: &mut Bencher,
    mut make_store: F1,
    mut make_hasher: F2,
) where
    H: Hasher,
    S: Store,
    F1: FnMut() -> S,
    F2: FnMut() -> H,
{
    let mut tree: MerkleTree<H, S, DEPTH> = MerkleTree::new(make_hasher(), make_store());

    // TODO: Not the best benchmarks, not all inserts are equally expensive.

    b.iter(|| {
        for _ in 0..NUM_BATCHES {
            let leaves: Vec<Node> = (0..BATCH_SIZE)
                .map(|_| black_box(Node::random()))
                .collect::<Vec<Node>>();
            tree.add_leaves(&leaves).unwrap();
        }
    });
}

fn bench_insertions(c: &mut Criterion) {
    let mut group = c.benchmark_group("inserts");

    group
        .sample_size(SAMPLE_SIZE as usize)
        .warm_up_time(std::time::Duration::from_millis(500));

    // Depth 32 benchmarks Keccak256
    group.throughput(Throughput::Elements((NUM_BATCHES * BATCH_SIZE) as u64));
    group.bench_function(BenchmarkId::new("memory_store", "depth32_keccak256"), |b| {
        bench_store::<Keccak256Hasher, MemoryStore, 32, _, _>(
            b,
            || MemoryStore::new(),
            || Keccak256Hasher,
        )
    });
    group.throughput(Throughput::Elements((NUM_BATCHES * BATCH_SIZE) as u64));
    group.bench_function(BenchmarkId::new("sqlite_store", "depth32_keccak256"), |b| {
        let _ = std::fs::remove_file("sqlite.db");
        bench_store::<Keccak256Hasher, SqliteStore, 32, _, _>(
            b,
            || SqliteStore::new("sqlite.db"),
            || Keccak256Hasher,
        )
    });
    group.throughput(Throughput::Elements((NUM_BATCHES * BATCH_SIZE) as u64));
    group.bench_function(BenchmarkId::new("sled_store", "depth32_keccak256"), |b| {
        let _ = std::fs::remove_dir_all("sled.db");
        bench_store::<Keccak256Hasher, SledStore, 32, _, _>(
            b,
            || SledStore::new("sled.db", false),
            || Keccak256Hasher,
        )
    });
    group.throughput(Throughput::Elements((NUM_BATCHES * BATCH_SIZE) as u64));
    group.bench_function(
        BenchmarkId::new("rocksdb_store", "depth32_keccak256"),
        |b| {
            let _ = std::fs::remove_file("rocksdb.db");
            bench_store::<Keccak256Hasher, RocksDbStore, 32, _, _>(
                b,
                || RocksDbStore::new("rocksdb.db"),
                || Keccak256Hasher,
            )
        },
    );

    // Depth 32 benchmarks Poseidon
    // TODO: Benchmarks not working due to inputs being bigger than the prime
    /*
    group.throughput(Throughput::Elements((NUM_BATCHES * BATCH_SIZE) as u64));
    group.bench_function(BenchmarkId::new("memory_store", "depth32_poseidon"), |b| {
        bench_store::<PoseidonHasher, MemoryStore, 32, _, _>(
            b,
            || MemoryStore::new(),
            || PoseidonHasher,
        )
    });
    group.throughput(Throughput::Elements((NUM_BATCHES * BATCH_SIZE) as u64));
    group.bench_function(BenchmarkId::new("sqlite_store", "depth32_poseidon"), |b| {
        let _ = std::fs::remove_file("sqlite.db");
        bench_store::<PoseidonHasher, SqliteStore, 32, _, _>(
            b,
            || SqliteStore::new("sqlite.db"),
            || PoseidonHasher,
        )
    });
    group.throughput(Throughput::Elements((NUM_BATCHES * BATCH_SIZE) as u64));
    group.bench_function(BenchmarkId::new("sled_store", "depth32_poseidon"), |b| {
        let _ = std::fs::remove_dir_all("sled.db");
        bench_store::<PoseidonHasher, SledStore, 32, _, _>(
            b,
            || SledStore::new("sled.db", false),
            || PoseidonHasher,
        )
    });
    group.throughput(Throughput::Elements((NUM_BATCHES * BATCH_SIZE) as u64));
    group.bench_function(BenchmarkId::new("rocksdb_store", "depth32_poseidon"), |b| {
        let _ = std::fs::remove_file("rocksdb.db");
        bench_store::<PoseidonHasher, RocksDbStore, 32, _, _>(
            b,
            || RocksDbStore::new("rocksdb.db"),
            || PoseidonHasher,
        )
    });
     */

    // Cleanup
    let _ = std::fs::remove_file("sqlite.db");
    let _ = std::fs::remove_dir_all("sled.db");
    let _ = std::fs::remove_file("rocksdb.db");

    group.finish();
}

criterion_group!(benches, bench_insertions);
criterion_main!(benches);
