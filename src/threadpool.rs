use crate::Worker;
use crate::PoolCreationError;
use std::sync::mpsc;
use std::sync::{Arc,Mutex};
pub struct Threadpool {
 workers: Vec<Worker>,
 sender: Option<mpsc::Sender<Job>>
}

//dyn FnOnce() + Send + 'static => F which a closure bounds
pub type Job = Box<dyn FnOnce() + Send + 'static>;

impl Threadpool {
    pub fn build(thread_num: usize)-> Result<Self, PoolCreationError> {
        //if num of thread was specified less than 1, invoke panic
        assert!(thread_num > 0);

        let mut workers = Vec::with_capacity(thread_num);

        let (sender, receiver) = mpsc::channel();
        
        let receiver = Arc::new(Mutex::new(receiver));
        
        for id in 0..thread_num {
            //To avoid value being moved in previous iteration of loop
            //use smart pointers to wrap the receiver: Arc<T>
            //std::sync::mpsc::Receiver<Job>  cannot be shared between threads safely
            //wrap it as Mutex, Mutex will ensure that only one worker gets a job from the receiver at a time.
            //In ThreadPool::new, we put the receiver in an Arc and a Mutex. For each new worker, we clone the Arc to bump the reference count so the workers can share ownership of the receiver.
            let worker = Worker::new(id, Arc::clone(&receiver))?;
            workers.push(worker);
        }

        Ok(
            Threadpool { 
                workers, 
                sender: Some(sender), //when sender was used as member of the threadpool, Sender<T> inferred to be Sender<Job>
            }
        )
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + 'static + Send,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

//impl Drop for threadpool to have graceful shutdown

impl Drop for Threadpool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            //explictly drop the sender before waiting for threads to finish
            drop(self.sender.take()); //then all calls to recv() in the loop with return an error
            //-> change the worker loop to handle the errors

            println!("Shutting down worker {}", worker.worker_id);
            //here is only one mutable borrow of each worker
            //join(self),the self here is JoinHandle<()>, join() takes its arguments' ownership
            //need to move the thread out of the Worker instance that owns it
            //thread: Option<thread::JoinHandle<()>>, Option.take()to move the value out of he some variant, leave None in its place
            // before: worker.handle.join(); //error!
            if let Some(_handle) = worker.handle.take() {
                 _handle.join().unwrap();
            }
        }
    }
}