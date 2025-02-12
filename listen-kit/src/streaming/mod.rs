use futures_util::{Stream, StreamExt};
use tokio::sync::mpsc::{channel, Receiver};

pub struct SendableStream<T: 'static> {
    receiver: Receiver<T>,
}

impl<T: 'static> SendableStream<T> {
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = T> + 'static,
    {
        let (tx, rx) = channel(32);
        let mut stream = Box::pin(stream);

        tokio::task::spawn_local(async move {
            while let Some(item) = stream.next().await {
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });

        Self { receiver: rx }
    }

    pub async fn next(&mut self) -> Option<T> {
        self.receiver.recv().await
    }
}
