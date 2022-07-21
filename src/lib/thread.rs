use core::num::NonZeroUsize;
use num_cpus;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use worker::{Message, Worker};

use super::errors::MyErrors;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Errors
    /// The ThreadPool creation failes when the number of threads grather than a half of all logical cores.
    pub fn new(size: NonZeroUsize) -> Result<ThreadPool, MyErrors> {
        let max = num_cpus::get() / 2;
        if max < size.get() {
            return Err(MyErrors::InvalidConfig(format!(
                "The `threads` size must be lower than {}",
                max
            )));
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size.get());

        for id in 0..size.get() {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool { workers, sender })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        if let Err(err) = self.sender.send(Message::NewJob(job)) {
            println!("{:?}", err);
        }
    }
}

/// Drops ThreadPool
///
/// The TheadPool will only be dropped when all workers are finished.
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap_or_default();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                match thread.join() {
                    Ok(_) => println!("Worker {} closes with success!", worker.id),
                    Err(err) => println!(
                        "Worker {} failed to closes with error: {:?}",
                        worker.id, err
                    ),
                }
            }
        }
    }
}

mod worker {
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;

    type Job = Box<dyn FnOnce() + Send + 'static>;

    pub enum Message {
        NewJob(Job),
        Terminate,
    }

    pub struct Worker {
        pub id: usize,
        pub thread: Option<thread::JoinHandle<()>>,
    }

    impl Worker {
        pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
            let thread = thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job();
                        println!("Worker {} finished a job; waiting for job.", id);
                    }
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);

                        break;
                    }
                }
            });

            Worker {
                id,
                thread: Some(thread),
            }
        }
    }
}
