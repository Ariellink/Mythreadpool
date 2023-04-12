use failure::Fail;
//use crate::Job;

#[derive(Fail,Debug)]
pub enum PoolCreationError {
    #[fail(display = "{}", _0)]
    IoError(std::io::Error),
    // #[fail(display = "{}", _0)]
    // SendErr(std::sync::mpsc::SendError<Box<dyn FnOnce() + Send + 'static>>),
    //#[fail(display = "{}", _0)]
    // PrisonErr(std::sync::PoisonError<std::sync::MutexGuard<'a, Job>>),
}

impl From<std::io::Error> for PoolCreationError {
    fn from(err: std::io::Error) -> Self {
        PoolCreationError::IoError(err)
    }
}

// impl From <std::sync::PoisonError<std::sync::MutexGuard<'a, Job>>> for PoolCreationError {
//     fn from(err: std::sync::PoisonError<std::sync::MutexGuard<Job>>) -> Self {
//         PoolCreationError::PrisonErr(err)
//     }
//}

// impl From<std::sync::mpsc::SendError<Box<dyn FnOnce() + Send + 'static>>> for PoolCreationError {
//     fn from(err:std::sync::mpsc::SendError<Box<dyn FnOnce() + Send + 'static>>) -> Self {
//         PoolCreationError::SendErr(err)
//     }
// }