# SafeAnyMap

Usage:


## Maps
```rust
use safe_any_map::SafeAnyMap;

fn main() {
    let mut map = SafeAnyMap::new();

    // Inserting values
    map.insert("age", 42);
    map.insert("name", "John Doe");
    map.insert("is_human", true);

    // Getting values
    let age = map.get::<i32>("age").unwrap();
    let name = map.get::<&str>("name").unwrap();
    let is_human = map.get::<bool>("is_human").unwrap();

    println!("age:      {}", age);
    println!("name:     {}", name);
    println!("is_human: {}", is_human);

    // safety: this will panic because age is not a string
    // therefore it technically doesn't exist.
    let age = map.get::<&str>("age").unwrap();
    println!("age:      {}", age);
}
```

## MultiValueTypeMap
```rust
use safe_any_map::MultiValueTypeMap;

fn main() {
    let mut store = MultiValueTypeMap::new();

    store.add("age", 42);
    store.add("age", "forty-two");

    // Getting values Some(42)
    let age = store.get::<i32>("age").unwrap();
    // Getting values Some("forty-two")
    let age_str = store.get::<&str>("age").unwrap();
}
```

## MultiTypeMapSet
Differs from MultiTypeMapSet in that it allows for multiple values of the same type to be stored under the same key.

```rust
use safe_any_map::MultiTypeMapSet;

fn main() {
    let mut store = MultiTypeMapSet::new();

    store.insert("age", 42);
    store.insert("age", 43);

    // Getting values Some([42, 43])
    let age = store.get::<i32>("age").unwrap();
}
```