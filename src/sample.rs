///A trait for Type.
pub trait Type {
    ///Returns the memory representation of self as a byte array in native byte order.
    fn ne_bytes(self) -> Vec<u8>;

    ///Returns the memory representation of self as a byte array in big-endian byte order.
    fn be_bytes(self) -> Vec<u8>;

    ///Returns the memory representation of self as a byte array in little-endian byte order.
    fn le_bytes(self) -> Vec<u8>;
}

macro_rules! type_for {
    ($t:ty) => {
        impl Type for $t {
            fn ne_bytes(self) -> Vec<u8> {
                Vec::from(self.to_ne_bytes())
            }

            fn be_bytes(self) -> Vec<u8> {
                Vec::from(self.to_be_bytes())
            }

            fn le_bytes(self) -> Vec<u8> {
                Vec::from(self.to_le_bytes())
            }
        }
    };
}

type_for!(f32);

type_for!(f64);

type_for!(i8);

type_for!(i16);

type_for!(i32);

type_for!(i64);

type_for!(u8);

type_for!(u16);

type_for!(u32);

type_for!(u64);

///A trait for Sample.
pub trait Sample {
    const CHANNEL_SIZE: u16;

    const BYTE_SIZE: usize;

    ///Copies self into a new `Vec<u8>` as a byte array in native byte order.
    fn copy_to_ne_bytes(&self) -> Vec<u8>;

    ///Copies self into a new `Vec<u8>` as a byte array in big-endian byte order.
    fn copy_to_be_bytes(&self) -> Vec<u8>;

    ///Copies self into a new `Vec<u8>` as a byte array in little-endian byte order.
    fn copy_to_le_bytes(&self) -> Vec<u8>;
}

impl<T> Sample for T
where
    T: Type + Clone,
{
    const CHANNEL_SIZE: u16 = 1;

    const BYTE_SIZE: usize = size_of::<T>();

    fn copy_to_ne_bytes(&self) -> Vec<u8> {
        self.clone().ne_bytes()
    }

    fn copy_to_be_bytes(&self) -> Vec<u8> {
        self.clone().be_bytes()
    }

    fn copy_to_le_bytes(&self) -> Vec<u8> {
        self.clone().le_bytes()
    }
}

macro_rules! sample_array {
    ($n:expr) => {
        impl<T> Sample for [T; $n]
        where
            T: Type + Clone,
        {
            const CHANNEL_SIZE: u16 = $n;

            const BYTE_SIZE: usize = size_of::<T>() * $n;

            fn copy_to_ne_bytes(&self) -> Vec<u8> {
                let mut o = self.clone();
                let ptr = o.as_mut_ptr() as *mut u8;
                unsafe { Vec::from_raw_parts(ptr, Self::BYTE_SIZE, Self::BYTE_SIZE) }
            }

            fn copy_to_be_bytes(&self) -> Vec<u8> {
                let mut v = Vec::with_capacity(Self::BYTE_SIZE);
                for i in self {
                    v.extend_from_slice(&i.clone().be_bytes())
                }
                v
            }

            fn copy_to_le_bytes(&self) -> Vec<u8> {
                let mut v = Vec::with_capacity(Self::BYTE_SIZE);
                for i in self {
                    v.extend_from_slice(&i.clone().le_bytes())
                }
                v
            }
        }
    };
}

sample_array!(2);

sample_array!(3);

sample_array!(4);

sample_array!(5);

sample_array!(6);

sample_array!(7);

sample_array!(8);
