use crate::from_value::FromSysValue;
use crate::to_value::ToValue;

pub struct Fn1<Arg, Res>
where
    Arg: ToValue,
    Res: 'static + FromSysValue,
{
    // This is not the right type in the RootedValue parameter
    // but this does not matter here.
    f: crate::RootedValue<Res>,
    phantom_data: std::marker::PhantomData<(Arg, Res)>,
}

impl<Arg, Res> FromSysValue for Fn1<Arg, Res>
where
    Arg: ToValue,
    Res: 'static + FromSysValue,
{
    unsafe fn from_value(f: ocaml_sys::Value) -> Self {
        let f = crate::RootedValue::create(f);
        Fn1 { f, phantom_data: std::marker::PhantomData }
    }
}

impl<Arg, Res> Fn1<Arg, Res>
where
    Arg: ToValue,
    Res: 'static + FromSysValue,
{
    // This uses [mut self] as this can result in side effects on the ocaml side.
    pub fn call1(&mut self, arg: Arg) -> Res {
        let f = self.f.value().value;
        let arg = arg.to_value(|x| x);
        let res = unsafe { ocaml_sys::caml_callback_exn(f, arg) };
        if ocaml_sys::is_exception_result(res) {
            panic!("TODO: got an ocaml exception")
        }
        unsafe { Res::from_value(res) }
    }
}
