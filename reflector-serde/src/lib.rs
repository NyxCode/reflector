pub mod de;
pub mod ser;
mod visit;

#[test]
fn works() {
    use reflector::Introspect;

    macro_rules! roundtrip {
        ($e:expr) => {{
            let json = serde_json::to_string(&ser::Reflect(&$e)).unwrap();
            let back = serde_json::from_str::<de::Reflect<_>>(&json).unwrap().0;
            assert_eq!($e, back);
        }};
    }

    #[derive(PartialEq, Debug, Introspect)]
    struct A<X> {
        a: i32,
        b: X,
    }
    roundtrip!(A { a: 42, b: 3u8 });
    roundtrip!(A { a: 42, b: "hey" });

    #[derive(PartialEq, Debug, Introspect)]
    struct B<'a>(i32, &'a str);
    roundtrip!(B(3, "hey"));

    #[derive(PartialEq, Debug, Introspect)]
    enum C<'a> {
        A,
        B(i32),
        C(i32, &'a str),
        D { x: &'a str },
    }
    roundtrip!(C::A);
    roundtrip!(C::B(42));
    roundtrip!(C::C(42, "hey"));
    roundtrip!(C::D { x: "hey" });
}
