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
    C0(Adt1, List<Adt13>, List<Adt15>, List<Str>),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(List<Adt2>, List<Adt3>, List<Adt6>, List<Adt7>, List<List<Str>>, List<Adt2>, List<Adt2>, bool, List<Adt8>, List<Adt9>, List<Adt10>, List<Adt12>),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt3 {
    C0(Str, List<(Str, Adt4)>),
}

#[derive(Clone)]
pub enum Adt4 {
    C0,
    C1,
    C2(Option<u64>),
    C3(u64),
    C4(Adt5),
    C5(Option<u64>),
    C6,
    C7,
    C8,
    C9,
    C10,
    C11(Str),
    C12(std::rc::Rc<Adt4>),
    C13(std::rc::Rc<Adt4>, std::rc::Rc<Adt4>),
    C14(std::rc::Rc<Adt4>),
    C15(List<Adt4>),
    C16(List<Adt4>),
    C17(std::rc::Rc<Adt4>, std::rc::Rc<Adt4>, bool),
    C18(std::rc::Rc<Adt4>, bool),
    C19(List<Adt4>, std::rc::Rc<Adt4>),
}

#[derive(Clone)]
pub enum Adt5 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt6 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(Str, List<(Str, List<(Option<Str>, Adt4)>)>),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str, Adt5),
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
    C0(Str, List<Adt4>, Adt4),
}

#[derive(Clone)]
pub enum Adt15 {
    C0(u64, List<Str>),
}

#[derive(Clone)]
pub enum Adt16 {
    C0(Str, List<Str>, Adt4, Str, List<Adt17>, List<Adt6>, bool, Option<List<Str>>, List<Str>, List<Adt6>, List<Adt18>, Option<Str>, List<Adt6>, List<Adt68>, List<Adt68>, Option<Adt4>, Option<Str>),
}

