use crate::sample::*;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::slice::{from_raw_parts, Iter};

///A contiguous growable block of sample.
#[repr(C)]
pub struct Block<T>(Vec<T>);

impl<T> std::fmt::Debug for Block<T>
where
    T: Sample,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("Block");
        f.field("channel_size", &self.channel_size())
            .field("byte_size", &self.byte_size())
            .field("bit_depth", &self.bit_depth())
            .field("data_size", &self.0.len())
            .finish()
    }
}

impl<T> Deref for Block<T>
where
    T: Sample,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Block<T>
where
    T: Sample,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Block<T>
where
    T: Sample,
{
    ///Constructs a new, empty Block with the specified capacity.
    pub fn new(n: usize) -> Self {
        Self(Vec::with_capacity(n))
    }

    ///Returns channel size.
    pub fn channel_size(&self) -> u16 {
        T::CHANNEL_SIZE
    }

    ///Returns byte size.
    pub fn byte_size(&self) -> usize {
        T::BYTE_SIZE
    }

    ///Returns bit depth.
    pub fn bit_depth(&self) -> usize {
        8 * (T::BYTE_SIZE / T::CHANNEL_SIZE as usize)
    }

    fn u8_size(&self) -> usize {
        T::BYTE_SIZE * self.0.len()
    }

    ///Returns a slice of u8 bytes.
    pub fn bytes_slice(&self) -> &[u8] {
        let n = self.u8_size();
        let ptr = self.0.as_ptr() as *mut u8;
        unsafe { from_raw_parts(ptr, n) }
    }

    ///Copies self into a new `Vec<u8>` as a byte array in native byte order.
    pub fn copy_to_ne_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.u8_size());
        for i in self.0.iter() {
            v.extend_from_slice(&i.copy_to_ne_bytes());
        }
        v
    }

    ///Copies self into a new `Vec<u8>` as a byte array in big-endian byte order.
    pub fn copy_to_be_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.u8_size());
        for i in self.0.iter() {
            v.extend_from_slice(&i.copy_to_be_bytes());
        }
        v
    }

    ///Copies self into a new `Vec<u8>` as a byte array in little-endian byte order.
    pub fn copy_to_le_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.u8_size());
        for i in self.0.iter() {
            v.extend_from_slice(&i.copy_to_le_bytes());
        }
        v
    }

    ///Returns an iterator.
    pub fn channel_iter(&self, n: usize) -> ChannelIter<'_, T> {
        ChannelIter::new(self.iter(), n)
    }
}

impl<T> From<Box<[T]>> for Block<T>
where
    T: Sample,
{
    fn from(o: Box<[T]>) -> Self {
        Self(Vec::from(o))
    }
}

impl<T> From<&[T]> for Block<T>
where
    T: Type + Clone,
{
    fn from(o: &[T]) -> Self {
        Self(o.to_vec())
    }
}

impl<T> From<Vec<T>> for Block<T>
where
    T: Sample,
{
    fn from(o: Vec<T>) -> Self {
        Self(o)
    }
}

impl<T> Into<Box<[u8]>> for Block<T>
where
    T: Sample,
{
    fn into(self) -> Box<[u8]> {
        Into::<Vec<u8>>::into(self).into_boxed_slice()
    }
}

impl<T> Into<Vec<u8>> for Block<T>
where
    T: Sample,
{
    fn into(self) -> Vec<u8> {
        let n = self.u8_size();
        let mut o = ManuallyDrop::new(self.0);
        let ptr = o.as_mut_ptr() as *mut u8;
        unsafe { Vec::<u8>::from_raw_parts(ptr, n, n) }
    }
}

impl<T> Into<ByteBlock> for Block<T>
where
    T: Sample,
{
    fn into(self) -> ByteBlock {
        ByteBlock::new(
            self.channel_size(),
            self.byte_size(),
            #[cfg(target_endian = "big")]
            true,
            #[cfg(target_endian = "little")]
            false,
            self.into(),
        )
    }
}

