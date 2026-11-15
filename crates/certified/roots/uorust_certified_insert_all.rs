#![forbid(unsafe_code)]
use core::cmp::Ordering;

use core::sync::atomic::{AtomicU64, Ordering as Memory};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Overflow;

pub type R<T> = Result<T, Overflow>;

static WORK: AtomicU64 = AtomicU64::new(0);

pub fn tick(units: u64) { WORK.fetch_add(units, Memory::Relaxed); }

pub fn work() -> u64 { WORK.load(Memory::Relaxed) }

pub trait Same {
    fn same(&self, other: &Self) -> bool;
}

pub trait Key {
    fn key(&self, other: &Self) -> Ordering;
}

impl Same for bool {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for bool {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for u8 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for u8 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for u16 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for u16 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for u32 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for u32 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for u64 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for u64 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for i8 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for i8 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for i16 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for i16 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for i32 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for i32 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for i64 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for i64 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for Ordering {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl<A: Key, B: Key> Key for (A, B) {
    fn key(&self, other: &Self) -> Ordering { match self.0.key(&other.0) { Ordering::Equal => self.1.key(&other.1), decided => decided, } }
}

pub fn equal<T: Same>(a: T, b: T) -> bool { a.same(&b) }

pub fn compare<T: Key>(a: T, b: T) -> Ordering { a.key(&b) }

pub fn nat_add(p0: u64, p1: u64) -> R<u64> {
    match u64::checked_add(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<u64, Overflow>(Overflow)
        }
    }
}

pub fn nat_sub(p0: u64, p1: u64) -> u64 {
    u64::saturating_sub(p0, p1)
}

pub fn nat_mul(p0: u64, p1: u64) -> R<u64> {
    match u64::checked_mul(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<u64, Overflow>(Overflow)
        }
    }
}

pub fn nat_quot(p0: u64, p1: u64, p2: u64) -> u64 {
    u64::checked_div(p0, p1).unwrap_or(p2)
}

pub fn nat_rem(p0: u64, p1: u64, p2: u64) -> u64 {
    u64::checked_rem(p0, p1).unwrap_or(p2)
}

pub fn nat_eq(p0: u64, p1: u64) -> bool {
    core::cmp::PartialEq::eq(&p0, &p1)
}

pub fn nat_le(p0: u64, p1: u64) -> bool {
    core::cmp::PartialOrd::le(&p0, &p1)
}

pub fn nat_lt(p0: u64, p1: u64) -> bool {
    core::cmp::PartialOrd::lt(&p0, &p1)
}

pub fn nat_succ(p0: u64) -> R<u64> {
    match u64::checked_add(p0, 1u64) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<u64, Overflow>(Overflow)
        }
    }
}

pub fn int_add(p0: i64, p1: i64) -> R<i64> {
    match i64::checked_add(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    }
}

pub fn int_sub(p0: i64, p1: i64) -> R<i64> {
    match i64::checked_sub(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    }
}

pub fn int_mul(p0: i64, p1: i64) -> R<i64> {
    match i64::checked_mul(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    }
}

pub fn int_neg(p0: i64) -> R<i64> {
    match i64::checked_neg(p0) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    }
}

pub fn int_quot(p0: i64, p1: i64, p2: i64) -> R<i64> {
    { let value = { let (a, b, z): (i64, i64, i64) = (p0, p1, p2); if b == 0 { Some(z) } else { a.checked_div(b) } }; match value {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    } }
}

pub fn int_rem(p0: i64, p1: i64, p2: i64) -> i64 {
    { let (a, b, z): (i64, i64, i64) = (p0, p1, p2); if b == 0 { z } else { a.wrapping_rem(b) } }
}

pub fn bool_not(p0: bool) -> bool {
    !p0
}

pub fn bool_and(p0: bool, p1: bool) -> bool {
    if p0 {
        p1
    } else {
        false
    }
}

pub fn bool_or(p0: bool, p1: bool) -> bool {
    if p0 {
        true
    } else {
        p1
    }
}

pub mod fixed_u8 {
    pub fn checked_add(p0: u8, p1: u8) -> Option<u8> {
        u8::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: u8, p1: u8) -> Option<u8> {
        u8::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: u8, p1: u8) -> Option<u8> {
        u8::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: u8, p1: u8) -> Option<u8> {
        u8::checked_div(p0, p1)
    }

    pub fn bit_and(p0: u8, p1: u8) -> u8 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: u8, p1: u8) -> u8 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: u8, p1: u8) -> u8 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: u8) -> u8 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: u8, p1: u32) -> Option<u8> {
        u8::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: u8, p1: u32) -> Option<u8> {
        u8::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<u8> {
        u8::try_from(p0).ok()
    }
}

pub mod fixed_u16 {
    pub fn checked_add(p0: u16, p1: u16) -> Option<u16> {
        u16::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: u16, p1: u16) -> Option<u16> {
        u16::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: u16, p1: u16) -> Option<u16> {
        u16::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: u16, p1: u16) -> Option<u16> {
        u16::checked_div(p0, p1)
    }

    pub fn bit_and(p0: u16, p1: u16) -> u16 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: u16, p1: u16) -> u16 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: u16, p1: u16) -> u16 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: u16) -> u16 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: u16, p1: u32) -> Option<u16> {
        u16::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: u16, p1: u32) -> Option<u16> {
        u16::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<u16> {
        u16::try_from(p0).ok()
    }
}

pub mod fixed_u32 {
    pub fn checked_add(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_div(p0, p1)
    }

    pub fn bit_and(p0: u32, p1: u32) -> u32 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: u32, p1: u32) -> u32 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: u32, p1: u32) -> u32 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: u32) -> u32 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<u32> {
        u32::try_from(p0).ok()
    }
}

pub mod fixed_u64 {
    pub fn checked_add(p0: u64, p1: u64) -> Option<u64> {
        u64::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: u64, p1: u64) -> Option<u64> {
        u64::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: u64, p1: u64) -> Option<u64> {
        u64::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: u64, p1: u64) -> Option<u64> {
        u64::checked_div(p0, p1)
    }

    pub fn bit_and(p0: u64, p1: u64) -> u64 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: u64, p1: u64) -> u64 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: u64, p1: u64) -> u64 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: u64) -> u64 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: u64, p1: u32) -> Option<u64> {
        u64::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: u64, p1: u32) -> Option<u64> {
        u64::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<u64> {
        u64::try_from(p0).ok()
    }
}

pub mod fixed_i8 {
    pub fn checked_add(p0: i8, p1: i8) -> Option<i8> {
        i8::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: i8, p1: i8) -> Option<i8> {
        i8::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: i8, p1: i8) -> Option<i8> {
        i8::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: i8, p1: i8) -> Option<i8> {
        i8::checked_div(p0, p1)
    }

    pub fn bit_and(p0: i8, p1: i8) -> i8 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: i8, p1: i8) -> i8 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: i8, p1: i8) -> i8 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: i8) -> i8 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: i8, p1: u32) -> Option<i8> {
        i8::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: i8, p1: u32) -> Option<i8> {
        i8::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<i8> {
        i8::try_from(p0).ok()
    }
}

pub fn checked_neg_i8(p0: i8) -> Option<i8> {
    i8::checked_neg(p0)
}

pub mod fixed_i16 {
    pub fn checked_add(p0: i16, p1: i16) -> Option<i16> {
        i16::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: i16, p1: i16) -> Option<i16> {
        i16::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: i16, p1: i16) -> Option<i16> {
        i16::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: i16, p1: i16) -> Option<i16> {
        i16::checked_div(p0, p1)
    }

