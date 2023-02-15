use hyperpipe::HyperPipe;
use std::{
    future::Future,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

pub struct AsyncHyperPipe {
    inner: HyperPipe,
}

impl AsyncHyperPipe {
    pub fn new<'p, P: Into<&'p Path>>(path: P) -> Self {
        Self {
            inner: HyperPipe::new(path).unwrap(),
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
            None => {
                ctx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

#[test]
fn in_out() {
    smol::block_on(async move {
        let temp_dir = tempfile::tempdir().unwrap();
        let pipe_path = temp_dir.into_path();

        let mut p1 = AsyncHyperPipe::new(pipe_path.as_path());
        let v1 = vec![1, 2, 3, 4, 5, 6];
        p1.push(v1.clone());

        let mut p2 = AsyncHyperPipe::new(pipe_path.as_path());
        let v2 = p2.pull().await;
        assert_eq!(v1, v2);
    })
}
