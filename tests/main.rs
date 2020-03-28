use lazing::lazy;
use std::ops::Deref;

#[lazy]
static I: String = "123".to_owned();
#[test]
fn it_works() {
    assert_eq!("main::I", get_type_name(&I));
    assert_eq!("123", I.deref());
}

fn get_type_name<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}
