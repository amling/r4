extern crate stream;
extern crate wns;

use std::collections::VecDeque;
use stream::Entry;
use stream::Stream;
use stream::StreamTrait;
use wns::WaitNotifyState;

#[derive(Default)]
struct OneBuffer {
    buf: VecDeque<Entry>,
    rclosed: bool,
    closed: bool,
}

#[derive(Default)]
struct BgopState {
    fe_to_be: OneBuffer,
    be_to_fe: OneBuffer,
}

pub struct BgopRbe {
    state: WaitNotifyState<BgopState>,
}

impl BgopRbe {
    pub fn read(&self) -> Option<Entry> {
        return self.state.wait(&mut |buffers| {
            if let Some(e) = buffers.fe_to_be.buf.pop_front() {
                return (Some(Some(e)), true);
            }
            if buffers.fe_to_be.closed {
                return (Some(None), false);
            }
            return (None, false);
        });
    }

    pub fn rclose(&self) {
        self.state.write(|buffers| {
            buffers.fe_to_be.rclosed = true;
            buffers.fe_to_be.buf.clear();
        });
    }
}

pub struct BgopWbe {
    state: WaitNotifyState<BgopState>,
}

impl BgopWbe {
    pub fn write(&self, e: Entry) -> bool {
        return self.state.wait(&mut |buffers| {
            if buffers.be_to_fe.rclosed {
                return (Some(false), false);
            }
            if buffers.be_to_fe.buf.len() < 1024 {
                buffers.be_to_fe.buf.push_back(e.clone());
                return (Some(true), true);
            }
            return (None, false);
        });
    }

    pub fn close(self) {
        self.state.write(&mut |buffers: &mut BgopState| {
            buffers.be_to_fe.closed = true;
        });
    }
}

pub struct BgopFe {
    state: WaitNotifyState<BgopState>,
}

impl BgopFe {
    fn ferry<R, F: FnMut(bool, &mut BgopState) -> Option<R>>(&self, mut f: F, w: &mut FnMut(Entry) -> bool) -> R {
        enum Ret<R> {
            Ferry(Vec<Entry>),
            Return(R),
        }
        loop {
            let f = &mut f;
            let ret = self.state.wait(&mut |buffers| {
                if !buffers.be_to_fe.buf.is_empty() {
                    return (Some(Ret::Ferry(buffers.be_to_fe.buf.drain(..).collect())), true);
                }

                if let Some(ret) = f(buffers.be_to_fe.closed, buffers) {
                    return (Some(Ret::Return(ret)), true);
                }

                return (None, false);
            });
            match ret {
                Ret::Ferry(es) => {
                    for e in es {
                        if !w(e) {
                            self.state.write(|buffers| {
                                buffers.be_to_fe.rclosed = true;
                                buffers.be_to_fe.buf.clear();
                            });
                            break;
                        }
                    }
                }
                Ret::Return(ret) => {
                    return ret;
                }
            }
        }
    }
}

impl StreamTrait for BgopFe {
    fn write(&mut self, e: Entry, w: &mut FnMut(Entry) -> bool) -> bool {
        return self.ferry(|_os_closed, buffers| {
            if buffers.fe_to_be.rclosed {
                return Some(false);
            }

            if buffers.fe_to_be.buf.len() < 1024 {
                buffers.fe_to_be.buf.push_back(e.clone());
                return Some(true);
            }

            return None;
        }, w);
    }

    fn close(self: Box<BgopFe>, w: &mut FnMut(Entry) -> bool) {
        self.state.write(|buffers| {
            buffers.fe_to_be.closed = true;
        });
        self.ferry(|os_closed, _buffers| {
            if os_closed {
                return Some(());
            }
            return None;
        }, w);
    }
}

pub fn new() -> (Stream, BgopRbe, BgopWbe) {
    let state = WaitNotifyState::new(BgopState::default());

    let fe = BgopFe {
        state: state.clone(),
    };

    let rbe = BgopRbe {
        state: state.clone(),
    };

    let wbe = BgopWbe {
        state: state.clone(),
    };

    return (Stream::new(fe), rbe, wbe);
}
