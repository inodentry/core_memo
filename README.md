# Simple Memoization Library

`core_memo` is a simple, straightforward, zero-cost Rust library for lazy
evaluation and memoization. It does not do memory allocations or dynamic
dispatch and it is `#![no_std]`-compatible. It has no dependencies.

See the
[![docs](https://docs.rs/core_memo/badge.svg)](https://docs.rs/core_memo)
for more info!


## Example

```rust
use core_memo::{Memoize, Memo, MemoExt};

#[derive(Debug, PartialEq, Eq)] // for assert_eq! later
struct MemoSum(i32);

impl Memoize for MemoSum {
    type Param = [i32];

    fn memoize(p: &[i32]) -> MemoSum {
        MemoSum(p.iter().sum())
    }
}


// The `Memo` type holds ownership over the parameter for the calculation
// There are also the `MemoExt` and `MemoOnce` types with different semantics

let mut memo: Memo<MemoSum, _> = Memo::new(vec![1, 2]);

// Our `memoize` method is called the first time we call `memo.get()`
assert_eq!(memo.get(), &MemoSum(3));

// Further calls to `memo.get()` return the cached value without reevaluating
assert_eq!(memo.get(), &MemoSum(3));


// We can mutate the parameter held inside the `Memo`:

// via a mutable reference
memo.param_mut().push(3);
// via a closure
memo.update_param(|p| p.push(4));

// either way, the `Memo` forgets any cached value and it will be
// reevaluated on the next call to `memo.get()`

assert_eq!(memo.get(), &MemoSum(10)); // the vec is now `[1, 2, 3, 4]`
```

