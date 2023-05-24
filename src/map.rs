use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use futures::ready;
use std::{
    borrow::Borrow,
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
    task::{Context, Poll},
    time::Duration,
};
use tokio_util::time::{delay_queue, DelayQueue};
pub struct ExpireMap<K, V, S = RandomState> {
    underlying_map: DashMap<K, (V, delay_queue::Key), S>,
    delay_queue: DelayQueue<K>,
    timeout: Duration,
}
impl<'a, K: 'a + Eq + Hash + Clone, V: 'a> ExpireMap<K, V, RandomState> {
    pub fn new(duration: Duration) -> Self {
        Self::with_duration_and_hasher(duration, RandomState::default())
    }
}
impl<'a, K: 'a + Eq + Hash + Clone, V: 'a, S: BuildHasher + Clone> ExpireMap<K, V, S> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let delay_key = self.delay_queue.insert(key.clone(), self.timeout);
        if let Some(entry) = self.underlying_map.insert(key, (value, delay_key)) {
            self.delay_queue.remove(&delay_key);
            Some(entry.0)
        } else {
            None
        }
    }
}
impl<'a, K: 'a + Eq + Hash + Clone, V: 'a, S: BuildHasher + Clone + Default> ExpireMap<K, V, S> {
    pub fn with_duration_and_hasher(duration: Duration, hasher: S) -> Self {
        Self {
            underlying_map: DashMap::with_hasher(hasher),
            delay_queue: DelayQueue::new(),
            timeout: duration,
        }
    }
    pub fn get<Q>(&'a mut self, key: &Q) -> Option<Ref<'a, K, (V, delay_queue::Key), S>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.purge();
        if let Some(entry) = self.underlying_map.get(key) {
            self.delay_queue.reset(&entry.1, self.timeout);
            Some(entry)
        } else {
            None
        }
    }
    pub fn get_mut<Q>(&'a mut self, key: &Q) -> Option<RefMut<'a, K, (V, delay_queue::Key), S>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.purge();
        if let Some(entry) = self.underlying_map.get_mut(key) {
            self.delay_queue.reset(&entry.1, self.timeout);
            Some(entry)
        } else {
            None
        }
    }
    pub fn poll_purge(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        while let Some(entry) = ready!(self.delay_queue.poll_expired(cx)) {
            self.underlying_map.remove(entry.get_ref());
        }
        Poll::Ready(())
    }
    pub fn purge(&mut self){
        let mut cx = std::task::Context::from_waker(futures::task::noop_waker_ref());
        let _ = self.poll_purge(&mut cx);
    }
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(entry) = self.underlying_map.remove(key) {
            self.delay_queue.remove(&entry.1 .1);
            Some(entry.1 .0)
        } else {
            None
        }
    }
    pub fn len(&self) -> usize {
        self.underlying_map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.underlying_map.is_empty()
    }
    pub fn clear(&mut self) {
        self.underlying_map.clear();
        self.delay_queue.clear();
    }
    pub fn contains_key<Q>(&mut self, key: &Q) -> bool
where
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized{
        self.purge();
        self.underlying_map.contains_key(key)
    }
}