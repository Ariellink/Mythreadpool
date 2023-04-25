# Overview  

## struct Threadpool  

Threadpool有：
1. 一组workers  
2. 一个sender(包一个Job(闭包)) ， 会创建一个通道并充当发送端。 

```rust
pub struct Threadpool {
 workers: Vec<Worker>,
 sender: Option<mpsc::Sender<Job>>
}
```

## struct Worker
Worker 将会充当通道的接收端。
Worker 有：  
1. worker id
2. 一个线程句柄, spawn线程得来  

```rust
pub struct Worker {
    pub worker_id: usize,
    //Option.take() move the ownership of the worker, so that join() can consume the thread
    pub handle: Option<thread::JoinHandle<()>>,
}
```

## impl Threadpool
1. `pub fn build(thread_num: usize)-> Result<Self, PoolCreationError> `  
-> 让用户决定开几个线程，以此来构造线程池结构体。  
    a. 新建了一个通道，并接着让线程池在接收端等待。  
    `let (sender, receiver) = mpsc::channel(); `  
    b. 利用线程数的循环，给`worker id`和`receiver`。
    🤢尝试将 `receiver` 传递给多个 Worker 实例是不行的。Rust 所提供的通道实现是**多生产者，单消费者**。我们希望通过在所有的 `worker` 中共享单一 `receiver`。为了在多个线程间共享所有权并允许线程修改其值，需要使用 `Arc<Mutex<T>>`.
    ```rust
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
    ```

2. 会在通道发送端发出期望执行的任务（闭包）。  
` pub fn execute<F>(&self, f: F)  
    where  
        F: FnOnce() + 'static + Send,`  
    a. 在使用 execute 得到的闭包新建 Job 实例之后，将这些任务从通道的发送端发出。  
    b. Job 将是一个有着 execute 接收到的闭包类型的 trait 对象的类型别名。`type Job = Box<dyn FnOnce() + Send + 'static>;`
    ```rust
       {
            let job = Box::new(f);
            self.sender.send(job).unwrap();
       } 
    ```

# 发送之后Worker端接收发生了什么？
## impl Worker
单个worker spawn一个线程来接收sender发过来的闭包。
在 worker 中，传递给 thread::spawn 的闭包仍然还只是**引用**了通道的接收端。  
1. 首先在 receiver 上调用了 lock 来获取互斥器，接着 unwrap 在出现任何错误时 panic。如果互斥器处于一种叫做 被污染（poisoned）的状态时获取锁可能会失败，这可能发生于其他线程在持有锁时 panic 了且没有释放锁。(第一个unwrap())
2. 某个Worker如果锁定了互斥器，接着调用 recv 从通道中接收 Job. `job`的类型就是一个type Job。
```rust
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // --snip--
        let thread = thread::spawn(move||loop {
            //每个Worker线程都尝试抢占receiver
            //recv会阻塞线程，因此receive
           let job = receiver.lock().unwrap().recv().unwarp();
            //抢到之后执行job
           job();
        });

        Worker {
            id,
            thread,
        }
    }
}
}
```