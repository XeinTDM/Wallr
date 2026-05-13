use crate::storage::get_pool;
use bk_tree::BKTree;
use std::sync::RwLock;

pub struct Hamming;
impl bk_tree::Metric<Vec<u8>> for Hamming {
    fn distance(&self, a: &Vec<u8>, b: &Vec<u8>) -> u32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum()
    }

    fn threshold_distance(&self, a: &Vec<u8>, b: &Vec<u8>, threshold: u32) -> Option<u32> {
        let dist = self.distance(a, b);
        if dist <= threshold { Some(dist) } else { None }
    }
}

pub static BANNED_HASH_TREE: std::sync::OnceLock<
    std::sync::RwLock<bk_tree::BKTree<Vec<u8>, Hamming>>,
> = std::sync::OnceLock::new();

pub static HASH_TREE_LOADED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub async fn check_banned_phash(phash: &[u8]) -> anyhow::Result<bool> {
    let tree_lock = BANNED_HASH_TREE.get_or_init(|| RwLock::new(BKTree::new(Hamming)));

    if !HASH_TREE_LOADED.load(std::sync::atomic::Ordering::SeqCst) {
        let pool = get_pool()?;
        let rows = sqlx::query!("SELECT phash FROM banned_hashes")
            .fetch_all(pool)
            .await?;

        let mut tree = tree_lock.write().unwrap();
        if !HASH_TREE_LOADED.load(std::sync::atomic::Ordering::SeqCst) {
            for row in rows {
                let banned: Vec<u8> = row.phash;
                tree.add(banned);
            }
            HASH_TREE_LOADED.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    let tree = tree_lock.read().unwrap();
    let phash_vec = phash.to_vec();
    let mut matches = tree.find(&phash_vec, 5);

    Ok(matches.next().is_some())
}

pub async fn add_banned_hash(phash: &[u8], admin_id: &str, reason: &str) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO banned_hashes (id, phash, reason, added_by) VALUES ($1, $2, $3, $4)",
        id,
        phash,
        reason,
        admin_id
    )
    .execute(pool)
    .await?;

    if let Some(tree_lock) = BANNED_HASH_TREE.get()
        && let Ok(mut tree) = tree_lock.write()
    {
        tree.add(phash.to_vec());
    }

    Ok(())
}

pub async fn check_banned_embedding(embedding: &[f32]) -> anyhow::Result<bool> {
    let pool = get_pool()?;

    let vector_embedding = pgvector::Vector::from(embedding.to_vec());

    let is_banned = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM banned_embeddings WHERE embedding <=> $1 < 0.05)",
        vector_embedding as pgvector::Vector
    )
    .fetch_one(pool)
    .await?;

    Ok(is_banned.unwrap_or(false))
}

pub async fn check_banned_exact_hash(sha256_hex: &str) -> anyhow::Result<bool> {
    let pool = get_pool()?;

    let is_banned = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM banned_exact_hashes WHERE sha256 = $1)",
        sha256_hex
    )
    .fetch_one(pool)
    .await?;

    Ok(is_banned.unwrap_or(false))
}

pub async fn add_banned_exact_hash(
    sha256_hex: &str,
    admin_id: &str,
    reason: &str,
) -> anyhow::Result<()> {
    let pool = get_pool()?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO banned_exact_hashes (id, sha256, reason, added_by) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        id, sha256_hex, reason, admin_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
