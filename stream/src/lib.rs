extern crate record;

use record::Record;
use std::sync::Arc;

#[derive(Clone)]
pub enum Entry {
    Record(Record),
    Line(Arc<str>),
}

impl Entry {
    pub fn parse(self) -> Record {
        return match self {
            Entry::Record(r) => r,
            Entry::Line(line) => Record::parse(&line),
        };
    }

    pub fn deparse(self) -> Arc<str> {
        return match self {
            Entry::Record(r) => Arc::from(r.deparse()),
            Entry::Line(line) => line,
        };
    }
}

pub trait StreamTrait {
    fn write(&mut self, r: Entry, w: &mut dyn FnMut(Entry) -> bool) -> bool;
    fn close(self: Box<Self>, w: &mut dyn FnMut(Entry) -> bool);
}

pub struct Stream(Box<dyn StreamTrait>);

impl Stream {
    pub fn new<F: StreamTrait + 'static>(f: F) -> Self {
        return Stream(Box::new(f));
    }

    pub fn write(&mut self, e: Entry, w: &mut dyn FnMut(Entry) -> bool) -> bool {
        let mut ret = true;
        let ret2 = self.0.write(e, &mut |e| {
            ret &= w(e);
            return ret;
        });
        return ret && ret2;
    }

    pub fn close(self, w: &mut dyn FnMut(Entry) -> bool) {
        self.0.close(w);
    }
}

pub fn id() -> Stream {
    return closures(
        (),
        |_s, e, w| {
            return w(e);
        },
        |_s, _w| {
        },
    );
}

pub fn compound(s1: Stream, s2: Stream) -> Stream {
    return closures(
        (s1, s2),
        |(s1, s2), e, w| {
            return s1.write(e, &mut |e| s2.write(e, w));
        },
        |(s1, mut s2), w| {
            s1.close(&mut |e| s2.write(e, w));
            s2.close(w);
        },
    );
}

pub fn parse() -> Stream {
    return closures(
        (),
        |_s, e, w| {
            return w(Entry::Record(e.parse()));
        },
        |_s, _w| {
        },
    );
}

pub fn deparse() -> Stream {
    return closures(
        (),
        |_s, e, w| {
            return w(Entry::Line(e.deparse()));
        },
        |_s, _w| {
        },
    );
}

pub fn transform_records<F: FnMut(Record) -> Record + 'static>(f: F) -> Stream {
    return closures(
        f,
        |f, e, w| {
            return w(match e {
                Entry::Record(r) => Entry::Record(f(r)),
                e => e,
            });
        },
        |_f, _w| {
        },
    );
}

struct ClosuresStream<S, W: Fn(&mut S, Entry, &mut dyn FnMut(Entry) -> bool) -> bool, C: Fn(S, &mut dyn FnMut(Entry) -> bool)> {
    s: S,
    w: W,
    c: C,
}

impl<S, W: Fn(&mut S, Entry, &mut dyn FnMut(Entry) -> bool) -> bool, C: Fn(S, &mut dyn FnMut(Entry) -> bool)> StreamTrait for ClosuresStream<S, W, C> {
    fn write(&mut self, e: Entry, w: &mut dyn FnMut(Entry) -> bool) -> bool {
        return (self.w)(&mut self.s, e, w);
    }

    fn close(self: Box<Self>, w: &mut dyn FnMut(Entry) -> bool) {
        let s = *self;
        return (s.c)(s.s, w);
    }
}

pub fn closures<S: 'static, W: Fn(&mut S, Entry, &mut dyn FnMut(Entry) -> bool) -> bool + 'static, C: Fn(S, &mut dyn FnMut(Entry) -> bool) + 'static>(s: S, w: W, c: C) -> Stream {
    return Stream::new(ClosuresStream {
        s: s,
        w: w,
        c: c,
    });
}
