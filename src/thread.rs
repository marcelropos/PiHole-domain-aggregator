use anyhow::{anyhow, Error};
use core::num::NonZeroUsize;
use std::{
    cmp::Ordering,
    sync::{mpsc, Arc, Mutex},
};
use worker::{Message, Worker};

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
    pub fn new(threads: Option<NonZeroUsize>) -> Result<ThreadPool, Error> {
        let capacity = {
            let limit = num_cpus::get() / 2;
            match threads {
                Some(threads) => match limit.cmp(&threads.get()) {
                    Ordering::Less | Ordering::Equal => threads.get(),
                    Ordering::Greater => {
                        return Err(anyhow!("The `threads` size must be lower than {limit}"))
                    }
                },
                None => limit,
            }
        };

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(capacity);

        for id in 0..capacity {
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
                    Err(err) => eprintln!(
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
                let message = receiver.lock().map(|guard| guard.recv());

                match message {
                    //Expected cases
                    Ok(Ok(Message::NewJob(job))) => {
                        println!("Worker {} got a job; executing.", id);
                        job();
                        println!("Worker {} finished a job; waiting for job.", id);
                    }
                    Ok(Ok(Message::Terminate)) => {
                        println!("Worker {} was told to terminate.", id);
                        break;
                    }
                    // Error cases
                    Ok(Err(_)) => {
                        eprintln!("Sender has disconnected. Worker {} terminates now!", id);
                        break;
                    }
                    Err(_) => {
                        eprintln!("Another Worker panicked. Worker {} terminates now!", id);
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
