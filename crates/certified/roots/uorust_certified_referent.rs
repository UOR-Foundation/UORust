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
    C0(Str, Str),
}

pub fn f0(v0: Str) -> R<Option<Str>> {
    let v1: Str = f1(v0)?;
    let m1 = f2(v1.clone(), Str::lit("&"));
    match m1 {
        None => {
            let m2 = f3(v1);
            match m2 {
                None => {
                    Ok(None::<Str>)
                }
                Some(v2) => {
                    Ok(Some(v2))
                }
            }
        }
        Some(v3) => {
            let v4: Str = f4(v3)?;
            let v5: Str = f5(v4)?;
            Ok(Some(f6(v5)))
        }
    }
}

pub fn f1(v0: Str) -> R<Str> {
    f7(f4(v0)?)
}

pub fn f2(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a3 = v0;
        utf8_encode(a3)
    };
    let v3: Bytes = {
        let a4 = v1;
        utf8_encode(a4)
    };
    if f8(v2.clone(), 0u64, v3.clone()) {
        Some(f9(v2.clone(), {
            let a5 = v3;
            length_bytes(a5)
        }, {
            let a6 = v2;
            length_bytes(a6)
        }))
    } else {
        None::<Str>
    }
}

pub fn f3(v0: Str) -> Option<Str> {
    let m7 = f10(v0);
    match m7 {
        None => {
            None::<Str>
        }
        Some(v1) => {
            f11(v1, Str::lit(">"))
        }
    }
}

pub fn f4(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a8 = v0;
        utf8_encode(a8)
    };
    let v2: u64 = {
        let a9 = v1.clone();
        length_bytes(a9)
    };
    Ok(f9(v1.clone(), f12(v2, v1, 0u64)?, v2))
}

pub fn f5(v0: Str) -> R<Str> {
    let m10 = f2(v0.clone(), Str::lit("'"));
    match m10 {
        None => {
            Ok(v0)
        }
        Some(v1) => {
            let m11 = f13(v1)?;
            match m11 {
                None => {
                    Ok(Str::lit(""))
                }
                Some(v2) => {
                    let _: Str = match v2.clone() {
                        Adt0::C0(h12, _) => {
                            h12
                        }
                    };
                    let v4: Str = match v2 {
                        Adt0::C0(_, h13) => {
                            h13
                        }
                    };
                    Ok(v4)
                }
            }
        }
    }
}

pub fn f6(v0: Str) -> Str {
    let m14 = f2(v0.clone(), Str::lit("mut "));
    match m14 {
        None => {
            v0
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f7(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a15 = v0;
        utf8_encode(a15)
    };
    let v2: u64 = {
        let a16 = v1.clone();
        length_bytes(a16)
    };
    Ok(f9(v1.clone(), 0u64, f14(v2, v1, v2)?))
}

pub fn f8(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m21 = {
        let a18 = v0;
        let a19 = v1;
        let a20 = {
            let a17 = v2.clone();
            length_bytes(a17)
        };
        slice_bytes(a18, a19, a20)
    };
    match m21 {
        None => {
            false
        }
        Some(v3) => {
            let a22 = v3;
            let a23 = v2;
            equal(a22, a23)
        }
    }
}

pub fn f9(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m29 = {
        let a26 = v0;
        let a27 = v1;
        let a28 = {
            let a24 = v2;
            let a25 = v1;
            nat_sub(a24, a25)
        };
        slice_bytes(a26, a27, a28)
    };
    match m29 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m31 = {
                let a30 = v3;
                utf8_decode(a30)
            };
            match m31 {
                None => {
                    Str::lit("")
                }
                Some(v4) => {
                    v4
                }
            }
        }
    }
}

pub fn f10(v0: Str) -> Option<Str> {
    let m32 = f2(v0.clone(), Str::lit("std::boxed::Box<"));
    match m32 {
        None => {
            f2(v0, Str::lit("Box<"))
        }
        Some(v1) => {
            Some(v1)
        }
    }
}