#[derive(Clone)]
pub enum Adt17 {
    C0(Str, (Str, Adt4)),
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
    C0(List<u64>, List<Str>, List<Str>, List<Adt4>, List<Str>, List<Adt70>, List<Adt4>),
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
    C2(Str, List<Adt2>, Option<Str>),
    C3(Str, List<Adt6>, Str, Adt71, List<Str>, Option<Str>, Option<Adt2>, Option<Str>),
    C4(Str, List<Adt6>, Str, Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt74 {
    C0(Str),
    C1(Str),
    C2,
    C3(Str),
    C4(Str, List<Adt15>),
    C5(Str),
    C6(Str),
    C7(Str),
    C8(Str),
    C9(Str),
    C10(Adt75),
}

#[derive(Clone)]
pub enum Adt75 {
    C0(Str, Str),
    C1(Str),
    C2(Str),
    C3,
    C4(Str),
    C5(Str, Str),
    C6(Str, List<Adt76>),
}

#[derive(Clone)]
pub enum Adt76 {
    C0(Str, Adt77),
}

#[derive(Clone)]
pub enum Adt77 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt78 {
    C0(Str, Adt69),
}

pub fn f0(v0: Str, v1: Str, v2: Adt0, v3: Adt16, v4: Adt69) -> R<Result<((Adt71, Adt4), Adt69), Adt74>> {
    let v5: Adt78 = f1(v3, v2, v4, Str::lit("value"))?;
    let v6: Str = match v5.clone() {
        Adt78::C0(h1, _) => {
            h1
        }
    };
    let v7: Adt69 = match v5 {
        Adt78::C0(_, h2) => {
            h2
        }
    };
    let v8: Adt6 = {
        let a3 = v0;
        let a4 = Str::lit("is_some");
        if equal(a3, a4) {
            Adt6::C0(Str::lit("false"), Str::lit("true"))
        } else {
            Adt6::C0(Str::lit("true"), Str::lit("false"))
        }
    };
    let v9: Str = match v8.clone() {
        Adt6::C0(h5, _) => {
            h5
        }
    };
    let v10: Str = match v8 {
        Adt6::C0(_, h6) => {
            h6
        }
    };
    Ok(Ok::<((Adt71, Adt4), Adt69), Adt74>(((Adt71::C1(v1, List::cons(Adt72::C0(Str::lit("Option.none"), Adt71::C0(v9)), List::cons(Adt72::C0({
        let a7 = List::cons(Str::lit("Option.some "), List::cons(v6, List::<Str>::nil()));
        let a8 = Str::lit("");
        join(a7, a8)
    }, Adt71::C0(v10)), List::<Adt72>::nil()))), Adt4::C1), v7)))
}

pub fn f1(v0: Adt16, v1: Adt0, v2: Adt69, v3: Str) -> R<Adt78> {
    let v4: Str = f2(v3.clone(), f3(v0, v1, v2.clone(), v3, List::<Str>::nil()))?;
    let v5: List<Str> = match v2.clone() {
        Adt69::C0(_, _, h9, _, _, _, _) => {
            h9
        }
    };
    let v6: List<Str> = {
        let a10 = v5;
        let a11 = List::cons(v4.clone(), List::<Str>::nil());
        append_list(a10, a11)
    };
    Ok(Adt78::C0(v4, Adt69::C0(match v2.clone() {
        Adt69::C0(h12, _, _, _, _, _, _) => {
            h12
        }
    }, match v2.clone() {
        Adt69::C0(_, h13, _, _, _, _, _) => {
            h13
        }
    }, v6, match v2.clone() {
        Adt69::C0(_, _, _, h14, _, _, _) => {
            h14
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, h15, _, _) => {
            h15
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, _, h16, _) => {
            h16
        }
    }, match v2 {
        Adt69::C0(_, _, _, _, _, _, h17) => {
            h17
        }
    })))
}

pub fn f2(v0: Str, v1: List<Str>) -> R<Str> {
    let m23 = f4(f5(f6(1u64, {
        let a21 = {
            let a19 = {
                let a18 = v1.clone();
                length_list(a18)
            };
            let a20 = 2u64;
            nat_add(a19, a20)?
        };
        let a22 = 1u64;
        nat_sub(a21, a22)
    })?, v0)?, v1);
    match m23 {
        None => {
            Ok(Str::lit(""))
        }
        Some(v2) => {
            Ok(v2)
        }
    }
}

pub fn f3(v0: Adt16, v1: Adt0, v2: Adt69, v3: Str, v4: List<Str>) -> List<Str> {
    f7({
        let a37 = {
            let a35 = {
                let a32 = {
                    let a29 = {
                        let a26 = f8(f9(match v0.clone() {
                            Adt16::C0(_, _, _, _, h24, _, _, _, _, _, _, _, _, _, _, _, _) => {
                                h24
                            }
                        }));
                        let a27 = match v0 {
                            Adt16::C0(_, _, _, _, _, _, _, _, h25, _, _, _, _, _, _, _, _) => {
                                h25
                            }
                        };
                        append_list(a26, a27)
                    };
                    let a30 = f10(f11(match v1.clone() {
                        Adt0::C0(_, h28, _, _) => {
                            h28
                        }
                    }));
                    append_list(a29, a30)
                };
                let a33 = match v1 {
                    Adt0::C0(_, _, _, h31) => {
                        h31
                    }
                };
                append_list(a32, a33)
            };
            let a36 = match v2 {
                Adt69::C0(_, _, h34, _, _, _, _) => {
                    h34
                }
            };
            append_list(a35, a36)
        };
        let a38 = v4;
        append_list(a37, a38)
    }, v3)
}

pub fn f4(p42: List<Str>, v1: List<Str>) -> Option<Str> {
    let m41 = p42;
    {
        let mut m41 = m41;
        loop {
            let step = {
                let v0 = m41;
                let m39 = v0;
                match m39.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a40 = f12(v1.clone(), v2.clone());
                        if bool_not(a40) {
                            Err::<List<Str>, Option<Str>>(Some(v2))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m41 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f5(v0: List<u64>, v1: Str) -> R<List<Str>> {
    let m43 = v0;
    match m43.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons({
                let a44 = List::cons(v1.clone(), List::cons(f13(v2)?, List::<Str>::nil()));
                let a45 = Str::lit("");
                join(a44, a45)
            }, f5(v3, v1)?))
        }
    }
}

pub fn f6(v0: u64, v1: u64) -> R<List<u64>> {
    let m46 = v1;
    if m46 == 0 {
        Ok(List::<u64>::nil())
    } else {
        let v2 = m46 - 1;
        Ok(List::cons(v0, f6({
            let a47 = v0;
            let a48 = 1u64;
            nat_add(a47, a48)?
        }, v2)?))
    }
}

pub fn f7(v0: List<Str>, v1: Str) -> List<Str> {
    let m49 = v0;
    match m49.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            if f14(v2.clone(), v1.clone()) {
                List::cons(v2, f7(v3, v1))
            } else {
                f7(v3, v1)
            }
        }
    }
}

pub fn f8(v0: List<(Str, Adt4)>) -> List<Str> {
    let m50 = v0;
    match m50.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            let v3: Str = {
                let (h51, _) = v1;
                h51
            };
            let _: Adt4 = {
                let (_, h52) = v1;
                h52
            };
            List::cons(v3, f8(v2))
        }
    }
}

