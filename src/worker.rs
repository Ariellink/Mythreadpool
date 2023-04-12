use std::thread;
use std::sync::{mpsc,Arc,Mutex};
use crate::{Job,PoolCreationError};

pub struct Worker {
    pub worker_id: usize,
    pub handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    //1. worker spawns a thread which contains a rx 
    //2. construct the Worker with id provided and the waiting rx
   pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Self, PoolCreationError> {
        //Loop: closure to loop forever, asking the receiving end of the channel for a job and running the job when it gets one. 
        let handle = thread::Builder::new().spawn(move || loop{
            //Blocking: blocks this thread and waiting availale job received
            let receive = receiver.lock().expect(&format!("mutex poisoned in thread {}", id)).recv(); //ignore the recv error
            
            // handle the recv() return error
            match receive {
               Ok(_job) => {
                    println!("Worker {id} got a job; executing.");

                    _job(); // execute the closure
               }
               Err(_job) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
               } 
            }

        })?;
        Ok(
            Worker {
                worker_id: id,
                handle: Some(handle),
            }
        )
    }
}