///Immutable channel iterator.
pub struct ChannelIter<'a, T>(Iter<'a, T>, bool, usize);

impl<'a, T> ChannelIter<'a, T>
where
    T: Sample,
{
    fn new(i: Iter<'a, T>, n: usize) -> Self {
        Self(i, n < (T::CHANNEL_SIZE as usize), n)
    }

    ///Advances the iterator and returns the next value.
    pub fn next<U>(&mut self, f: impl Fn(&T, usize) -> U) -> Option<U>
    where
        U: Type,
    {
        if self.1 {
            return self.0.next().map(|o| f(o, self.2));
        }
        None
    }
}

///A byte array of sample.
#[repr(C)]
pub struct ByteBlock {
    channel_size: u16,
    byte_size: usize,
    big_endian: bool,
    data: Vec<u8>,
}

impl std::fmt::Debug for ByteBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("ByteBlock");
        f.field("channel_size", &self.channel_size())
            .field("byte_size", &self.byte_size())
            .field("bit_depth", &self.bit_depth())
            .field("big_endian", &self.big_endian)
            .field("data_size", &self.data.len())
            .finish()
    }
}

impl Deref for ByteBlock {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for ByteBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl ByteBlock {
    ///Constructs a new ByteBlock.
    pub fn new(channel_size: u16, byte_size: usize, big_endian: bool, data: Vec<u8>) -> Self {
        Self {
            channel_size,
            byte_size,
            big_endian,
            data,
        }
    }

    ///Constructs a new ByteBlock from Block as a byte array in big-endian/little-endian byte order.
    pub fn from_block<T: Sample>(big_endian: bool, o: &Block<T>) -> Self {
        Self::new(
            o.channel_size(),
            o.byte_size(),
            big_endian,
            if big_endian {
                o.copy_to_be_bytes()
            } else {
                o.copy_to_le_bytes()
            },
        )
    }

    ///Returns channel size.
    pub fn channel_size(&self) -> u16 {
        self.channel_size
    }

    ///Returns byte size.
    pub fn byte_size(&self) -> usize {
        self.byte_size
    }

    ///Returns bit depth.
    pub fn bit_depth(&self) -> usize {
        8 * (self.byte_size / self.channel_size as usize)
    }

    ///Converts Self into Block.
    pub fn into_block<T: Sample>(self, f: impl Fn(&[u8]) -> T) -> Block<T> {
        let byte_size = self.byte_size;
        let len = self.data.len();
        let data = &self.data;
        let mut v = Vec::new();
        let a = len % byte_size;
        let b = len - a;
        let mut i = 0;
        while i < b {
            let o = unsafe { from_raw_parts(&data[i], byte_size) };
            v.push(f(o));
            i += byte_size;
        }
        if a > 0 {
            let o = unsafe { from_raw_parts(&data[i], a) };
            v.push(f(o));
        }
        Block::from(v)
    }
}

impl<T> From<&Block<T>> for ByteBlock
where
    T: Sample,
{
    fn from(o: &Block<T>) -> Self {
        Self::new(
            o.channel_size(),
            o.byte_size(),
            #[cfg(target_endian = "big")]
            true,
            #[cfg(target_endian = "little")]
            false,
            o.copy_to_ne_bytes(),
        )
    }
}

impl Into<Box<[u8]>> for ByteBlock {
    fn into(self) -> Box<[u8]> {
        self.data.into_boxed_slice()
    }
}

impl Into<Vec<u8>> for ByteBlock {
    fn into(self) -> Vec<u8> {
        self.data
    }
}

///Whole info of audio.
#[repr(C)]
pub struct Whole<T> {
    sample_rate: u32,
    data: Block<T>,
}

impl<T> std::fmt::Debug for Whole<T>
where
    T: Sample,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("Whole");
        f.field("sample_rate", &self.sample_rate)
            .field("channel_size", &self.channel_size())
            .field("byte_size", &self.byte_size())
            .field("bit_depth", &self.bit_depth())
            .field("data_size", &self.0.len())
            .finish()
    }
}

