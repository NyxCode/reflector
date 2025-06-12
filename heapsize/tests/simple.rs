use heapsize::{HeapSize, Reflect};
use reflector::Reflect;

#[test]
fn asdf() {
    #[derive(Reflect)]
    struct MyStruct(i32);
}

#[test]
fn simple_struct() {
    #[derive(Reflect)]
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
    #[derive(Reflect)]
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
    #[derive(Reflect)]
    pub struct X<'a, 'b>(&'a i32, &'b i32);

    assert_eq!(Reflect(&X(&0, &0)).heap_size(), 0);
}

#[test]
fn with_generics() {
    #[derive(Reflect)]
    pub struct X<A, B>(A, Box<B>);

    assert_eq!(Reflect(&X(Box::new(0u8), Box::new(0u8))).heap_size(), 2);
}

#[test]
fn unit() {
    #[derive(Reflect)]
    pub struct A;
    #[derive(Reflect)]
    pub struct B();
    #[derive(Reflect)]
    pub struct C {};

    assert_eq!(Reflect(&A).heap_size(), 0);
    assert_eq!(Reflect(&B()).heap_size(), 0);
    assert_eq!(Reflect(&C{}).heap_size(), 0);
}

fn simple_enum() {
    #[derive(Reflect)]
    pub enum Simple {
        A(i32),
        B(Box<[u8; 128]>),
        C(Vec<u8>),
    }
    
    assert_eq!(Reflect(&Simple::A(0)).heap_size(), 0);
    assert_eq!(Reflect(&Simple::B(Box::new([0; 128]))).heap_size(), 128);
    assert_eq!(Reflect(&Simple::C(vec![0; 1024])).heap_size(), 1024);
}