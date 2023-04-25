
# Drop
drop函数一次循环找到所有的worker，在所有的worker的线程中都调用join.


```rust
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
        }
    } 
}
```
但是，调用 join 并不会关闭线程，因为他们一直 loop 来寻找任务。如果采用这个实现来尝试丢弃 ThreadPool ，则主线程会永远阻塞在等待第一个线程结束上。    

为了修复这个问题，修改线程既监听是否有 Job 运行也要监听一个应该停止监听并退出无限循环的信号。所以通道将发送这个枚举的两个成员之一而不是 Job 实例：

```rust
enum Message {
    NewJob(Job),
    Terminate,
}
```
## 死锁


```rust
 for worker in &mut self.workers {
            //explictly drop the sender before waiting for threads to finish
            //drop(self.sender.take()); //then all calls to recv() in the loop with return an error
            //-> change the worker loop to handle the errors
            println!("Shutting down worker {}", worker.worker_id);
            //here is only one mutable borrow of each worker
            //join(self),the self here is JoinHandle<()>, join() takes its arguments' ownership
            if let Some(m) = &self.sender {
                m.send(Message::Terminate).unwrap()
             }
            println!("Shutted down worker {}", worker.worker_id);
            //need to move the thread out of the Worker instance that owns it
            //thread: Option<thread::JoinHandle<()>>, Option.take()to move the value out of he some variant, leave None in its place
             //worker.handle.join(); //error!
            //等待work n结束
            if let Some(_handle) = worker.handle.take() {
                 _handle.join().unwrap();
            }
        }
    }
```

```rust
➜  mythreadpool git:(main) ✗ cargo run          
   Compiling mythreadpool v0.1.0 (/home/chenxi0912/rusttest/talentplan/github/mythreadpool)
    Finished dev [unoptimized + debuginfo] target(s) in 1.69s
     Running `target/debug/main`
Worker 1 got a job; executing. //worker 1 抢到第一个消息 执行
Shutting down. //客户端发出第二个消息，main函数执行完调用drop
Shutting down all workers.//drop()
Shutting down worker 0 //循环到 worker0
// sender 发出第一个terminate
Shutted down worker 0
Worker 0 got a job; executing. //worker 0 抢到第二个消息 执行
Worker 1 was told to terminate; shutting down. //worker1 抢到了 terminate 消息，而不是worker0
```
worker0的循环一直在等worker0 join完，但是terminate消息被worker1抢走了。  

·pub fn join(self) -> Result<T>·
Waits for the associated thread to finish.

This function will return immediately if the associated thread has already finished.

`            if let Some(_handle) = worker.handle.take() {
                 _handle.join().unwrap();
`