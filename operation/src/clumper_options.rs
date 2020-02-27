use clumper::BoxedClumper;
use opts::parser::OptionsPile;
use opts::parser::Optionsable;
use opts::vals::UnvalidatedOption;
use record::Record;
use registry::Registrant;
use registry::args::OneKeyRegistryArgs;
use std::rc::Rc;
use std::sync::Arc;
use stream::Stream;

#[derive(Default)]
#[derive(Validates)]
pub struct ClumperOptions(UnvalidatedOption<Vec<BoxedClumper>>);

impl Optionsable for ClumperOptions {
    type Options = ClumperOptions;

    fn options(opt: &mut OptionsPile<ClumperOptions>) {
        opt.add_sub(|p| &mut (p.0).0, clumper::REGISTRY.single_options(&["c", "clumper"], "clumpers to bucket by"));
        opt.add_sub(|p| &mut (p.0).0, clumper::REGISTRY.multiple_options(&["c", "clumper"]));
        opt.add(clumper::REGISTRY.help_options("clumper"));
        opt.match_single(&["k", "key"], |p, a| {
            for a in a.split(',') {
                (p.0).0.push(clumper::key::Impl::init(OneKeyRegistryArgs::new(a)));
            }
            return Result::Ok(());
        }, "keys to bucket by");
    }
}

impl ClumperOptionsValidated {
    pub fn stream<F: Fn(Vec<(Arc<str>, Record)>) -> Stream + 'static>(&self, f: F) -> Stream {
        let mut bsw: Rc<dyn Fn(Vec<(Arc<str>, Record)>) -> Stream> = Rc::new(f);

        bsw = self.0.iter().rev().fold(bsw, |bsw, cw| {
            let cw = cw.clone();
            return Rc::new(move |bucket_outer| {
                let bucket_outer = bucket_outer.clone();
                let bsw = bsw.clone();
                return cw.stream(Box::new(move |bucket_inner| {
                    let mut bucket = bucket_outer.clone();
                    bucket.extend(bucket_inner);
                    return bsw(bucket);
                }));
            });
        });

        return bsw(vec![]);
    }
}