impl<T> Deref for Whole<T>
where
    T: Sample,
{
    type Target = Block<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Whole<T>
where
    T: Sample,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> Whole<T>
where
    T: Sample,
{
    ///Constructs a new, empty Whole with the specified capacity.
    pub fn new(sample_rate: u32, n: usize) -> Self {
        Self::from_block(sample_rate, Block::new(n))
    }

    ///Constructs a new Whole from Block.
    pub fn from_block(sample_rate: u32, data: Block<T>) -> Self {
        Self { sample_rate, data }
    }

    ///Add elements of other into self.
    pub fn add(&mut self, mut o: Block<T>) {
        self.data.append(&mut o);
    }
}

impl<T> Into<Block<T>> for Whole<T>
where
    T: Sample,
{
    fn into(self) -> Block<T> {
        self.data
    }
}

macro_rules! min_merge {
    ($a:ident $(, $o:ident )+) => {{
        let mut min = $a.len();
        $(
            let o_len = $o.len();
            if min > o_len {
                min = o_len;
            }
        )*
        let mut v = Block::new(min);
        for i in 0..min {
            v.push( [$a[i].clone()$(, $o[i].clone())*]);
        }
        v
    }};
}

macro_rules! max_merge {
    ($a:ident $(, $o:ident )+) => {{
        let a_len = $a.len();
        let mut min = a_len;
        let mut max = a_len;
        $(
            let o_len = $o.len();
            if min > o_len {
                min = o_len;
            }
            if max < o_len {
                max = o_len;
            }
        )*
        let mut v = Block::new(max);
        for i in 0..min {
            v.push([$a[i].clone()$(, $o[i].clone())*]);
        }
        for i in min..max {
            let o = [
                if i < a_len {
                    $a[i].clone()
                } else {
                    T::default()
                }$(, if i < $o.len() {
                    $o[i].clone()
                } else {
                    T::default()
                })*
            ];
            v.push(o);
        }
        v
    }};
}

///Build a channel to block.
pub fn build_mono<T>(a: &[T]) -> Block<T>
where
    T: Type + Clone,
{
    Block::from(a)
}

///Build 2 planar channels to packed block.
pub fn build_2<T>(a: &[T], b: &[T]) -> Block<[T; 2]>
where
    T: Type + Clone,
{
    min_merge!(a, b)
}

///Build 2 planar channels to packed block. Padding if slice's number are different.
pub fn build_2_padding<T>(a: &[T], b: &[T]) -> Block<[T; 2]>
where
    T: Type + Clone + Default,
{
    max_merge!(a, b)
}

///Build 2 planar channels to packed block.
pub fn build_stereo<T>(a: &[T], b: &[T]) -> Block<[T; 2]>
where
    T: Type + Clone + Default,
{
    build_2(a, b)
}

///Build 3 planar channels to packed block.
pub fn build_3<T>(a: &[T], b: &[T], c: &[T]) -> Block<[T; 3]>
where
    T: Type + Clone,
{
    min_merge!(a, b, c)
}

///Build 3 planar channels to packed block. Padding if slice's number are different.
pub fn build_3_padding<T>(a: &[T], b: &[T], c: &[T]) -> Block<[T; 3]>
where
    T: Type + Clone + Default,
{
    max_merge!(a, b, c)
}

///Build 3 planar channels to packed block.
pub fn build_surround<T>(a: &[T], b: &[T], c: &[T]) -> Block<[T; 3]>
where
    T: Type + Clone + Default,
{
    build_3(a, b, c)
}

///Build 4 planar channels to packed block.
pub fn build_4<T>(a: &[T], b: &[T], c: &[T], d: &[T]) -> Block<[T; 4]>
where
    T: Type + Clone,
{
    min_merge!(a, b, c, d)
}

///Build 4 planar channels to packed block. Padding if slice's number are different.
pub fn build_4_padding<T>(a: &[T], b: &[T], c: &[T], d: &[T]) -> Block<[T; 4]>
where
    T: Type + Clone + Default,
{
    max_merge!(a, b, c, d)
}

///Build 5 planar channels to packed block.
pub fn build_5<T>(a: &[T], b: &[T], c: &[T], d: &[T], e: &[T]) -> Block<[T; 5]>
where
    T: Type + Clone,
{
    min_merge!(a, b, c, d, e)
}

///Build 5 planar channels to packed block. Padding if slice's number are different.
pub fn build_5_padding<T>(a: &[T], b: &[T], c: &[T], d: &[T], e: &[T]) -> Block<[T; 5]>
where
    T: Type + Clone + Default,
{
    max_merge!(a, b, c, d, e)
}

///Build 6 planar channels to packed block.
pub fn build_6<T>(a: &[T], b: &[T], c: &[T], d: &[T], e: &[T], f: &[T]) -> Block<[T; 6]>
where
    T: Type + Clone,
{
    min_merge!(a, b, c, d, e, f)
}

///Build 6 planar channels to packed block. Padding if slice's number are different.
pub fn build_6_padding<T>(a: &[T], b: &[T], c: &[T], d: &[T], e: &[T], f: &[T]) -> Block<[T; 6]>
where
    T: Type + Clone + Default,
{
    max_merge!(a, b, c, d, e, f)
}

///Build 6 planar channels to packed block.
pub fn build_hexagonal<T>(a: &[T], b: &[T], c: &[T], d: &[T], e: &[T], f: &[T]) -> Block<[T; 6]>
where
    T: Type + Clone + Default,
{
    build_6(a, b, c, d, e, f)
}

///Build 7 planar channels to packed block.
pub fn build_7<T>(a: &[T], b: &[T], c: &[T], d: &[T], e: &[T], f: &[T], g: &[T]) -> Block<[T; 7]>
where
    T: Type + Clone,
{
    min_merge!(a, b, c, d, e, f, g)
}

///Build 7 planar channels to packed block. Padding if slice's number are different.
pub fn build_7_padding<T>(
    a: &[T],
    b: &[T],
    c: &[T],
    d: &[T],
    e: &[T],
    f: &[T],
    g: &[T],
) -> Block<[T; 7]>
where
    T: Type + Clone + Default,
{
    max_merge!(a, b, c, d, e, f, g)
}

///Build 8 planar channels to packed block.
pub fn build_8<T>(
    a: &[T],
    b: &[T],
    c: &[T],
    d: &[T],
    e: &[T],
    f: &[T],
    g: &[T],
    h: &[T],
) -> Block<[T; 8]>
where
    T: Type + Clone,
{
    min_merge!(a, b, c, d, e, f, g, h)
}

///Build 8 planar channels to packed block. Padding if slice's number are different.
pub fn build_8_padding<T>(
    a: &[T],
    b: &[T],
    c: &[T],
    d: &[T],
    e: &[T],
    f: &[T],
    g: &[T],
    h: &[T],
) -> Block<[T; 8]>
where
    T: Type + Clone + Default,
{
    max_merge!(a, b, c, d, e, f, g, h)
}

///Build 8 planar channels to packed block.
pub fn build_octagonal<T>(
    a: &[T],
    b: &[T],
    c: &[T],
    d: &[T],
    e: &[T],
    f: &[T],
    g: &[T],
    h: &[T],
) -> Block<[T; 8]>
where
    T: Type + Clone + Default,
{
    build_8(a, b, c, d, e, f, g, h)
}

///Build 8 planar channels to packed block.
pub fn build_cube<T>(
    a: &[T],
    b: &[T],
    c: &[T],
    d: &[T],
    e: &[T],
    f: &[T],
    g: &[T],
    h: &[T],
) -> Block<[T; 8]>
where
    T: Type + Clone + Default,
{
    build_8(a, b, c, d, e, f, g, h)
}
