use std::fmt::Debug;
use std::time::Duration;
use tokio::sync::watch::{self, Sender, Receiver};
use tokio::time::{timeout as tokio_timeout, Elapsed};

#[derive(Debug)]
pub struct Waiter<T> {
    ready_tx: Sender<T>,
    ready_rx: Receiver<T>,
}

impl<T> Waiter<T>
    where T: Clone + Debug + PartialEq + Send + Sync
{
    pub fn new(init: T) -> Self {
        let (tx, rx) = watch::channel(init);
        Waiter {
            ready_tx: tx,
            ready_rx: rx,
        }
    }

    pub fn set(&self, value: T) {
        self.ready_tx.broadcast(value).unwrap();
    }

    pub async fn wait(&self, desired: &T) {
        let mut rx = self.ready_rx.clone();
        loop {
            match rx.recv().await {
                None => panic!("self.ready_tx should not be dropped"),
                Some(ref v) if v == desired => break,
                Some(_) => continue,
            }
        }
    }

    pub async fn wait_timeout(&self, timeout: Duration, desired: &T)
        -> Result<(), Elapsed>
    {
        tokio_timeout(timeout, self.wait(desired)).await?;
        Ok(())
    }

}

impl<C: Copy> Waiter<C> {
    pub fn value(&self) -> C {
        *self.ready_rx.borrow()
    }
}

impl Waiter<bool> {
    pub fn set_true(&self) {
        self.set(true)
    }

    pub fn set_false(&self) {
        self.set(false)
    }

    pub async fn wait_true(&self) {
        self.wait(&true).await
    }

    pub async fn wait_false(&self) {
        self.wait(&false).await
    }

    pub async fn wait_ready_timeout(&self, timeout: Duration)
        -> Result<(), Elapsed>
    {
        self.wait_timeout(timeout, &true).await?;
        Ok(())
    }

    pub async fn wait_false_timeout(&self, timeout: Duration)
        -> Result<(), Elapsed>
    {
        self.wait_timeout(timeout, &false).await?;
        Ok(())
    }
}