    pub fn bit_and(p0: i16, p1: i16) -> i16 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: i16, p1: i16) -> i16 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: i16, p1: i16) -> i16 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: i16) -> i16 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: i16, p1: u32) -> Option<i16> {
        i16::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: i16, p1: u32) -> Option<i16> {
        i16::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<i16> {
        i16::try_from(p0).ok()
    }
}

pub fn checked_neg_i16(p0: i16) -> Option<i16> {
    i16::checked_neg(p0)
}

pub mod fixed_i32 {
    pub fn checked_add(p0: i32, p1: i32) -> Option<i32> {
        i32::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: i32, p1: i32) -> Option<i32> {
        i32::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: i32, p1: i32) -> Option<i32> {
        i32::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: i32, p1: i32) -> Option<i32> {
        i32::checked_div(p0, p1)
    }

    pub fn bit_and(p0: i32, p1: i32) -> i32 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: i32, p1: i32) -> i32 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: i32, p1: i32) -> i32 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: i32) -> i32 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: i32, p1: u32) -> Option<i32> {
        i32::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: i32, p1: u32) -> Option<i32> {
        i32::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<i32> {
        i32::try_from(p0).ok()
    }
}

pub fn checked_neg_i32(p0: i32) -> Option<i32> {
    i32::checked_neg(p0)
}

pub mod fixed_i64 {
    pub fn checked_add(p0: i64, p1: i64) -> Option<i64> {
        i64::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: i64, p1: i64) -> Option<i64> {
        i64::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: i64, p1: i64) -> Option<i64> {
        i64::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: i64, p1: i64) -> Option<i64> {
        i64::checked_div(p0, p1)
    }

    pub fn bit_and(p0: i64, p1: i64) -> i64 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: i64, p1: i64) -> i64 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: i64, p1: i64) -> i64 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: i64) -> i64 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: i64, p1: u32) -> Option<i64> {
        i64::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: i64, p1: u32) -> Option<i64> {
        i64::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<i64> {
        i64::try_from(p0).ok()
    }
}

pub fn checked_neg_i64(p0: i64) -> Option<i64> {
    i64::checked_neg(p0)
}
use core::str;

use std::mem;

use std::rc::Rc;

#[derive(Clone)]
pub struct Str(Rc<str>);

impl Str {
    pub fn lit(text: &str) -> Str { Str(Rc::from(text)) }
    pub fn text(&self) -> &str { &self.0 }
    pub fn length(&self) -> u64 { chars(&self.0) }
    pub fn utf8_encode(&self) -> Bytes { tick(self.0.len() as u64); Bytes::lit(self.0.as_bytes()) }
    pub fn split_exact(&self, delimiter: &Str, maximum: u32) -> Option<List<Str>> { if delimiter.0.is_empty() { return None; }; chars(&self.0); let mut fields: Vec<Str> = Vec::new(); for piece in self.0.split(&*delimiter.0) { fields.push(Str::lit(piece)); }; if fields.len() as u128 > u128::from(maximum) { return None; }; Some(List::onto(fields, List::nil())) }
    pub fn join(texts: List<Str>, delimiter: &Str) -> Str { let mut out = String::new(); let mut first = true; let mut cursor = &texts; while let Repr::Cell(cell) = &cursor.0 { tick(1); if !first { chars(&delimiter.0); out.push_str(&delimiter.0); }; first = false; chars(&cell.head.0); out.push_str(&cell.head.0); cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for text in buffer[..*len].iter().rev() { tick(1); if !first { chars(&delimiter.0); out.push_str(&delimiter.0); }; first = false; chars(&text.0); out.push_str(&text.0); } }, _ => { }, }; Str::lit(&out) }
}

#[derive(Clone)]
pub struct Bytes(Rc<[u8]>);

impl Bytes {
    pub fn lit(octets: &[u8]) -> Bytes { Bytes(Rc::from(octets)) }
    pub fn octets(&self) -> &[u8] { &self.0 }
    pub fn append(a: Bytes, b: Bytes) -> Bytes { tick((a.0.len() + b.0.len()) as u64); let mut out = Vec::with_capacity(a.0.len() + b.0.len()); out.extend_from_slice(&a.0); out.extend_from_slice(&b.0); Bytes(Rc::from(out)) }
    pub fn length(&self) -> u64 { self.0.len() as u64 }
    pub fn index(&self, i: u64) -> Option<u8> { match usize::try_from(i) { Ok(at) => self.0.get(at).copied(), Err(_) => None, } }
    pub fn slice(&self, start: u64, count: u64) -> Option<Bytes> { let end = u128::from(start) + u128::from(count); if end > self.0.len() as u128 { return None; }; tick(count); Some(Bytes::lit(&self.0[start as usize..end as usize])) }
    pub fn utf8_decode(&self) -> Option<Str> { tick(self.0.len() as u64); match str::from_utf8(&self.0) { Ok(text) => Some(Str::lit(text)), Err(_) => None, } }
    pub fn compare(&self, other: &Bytes) -> Ordering { tick(1 + self.0.len().min(other.0.len()) as u64); self.0.cmp(&other.0) }
}

pub struct Node<T> { head: T, tail: List<T> }

pub enum Repr<T> { Nil, Run { buffer: Rc<Vec<T>>, len: usize }, Cell(Rc<Node<T>>) }

pub struct List<T>(Repr<T>);

impl<T> Clone for List<T> {
    fn clone(&self) -> Self { List(match &self.0 { Repr::Nil => Repr::Nil, Repr::Run { buffer, len } => Repr::Run { buffer: buffer.clone(), len: *len }, Repr::Cell(cell) => Repr::Cell(cell.clone()), }) }
}

impl<T> List<T> {
    fn take(&mut self) -> Repr<T> { mem::replace(&mut self.0, Repr::Nil) }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) { let mut next = match self.tail.take() { Repr::Cell(cell) => Some(cell), _ => None, }; while let Some(cell) = next { match Rc::try_unwrap(cell) { Ok(mut node) => next = match node.tail.take() { Repr::Cell(cell) => Some(cell), _ => None, }, Err(_) => break, } } }
}

