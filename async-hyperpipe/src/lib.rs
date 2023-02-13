use atomptr::AtomPtr;
use hyperpipe::HyperPipe;
use inotify::{Inotify, WatchMask};
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    thread,
};

pub struct AsyncHyperPipe {
    inner: HyperPipe,
    notifier: Arc<AtomPtr<Option<Waker>>>,
    init: bool,
}

impl AsyncHyperPipe {
    pub fn consumer<'p, P: Into<&'p Path>>(path: P) -> Self {
        let inner = HyperPipe::new(path).unwrap();
        let notifier = Arc::new(AtomPtr::new(None));
        NotifierThread::new(inner.root_path(), Arc::clone(&notifier));
        Self {
            inner,
            notifier,
            init: false,
        }
    }

    pub fn producer<'p, P: Into<&'p Path>>(path: P) -> Self {
        let inner = HyperPipe::new(path).unwrap();
        let notifier = Arc::new(AtomPtr::new(None));
        Self {
            inner,
            notifier,
            init: false,
        }
    }

    pub async fn pull(&mut self) -> Vec<u8> {
        self.await
    }

    pub fn push(&mut self, data: Vec<u8>) {
        self.inner.push(data);
    }
}

impl Future for AsyncHyperPipe {
    type Output = Vec<u8>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        match self.inner.pull() {
            Some(item) => Poll::Ready(item),
            None if self.init => Poll::Pending,
            None => {
                eprintln!("Set waker on notifier thread...");
                self.notifier.swap(Some(ctx.waker().clone()));
                self.init = true;
                Poll::Pending
            }
        }
    }
}

pub struct NotifierThread {
    waker: Arc<AtomPtr<Option<Waker>>>,
    notify: Inotify,
}

impl NotifierThread {
    pub fn new(path: PathBuf, waker: Arc<AtomPtr<Option<Waker>>>) {
        let mut notify = Inotify::init().expect("failed to initialise Inotify structure");
        notify
            .add_watch(&path, WatchMask::MOVE)
            .expect(&format!("failed to watch directory {:?}", path));
        eprintln!("Start INOTIFY watch for path {:?}", path);

        Self { waker, notify }.run()
    }

    pub fn run(mut self) {
        thread::spawn(move || {
            let mut buffer = [0; 64];

            loop {
                let _e = self
                    .notify
                    .read_events_blocking(&mut buffer)
                    .expect("Error while reading events");

                let events: Vec<_> = _e.collect();

                // For any event we want to wake the reading future so it
                // can update.  We don't have to check for what type of
                // event we're handling here because we're only listening
                // for Modify events.
                if let Some(ref w) = **self.waker.get_ref() {
                    eprintln!("[{} events]: waking receiver...", events.len());
                    w.wake_by_ref();
                }
            }
        });
    }
}

/// A simple test that constructs a tempfile pipe that we write into
/// and read from once.  We use `smol` as a test runtime here to await
/// our future.  We could probably use something more low-level too
/// but smol will do the trick too.
#[test]
fn in_out() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pipe_path = temp_dir.into_path();

    let mut p1 = AsyncHyperPipe::producer(pipe_path.as_path());
    let v1 = vec![1, 2, 3, 4, 5, 6];
    p1.push(v1.clone());

    let mut p2 = AsyncHyperPipe::consumer(pipe_path.as_path());
    let v2 = smol::block_on(async move { p2.pull().await });

    assert_eq!(v1, v2);
}

#[test]
fn staggered_writes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pipe_path = temp_dir.into_path();

    let mut p1 = AsyncHyperPipe::producer(pipe_path.as_path());
    let mut p2 = AsyncHyperPipe::consumer(pipe_path.as_path());

    let v1 = vec![1, 2, 3, 4, 5, 6];
    let v1_1 = v1.clone();
    thread::spawn(move || {
        for _ in 0..4 {
            eprintln!("Sending payload...");
            p1.push(v1.clone());
            #[allow(deprecated)]
            thread::sleep_ms(250);
        }
    });

    smol::block_on(async move {
        let v1 = v1_1.clone();
        for _ in 0..4 {
            let v2 = p2.pull().await;
            eprintln!("Receiving payload!");
            assert_eq!(v1, v2);
        }
    });
}
