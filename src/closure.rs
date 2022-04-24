use crate::from_value::FromSysValue;
use crate::to_value::ToValue;
use crate::RootedValue;

pub struct Fn0<Res>
where
    Res: 'static + FromSysValue,
{
    // This is not the right type in the RootedValue parameter
    // but this does not matter here.
    f: crate::RootedValue<Res>,
    phantom_data: std::marker::PhantomData<Res>,
}

impl<Res> FromSysValue for Fn0<Res>
where
    Res: 'static + FromSysValue,
{
    unsafe fn from_value(f: ocaml_sys::Value) -> Self {
        let f = crate::RootedValue::create(f);
        Fn0 { f, phantom_data: std::marker::PhantomData }
    }
}

fn handle_exn<'a, R: 'static + FromSysValue>(r: ocaml_sys::Value) -> crate::exn::Result<'a, R> {
    if ocaml_sys::is_exception_result(r) {
        let exn = ocaml_sys::extract_exception(r);
        let exn: crate::exn::OCamlError<'a> = unsafe { crate::value::Value::new(exn) };
        Err(exn)
    } else {
        Ok(unsafe { R::from_value(r) })
    }
}

impl<Res> Fn0<Res>
where
    Res: 'static + FromSysValue,
{
    // This uses [mut self] as this can result in side effects on the ocaml side.
    pub fn call0<'a>(&mut self) -> crate::exn::Result<'a, Res> {
        let f = self.f.value().value;
        handle_exn(unsafe { ocaml_sys::caml_callback_exn(f, ocaml_sys::UNIT) })
    }
}

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
    pub fn call1<'a>(&mut self, arg: Arg) -> crate::exn::Result<'a, Res> {
        let arg = arg.to_value();
        handle_exn(unsafe { ocaml_sys::caml_callback_exn(self.f.value().value, arg) })
    }
}

pub struct Fn2<Arg1, Arg2, Res>
where
    Arg1: ToValue,
    Arg2: ToValue,
    Res: 'static + FromSysValue,
{
    f: crate::RootedValue<Res>,
    phantom_data: std::marker::PhantomData<(Arg1, Arg2, Res)>,
}

impl<Arg1, Arg2, Res> FromSysValue for Fn2<Arg1, Arg2, Res>
where
    Arg1: ToValue,
    Arg2: ToValue,
    Res: 'static + FromSysValue,
{
    unsafe fn from_value(f: ocaml_sys::Value) -> Self {
        let f = crate::RootedValue::create(f);
        Fn2 { f, phantom_data: std::marker::PhantomData }
    }
}

impl<Arg1, Arg2, Res> Fn2<Arg1, Arg2, Res>
where
    Arg1: ToValue,
    Arg2: ToValue,
    Res: 'static + FromSysValue,
{
    // This uses [mut self] as this can result in side effects on the ocaml side.
    pub fn call2<'a>(&mut self, arg1: Arg1, arg2: Arg2) -> crate::exn::Result<'a, Res> {
        let arg1: RootedValue<()> = RootedValue::create(arg1.to_value());
        let arg2: RootedValue<()> = RootedValue::create(arg2.to_value());
        handle_exn(unsafe {
            ocaml_sys::caml_callback2_exn(
                self.f.value().value,
                arg1.value().value,
                arg2.value().value,
            )
        })
    }
}

pub struct Fn3<Arg1, Arg2, Arg3, Res>
where
    Arg1: ToValue,
    Arg2: ToValue,
    Arg3: ToValue,
    Res: 'static + FromSysValue,
{
    f: crate::RootedValue<Res>,
    phantom_data: std::marker::PhantomData<(Arg1, Arg2, Arg3, Res)>,
}

impl<Arg1, Arg2, Arg3, Res> FromSysValue for Fn3<Arg1, Arg2, Arg3, Res>
where
    Arg1: ToValue,
    Arg2: ToValue,
    Arg3: ToValue,
    Res: 'static + FromSysValue,
{
    unsafe fn from_value(f: ocaml_sys::Value) -> Self {
        let f = crate::RootedValue::create(f);
        Fn3 { f, phantom_data: std::marker::PhantomData }
    }
}

impl<Arg1, Arg2, Arg3, Res> Fn3<Arg1, Arg2, Arg3, Res>
where
    Arg1: ToValue,
    Arg2: ToValue,
    Arg3: ToValue,
    Res: 'static + FromSysValue,
{
    // This uses [mut self] as this can result in side effects on the ocaml side.
    pub fn call3<'a>(&mut self, arg1: Arg1, arg2: Arg2, arg3: Arg3) -> crate::exn::Result<'a, Res> {
        let arg1: RootedValue<()> = crate::RootedValue::create(arg1.to_value());
        let arg2: RootedValue<()> = crate::RootedValue::create(arg2.to_value());
        let arg3: RootedValue<()> = crate::RootedValue::create(arg3.to_value());
        handle_exn(unsafe {
            ocaml_sys::caml_callback3_exn(
                self.f.value().value,
                arg1.value().value,
                arg2.value().value,
                arg3.value().value,
            )
        })
    }
}
