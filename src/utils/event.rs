use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

/// Creates a new event notifier and listener.
pub fn event() -> (Notifier, Listener) {
    let inner = Rc::new(Inner::new());
    (Notifier(inner.clone()), Listener(inner))
}

/// Event notifier
#[derive(Clone)]
pub struct Notifier(Rc<Inner>);

impl Notifier {
    /// Make a notification.
    pub fn notify(&self) {
        self.0.notify();
    }
}

/// Event listener
pub struct Listener(Rc<Inner>);

impl Listener {
    /// Returns a [`Notified`] future that completes when the event is notified.
    ///
    /// Take mutable reference here to ensure only one future exists at a time.
    pub fn notified(&mut self) -> Notified {
        Notified(self)
    }
}

/// A future that completes when it's notified.
///
/// It resolves immediately if [`notify()`] has been called before it's awaited.
pub struct Notified<'a>(&'a mut Listener);

impl Future for Notified<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.0.poll(cx)
    }
}

struct Inner {
    state: Cell<bool>,
    waker: Cell<Option<Waker>>,
}

impl Inner {
    #[inline(always)]
    const fn new() -> Self {
        Inner {
            state: Cell::new(false),
            waker: Cell::new(None),
        }
    }

    #[inline(always)]
    fn notify(&self) {
        // Set notified
        self.state.set(true);

        // Wake up waker
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    #[inline(always)]
    fn poll(&self, cx: &mut Context<'_>) -> Poll<()> {
        if self.state.replace(false) {
            Poll::Ready(())
        } else {
            self.waker.set(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}
