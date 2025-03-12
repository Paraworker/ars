use std::{
    cell::{Cell, UnsafeCell},
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use smallvec::SmallVec;

/// Async notifier
pub struct Notify {
    count: Cell<u64>,
    waker: WakerList,
}

impl Notify {
    /// Create a new [`Notify`].
    pub const fn new() -> Self {
        Notify {
            count: Cell::new(0),
            waker: WakerList::new(),
        }
    }

    /// Returns a [`Notified`] future.
    ///
    /// The future completes when it's notified.
    ///
    /// See [`Notified`] for more details.
    pub fn notified(&self) -> Notified {
        Notified {
            notify: self,
            count: self.count.get(),
        }
    }

    /// Make a notification.
    ///
    /// This will wake all associated [`Notified`]s.
    pub fn notify(&self) {
        // Increment count
        self.count.set(self.count.get() + 1);

        // Wake all wakers
        self.waker.wake_all();
    }
}

/// A future that completes when it's notified.
///
/// It resolves immediately if `notify()` has been called before it's awaited.
pub struct Notified<'a> {
    notify: &'a Notify,
    count: u64,
}

impl Future for Notified<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.notify.count.get() > self.count {
            Poll::Ready(())
        } else {
            self.notify.waker.push(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// A list of wakers with interior mutability
struct WakerList(UnsafeCell<SmallVec<[Waker; 1]>>);

impl WakerList {
    /// Create a new empty list
    const fn new() -> Self {
        WakerList(UnsafeCell::new(SmallVec::new_const()))
    }

    /// Push a waker onto the list
    fn push(&self, waker: Waker) {
        // SAFETY: Unique access since [`WakerList`] is `!Sync`
        let list = unsafe { &mut *self.0.get() };

        list.push(waker);
    }

    /// Wake all wakers in the list
    ///
    /// Wakers are taken from the list and consumed
    fn wake_all(&self) {
        // SAFETY: Unique access since [`WakerList`] is `!Sync`
        let list = unsafe { &mut *self.0.get() };

        list.drain(..).for_each(|waker| waker.wake());
    }
}
