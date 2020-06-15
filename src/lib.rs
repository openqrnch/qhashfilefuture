mod err;

use {
  std::{
    future::Future,
    sync::{Arc,Mutex},
    task::{Context,Poll,Waker},
    pin::Pin,
    thread,
    path::PathBuf
  },
  sha2::{Sha256, Sha512, Digest}
};

pub use err::Error;

pub enum HashAlg {
  Sha2_256,
  Sha2_512
}

pub struct HasherFuture {
  shared_state: Arc<Mutex<SharedState>>
}

struct SharedState {
  waker: Option<Waker>,
  hash: Option<Vec<u8>>,
  id: String
}

pub struct HashRet {
  pub id: String,
  pub hash: Vec<u8>,
  pub hex: String
}

impl Future for HasherFuture {
  type Output = HashRet;
  fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
    let mut shared_state = self.shared_state.lock().unwrap();
    if let Some(hash) = &shared_state.hash {
      let hex = hex::encode(hash);
      let hr = HashRet { id: shared_state.id.clone(), hash: hash.clone(),
          hex };
      Poll::Ready(hr)
    } else {
      shared_state.waker = Some(ctx.waker().clone());
      Poll::Pending
    }
  }
}

impl HasherFuture {
  pub fn new(id: &str, alg: HashAlg, fname: &PathBuf) -> Result<Self, Error> {
    let shstate = SharedState { waker: None, hash: None, id: id.to_string() };
    let shared_state = Arc::new(Mutex::new(shstate));

    let mut file = std::fs::File::open(&fname)?;

    // Spawn the new thread
    let thread_shared_state = shared_state.clone();
    thread::spawn(move || {
      let hash = match alg {
        HashAlg::Sha2_256 => {
          let mut hasher = Sha256::new();
          let _n = std::io::copy(&mut file, &mut hasher);
          hasher.finalize().to_vec()
        }
        HashAlg::Sha2_512 => {
          let mut hasher = Sha512::new();
          let _n = std::io::copy(&mut file, &mut hasher);
          hasher.finalize().to_vec()
        }
      };

      // How would one recover from lock() failing?
      let mut shared_state = thread_shared_state.lock().unwrap();

      // Signal that hash has been acquired
      shared_state.hash = Some(hash);

      // Wake up the last task on which the future was polled, if one exists.
      if let Some(waker) = shared_state.waker.take() {
        waker.wake()
      }
    });

    Ok(HasherFuture { shared_state })
  }
}

/* vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :*/
