use std::sync::{mpsc, Mutex, Arc};
use std::thread;

type Job = Box<dyn FnOnce() + 'static + Send>;

enum Message {
    Exec(Job),
    Term,
}

struct Workers{
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

pub struct ThreadPool{
    workers: Vec<Workers>,
    sender: mpsc::Sender<Message>, 
}

impl Workers{
    fn new(id: usize, job: Arc<Mutex<mpsc::Receiver<Message>>>) -> Workers{
        let handle = thread::spawn(move || loop{
            let msg = job.lock().unwrap().recv().unwrap();
            match msg {
                Message::Exec(job) => {
                    println!("Worker {} executing", id);
                    job()
                },
                Message::Term => {
                    println!("Worker {} exiting", id); 
                    break 
                },
            }
        });

        Workers{
            id,
            thread: Some(handle),
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> Self{
        assert!(size>0);
        let (sender, receiver) = mpsc::channel();
        let rcvs = Arc::new( Mutex::new(receiver) );
        let mut wrks = Vec::with_capacity(size);

        for i in 1..=size {
            let thread_ = Workers::new(i, Arc::clone(&rcvs));
            wrks.push(thread_);
        }

        ThreadPool{
            workers: wrks,
            sender:  sender,
        }
    }

    pub fn execute<F>(&self, job: F) 
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Message::Exec(Box::new(job))).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for wrker in &mut self.workers {
            self.sender.send(Message::Term).unwrap();
            if let Some(wrk) = wrker.thread.take() {
                wrk.join().unwrap();
            }
        }
    }
}