impl<T: Clone> List<T> {
    pub fn nil() -> Self { List(Repr::Nil) }
    pub fn cons(head: T, mut tail: List<T>) -> Self { List(match tail.take() { Repr::Nil => { let mut buffer = Vec::with_capacity(16); buffer.push(head); Repr::Run { buffer: Rc::new(buffer), len: 1 } }, Repr::Run { mut buffer, len } => { match Rc::get_mut(&mut buffer) { Some(cells) => { cells.truncate(len); cells.push(head); Repr::Run { buffer, len: len + 1 } }, None => Repr::Cell(Rc::new(Node { head, tail: List(Repr::Run { buffer, len }) })), } }, rest => Repr::Cell(Rc::new(Node { head, tail: List(rest) })), }) }
    pub fn uncons(&self) -> Option<(T, List<T>)> { match &self.0 { Repr::Nil => None, Repr::Run { buffer, len } => { let head = buffer[len - 1].clone(); Some((head, List(if *len == 1 { Repr::Nil } else { Repr::Run { buffer: buffer.clone(), len: len - 1 } }))) }, Repr::Cell(cell) => Some((cell.head.clone(), cell.tail.clone())), } }
    pub fn length(&self) -> u64 { let mut count = 0; let mut cursor = self; while let Repr::Cell(cell) = &cursor.0 { tick(1); count += 1; cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for _ in buffer[..*len].iter().rev() { tick(1); count += 1; } }, _ => { }, }; count }
    pub fn append(a: List<T>, b: List<T>) -> List<T> { List::onto(a.items(), b) }
    pub fn index(&self, i: u64) -> Option<T> { let mut position = 0; let mut cursor = self; while let Repr::Cell(cell) = &cursor.0 { tick(1); if position == i { return Some(cell.head.clone()); }; position += 1; cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for item in buffer[..*len].iter().rev() { tick(1); if position == i { return Some(item.clone()); }; position += 1; } }, _ => { }, }; None }
    pub fn slice(&self, start: u64, count: u64) -> Option<List<T>> { let end = u128::from(start) + u128::from(count); let mut part = Vec::new(); let mut position: u128 = 0; let mut cursor = self; while let Repr::Cell(cell) = &cursor.0 { tick(1); if position >= end { return Some(List::onto(part, List::nil())); }; if position >= u128::from(start) { part.push(cell.head.clone()); }; position += 1; cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for item in buffer[..*len].iter().rev() { tick(1); if position >= end { return Some(List::onto(part, List::nil())); }; if position >= u128::from(start) { part.push(item.clone()); }; position += 1; } }, _ => { }, }; if position < end { None } else { Some(List::onto(part, List::nil())) } }
    fn items(&self) -> Vec<T> { let mut out = Vec::new(); let mut cursor = self; while let Repr::Cell(cell) = &cursor.0 { tick(1); out.push(cell.head.clone()); cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for item in buffer[..*len].iter().rev() { tick(1); out.push(item.clone()); } }, _ => { }, }; out }
    fn onto(mut items: Vec<T>, tail: List<T>) -> List<T> { let mut out = tail; while let Some(item) = items.pop() { tick(1); out = List::cons(item, out); }; out }
}

pub fn chars(text: &str) -> u64 { let mut count = 0; for _ in text.chars() { tick(1); count += 1; }; count }

impl Same for Str {
    fn same(&self, other: &Self) -> bool { let mut left = self.0.chars(); let mut right = other.0.chars(); loop { tick(1); match (left.next(), right.next()) { (None, None) => return true, (a, b) if a != b => return false, _ => { }, } } }
}

impl Same for Bytes {
    fn same(&self, other: &Self) -> bool { tick(1 + self.0.len().min(other.0.len()) as u64); self.0 == other.0 }
}

impl Key for Str {
    fn key(&self, other: &Self) -> Ordering { let mut left = self.0.chars(); let mut right = other.0.chars(); loop { tick(1); match (left.next(), right.next()) { (None, None) => return Ordering::Equal, (None, Some(_)) => return Ordering::Less, (Some(_), None) => return Ordering::Greater, (Some(a), Some(b)) => match a.cmp(&b) { Ordering::Equal => { }, decided => return decided, }, } } }
}

pub fn canonical_decimal(text: &str) -> Option<Option<i128>> { let digits = match text.strip_prefix('-') { Some(rest) => rest, None => text, }; let mut canonical = !digits.is_empty() && (digits == "0" || !digits.starts_with('0')) && text != "-0"; for c in digits.chars() { tick(1); canonical = canonical && c.is_ascii_digit(); }; if !canonical { return None; }; Some(text.parse::<i128>().ok()) }

pub fn parse_int(text: Str) -> R<Option<i64>> { match canonical_decimal(&text.0) { None => Ok(None), Some(Some(n)) => match i64::try_from(n) { Ok(value) => Ok(Some(value)), Err(_) => Err(Overflow), }, Some(None) => Err(Overflow), } }

pub fn format_u8(a: u8) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_u8(text: Str) -> Option<u8> { match canonical_decimal(&text.0) { Some(Some(n)) => u8::try_from(n).ok(), _ => None, } }

pub fn format_u16(a: u16) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_u16(text: Str) -> Option<u16> { match canonical_decimal(&text.0) { Some(Some(n)) => u16::try_from(n).ok(), _ => None, } }

pub fn format_u32(a: u32) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_u32(text: Str) -> Option<u32> { match canonical_decimal(&text.0) { Some(Some(n)) => u32::try_from(n).ok(), _ => None, } }

pub fn format_u64(a: u64) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_u64(text: Str) -> Option<u64> { match canonical_decimal(&text.0) { Some(Some(n)) => u64::try_from(n).ok(), _ => None, } }

pub fn format_i8(a: i8) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_i8(text: Str) -> Option<i8> { match canonical_decimal(&text.0) { Some(Some(n)) => i8::try_from(n).ok(), _ => None, } }

pub fn format_i16(a: i16) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_i16(text: Str) -> Option<i16> { match canonical_decimal(&text.0) { Some(Some(n)) => i16::try_from(n).ok(), _ => None, } }

pub fn format_i32(a: i32) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_i32(text: Str) -> Option<i32> { match canonical_decimal(&text.0) { Some(Some(n)) => i32::try_from(n).ok(), _ => None, } }

pub fn format_i64(a: i64) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_i64(text: Str) -> Option<i64> { match canonical_decimal(&text.0) { Some(Some(n)) => i64::try_from(n).ok(), _ => None, } }

pub fn append_list<T: Clone>(p0: List<T>, p1: List<T>) -> List<T> {
    List::append(p0, p1)
}

pub fn length_list<T: Clone>(p0: List<T>) -> u64 {
    List::length(&p0)
}

pub fn index_list<T: Clone>(p0: List<T>, p1: u64) -> Option<T> {
    List::index(&p0, p1)
}

pub fn slice_list<T: Clone>(p0: List<T>, p1: u64, p2: u64) -> Option<List<T>> {
    List::slice(&p0, p1, p2)
}

pub fn append_bytes(p0: Bytes, p1: Bytes) -> Bytes {
    Bytes::append(p0, p1)
}

pub fn length_bytes(p0: Bytes) -> u64 {
    Bytes::length(&p0)
}

pub fn length_string(p0: Str) -> u64 {
    Str::length(&p0)
}

pub fn index_bytes(p0: Bytes, p1: u64) -> Option<u8> {
    Bytes::index(&p0, p1)
}

pub fn slice_bytes(p0: Bytes, p1: u64, p2: u64) -> Option<Bytes> {
    Bytes::slice(&p0, p1, p2)
}

pub fn utf8_encode(p0: Str) -> Bytes {
    Str::utf8_encode(&p0)
}

pub fn utf8_decode(p0: Bytes) -> Option<Str> {
    Bytes::utf8_decode(&p0)
}

pub fn compare_bytes(p0: Bytes, p1: Bytes) -> Ordering {
    Bytes::compare(&p0, &p1)
}

pub fn split_exact(p0: Str, p1: Str, p2: u32) -> Option<List<Str>> {
    Str::split_exact(&p0, &p1, p2)
}

pub fn join(p0: List<Str>, p1: Str) -> Str {
    Str::join(p0, &p1)
}

#[derive(Clone)]
pub enum Adt0 {
    C0,
    C1,
    C2(Option<u64>),
    C3(u64),
    C4(Adt1),
    C5(Option<u64>),
    C6,
    C7,
    C8,
    C9,
    C10,
    C11(Str),
    C12(std::rc::Rc<Adt0>),
    C13(std::rc::Rc<Adt0>, std::rc::Rc<Adt0>),
    C14(std::rc::Rc<Adt0>),
    C15(List<Adt0>),
    C16(List<Adt0>),
    C17(std::rc::Rc<Adt0>, std::rc::Rc<Adt0>, bool),
    C18(std::rc::Rc<Adt0>, bool),
    C19(List<Adt0>, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Adt3, List<Adt13>, List<Adt15>, List<Str>),
}

