use reflector::*;

pub use reflect::Reflect;

/// Compute the number of bytes a value uses on the heap
pub trait HeapSize {
    const HAS_HEAP: bool;

    fn heap_size(&self) -> usize;
}

// instead of a derive macro, this enables users to get an impl of `HeapSize` for their structs.
mod reflect {
    use super::HeapSize;
    use reflector::*;

    pub struct Reflect<'a, T>(pub &'a T);

    trait ReflectHelper<Shape> {
        const HAS_HEAP: bool;

        fn heap_size(&self) -> usize;
    }

    impl<'a, T> HeapSize for Reflect<'a, T>
    where
        T: Introspect,
        T: ReflectHelper<T::Kind>,
    {
        const HAS_HEAP: bool = T::HAS_HEAP;

        fn heap_size(&self) -> usize {
            T::heap_size(&self.0)
        }
    }

    impl<T> ReflectHelper<StructType> for T
    where
        T: Struct,
        T::Fields: HeapSizeFields<T>,
    {
        const HAS_HEAP: bool = <T::Fields as HeapSizeFields<T>>::HAS_HEAP;

        fn heap_size(&self) -> usize {
            <T::Fields as HeapSizeFields<T>>::heap_size(&self)
        }
    }

    impl<T> ReflectHelper<EnumType> for T
    where
        T: Enum,
        T::Variants: HeapSizeVariants<T>,
    {
        const HAS_HEAP: bool = <T::Variants as HeapSizeVariants<T>>::HAS_HEAP;

        fn heap_size(&self) -> usize {
            <T::Variants as HeapSizeVariants<T>>::heap_size(&self)
        }
    }

    // helper trait implemented recursively for a list of fields, e.g `(Field0, (Field1, ()))`
    trait HeapSizeFields<P> {
        const HAS_HEAP: bool;

        fn heap_size(parent: &P) -> usize;
    }
    // end of the recursion
    impl<P> HeapSizeFields<P> for () {
        const HAS_HEAP: bool = false;

        fn heap_size(parent: &P) -> usize {
            0
        }
    }
    impl<P, Head, Tail> HeapSizeFields<P> for (Head, Tail)
    where
        Head: Field<Root = P>,
        Head::Type: HeapSize,
        Tail: HeapSizeFields<P>,
    {
        const HAS_HEAP: bool = Head::Type::HAS_HEAP || Tail::HAS_HEAP;

        fn heap_size(parent: &P) -> usize {
            Head::try_get_ref(parent).unwrap().heap_size() + Tail::heap_size(parent)
        }
    }

    // helper trait implemented recursively for a list of fields, e.g `(Field0, (Field1, ()))`
    trait HeapSizeVariants<P> {
        const HAS_HEAP: bool;

        fn heap_size(parent: &P) -> usize;
    }
    // end of the recursion
    impl<P> HeapSizeVariants<P> for () {
        const HAS_HEAP: bool = false;

        fn heap_size(parent: &P) -> usize {
            0
        }
    }
    impl<P, Head, Tail> HeapSizeVariants<P> for (Head, Tail)
    where
        Head: Variant<Root = P>,
        Head::Fields: HeapSizeFields<P>,
        Tail: HeapSizeVariants<P>,
    {
        const HAS_HEAP: bool = Head::Fields::HAS_HEAP || Tail::HAS_HEAP;

        fn heap_size(parent: &P) -> usize {
            if Head::is_active(parent) {
                Head::Fields::heap_size(parent)
            } else {
                Tail::heap_size(parent)
            }
        }
    }
}

// very boring impls for primitive types & stuff from std
mod impls {
    use super::HeapSize;

    macro_rules! primitives {
        ($($t:ty),*) => {$(
            impl HeapSize for $t  {
                const HAS_HEAP: bool = false;

                fn heap_size(&self) -> usize { 0 }
            }
        )*};
    }

    primitives!(
        u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, str
    );

    impl<T: HeapSize> HeapSize for [T] {
        const HAS_HEAP: bool = T::HAS_HEAP;

        fn heap_size(&self) -> usize {
            if T::HAS_HEAP {
                self.iter().map(T::heap_size).sum::<usize>()
            } else {
                0
            }
        }
    }

    impl<'a, T: HeapSize + ?Sized> HeapSize for &'a T {
        const HAS_HEAP: bool = T::HAS_HEAP;

        fn heap_size(&self) -> usize {
            T::heap_size(self)
        }
    }

    impl<T: HeapSize> HeapSize for Vec<T> {
        const HAS_HEAP: bool = true;

        fn heap_size(&self) -> usize {
            let direct = self.capacity() * size_of::<T>();
            if T::HAS_HEAP {
                let indirect = self.iter().map(T::heap_size).sum::<usize>();
                direct + indirect
            } else {
                direct
            }
        }
    }

    impl<T: HeapSize + ?Sized> HeapSize for Box<T> {
        const HAS_HEAP: bool = true;

        fn heap_size(&self) -> usize {
            let direct = size_of_val::<T>(self);
            if T::HAS_HEAP {
                direct + T::heap_size(self)
            } else {
                direct
            }
        }
    }

    impl<const N: usize, T: HeapSize> HeapSize for [T; N] {
        const HAS_HEAP: bool = T::HAS_HEAP;

        fn heap_size(&self) -> usize {
            if T::HAS_HEAP {
                self.iter().map(T::heap_size).sum::<usize>()
            } else {
                0
            }
        }
    }
}
