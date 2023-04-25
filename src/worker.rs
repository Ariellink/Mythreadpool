use std::thread;
use std::sync::{mpsc,Arc,Mutex};
use crate::{PoolCreationError,Message};

pub struct Worker {
    pub worker_id: usize,
    //Option.take() move the ownership of the worker, so that join() can consume the thread
    pub handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    //1. worker spawns a thread which contains a rx 
    //2. construct the Worker with id provided and the waiting rx
   //pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Self, PoolCreationError> {
     pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Result<Self, PoolCreationError> {
        //Loop: closure to loop forever, asking the receiving end of the channel for a job and running the job when it gets one. 
        let handle = thread::Builder::new().spawn(move || loop{
            //Blocking: blocks this thread and waiting availale job received
            let message = receiver.lock().expect(&format!("mutex poisoned in thread {}", id)).recv(); //ignore the recv error
            
            match message {
               Ok(Message::NewJob(message)) => {
                    println!("Worker {id} got a job; executing.");

                    message(); // execute the closure
               }
               Ok(Message::Terminate) => {
                    println!("Worker {id} was told to terminate; shutting down.");
                    break;
               } 
               Err(e) => {
                    println!("Got a receiver error: {:?}",e);
               }
            }

        })?;
        Ok(
            Worker {
                worker_id: id,
                handle: Some(handle), //for Option<thread::JoinHandle<()>>
            }
        )
    }
}