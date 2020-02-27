use record::Record;
use std::collections::VecDeque;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::ClumperBe;
use super::ClumperRegistrant;

#[derive(RegistryArgs)]
pub struct Args {
    count: usize,
}

pub type Impl = ClumperRegistrant<ImplBe>;

pub struct ImplBe();

impl ClumperBe for ImplBe {
    type Args = Args;

    fn names() -> Vec<&'static str> {
        return vec!["window"];
    }

    fn help_msg() -> &'static str {
        return "'bucket' records by making a bucket for each [overlapping] window of a specified size";
    }

    fn stream(a: &Args, bsw: Box<dyn Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        let size = a.count;

        return stream::closures(
            VecDeque::new(),
            move |s, e, w| {
                let r = e.parse();

                s.push_back(r);
                if s.len() > size {
                    s.pop_front();
                }
                if s.len() == size {
                    let mut substream = bsw(vec![]);

                    for r in s {
                        // Disregard flow since one substream
                        // ending does not mean we're done (e.g.
                        // each substream could be head -n 1).
                        substream.write(Entry::Record(r.clone()), w);
                    }

                    substream.close(w);
                }

                return true;
            },
            |_s, _w| {
            },
        );
    }
}
