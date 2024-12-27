#![allow(unused_imports)]
#![allow(dead_code)]

use std::future::Future;

use tokio::{
    sync::mpsc::{self, Sender},
    time::{timeout, Duration, Timeout},
};

pub async fn add(left: u64, right: u64) -> u64 {
    left + right
}

fn authenticate() -> (Sender<String>, impl Future<Output = Result<String, String>>) {
    let (tx, mut rx) = mpsc::channel(10);

    let fut = async move {
        while let Some(password) = rx.recv().await {
            if password == "pass123".to_string() {
                return Ok("secret data".to_string());
            }
        }

        Err("Sender Dropped".to_string())
    };

    (tx, fut)
}

fn timeout_future<F, T>(fut: F) -> Timeout<F>
where
    F: Future<Output = T>,
{
    timeout(Duration::from_secs(5), fut)
}

#[cfg(test)]
mod tests {

    use std::future;

    use super::*;

    use tokio::time::{timeout, Duration};

    use tokio_test::{
        assert_pending, assert_ready, assert_ready_eq, assert_ready_err, assert_ready_ok, task,
    };

    #[tokio::test(start_paused = true)]
    async fn test_timeout_future() {
        let fut = async {
            future::pending::<()>().await;
        };

        let mut task = task::spawn(timeout_future(fut));

        tokio::time::sleep(Duration::from_secs(5)).await;

        assert_ready_err!(task.poll());
    }

    #[tokio::test]
    async fn test_authenticate() {
        let (tx, fut) = authenticate();

        let mut task = task::spawn(fut);

        tx.send("password123".into()).await.ok();

        assert_pending!(task.poll());

        tx.send("pass123".into()).await.ok();

        // let res = assert_ready!(task.poll());
        //
        // assert_eq!(res, Ok("secret data".to_string()));

        // assert_ready_eq!(task.poll(), Ok("secret data".to_string()));

        assert_ready_ok!(task.poll());
    }

    #[tokio::test]
    async fn test_authenticate_err() {
        let (tx, fut) = authenticate();

        let mut task = task::spawn(fut);

        drop(tx);

        assert_ready_err!(task.poll());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn it_works() {
        let task1 = tokio::spawn(async {
            println!("hello");
            println!("hello");
            println!("hello");
        });

        let task2 = tokio::spawn(async {
            println!("world");
            println!("world");
            println!("world");
        });

        let _ = tokio::join!(task1, task2);

        let result = add(2, 2).await;
        assert_eq!(result, 4);
    }

    #[test]
    fn _it_works() {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let result = add(2, 2).await;
                assert_eq!(result, 4);
            })
    }
}
