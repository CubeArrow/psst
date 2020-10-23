use crate::promise::Promise;
use druid::{
    lens::{Field, Map},
    Data, Lens,
};

#[derive(Clone, Data)]
pub struct Ctx<C: Data, T: Data> {
    pub ctx: C,
    pub data: T,
}

impl<C: Data, T: Data> Ctx<C, T> {
    pub fn new(c: C, t: T) -> Self {
        Self { ctx: c, data: t }
    }

    pub fn make<S: Data>(cl: impl Lens<S, C>, tl: impl Lens<S, T>) -> impl Lens<S, Self> {
        CtxMake { cl, tl }
    }

    pub fn ctx() -> impl Lens<Self, C> {
        Field::new(|c: &Self| &c.ctx, |c: &mut Self| &mut c.ctx)
    }

    pub fn data() -> impl Lens<Self, T> {
        Field::new(|c: &Self| &c.data, |c: &mut Self| &mut c.data)
    }

    pub fn map<U: Data>(map: impl Lens<T, U>) -> impl Lens<Self, Ctx<C, U>> {
        CtxMap { map }
    }
}

struct CtxMake<CL, TL> {
    cl: CL,
    tl: TL,
}

impl<C: Data, T: Data, S: Data, CL, TL> Lens<S, Ctx<C, T>> for CtxMake<CL, TL>
where
    CL: Lens<S, C>,
    TL: Lens<S, T>,
{
    fn with<V, F: FnOnce(&Ctx<C, T>) -> V>(&self, data: &S, f: F) -> V {
        self.cl.with(data, |c| {
            self.tl.with(data, |t| {
                let ct = Ctx::new(c.to_owned(), t.to_owned());
                f(&ct)
            })
        })
    }

    fn with_mut<V, F: FnOnce(&mut Ctx<C, T>) -> V>(&self, data: &mut S, f: F) -> V {
        let mut t_data = data.to_owned();
        let v = self.cl.with_mut(data, |c| {
            self.tl.with_mut(&mut t_data, |t| {
                let mut ct = Ctx::new(c.to_owned(), t.to_owned());
                let v = f(&mut ct);
                *c = ct.ctx;
                *t = ct.data;
                v
            })
        });
        *data = t_data;
        v
    }
}

struct CtxMap<Map> {
    map: Map,
}

impl<C: Data, T: Data, U: Data, Map> Lens<Ctx<C, T>, Ctx<C, U>> for CtxMap<Map>
where
    Map: Lens<T, U>,
{
    fn with<V, F: FnOnce(&Ctx<C, U>) -> V>(&self, c: &Ctx<C, T>, f: F) -> V {
        self.map.with(&c.data, |u| {
            let cu = Ctx::new(c.ctx.to_owned(), u.to_owned());
            f(&cu)
        })
    }

    fn with_mut<V, F: FnOnce(&mut Ctx<C, U>) -> V>(&self, c: &mut Ctx<C, T>, f: F) -> V {
        let t = &mut c.data;
        let c = &mut c.ctx;
        self.map.with_mut(t, |u| {
            let mut cu = Ctx::new(c.to_owned(), u.to_owned());
            let v = f(&mut cu);
            *c = cu.ctx;
            *u = cu.data;
            v
        })
    }
}

impl<C: Data, PT: Data, PD: Data, PE: Data> Ctx<C, Promise<PT, PD, PE>> {
    pub fn in_promise() -> impl Lens<Self, Promise<Ctx<C, PT>, Ctx<C, PD>, Ctx<C, PE>>> {
        Map::new(
            |c: &Self| match &c.data {
                Promise::Empty => Promise::Empty,
                Promise::Resolved(res) => {
                    Promise::Resolved(Ctx::new(c.ctx.to_owned(), res.to_owned()))
                }
                Promise::Deferred(def) => {
                    Promise::Deferred(Ctx::new(c.ctx.to_owned(), def.to_owned()))
                }
                Promise::Rejected(err) => {
                    Promise::Rejected(Ctx::new(c.ctx.to_owned(), err.to_owned()))
                }
            },
            |c: &mut Self, p: Promise<Ctx<C, PT>, Ctx<C, PD>, Ctx<C, PE>>| match p {
                Promise::Empty => {
                    c.data = Promise::Empty;
                }
                Promise::Resolved(pc) => {
                    c.data = Promise::Resolved(pc.data);
                    c.ctx = pc.ctx;
                }
                Promise::Deferred(pc) => {
                    c.data = Promise::Deferred(pc.data);
                    c.ctx = pc.ctx;
                }
                Promise::Rejected(pc) => {
                    c.data = Promise::Rejected(pc.data);
                    c.ctx = pc.ctx;
                }
            },
        )
    }
}
