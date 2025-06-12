use heapsize::{HeapSize, Reflect};
use reflector::Introspect;

#[test]
fn asdf() {
    #[derive(Introspect)]
    struct MyStruct(i32);
}

#[test]
fn simple_struct() {
    #[derive(Introspect)]
    pub struct MyStruct {
        a: i32,
        b: Vec<u8>,
    }

    assert_eq!(
        Reflect(&MyStruct {
            a: 42,
            b: vec![42; 42],
        })
        .heap_size(),
        42
    );
}

#[test]
fn tuple_struct() {
    #[derive(Introspect)]
    pub struct MyStruct(&'static str, Box<str>, Vec<Self>);

    impl HeapSize for MyStruct {
        const HAS_HEAP: bool = Reflect::<Self>::HAS_HEAP;

        fn heap_size(&self) -> usize {
            Reflect(self).heap_size()
        }
    }

    assert_eq!(
        MyStruct("", "x".into(), vec![MyStruct("", "xy".into(), vec![])]).heap_size(),
        size_of::<MyStruct>() + 3
    );
}

#[test]
fn with_lifetime() {
    #[derive(Introspect)]
    pub struct X<'a, 'b>(&'a i32, &'b i32);

    assert_eq!(Reflect(&X(&0, &0)).heap_size(), 0);
}

#[test]
fn with_generics() {
    #[derive(Introspect)]
    pub struct X<A, B>(A, Box<B>);

    assert_eq!(Reflect(&X(Box::new(0u8), Box::new(0u8))).heap_size(), 2);
}

#[test]
fn unit() {
    #[derive(Introspect)]
    pub struct A;
    #[derive(Introspect)]
    pub struct B();
    #[derive(Introspect)]
    pub struct C {};

    assert_eq!(Reflect(&A).heap_size(), 0);
    assert_eq!(Reflect(&B()).heap_size(), 0);
    assert_eq!(Reflect(&C {}).heap_size(), 0);
}

#[test]
fn simple_enum() {
    #[derive(Introspect)]
    pub enum Simple {
        A(i32),
        B(Box<[u8; 128]>),
        C(Vec<u8>),
    }

    assert_eq!(Reflect(&Simple::A(0)).heap_size(), 0);
    assert_eq!(Reflect(&Simple::B(Box::new([0; 128]))).heap_size(), 128);
    assert_eq!(Reflect(&Simple::C(vec![0; 1024])).heap_size(), 1024);
}

#[test]
fn gigantic() {
    #[derive(Introspect)]
    #[rustfmt::skip]
    pub struct Gigantic {
        f00: u8, f01: u8, f02: u8, f03: u8, f04: u8, f05: u8, f06: u8, f07: u8, f08: u8, f09: u8, 
        f10: u8, f11: u8, f12: u8, f13: u8, f14: u8, f15: u8, f16: u8, f17: u8, f18: u8, f19: u8, 
        f20: u8, f21: u8, f22: u8, f23: u8, f24: u8, f25: u8, f26: u8, f27: u8, f28: u8, f29: u8, 
        f30: u8, f31: u8, f32: u8, f33: u8, f34: u8, f35: u8, f36: u8, f37: u8, f38: u8, f39: u8, 
        f40: u8, f41: u8, f42: u8, f43: u8, f44: u8, f45: u8, f46: u8, f47: u8, f48: u8, f49: u8, 
        f50: u8, f51: u8, f52: u8, f53: u8, f54: u8, f55: u8, f56: u8, f57: u8, f58: u8, f59: u8, 
        f60: u8, f61: u8, f62: u8, f63: u8, f64: u8, f65: u8, f66: u8, f67: u8, f68: u8, f69: u8, 
        f70: u8, f71: u8, f72: u8, f73: u8, f74: u8, f75: u8, f76: u8, f77: u8, f78: u8, f79: u8, 
        f80: u8, f81: u8, f82: u8, f83: u8, f84: u8, f85: u8, f86: u8, f87: u8, f88: u8, f89: u8, 
        f90: u8, f91: u8, f92: u8, f93: u8, f94: u8, f95: u8, f96: u8, f97: u8, f98: u8, f99: u8, 
    }
}