pub fn f11(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a33 = v0.clone();
        utf8_encode(a33)
    };
    let v3: u64 = {
        let a34 = v2.clone();
        length_bytes(a34)
    };
    let v4: u64 = f15(v1.clone());
    if f16(v0, v1) {
        Some(f9(v2, 0u64, {
            let a35 = v3;
            let a36 = v4;
            nat_sub(a35, a36)
        }))
    } else {
        None::<Str>
    }
}

pub fn f12(p43: u64, v1: Bytes, p44: u64) -> R<u64> {
    let m42 = (p43, p44);
    {
        let mut m42 = m42;
        loop {
            let step = {
                let (v0, v2) = m42;
                let m37 = v0;
                if m37 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m37 - 1;
                    let v4: u64 = f17(v1.clone(), v2)?;
                    let a38 = v4;
                    let a39 = 0u64;
                    if nat_eq(a38, a39) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a40 = v2;
                            let a41 = v4;
                            nat_add(a40, a41)?
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m42 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f13(v0: Str) -> R<Option<Adt0>> {
    let m45 = f18(v0.clone(), Str::lit(" "))?;
    match m45 {
        None => {
            Ok(None::<Adt0>)
        }
        Some(v1) => {
            Ok(Some(Adt0::C0(f19(v0.clone(), 0u64, v1), f19(v0.clone(), {
                let a46 = v1;
                let a47 = f15(Str::lit(" "));
                nat_add(a46, a47)?
            }, f15(v0)))))
        }
    }
}

pub fn f14(p54: u64, v1: Bytes, p55: u64) -> R<u64> {
    let m53 = (p54, p55);
    {
        let mut m53 = m53;
        loop {
            let step = {
                let (v0, v2) = m53;
                let m48 = v0;
                if m48 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m48 - 1;
                    let v4: u64 = f20(v1.clone(), v2)?;
                    let a49 = v4;
                    let a50 = 0u64;
                    if nat_eq(a49, a50) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a51 = v2;
                            let a52 = v4;
                            nat_sub(a51, a52)
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m53 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f15(v0: Str) -> u64 {
    let a57 = {
        let a56 = v0;
        utf8_encode(a56)
    };
    length_bytes(a57)
}

pub fn f16(v0: Str, v1: Str) -> bool {
    let v2: Bytes = {
        let a58 = v0;
        utf8_encode(a58)
    };
    let v3: Bytes = {
        let a59 = v1;
        utf8_encode(a59)
    };
    let v4: u64 = {
        let a60 = v2.clone();
        length_bytes(a60)
    };
    let v5: u64 = {
        let a61 = v3.clone();
        length_bytes(a61)
    };
    let a62 = v5;
    let a63 = v4;
    if nat_le(a62, a63) {
        f8(v2, {
            let a64 = v4;
            let a65 = v5;
            nat_sub(a64, a65)
        }, v3)
    } else {
        false
    }
}

pub fn f17(v0: Bytes, v1: u64) -> R<u64> {
    let v2: u64 = f21(v0.clone(), v1)?;
    let v3: u64 = f21(v0.clone(), {
        let a66 = v1;
        let a67 = 1u64;
        nat_add(a66, a67)?
    })?;
    let v4: u64 = f21(v0, {
        let a68 = v1;
        let a69 = 2u64;
        nat_add(a68, a69)?
    })?;
    if f22(v2) {
        Ok(1u64)
    } else {
        let a70 = v2;
        let a71 = 194u64;
        if nat_eq(a70, a71) {
            let a72 = v3;
            let a73 = 133u64;
            if if nat_eq(a72, a73) {
                true
            } else {
                let a74 = v3;
                let a75 = 160u64;
                nat_eq(a74, a75)
            } {
                Ok(2u64)
            } else {
                Ok(0u64)
            }
        } else {
            let a76 = v2;
            let a77 = 225u64;
            if nat_eq(a76, a77) {
                let a78 = v3;
                let a79 = 154u64;
                if if nat_eq(a78, a79) {
                    let a80 = v4;
                    let a81 = 128u64;
                    nat_eq(a80, a81)
                } else {
                    false
                } {
                    Ok(3u64)
                } else {
                    Ok(0u64)
                }
            } else {
                let a82 = v2;
                let a83 = 226u64;
                if nat_eq(a82, a83) {
                    let a84 = v3;
                    let a85 = 128u64;
                    if nat_eq(a84, a85) {
                        let a86 = 128u64;
                        let a87 = v4;
                        if if if nat_le(a86, a87) {
                            let a88 = v4;
                            let a89 = 138u64;
                            nat_le(a88, a89)
                        } else {
                            false
                        } {
                            true
                        } else {
                            let a90 = v4;
                            let a91 = 168u64;
                            if nat_eq(a90, a91) {
                                true
                            } else {
                                let a92 = v4;
                                let a93 = 169u64;
                                if nat_eq(a92, a93) {
                                    true
                                } else {
                                    let a94 = v4;
                                    let a95 = 175u64;
                                    nat_eq(a94, a95)
                                }
                            }
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    } else {
                        let a96 = v3;
                        let a97 = 129u64;
                        if if nat_eq(a96, a97) {
                            let a98 = v4;
                            let a99 = 159u64;
                            nat_eq(a98, a99)
                        } else {
                            false
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    }
                } else {
                    let a100 = v2;
                    let a101 = 227u64;
                    if if nat_eq(a100, a101) {
                        let a102 = v3;
                        let a103 = 128u64;
                        if nat_eq(a102, a103) {
                            let a104 = v4;
                            let a105 = 128u64;
                            nat_eq(a104, a105)
                        } else {
                            false
                        }
                    } else {
                        false
                    } {
                        Ok(3u64)
                    } else {
                        Ok(0u64)
                    }
                }
            }
        }
    }
}

pub fn f18(v0: Str, v1: Str) -> R<Option<u64>> {
    let v2: Bytes = {
        let a106 = v0;
        utf8_encode(a106)
    };
    f23({
        let a108 = {
            let a107 = v2.clone();
            length_bytes(a107)
        };
        let a109 = 1u64;
        nat_add(a108, a109)?
    }, v2, {
        let a110 = v1;
        utf8_encode(a110)
    }, 0u64)
}

pub fn f19(v0: Str, v1: u64, v2: u64) -> Str {
    f9({
        let a111 = v0;
        utf8_encode(a111)
    }, v1, v2)
}

pub fn f20(v0: Bytes, v1: u64) -> R<u64> {
    let a112 = v1;
    let a113 = 0u64;
    if nat_eq(a112, a113) {
        Ok(0u64)
    } else {
        if f22(f21(v0.clone(), {
            let a114 = v1;
            let a115 = 1u64;
            nat_sub(a114, a115)
        })?) {
            Ok(1u64)
        } else {
            let a116 = 2u64;
            let a117 = v1;
            if if nat_le(a116, a117) {
                let a120 = f17(v0.clone(), {
                    let a118 = v1;
                    let a119 = 2u64;
                    nat_sub(a118, a119)
                })?;
                let a121 = 2u64;
                nat_eq(a120, a121)
            } else {
                false
            } {
                Ok(2u64)
            } else {
                let a122 = 3u64;
                let a123 = v1;
                if if nat_le(a122, a123) {
                    let a126 = f17(v0, {
                        let a124 = v1;
                        let a125 = 3u64;
                        nat_sub(a124, a125)
                    })?;
                    let a127 = 3u64;
                    nat_eq(a126, a127)
                } else {
                    false
                } {
                    Ok(3u64)
                } else {
                    Ok(0u64)
                }
            }
        }
    }
}

pub fn f21(v0: Bytes, v1: u64) -> R<u64> {
    let m130 = {
        let a128 = v0;
        let a129 = v1;
        index_bytes(a128, a129)
    };
    match m130 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a175 = {
                let a151 = {
                    let a139 = {
                        let a133 = {
                            let a131 = v3;
                            let a132 = 128u8;
                            fixed_u8::bit_and(a131, a132)
                        };
                        let a134 = 128u8;
                        if equal(a133, a134) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a140 = {
                        let a137 = {
                            let a135 = v3;
                            let a136 = 64u8;
                            fixed_u8::bit_and(a135, a136)
                        };
                        let a138 = 64u8;
                        if equal(a137, a138) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a139, a140)?
                };
                let a152 = {
                    let a149 = {
                        let a143 = {
                            let a141 = v3;
                            let a142 = 32u8;
                            fixed_u8::bit_and(a141, a142)
                        };
                        let a144 = 32u8;
                        if equal(a143, a144) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a150 = {
                        let a147 = {
                            let a145 = v3;
                            let a146 = 16u8;
                            fixed_u8::bit_and(a145, a146)
                        };
                        let a148 = 16u8;
                        if equal(a147, a148) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a149, a150)?
                };
                nat_add(a151, a152)?
            };
            let a176 = {
                let a173 = {
                    let a161 = {
                        let a155 = {
                            let a153 = v3;
                            let a154 = 8u8;
                            fixed_u8::bit_and(a153, a154)
                        };
                        let a156 = 8u8;
                        if equal(a155, a156) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a162 = {
                        let a159 = {
                            let a157 = v3;
                            let a158 = 4u8;
                            fixed_u8::bit_and(a157, a158)
                        };
                        let a160 = 4u8;
                        if equal(a159, a160) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a161, a162)?
                };
                let a174 = {
                    let a171 = {
                        let a165 = {
                            let a163 = v3;
                            let a164 = 2u8;
                            fixed_u8::bit_and(a163, a164)
                        };
                        let a166 = 2u8;
                        if equal(a165, a166) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a172 = {
                        let a169 = {
                            let a167 = v3;
                            let a168 = 1u8;
                            fixed_u8::bit_and(a167, a168)
                        };
                        let a170 = 1u8;
                        if equal(a169, a170) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a171, a172)?
                };
                nat_add(a173, a174)?
            };
            nat_add(a175, a176)
        }
    }
}

pub fn f22(v0: u64) -> bool {
    let a177 = v0;
    let a178 = 32u64;
    if nat_eq(a177, a178) {
        true
    } else {
        let a179 = 9u64;
        let a180 = v0;
        if nat_le(a179, a180) {
            let a181 = v0;
            let a182 = 13u64;
            nat_le(a181, a182)
        } else {
            false
        }
    }
}

pub fn f23(p193: u64, v1: Bytes, v2: Bytes, p194: u64) -> R<Option<u64>> {
    let m192 = (p193, p194);
    {
        let mut m192 = m192;
        loop {
            let step = {
                let (v0, v3) = m192;
                let m183 = v0;
                if m183 == 0 {
                    Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                } else {
                    let v4 = m183 - 1;
                    let a188 = {
                        let a184 = v1.clone();
                        length_bytes(a184)
                    };
                    let a189 = {
                        let a186 = v3;
                        let a187 = {
                            let a185 = v2.clone();
                            length_bytes(a185)
                        };
                        nat_add(a186, a187)?
                    };
                    if nat_lt(a188, a189) {
                        Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                    } else {
                        if f8(v1.clone(), v3, v2.clone()) {
                            Err::<(u64, u64), R<Option<u64>>>(Ok(Some(v3)))
                        } else {
                            Ok::<(u64, u64), R<Option<u64>>>((v4, {
                                let a190 = v3;
                                let a191 = 1u64;
                                nat_add(a190, a191)?
                            }))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m192 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Str) -> R<Option<Str>> {
    f0(p0)
}
