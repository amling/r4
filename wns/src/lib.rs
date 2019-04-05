use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

struct WaitNotifyStateBe<S> {
    c: Condvar,
    m: Mutex<S>,
}

pub struct WaitNotifyState<S>(Arc<WaitNotifyStateBe<S>>);

impl<S> Clone for WaitNotifyState<S> {
    fn clone(&self) -> Self {
        return WaitNotifyState(self.0.clone());
    }
}

impl<S> WaitNotifyState<S> {
    pub fn new(s: S) -> Self {
        return WaitNotifyState(Arc::from(WaitNotifyStateBe {
            c: Condvar::new(),
            m: Mutex::new(s),
        }));
    }

    pub fn read<F: FnOnce(&S) -> R, R>(&self, f: F) -> R {
        let mg = self.0.m.lock().unwrap();
        return f(&mg);
    }

    pub fn write<F: FnOnce(&mut S) -> R, R>(&self, f: F) -> R {
        let mut mg = self.0.m.lock().unwrap();
        self.0.c.notify_all();
        return f(&mut mg);
    }

    pub fn wait<F: FnMut(&mut S) -> (Option<R>, bool), R>(&self, f: &mut F) -> R {
        let mut mg = self.0.m.lock().unwrap();
        loop {
            let (r, n) = f(&mut mg);
            if n {
                self.0.c.notify_all();
            }
            if let Some(r) = r {
                return r;
            }
            mg = self.0.c.wait(mg).unwrap();
        }
    }
}
