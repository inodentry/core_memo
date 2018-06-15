use core::cell::Cell;
use Memoize;

const MAGIC: i32 = -420;

#[derive(Debug)]
struct CallTracker {
    count: Cell<usize>,
}

impl CallTracker {
    fn new() -> Self {
        Self {
            count: Cell::new(0),
        }
    }

    fn count(&self) -> usize {
        self.count.get()
    }

    fn incr(&self) {
        self.count.set(self.count.get() + 1);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct TestOut(i32);

impl Memoize for TestOut {
    type Param = CallTracker;
    fn memoize(p: &CallTracker) -> Self {
        p.incr();
        TestOut(MAGIC)
    }
}

#[test]
fn track_calls_ext() {
    use MemoExt;

    let expected = TestOut(MAGIC);
    let track = CallTracker::new();
    let mut memo: MemoExt<TestOut> = MemoExt::new();

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.try_get(), None);
    assert_eq!(track.count(), 0);

    memo.ready(&track);

    assert_eq!(memo.is_ready(), true);
    assert_eq!(track.count(), 1);

    assert_eq!(memo.try_get(), Some(&expected));
    assert_eq!(memo.try_get(), Some(&expected));
    assert_eq!(track.count(), 1);

    assert_eq!(memo.get(&track), &expected);
    assert_eq!(memo.get(&track), &expected);
    assert_eq!(track.count(), 1);

    memo.clear();

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.try_get(), None);
    assert_eq!(track.count(), 1);

    assert_eq!(memo.get(&track), &expected);
    assert_eq!(memo.get(&track), &expected);
    assert_eq!(track.count(), 2);
    assert_eq!(memo.is_ready(), true);

    memo.update(&track);
    assert_eq!(track.count(), 3);
}

#[test]
fn track_calls_own() {
    use Memo;

    let expected = TestOut(MAGIC);
    let track = CallTracker::new();
    let mut memo: Memo<TestOut> = Memo::new(track);

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.try_get(), None);
    assert_eq!(memo.param().count(), 0);

    memo.ready();

    assert_eq!(memo.is_ready(), true);
    assert_eq!(memo.param().count(), 1);

    assert_eq!(memo.try_get(), Some(&expected));
    assert_eq!(memo.try_get(), Some(&expected));
    assert_eq!(memo.param().count(), 1);

    assert_eq!(memo.get(), &expected);
    assert_eq!(memo.get(), &expected);
    assert_eq!(memo.param().count(), 1);

    memo.clear();

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.try_get(), None);
    assert_eq!(memo.param().count(), 1);

    assert_eq!(memo.get(), &expected);
    assert_eq!(memo.get(), &expected);
    assert_eq!(memo.param().count(), 2);
    assert_eq!(memo.is_ready(), true);

    memo.update();
    assert_eq!(memo.param().count(), 3);
}

#[test]
fn track_calls_ref() {
    use MemoOnce;

    let expected = TestOut(MAGIC);
    let track = CallTracker::new();
    let mut memo: MemoOnce<TestOut> = MemoOnce::new(&track);

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.try_get(), None);
    assert_eq!(track.count(), 0);

    memo.ready();

    assert_eq!(memo.is_ready(), true);
    assert_eq!(track.count(), 1);

    assert_eq!(memo.try_get(), Some(&expected));
    assert_eq!(memo.try_get(), Some(&expected));
    assert_eq!(track.count(), 1);

    assert_eq!(memo.get(), &expected);
    assert_eq!(memo.get(), &expected);
    assert_eq!(track.count(), 1);

    memo.clear();

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.try_get(), None);
    assert_eq!(track.count(), 1);

    assert_eq!(memo.get(), &expected);
    assert_eq!(memo.get(), &expected);
    assert_eq!(track.count(), 2);
    assert_eq!(memo.is_ready(), true);

    memo.update();
    assert_eq!(track.count(), 3);
}

#[test]
fn track_calls_mutation() {
    use Memo;

    let track = CallTracker::new();
    let mut memo: Memo<TestOut> = Memo::new(track);

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.try_get(), None);
    assert_eq!(memo.param().count(), 0);

    memo.get();

    assert_eq!(memo.is_ready(), true);
    assert_eq!(memo.param().count(), 1);

    memo.update_param(|_p| ());

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.param().count(), 1);

    memo.get();

    assert_eq!(memo.is_ready(), true);
    assert_eq!(memo.param().count(), 2);

    memo.param_mut();

    assert_eq!(memo.is_ready(), false);
    assert_eq!(memo.param().count(), 2);

    memo.get();

    assert_eq!(memo.is_ready(), true);
    assert_eq!(memo.param().count(), 3);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct MemoSum(i32);

impl Memoize for MemoSum {
    type Param = [i32];

    fn memoize(p: &[i32]) -> MemoSum {
        MemoSum(p.iter().sum())
    }
}

#[test]
fn sums() {
    use Memo;

    let vals = vec![1, 2];
    let mut memo: Memo<MemoSum, _> = Memo::new(vals);

    assert_eq!(memo.get(), &MemoSum(3));

    memo.param_mut().push(3);

    assert_eq!(memo.get(), &MemoSum(6));

    memo.update_param(|p| p.push(4));

    assert_eq!(memo.get(), &MemoSum(10));
}
