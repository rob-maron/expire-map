# expire-map

## A hyper-efficient, concurrent hashmap designed for eager value expiration. Underpinned by the power of [`DashMap`](https://github.com/xacrimon/dashmap) and [`tokio`](https://github.com/tokio-rs/tokio).

Note: Requires a `tokio` runtime.

Values being retrieved trigger an automatic expiry timer reset, rendering this library ideal for managing dynamic scenarios such as UDP streams or caches which require semi-stale data retention.

## Example
```rust
use std::time::Duration;

use expire_map::ExpireMap;

#[tokio::main]
async fn main() {
    // new expire map with 5 second expiry
    let mut my_map:ExpireMap<u32,u32> = ExpireMap::new(Duration::from_secs(5));

    // insert a value which should expire in 5 seconds
    my_map.insert(1,2);

    // wait 4 seconds
    tokio::time::sleep(Duration::from_secs(4)).await;

    // get the value, which still exists, resetting the timer
    let value = my_map.get(&1);
    assert_eq!(value.unwrap().0, 2);

    // wait 6 seconds (to ensure the value has now expired)
    tokio::time::sleep(Duration::from_secs(6)).await;

    // get the value, which is expired
    let value = my_map.get(&1);
    assert_eq!(value.is_none(), true);
}
```