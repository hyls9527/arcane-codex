use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl<T> CacheEntry<T> {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

#[derive(Debug)]
pub struct Cache<K, V> {
    entries: RwLock<HashMap<K, CacheEntry<V>>>,
    default_ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read().unwrap();
        entries.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.value.clone())
            }
        })
    }

    pub fn set(&self, key: K, value: V) {
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            ttl: self.default_ttl,
        };
        self.entries.write().unwrap().insert(key, entry);
    }

    pub fn set_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            ttl,
        };
        self.entries.write().unwrap().insert(key, entry);
    }

    pub fn remove(&self, key: &K) -> bool {
        self.entries.write().unwrap().remove(key).is_some()
    }

    pub fn clear(&self) {
        self.entries.write().unwrap().clear();
    }

    pub fn invalidate_expired(&self) -> usize {
        let mut entries = self.entries.write().unwrap();
        let before = entries.len();
        entries.retain(|_, entry| !entry.is_expired());
        before - entries.len()
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }
}

pub type ImageListCache = Cache<String, Vec<serde_json::Value>>;
pub type AIResultCache = Cache<i64, serde_json::Value>;

pub fn create_image_list_cache() -> Arc<ImageListCache> {
    Arc::new(Cache::new(Duration::from_secs(30)))
}

pub fn create_ai_result_cache() -> Arc<AIResultCache> {
    Arc::new(Cache::new(Duration::from_secs(300)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_and_get() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set("key1".to_string(), "value1".to_string());

        let result = cache.get(&"key1".to_string());
        assert_eq!(result, Some("value1".to_string()));
    }

    #[test]
    fn test_cache_get_nonexistent() {
        let cache: Cache<String, String> = Cache::new(Duration::from_secs(60));
        let result = cache.get(&"nonexistent".to_string());
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_expiry() {
        let cache = Cache::new(Duration::from_millis(50));
        cache.set("key1".to_string(), "value1".to_string());

        assert!(cache.get(&"key1".to_string()).is_some());

        std::thread::sleep(Duration::from_millis(100));
        assert!(cache.get(&"key1".to_string()).is_none());
    }

    #[test]
    fn test_cache_set_with_custom_ttl() {
        let cache: Cache<String, String> = Cache::new(Duration::from_secs(3600));
        cache.set_with_ttl("key1".to_string(), "value1".to_string(), Duration::from_millis(50));

        assert!(cache.get(&"key1".to_string()).is_some());

        std::thread::sleep(Duration::from_millis(100));
        assert!(cache.get(&"key1".to_string()).is_none());
    }

    #[test]
    fn test_cache_remove() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set("key1".to_string(), "value1".to_string());

        assert!(cache.remove(&"key1".to_string()));
        assert!(cache.get(&"key1".to_string()).is_none());
    }

    #[test]
    fn test_cache_remove_nonexistent() {
        let cache: Cache<String, String> = Cache::new(Duration::from_secs(60));
        assert!(!cache.remove(&"nonexistent".to_string()));
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_invalidate_expired() {
        let cache = Cache::new(Duration::from_secs(60));
        cache.set_with_ttl("key1".to_string(), "value1".to_string(), Duration::from_millis(50));
        cache.set("key2".to_string(), "value2".to_string());

        std::thread::sleep(Duration::from_millis(100));

        let removed = cache.invalidate_expired();
        assert_eq!(removed, 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_len_and_is_empty() {
        let cache: Cache<String, String> = Cache::new(Duration::from_secs(60));
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        cache.set("key1".to_string(), "value1".to_string());
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_entry_is_expired() {
        let entry = CacheEntry {
            value: "test".to_string(),
            created_at: Instant::now(),
            ttl: Duration::from_secs(60),
        };
        assert!(!entry.is_expired());

        let expired_entry = CacheEntry {
            value: "test".to_string(),
            created_at: Instant::now() - Duration::from_secs(100),
            ttl: Duration::from_secs(60),
        };
        assert!(expired_entry.is_expired());
    }

    #[test]
    fn test_concurrent_cache_access() {
        let cache = Arc::new(Cache::new(Duration::from_secs(60)));
        let mut handles = vec![];

        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = std::thread::spawn(move || {
                cache_clone.set(format!("key{}", i), format!("value{}", i));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(cache.len(), 10);
    }

    #[test]
    fn test_create_image_list_cache() {
        let cache = create_image_list_cache();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_create_ai_result_cache() {
        let cache = create_ai_result_cache();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }
}