#[derive(Clone)]
pub enum Adt3 {
    C0(List<Adt4>, List<Adt5>, List<Adt6>, List<Adt7>, List<List<Str>>, List<Adt4>, List<Adt4>, bool, List<Adt8>, List<Adt9>, List<Adt10>, List<Adt12>),
}

#[derive(Clone)]
pub enum Adt4 {
    C0(Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt5 {
    C0(Str, List<(Str, Adt0)>),
}

#[derive(Clone)]
pub enum Adt6 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(Str, List<(Str, List<(Option<Str>, Adt0)>)>),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str, Adt1),
}

#[derive(Clone)]
pub enum Adt9 {
    C0(Str, Adt6),
}

#[derive(Clone)]
pub enum Adt10 {
    C0(Str, List<Adt11>),
}

#[derive(Clone)]
pub enum Adt11 {
    C0(u64, Str),
}

#[derive(Clone)]
pub enum Adt12 {
    C0(Str, List<i64>),
}

#[derive(Clone)]
pub enum Adt13 {
    C0(Str, Adt14),
}

#[derive(Clone)]
pub enum Adt14 {
    C0(Str, List<Adt0>, Adt0),
}

#[derive(Clone)]
pub enum Adt15 {
    C0(u64, List<Str>),
}

#[derive(Clone)]
pub enum Adt16 {
    C0(Str, List<Str>, Adt0, Str, List<Adt17>, List<Adt6>, bool, Option<List<Str>>, List<Str>, List<Adt6>, List<Adt18>, Option<Str>, List<Adt6>, List<Adt68>, List<Adt68>, Option<Adt0>, Option<Str>),
}

#[derive(Clone)]
pub enum Adt17 {
    C0(Str, (Str, Adt0)),
}

#[derive(Clone)]
pub enum Adt18 {
    C0(Str, Adt19),
}

#[derive(Clone)]
pub enum Adt19 {
    C0(List<Adt20>, Adt35, List<Adt17>),
}

#[derive(Clone)]
pub enum Adt20 {
    C0(std::rc::Rc<Adt21>),
    C1,
    C2(std::rc::Rc<Adt20>),
    C3(std::rc::Rc<Adt22>),
    C4(List<Adt20>),
    C5(Adt29),
    C6(Adt30),
    C7(List<Adt20>),
    C8,
    C9(Adt33),
    C10(std::rc::Rc<Adt34>),
    C11(Adt25),
    C12(List<Adt20>),
    C13(std::rc::Rc<Adt67>),
    C14,
}

#[derive(Clone)]
pub enum Adt21 {
    C0(Str, bool, std::rc::Rc<Option<Adt20>>),
}

#[derive(Clone)]
pub enum Adt22 {
    C0(std::rc::Rc<Adt20>, Adt23),
}

#[derive(Clone)]
pub enum Adt23 {
    C0(Str, std::rc::Rc<Adt24>),
}

#[derive(Clone)]
pub enum Adt24 {
    C0(bool, std::rc::Rc<Adt23>),
    C1(std::rc::Rc<Adt23>),
    C2(std::rc::Rc<Adt23>, Str),
    C3(List<Adt23>, std::rc::Rc<Adt23>),
    C4(std::rc::Rc<Adt23>),
    C5(List<Adt23>),
    C6(bool, Adt25),
    C7,
}

#[derive(Clone)]
pub enum Adt25 {
    C0(List<Adt26>, Str),
}

#[derive(Clone)]
pub enum Adt26 {
    C0(Str, Adt27),
}

#[derive(Clone)]
pub enum Adt27 {
    C0,
    C1(List<Adt28>),
    C2,
}

#[derive(Clone)]
pub enum Adt28 {
    C0(Adt23),
    C1(Str),
}

#[derive(Clone)]
pub enum Adt29 {
    C0(Adt25, List<Adt20>),
}

#[derive(Clone)]
pub enum Adt30 {
    C0(Adt25, List<Adt31>, bool),
}

#[derive(Clone)]
pub enum Adt31 {
    C0(Adt32, Adt20),
}

#[derive(Clone)]
pub enum Adt32 {
    C0(Str),
    C1(u64),
}

#[derive(Clone)]
pub enum Adt33 {
    C0(bool),
    C1(Option<u64>, Str),
    C2(Str),
    C3(Str),
    C4(u64),
    C5(List<u64>),
    C6,
}

#[derive(Clone)]
pub enum Adt34 {
    C0(std::rc::Rc<Option<Adt35>>, std::rc::Rc<Option<Adt35>>, bool),
}

#[derive(Clone)]
pub enum Adt35 {
    C0(List<Adt35>),
    C1(std::rc::Rc<Adt36>),
    C2(std::rc::Rc<Adt37>),
    C3(Adt39),
    C4(std::rc::Rc<Adt47>),
    C5(std::rc::Rc<Adt48>),
    C6(std::rc::Rc<Adt49>),
    C7(std::rc::Rc<Adt50>),
    C8(bool),
    C9(std::rc::Rc<Adt51>),
    C10(std::rc::Rc<Adt52>),
    C11(std::rc::Rc<Adt53>),
    C12(std::rc::Rc<Adt54>),
    C13(std::rc::Rc<Adt55>),
    C14(Adt33),
    C15(Adt56),
    C16(std::rc::Rc<Adt44>),
    C17(std::rc::Rc<Adt57>),
    C18(std::rc::Rc<Adt59>),
    C19(std::rc::Rc<Adt35>),
    C20(Adt25),
    C21(std::rc::Rc<Adt60>),
    C22(std::rc::Rc<Option<Adt35>>),
    C23(std::rc::Rc<Adt61>),
    C24(std::rc::Rc<Adt35>),
    C25(List<Adt35>),
    C26(std::rc::Rc<Adt63>),
    C27(std::rc::Rc<Adt65>),
    C28(Adt66),
}