pub fn f9(v0: List<Adt17>) -> List<(Str, Adt4)> {
    let m53 = v0;
    match m53.uncons() {
        None => {
            List::<(Str, Adt4)>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt17::C0(_, h54) => {
                    h54
                }
            }, f9(v2))
        }
    }
}

pub fn f10(v0: List<Adt14>) -> List<Str> {
    let m55 = v0;
    match m55.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt14::C0(h56, _, _) => {
                    h56
                }
            }, f10(v2))
        }
    }
}

pub fn f11(v0: List<Adt13>) -> List<Adt14> {
    let m57 = v0;
    match m57.uncons() {
        None => {
            List::<Adt14>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt13::C0(_, h58) => {
                    h58
                }
            }, f11(v2))
        }
    }
}

pub fn f12(p63: List<Str>, v1: Str) -> bool {
    let m62 = p63;
    {
        let mut m62 = m62;
        loop {
            let step = {
                let v0 = m62;
                let m59 = v0;
                match m59.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a60 = v2;
                        let a61 = v1.clone();
                        if equal(a60, a61) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m62 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f13(v0: u64) -> R<Str> {
    Ok(f15({
        let a64 = v0;
        let a65 = 1u64;
        nat_add(a64, a65)?
    }, v0))
}

pub fn f14(v0: Str, v1: Str) -> bool {
    f16({
        let a66 = v0;
        utf8_encode(a66)
    }, 0u64, {
        let a67 = v1;
        utf8_encode(a67)
    })
}

pub fn f15(v0: u64, v1: u64) -> Str {
    let m68 = v0;
    if m68 == 0 {
        Str::lit("")
    } else {
        let v2 = m68 - 1;
        let a69 = v1;
        let a70 = 10u64;
        if nat_lt(a69, a70) {
            f17(v1)
        } else {
            let a77 = List::cons(f15(v2, {
                let a71 = v1;
                let a72 = 10u64;
                let a73 = 0u64;
                nat_quot(a71, a72, a73)
            }), List::cons(f17({
                let a74 = v1;
                let a75 = 10u64;
                let a76 = 0u64;
                nat_rem(a74, a75, a76)
            }), List::<Str>::nil()));
            let a78 = Str::lit("");
            join(a77, a78)
        }
    }
}

pub fn f16(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m83 = {
        let a80 = v0;
        let a81 = v1;
        let a82 = {
            let a79 = v2.clone();
            length_bytes(a79)
        };
        slice_bytes(a80, a81, a82)
    };
    match m83 {
        None => {
            false
        }
        Some(v3) => {
            let a84 = v3;
            let a85 = v2;
            equal(a84, a85)
        }
    }
}

pub fn f17(v0: u64) -> Str {
    let a86 = v0;
    let a87 = 0u64;
    if nat_eq(a86, a87) {
        Str::lit("0")
    } else {
        let a88 = v0;
        let a89 = 1u64;
        if nat_eq(a88, a89) {
            Str::lit("1")
        } else {
            let a90 = v0;
            let a91 = 2u64;
            if nat_eq(a90, a91) {
                Str::lit("2")
            } else {
                let a92 = v0;
                let a93 = 3u64;
                if nat_eq(a92, a93) {
                    Str::lit("3")
                } else {
                    let a94 = v0;
                    let a95 = 4u64;
                    if nat_eq(a94, a95) {
                        Str::lit("4")
                    } else {
                        let a96 = v0;
                        let a97 = 5u64;
                        if nat_eq(a96, a97) {
                            Str::lit("5")
                        } else {
                            let a98 = v0;
                            let a99 = 6u64;
                            if nat_eq(a98, a99) {
                                Str::lit("6")
                            } else {
                                let a100 = v0;
                                let a101 = 7u64;
                                if nat_eq(a100, a101) {
                                    Str::lit("7")
                                } else {
                                    let a102 = v0;
                                    let a103 = 8u64;
                                    if nat_eq(a102, a103) {
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

pub fn run(p0: Str, p1: Str, p2: Adt0, p3: Adt16, p4: Adt69) -> R<Result<((Adt71, Adt4), Adt69), Adt74>> {
    f0(p0, p1, p2, p3, p4)
}
