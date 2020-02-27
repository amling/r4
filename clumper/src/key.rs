use record::Record;
use registry::args::OneKeyRegistryArgs;
use std::collections::HashMap;
use std::sync::Arc;
use stream::Entry;
use stream::Stream;
use super::ClumperBe;
use super::ClumperRegistrant;

pub type Impl = ClumperRegistrant<ImplBe>;

pub struct ImplBe();

impl ClumperBe for ImplBe {
    type Args = OneKeyRegistryArgs;

    fn names() -> Vec<&'static str> {
        return vec!["key", "k"];
    }

    fn help_msg() -> &'static str {
        return "bucket records by values of one key";
    }

    fn stream(a: &OneKeyRegistryArgs, bsw: Box<dyn Fn(Vec<(Arc<str>, Record)>) -> Stream>) -> Stream {
        let k = a.key.clone();

        return stream::closures(
            HashMap::new(),
            move |s, e, w| {
                let r = e.parse();

                let v = r.get_path(&k);

                let substream = s.entry(v.clone()).or_insert_with(|| {
                    return bsw(vec![(k.clone(), v)]);
                });

                // Disregard flow since one substream ending does
                // not mean we're done (e.g.  each substream could
                // be head -n 1).
                substream.write(Entry::Record(r), w);

                return true;
            },
            |s, w| {
                for (_, substream) in s.into_iter() {
                    substream.close(w);
                }
            },
        );
    }
}