#[derive(Clone)]
pub enum Adt36 {
    C0(std::rc::Rc<Adt35>, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt37 {
    C0(std::rc::Rc<Adt35>, Adt38, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt38 {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,
    C11,
    C12,
    C13,
    C14,
    C15,
    C16,
    C17,
    C18,
    C19,
    C20,
    C21,
    C22,
    C23,
    C24,
    C25,
    C26,
    C27,
}

#[derive(Clone)]
pub enum Adt39 {
    C0(bool, Adt40),
}

#[derive(Clone)]
pub enum Adt40 {
    C0(List<Adt41>),
}

#[derive(Clone)]
pub enum Adt41 {
    C0(Adt42),
    C1(Adt35, bool),
    C2(Adt44),
    C3,
}

#[derive(Clone)]
pub enum Adt42 {
    C0(Adt20, Option<Adt43>),
}

#[derive(Clone)]
pub enum Adt43 {
    C0(Adt35, Option<Adt35>),
}

#[derive(Clone)]
pub enum Adt44 {
    C0(Adt25, Option<Adt45>, std::rc::Rc<Option<Adt46>>),
}

#[derive(Clone)]
pub enum Adt45 {
    C0(List<Adt35>),
}

#[derive(Clone)]
pub enum Adt46 {
    C0(std::rc::Rc<Adt35>, std::rc::Rc<Adt20>),
}

#[derive(Clone)]
pub enum Adt47 {
    C0(bool, std::rc::Rc<Option<Adt35>>),
}

#[derive(Clone)]
pub enum Adt48 {
    C0(std::rc::Rc<Adt35>, List<Adt35>),
}

#[derive(Clone)]
pub enum Adt49 {
    C0(std::rc::Rc<Adt35>, Adt23),
}

#[derive(Clone)]
pub enum Adt50 {
    C0(List<Adt20>, std::rc::Rc<Adt35>, bool),
}

#[derive(Clone)]
pub enum Adt51 {
    C0(std::rc::Rc<Adt35>, Adt32),
}

#[derive(Clone)]
pub enum Adt52 {
    C0(bool, std::rc::Rc<Adt20>, std::rc::Rc<Adt35>, Adt40),
}

#[derive(Clone)]
pub enum Adt53 {
    C0(std::rc::Rc<Adt35>, Adt40, std::rc::Rc<Option<Adt35>>),
}

#[derive(Clone)]
pub enum Adt54 {
    C0(std::rc::Rc<Adt35>, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt55 {
    C0(std::rc::Rc<Adt20>, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt56 {
    C0(bool, Adt40),
}

#[derive(Clone)]
pub enum Adt57 {
    C0(std::rc::Rc<Adt35>, List<Adt58>),
}

#[derive(Clone)]
pub enum Adt58 {
    C0(Adt20, Adt35),
}

#[derive(Clone)]
pub enum Adt59 {
    C0(std::rc::Rc<Adt35>, Str, Option<List<Adt28>>, List<Adt35>),
}

#[derive(Clone)]
pub enum Adt60 {
    C0(bool, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt61 {
    C0(bool, Adt25, List<Adt62>, std::rc::Rc<Option<Adt35>>),
}

#[derive(Clone)]
pub enum Adt62 {
    C0(Adt32, Adt35),
}

#[derive(Clone)]
pub enum Adt63 {
    C0(Adt64, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt64 {
    C0,
    C1,
    C2,
}

#[derive(Clone)]
pub enum Adt65 {
    C0(bool, std::rc::Rc<Adt35>, Adt40),
}

#[derive(Clone)]
pub enum Adt66 {
    C0(Str, List<Adt35>, List<Adt40>),
}

#[derive(Clone)]
pub enum Adt67 {
    C0(std::rc::Rc<Adt20>, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt68 {
    C0(Str, u64),
}

#[derive(Clone)]
pub enum Adt69 {
    C0(List<u64>, List<Str>, List<Str>, List<Adt0>, List<Str>, List<Adt70>, List<Adt0>),
}

#[derive(Clone)]
pub enum Adt70 {
    C0(Str, List<Adt6>, Str, Adt71, List<Str>, List<Str>, bool, bool, Option<(Str, List<Adt73>)>),
}

#[derive(Clone)]
pub enum Adt71 {
    C0(Str),
    C1(Str, List<Adt72>),
    C2(std::rc::Rc<Adt71>, std::rc::Rc<Adt71>, std::rc::Rc<Adt71>),
    C3(Str, Str, std::rc::Rc<Adt71>, std::rc::Rc<Adt71>),
}

#[derive(Clone)]
pub enum Adt72 {
    C0(Str, Adt71),
}

#[derive(Clone)]
pub enum Adt73 {
    C0(Str, List<Adt6>),
    C1(Str, List<Str>),
    C2(Str, List<Adt4>, Option<Str>),
    C3(Str, List<Adt6>, Str, Adt71, List<Str>, Option<Str>, Option<Adt4>, Option<Str>),
    C4(Str, List<Adt6>, Str, Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt74 {
    C0(Str, Adt69),
}

pub fn f0(v0: Adt0, v1: Adt0, v2: Str, v3: Adt2, v4: Adt16, v5: Adt69) -> R<Adt74> {
    let v6: Adt74 = f1(v4.clone(), v3.clone(), v5, Str::lit("set"))?;
    let v7: Str = match v6.clone() {
        Adt74::C0(h1, _) => {
            h1
        }
    };
    let v8: Adt69 = match v6 {
        Adt74::C0(_, h2) => {
            h2
        }
    };
    let v9: Adt74 = f1(v4.clone(), v3.clone(), v8, Str::lit("items"))?;
    let v10: Str = match v9.clone() {
        Adt74::C0(h3, _) => {
            h3
        }
    };
    let v11: Adt69 = match v9 {
        Adt74::C0(_, h4) => {
            h4
        }
    };
    let v12: Adt74 = f1(v4.clone(), v3.clone(), v11, Str::lit("head"))?;
    let v13: Str = match v12.clone() {
        Adt74::C0(h5, _) => {
            h5
        }
    };
    let v14: Adt69 = match v12 {
        Adt74::C0(_, h6) => {
            h6
        }
    };
    let v15: Adt74 = f1(v4.clone(), v3.clone(), v14, Str::lit("rest"))?;
    let v16: Str = match v15.clone() {
        Adt74::C0(h7, _) => {
            h7
        }
    };
    let v17: Adt69 = match v15 {
        Adt74::C0(_, h8) => {
            h8
        }
    };
    let v18: Str = f2(v4, v3, v17.clone(), Str::lit("InsertAll"))?;
    let v19: Adt71 = Adt71::C1(v10.clone(), List::cons(Adt72::C0(Str::lit("List.nil"), Adt71::C0(v7.clone())), List::cons(Adt72::C0({
        let a9 = List::cons(Str::lit("List.cons "), List::cons(v13.clone(), List::cons(Str::lit(" "), List::cons(v16.clone(), List::<Str>::nil()))));
        let a10 = Str::lit("");
        join(a9, a10)
    }, Adt71::C0({
        let a11 = List::cons(Str::lit("("), List::cons(v18.clone(), List::cons(Str::lit(" "), List::cons(v16, List::cons(Str::lit(" ("), List::cons(v2.clone(), List::cons(Str::lit(" "), List::cons(v7.clone(), List::cons(Str::lit(" "), List::cons(v13, List::cons(Str::lit("))"), List::<Str>::nil())))))))))));
        let a12 = Str::lit("");
        join(a11, a12)
    })), List::<Adt72>::nil())));
    let v20: List<Str> = f3(v17.clone(), v2.clone());
    let v21: Adt69 = f4(v17, Adt70::C0(v18.clone(), List::cons(Adt6::C0(v10, f5(v1)), List::cons(Adt6::C0(v7, f5(v0.clone())), List::<Adt6>::nil())), f5(v0), v19, f6(List::cons({
        let a13 = List::cons(Str::lit("#"), List::cons(v2, List::<Str>::nil()));
        let a14 = Str::lit("");
        join(a13, a14)
    }, List::<Str>::nil()), List::<Str>::nil()), v20, true, false, None::<(Str, List<Adt73>)>));
    let v22: Adt69 = f7(v21, {
        let a15 = List::cons(Str::lit("#"), List::cons(v18.clone(), List::<Str>::nil()));
        let a16 = Str::lit("");
        join(a15, a16)
    });
    Ok(Adt74::C0(v18, v22))
}

pub fn f1(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str) -> R<Adt74> {
    let v4: Str = f8(v3.clone(), f9(v0, v1, v2.clone(), v3, List::<Str>::nil()))?;
    let v5: List<Str> = match v2.clone() {
        Adt69::C0(_, _, h17, _, _, _, _) => {
            h17
        }
    };
    let v6: List<Str> = {
        let a18 = v5;
        let a19 = List::cons(v4.clone(), List::<Str>::nil());
        append_list(a18, a19)
    };
    Ok(Adt74::C0(v4, Adt69::C0(match v2.clone() {
        Adt69::C0(h20, _, _, _, _, _, _) => {
            h20
        }
    }, match v2.clone() {
        Adt69::C0(_, h21, _, _, _, _, _) => {
            h21
        }
    }, v6, match v2.clone() {
        Adt69::C0(_, _, _, h22, _, _, _) => {
            h22
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, h23, _, _) => {
            h23
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, _, h24, _) => {
            h24
        }
    }, match v2 {
        Adt69::C0(_, _, _, _, _, _, h25) => {
            h25
        }
    })))
}

pub fn f2(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str) -> R<Str> {
    let v4: Str = {
        let a27 = List::cons(match v0.clone() {
            Adt16::C0(_, _, _, h26, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                h26
            }
        }, List::cons(v3, List::<Str>::nil()));
        let a28 = Str::lit("");
        join(a27, a28)
    };
    let v5: List<Str> = f10(match v2.clone() {
        Adt69::C0(_, _, _, _, _, h29, _) => {
            h29
        }
    });
    f8(v4.clone(), f9(v0, v1, v2, v4, v5))
}

pub fn f3(v0: Adt69, v1: Str) -> List<Str> {
    let m30 = f11(v0, v1);
    match m30 {
        None => {
            List::<Str>::nil()
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f4(v0: Adt69, v1: Adt70) -> Adt69 {
    let v2: List<Adt70> = match v0.clone() {
        Adt69::C0(_, _, _, _, _, h31, _) => {
            h31
        }
    };
    let v3: List<Adt70> = {
        let a32 = v2;
        let a33 = List::cons(v1, List::<Adt70>::nil());
        append_list(a32, a33)
    };
    Adt69::C0(match v0.clone() {
        Adt69::C0(h34, _, _, _, _, _, _) => {
            h34
        }
    }, match v0.clone() {
        Adt69::C0(_, h35, _, _, _, _, _) => {
            h35
        }
    }, match v0.clone() {
        Adt69::C0(_, _, h36, _, _, _, _) => {
            h36
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, h37, _, _, _) => {
            h37
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, h38, _, _) => {
            h38
        }
    }, v3, match v0 {
        Adt69::C0(_, _, _, _, _, _, h39) => {
            h39
        }
    })
}

pub fn f5(v0: Adt0) -> Str {
    let m40 = v0;
    match m40 {
        Adt0::C0 => {
            Str::lit("Unit")
        }
        Adt0::C1 => {
            Str::lit("Bool")
        }
        Adt0::C2(_) => {
            Str::lit("Nat")
        }
        Adt0::C3(_) => {
            Str::lit("Int64")
        }
        Adt0::C4(_) => {
            Str::lit("Nat")
        }
        Adt0::C5(_) => {
            Str::lit("Nat")
        }
        Adt0::C6 => {
            Str::lit("String")
        }
        Adt0::C7 => {
            Str::lit("String")
        }
        Adt0::C8 => {
            Str::lit("Nat")
        }
        Adt0::C9 => {
            let a41 = List::cons(f12(), List::cons(Str::lit("::ParseIntError"), List::<Str>::nil()));
            let a42 = Str::lit("");
            join(a41, a42)
        }
        Adt0::C10 => {
            let a43 = List::cons(f12(), List::cons(Str::lit("::FilePath"), List::<Str>::nil()));
            let a44 = Str::lit("");
            join(a43, a44)
        }
        Adt0::C11(v5) => {
            v5
        }
        Adt0::C12(r67) => {
            let v6 = (*r67).clone();
            let a45 = List::cons(Str::lit("(Option "), List::cons(f5(v6), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a46 = Str::lit("");
            join(a45, a46)
        }
        Adt0::C13(r68, r69) => {
            let v7 = (*r68).clone();
            let v8 = (*r69).clone();
            let a47 = List::cons(Str::lit("(Result "), List::cons(f5(v7), List::cons(Str::lit(" "), List::cons(f5(v8), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a48 = Str::lit("");
            join(a47, a48)
        }
        Adt0::C14(r70) => {
            let v9 = (*r70).clone();
            let a49 = List::cons(Str::lit("(List "), List::cons(f5(v9), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a50 = Str::lit("");
            join(a49, a50)
        }
        Adt0::C15(v10) => {
            let a53 = List::cons(Str::lit("Tuple"), List::cons({
                let a51 = f13(v10);
                let a52 = Str::lit("");
                join(a51, a52)
            }, List::<Str>::nil()));
            let a54 = Str::lit("");
            join(a53, a54)
        }
        Adt0::C16(v11) => {
            let a57 = List::cons(Str::lit("("), List::cons({
                let a55 = f14(v11);
                let a56 = Str::lit(" * ");
                join(a55, a56)
            }, List::cons(Str::lit(")"), List::<Str>::nil())));
            let a58 = Str::lit("");
            join(a57, a58)
        }
        Adt0::C17(r71, r72, _) => {
            let v12 = (*r71).clone();
            let v13 = (*r72).clone();
            let a59 = List::cons(Str::lit("(List Tuple"), List::cons(f15(v12), List::cons(f15(v13), List::cons(Str::lit(")"), List::<Str>::nil()))));
            let a60 = Str::lit("");
            join(a59, a60)
        }
        Adt0::C18(r73, _) => {
            let v15 = (*r73).clone();
            let a61 = List::cons(Str::lit("(List "), List::cons(f5(v15), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a62 = Str::lit("");
            join(a61, a62)
        }
        Adt0::C19(v17, r74) => {
            let v18 = (*r74).clone();
            let a65 = List::cons(Str::lit("(Fn ("), List::cons({
                let a63 = f16(v17);
                let a64 = Str::lit(" ");
                join(a63, a64)
            }, List::cons(Str::lit(") "), List::cons(f5(v18), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a66 = Str::lit("");
            join(a65, a66)
        }
    }
}

pub fn f6(p77: List<Str>, p78: List<Str>) -> List<Str> {
    let m76 = (p77, p78);
    {
        let mut m76 = m76;
        loop {
            let step = {
                let (v0, v1) = m76;
                let m75 = v0;
                match m75.uncons() {
                    None => {
                        Err::<(List<Str>, List<Str>), List<Str>>(v1)
                    }
                    Some((v2, v3)) => {
                        Ok::<(List<Str>, List<Str>), List<Str>>((v3, f17(v1, v2)))
                    }
                }
            };
            match step {
                Ok(next) => { m76 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f7(v0: Adt69, v1: Str) -> Adt69 {
    let v2: List<Str> = match v0.clone() {
        Adt69::C0(_, h79, _, _, _, _, _) => {
            h79
        }
    };
    let v3: List<Str> = f18(v2, v1);
    Adt69::C0(match v0.clone() {
        Adt69::C0(h80, _, _, _, _, _, _) => {
            h80
        }
    }, v3, match v0.clone() {
        Adt69::C0(_, _, h81, _, _, _, _) => {
            h81
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, h82, _, _, _) => {
            h82
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, h83, _, _) => {
            h83
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, _, h84, _) => {
            h84
        }
    }, match v0 {
        Adt69::C0(_, _, _, _, _, _, h85) => {
            h85
        }
    })
}

pub fn f8(v0: Str, v1: List<Str>) -> R<Str> {
    let m91 = f19(f20(f21(1u64, {
        let a89 = {
            let a87 = {
                let a86 = v1.clone();
                length_list(a86)
            };
            let a88 = 2u64;
            nat_add(a87, a88)?
        };
        let a90 = 1u64;
        nat_sub(a89, a90)
    })?, v0)?, v1);
    match m91 {
        None => {
            Ok(Str::lit(""))
        }
        Some(v2) => {
            Ok(v2)
        }
    }
}

pub fn f9(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str, v4: List<Str>) -> List<Str> {
    f22({
        let a105 = {
            let a103 = {
                let a100 = {
                    let a97 = {
                        let a94 = f23(f24(match v0.clone() {
                            Adt16::C0(_, _, _, _, h92, _, _, _, _, _, _, _, _, _, _, _, _) => {
                                h92
                            }
                        }));
                        let a95 = match v0 {
                            Adt16::C0(_, _, _, _, _, _, _, _, h93, _, _, _, _, _, _, _, _) => {
                                h93
                            }
                        };
                        append_list(a94, a95)
                    };
                    let a98 = f25(f26(match v1.clone() {
                        Adt2::C0(_, h96, _, _) => {
                            h96
                        }
                    }));
                    append_list(a97, a98)
                };
                let a101 = match v1 {
                    Adt2::C0(_, _, _, h99) => {
                        h99
                    }
                };
                append_list(a100, a101)
            };
            let a104 = match v2 {
                Adt69::C0(_, _, h102, _, _, _, _) => {
                    h102
                }
            };
            append_list(a103, a104)
        };
        let a106 = v4;
        append_list(a105, a106)
    }, v3)
}

pub fn f10(v0: List<Adt70>) -> List<Str> {
    let m107 = v0;
    match m107.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt70::C0(h108, _, _, _, _, _, _, _, _) => {
                    h108
                }
            }, f10(v2))
        }
    }
}

pub fn f11(v0: Adt69, v1: Str) -> Option<List<Str>> {
    let m110 = f27(match v0 {
        Adt69::C0(_, _, _, _, _, h109, _) => {
            h109
        }
    }, v1);
    match m110 {
        None => {
            None::<List<Str>>
        }
        Some(v2) => {
            Some(match v2 {
                Adt70::C0(_, _, _, _, _, h111, _, _, _) => {
                    h111
                }
            })
        }
    }
}

pub fn f12() -> Str {
    Str::lit("Prelude")
}

pub fn f13(v0: List<Adt0>) -> List<Str> {
    let m112 = v0;
    match m112.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f13(v2))
        }
    }
}

pub fn f14(v0: List<Adt0>) -> List<Str> {
    let m113 = v0;
    match m113.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f5(v1), f14(v2))
        }
    }
}

pub fn f15(v0: Adt0) -> Str {
    let m114 = v0;
    match m114 {
        Adt0::C0 => {
            Str::lit("Unit")
        }
        Adt0::C1 => {
            Str::lit("Bool")
        }
        Adt0::C2(_) => {
            Str::lit("Nat")
        }
        Adt0::C3(_) => {
            Str::lit("Int")
        }
        Adt0::C4(_) => {
            Str::lit("Nat")
        }
        Adt0::C5(_) => {
            Str::lit("Nat")
        }
        Adt0::C6 => {
            Str::lit("String")
        }
        Adt0::C7 => {
            Str::lit("String")
        }
        Adt0::C8 => {
            Str::lit("Nat")
        }
        Adt0::C9 => {
            Str::lit("ParseIntError")
        }
        Adt0::C10 => {
            Str::lit("FilePath")
        }
        Adt0::C11(v5) => {
            v5
        }
        Adt0::C12(r137) => {
            let v6 = (*r137).clone();
            let a115 = List::cons(Str::lit("Option"), List::cons(f15(v6), List::<Str>::nil()));
            let a116 = Str::lit("");
            join(a115, a116)
        }
        Adt0::C13(r138, r139) => {
            let v7 = (*r138).clone();
            let v8 = (*r139).clone();
            let a117 = List::cons(Str::lit("Result"), List::cons(f15(v7), List::cons(f15(v8), List::<Str>::nil())));
            let a118 = Str::lit("");
            join(a117, a118)
        }
        Adt0::C14(r140) => {
            let v9 = (*r140).clone();
            let a119 = List::cons(Str::lit("List"), List::cons(f15(v9), List::<Str>::nil()));
            let a120 = Str::lit("");
            join(a119, a120)
        }
        Adt0::C15(v10) => {
            let a123 = List::cons(Str::lit("Tuple"), List::cons({
                let a121 = f28(v10);
                let a122 = Str::lit("");
                join(a121, a122)
            }, List::<Str>::nil()));
            let a124 = Str::lit("");
            join(a123, a124)
        }
        Adt0::C16(v11) => {
            let a127 = List::cons(Str::lit("Pair"), List::cons({
                let a125 = f29(v11);
                let a126 = Str::lit("");
                join(a125, a126)
            }, List::<Str>::nil()));
            let a128 = Str::lit("");
            join(a127, a128)
        }
        Adt0::C17(r141, r142, _) => {
            let v12 = (*r141).clone();
            let v13 = (*r142).clone();
            let a129 = List::cons(Str::lit("ListTuple"), List::cons(f15(v12), List::cons(f15(v13), List::<Str>::nil())));
            let a130 = Str::lit("");
            join(a129, a130)
        }
        Adt0::C18(r143, _) => {
            let v15 = (*r143).clone();
            let a131 = List::cons(Str::lit("List"), List::cons(f15(v15), List::<Str>::nil()));
            let a132 = Str::lit("");
            join(a131, a132)
        }
        Adt0::C19(v17, r144) => {
            let v18 = (*r144).clone();
            let a135 = List::cons(Str::lit("Fn"), List::cons({
                let a133 = f30(v17);
                let a134 = Str::lit("");
                join(a133, a134)
            }, List::cons(Str::lit("To"), List::cons(f15(v18), List::<Str>::nil()))));
            let a136 = Str::lit("");
            join(a135, a136)
        }
    }
}

pub fn f16(v0: List<Adt0>) -> List<Str> {
    let m145 = v0;
    match m145.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f5(v1), f16(v2))
        }
    }
}

pub fn f17(v0: List<Str>, v1: Str) -> List<Str> {
    let m146 = v0;
    match m146.uncons() {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some((v2, v3)) => {
            let a147 = v1.clone();
            let a148 = v2.clone();
            if equal(a147, a148) {
                List::cons(v2, v3)
            } else {
                let m151 = {
                    let a149 = v1.clone();
                    let a150 = v2.clone();
                    compare(a149, a150)
                };
                if match m151 {
                    Ordering::Less => {
                        true
                    }
                    Ordering::Equal => {
                        false
                    }
                    Ordering::Greater => {
                        false
                    }
                } {
                    List::cons(v1, List::cons(v2, v3))
                } else {
                    List::cons(v2, f17(v3, v1))
                }
            }
        }
    }
}

pub fn f18(v0: List<Str>, v1: Str) -> List<Str> {
    let m152 = v0;
    match m152.uncons() {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some((v2, v3)) => {
            let a153 = v1.clone();
            let a154 = v2.clone();
            if equal(a153, a154) {
                List::cons(v2, v3)
            } else {
                let m157 = {
                    let a155 = v1.clone();
                    let a156 = v2.clone();
                    compare(a155, a156)
                };
                if match m157 {
                    Ordering::Less => {
                        true
                    }
                    Ordering::Equal => {
                        false
                    }
                    Ordering::Greater => {
                        false
                    }
                } {
                    List::cons(v1, List::cons(v2, v3))
                } else {
                    List::cons(v2, f18(v3, v1))
                }
            }
        }
    }
}

pub fn f19(p161: List<Str>, v1: List<Str>) -> Option<Str> {
    let m160 = p161;
    {
        let mut m160 = m160;
        loop {
            let step = {
                let v0 = m160;
                let m158 = v0;
                match m158.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a159 = f31(v1.clone(), v2.clone());
                        if bool_not(a159) {
                            Err::<List<Str>, Option<Str>>(Some(v2))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m160 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f20(v0: List<u64>, v1: Str) -> R<List<Str>> {
    let m162 = v0;
    match m162.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons({
                let a163 = List::cons(v1.clone(), List::cons(f32(v2)?, List::<Str>::nil()));
                let a164 = Str::lit("");
                join(a163, a164)
            }, f20(v3, v1)?))
        }
    }
}

pub fn f21(v0: u64, v1: u64) -> R<List<u64>> {
    let m165 = v1;
    if m165 == 0 {
        Ok(List::<u64>::nil())
    } else {
        let v2 = m165 - 1;
        Ok(List::cons(v0, f21({
            let a166 = v0;
            let a167 = 1u64;
            nat_add(a166, a167)?
        }, v2)?))
    }
}

pub fn f22(v0: List<Str>, v1: Str) -> List<Str> {
    let m168 = v0;
    match m168.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            if f33(v2.clone(), v1.clone()) {
                List::cons(v2, f22(v3, v1))
            } else {
                f22(v3, v1)
            }
        }
    }
}

pub fn f23(v0: List<(Str, Adt0)>) -> List<Str> {
    let m169 = v0;
    match m169.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            let v3: Str = {
                let (h170, _) = v1;
                h170
            };
            let _: Adt0 = {
                let (_, h171) = v1;
                h171
            };
            List::cons(v3, f23(v2))
        }
    }
}

pub fn f24(v0: List<Adt17>) -> List<(Str, Adt0)> {
    let m172 = v0;
    match m172.uncons() {
        None => {
            List::<(Str, Adt0)>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt17::C0(_, h173) => {
                    h173
                }
            }, f24(v2))
        }
    }
}

pub fn f25(v0: List<Adt14>) -> List<Str> {
    let m174 = v0;
    match m174.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt14::C0(h175, _, _) => {
                    h175
                }
            }, f25(v2))
        }
    }
}

pub fn f26(v0: List<Adt13>) -> List<Adt14> {
    let m176 = v0;
    match m176.uncons() {
        None => {
            List::<Adt14>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt13::C0(_, h177) => {
                    h177
                }
            }, f26(v2))
        }
    }
}

pub fn f27(p183: List<Adt70>, v1: Str) -> Option<Adt70> {
    let m182 = p183;
    {
        let mut m182 = m182;
        loop {
            let step = {
                let v0 = m182;
                let m178 = v0;
                match m178.uncons() {
                    None => {
                        Err::<List<Adt70>, Option<Adt70>>(None::<Adt70>)
                    }
                    Some((v2, v3)) => {
                        let a180 = match v2.clone() {
                            Adt70::C0(h179, _, _, _, _, _, _, _, _) => {
                                h179
                            }
                        };
                        let a181 = v1.clone();
                        if equal(a180, a181) {
                            Err::<List<Adt70>, Option<Adt70>>(Some(v2))
                        } else {
                            Ok::<List<Adt70>, Option<Adt70>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m182 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f28(v0: List<Adt0>) -> List<Str> {
    let m184 = v0;
    match m184.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f28(v2))
        }
    }
}

pub fn f29(v0: List<Adt0>) -> List<Str> {
    let m185 = v0;
    match m185.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f29(v2))
        }
    }
}

pub fn f30(v0: List<Adt0>) -> List<Str> {
    let m186 = v0;
    match m186.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f30(v2))
        }
    }
}

pub fn f31(p191: List<Str>, v1: Str) -> bool {
    let m190 = p191;
    {
        let mut m190 = m190;
        loop {
            let step = {
                let v0 = m190;
                let m187 = v0;
                match m187.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a188 = v2;
                        let a189 = v1.clone();
                        if equal(a188, a189) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m190 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f32(v0: u64) -> R<Str> {
    Ok(f34({
        let a192 = v0;
        let a193 = 1u64;
        nat_add(a192, a193)?
    }, v0))
}

pub fn f33(v0: Str, v1: Str) -> bool {
    f35({
        let a194 = v0;
        utf8_encode(a194)
    }, 0u64, {
        let a195 = v1;
        utf8_encode(a195)
    })
}

pub fn f34(v0: u64, v1: u64) -> Str {
    let m196 = v0;
    if m196 == 0 {
        Str::lit("")
    } else {
        let v2 = m196 - 1;
        let a197 = v1;
        let a198 = 10u64;
        if nat_lt(a197, a198) {
            f36(v1)
        } else {
            let a205 = List::cons(f34(v2, {
                let a199 = v1;
                let a200 = 10u64;
                let a201 = 0u64;
                nat_quot(a199, a200, a201)
            }), List::cons(f36({
                let a202 = v1;
                let a203 = 10u64;
                let a204 = 0u64;
                nat_rem(a202, a203, a204)
            }), List::<Str>::nil()));
            let a206 = Str::lit("");
            join(a205, a206)
        }
    }
}

pub fn f35(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m211 = {
        let a208 = v0;
        let a209 = v1;
        let a210 = {
            let a207 = v2.clone();
            length_bytes(a207)
        };
        slice_bytes(a208, a209, a210)
    };
    match m211 {
        None => {
            false
        }
        Some(v3) => {
            let a212 = v3;
            let a213 = v2;
            equal(a212, a213)
        }
    }
}

pub fn f36(v0: u64) -> Str {
    let a214 = v0;
    let a215 = 0u64;
    if nat_eq(a214, a215) {
        Str::lit("0")
    } else {
        let a216 = v0;
        let a217 = 1u64;
        if nat_eq(a216, a217) {
            Str::lit("1")
        } else {
            let a218 = v0;
            let a219 = 2u64;
            if nat_eq(a218, a219) {
                Str::lit("2")
            } else {
                let a220 = v0;
                let a221 = 3u64;
                if nat_eq(a220, a221) {
                    Str::lit("3")
                } else {
                    let a222 = v0;
                    let a223 = 4u64;
                    if nat_eq(a222, a223) {
                        Str::lit("4")
                    } else {
                        let a224 = v0;
                        let a225 = 5u64;
                        if nat_eq(a224, a225) {
                            Str::lit("5")
                        } else {
                            let a226 = v0;
                            let a227 = 6u64;
                            if nat_eq(a226, a227) {
                                Str::lit("6")
                            } else {
                                let a228 = v0;
                                let a229 = 7u64;
                                if nat_eq(a228, a229) {
                                    Str::lit("7")
                                } else {
                                    let a230 = v0;
                                    let a231 = 8u64;
                                    if nat_eq(a230, a231) {
                                        Str::lit("8")
                                    } else {
                                        Str::lit("9")
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Adt0, p2: Str, p3: Adt2, p4: Adt16, p5: Adt69) -> R<Adt74> {
    f0(p0, p1, p2, p3, p4, p5)
}
