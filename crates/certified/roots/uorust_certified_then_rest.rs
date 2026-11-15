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
    C0(List<Adt0>),
    C1(std::rc::Rc<Adt1>),
    C2(std::rc::Rc<Adt2>),
    C3(Adt4),
    C4(std::rc::Rc<Adt28>),
    C5(std::rc::Rc<Adt29>),
    C6(std::rc::Rc<Adt30>),
    C7(std::rc::Rc<Adt31>),
    C8(bool),
    C9(std::rc::Rc<Adt32>),
    C10(std::rc::Rc<Adt33>),
    C11(std::rc::Rc<Adt34>),
    C12(std::rc::Rc<Adt35>),
    C13(std::rc::Rc<Adt36>),
    C14(Adt21),
    C15(Adt37),
    C16(std::rc::Rc<Adt25>),
    C17(std::rc::Rc<Adt38>),
    C18(std::rc::Rc<Adt40>),
    C19(std::rc::Rc<Adt0>),
    C20(Adt13),
    C21(std::rc::Rc<Adt41>),
    C22(std::rc::Rc<Option<Adt0>>),
    C23(std::rc::Rc<Adt42>),
    C24(std::rc::Rc<Adt0>),
    C25(List<Adt0>),
    C26(std::rc::Rc<Adt44>),
    C27(std::rc::Rc<Adt46>),
    C28(Adt47),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(std::rc::Rc<Adt0>, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(std::rc::Rc<Adt0>, Adt3, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt3 {
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
pub enum Adt4 {
    C0(bool, Adt5),
}

#[derive(Clone)]
pub enum Adt5 {
    C0(List<Adt6>),
}

#[derive(Clone)]
pub enum Adt6 {
    C0(Adt7),
    C1(Adt0, bool),
    C2(Adt25),
    C3,
}

#[derive(Clone)]
pub enum Adt7 {
    C0(Adt8, Option<Adt24>),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(std::rc::Rc<Adt9>),
    C1,
    C2(std::rc::Rc<Adt8>),
    C3(std::rc::Rc<Adt10>),
    C4(List<Adt8>),
    C5(Adt17),
    C6(Adt18),
    C7(List<Adt8>),
    C8,
    C9(Adt21),
    C10(std::rc::Rc<Adt22>),
    C11(Adt13),
    C12(List<Adt8>),
    C13(std::rc::Rc<Adt23>),
    C14,
}

#[derive(Clone)]
pub enum Adt9 {
    C0(Str, bool, std::rc::Rc<Option<Adt8>>),
}

#[derive(Clone)]
pub enum Adt10 {
    C0(std::rc::Rc<Adt8>, Adt11),
}

#[derive(Clone)]
pub enum Adt11 {
    C0(Str, std::rc::Rc<Adt12>),
}

#[derive(Clone)]
pub enum Adt12 {
    C0(bool, std::rc::Rc<Adt11>),
    C1(std::rc::Rc<Adt11>),
    C2(std::rc::Rc<Adt11>, Str),
    C3(List<Adt11>, std::rc::Rc<Adt11>),
    C4(std::rc::Rc<Adt11>),
    C5(List<Adt11>),
    C6(bool, Adt13),
    C7,
}

#[derive(Clone)]
pub enum Adt13 {
    C0(List<Adt14>, Str),
}

#[derive(Clone)]
pub enum Adt14 {
    C0(Str, Adt15),
}

#[derive(Clone)]
pub enum Adt15 {
    C0,
    C1(List<Adt16>),
    C2,
}

#[derive(Clone)]
pub enum Adt16 {
    C0(Adt11),
    C1(Str),
}

#[derive(Clone)]
pub enum Adt17 {
    C0(Adt13, List<Adt8>),
}

#[derive(Clone)]
pub enum Adt18 {
    C0(Adt13, List<Adt19>, bool),
}

#[derive(Clone)]
pub enum Adt19 {
    C0(Adt20, Adt8),
}

#[derive(Clone)]
pub enum Adt20 {
    C0(Str),
    C1(u64),
}

#[derive(Clone)]
pub enum Adt21 {
    C0(bool),
    C1(Option<u64>, Str),
    C2(Str),
    C3(Str),
    C4(u64),
    C5(List<u64>),
    C6,
}

#[derive(Clone)]
pub enum Adt22 {
    C0(std::rc::Rc<Option<Adt0>>, std::rc::Rc<Option<Adt0>>, bool),
}

#[derive(Clone)]
pub enum Adt23 {
    C0(std::rc::Rc<Adt8>, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt24 {
    C0(Adt0, Option<Adt0>),
}

#[derive(Clone)]
pub enum Adt25 {
    C0(Adt13, Option<Adt26>, std::rc::Rc<Option<Adt27>>),
}

#[derive(Clone)]
pub enum Adt26 {
    C0(List<Adt0>),
}

#[derive(Clone)]
pub enum Adt27 {
    C0(std::rc::Rc<Adt0>, std::rc::Rc<Adt8>),
}

#[derive(Clone)]
pub enum Adt28 {
    C0(bool, std::rc::Rc<Option<Adt0>>),
}

#[derive(Clone)]
pub enum Adt29 {
    C0(std::rc::Rc<Adt0>, List<Adt0>),
}

#[derive(Clone)]
pub enum Adt30 {
    C0(std::rc::Rc<Adt0>, Adt11),
}

#[derive(Clone)]
pub enum Adt31 {
    C0(List<Adt8>, std::rc::Rc<Adt0>, bool),
}

#[derive(Clone)]
pub enum Adt32 {
    C0(std::rc::Rc<Adt0>, Adt20),
}

#[derive(Clone)]
pub enum Adt33 {
    C0(bool, std::rc::Rc<Adt8>, std::rc::Rc<Adt0>, Adt5),
}

#[derive(Clone)]
pub enum Adt34 {
    C0(std::rc::Rc<Adt0>, Adt5, std::rc::Rc<Option<Adt0>>),
}

#[derive(Clone)]
pub enum Adt35 {
    C0(std::rc::Rc<Adt0>, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt36 {
    C0(std::rc::Rc<Adt8>, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt37 {
    C0(bool, Adt5),
}

#[derive(Clone)]
pub enum Adt38 {
    C0(std::rc::Rc<Adt0>, List<Adt39>),
}

#[derive(Clone)]
pub enum Adt39 {
    C0(Adt8, Adt0),
}

#[derive(Clone)]
pub enum Adt40 {
    C0(std::rc::Rc<Adt0>, Str, Option<List<Adt16>>, List<Adt0>),
}

#[derive(Clone)]
pub enum Adt41 {
    C0(bool, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt42 {
    C0(bool, Adt13, List<Adt43>, std::rc::Rc<Option<Adt0>>),
}

#[derive(Clone)]
pub enum Adt43 {
    C0(Adt20, Adt0),
}

#[derive(Clone)]
pub enum Adt44 {
    C0(Adt45, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt45 {
    C0,
    C1,
    C2,
}

#[derive(Clone)]
pub enum Adt46 {
    C0(bool, std::rc::Rc<Adt0>, Adt5),
}

#[derive(Clone)]
pub enum Adt47 {
    C0(Str, List<Adt0>, List<Adt5>),
}

pub fn f0(v0: Adt0, v1: List<Adt6>) -> Adt0 {
    let v2: List<Adt6> = f1(v0);
    let v3: List<Adt6> = {
        let a1 = v2;
        let a2 = v1;
        append_list(a1, a2)
    };
    Adt0::C3(Adt4::C0(false, Adt5::C0(v3)))
}

pub fn f1(v0: Adt0) -> List<Adt6> {
    let m3 = v0;
    match m3 {
        Adt0::C0(v1) => {
            let v2: Adt0 = Adt0::C0(v1);
            List::cons(Adt6::C1(v2, true), List::<Adt6>::nil())
        }
        Adt0::C1(r8) => {
            let v3 = (*r8).clone();
            let v4: Adt0 = Adt0::C1(std::rc::Rc::new(v3));
            List::cons(Adt6::C1(v4, true), List::<Adt6>::nil())
        }
        Adt0::C2(r9) => {
            let v5 = (*r9).clone();
            let v6: Adt0 = Adt0::C2(std::rc::Rc::new(v5));
            List::cons(Adt6::C1(v6, true), List::<Adt6>::nil())
        }
        Adt0::C3(v7) => {
            let a4 = f2(v7.clone());
            if bool_not(a4) {
                f3(f4(v7))
            } else {
                let v8: Adt0 = Adt0::C3(v7);
                List::cons(Adt6::C1(v8, true), List::<Adt6>::nil())
            }
        }
        Adt0::C4(r10) => {
            let v9 = (*r10).clone();
            let v10: Adt0 = Adt0::C4(std::rc::Rc::new(v9));
            List::cons(Adt6::C1(v10, true), List::<Adt6>::nil())
        }
        Adt0::C5(r11) => {
            let v11 = (*r11).clone();
            let v12: Adt0 = Adt0::C5(std::rc::Rc::new(v11));
            List::cons(Adt6::C1(v12, true), List::<Adt6>::nil())
        }
        Adt0::C6(r12) => {
            let v13 = (*r12).clone();
            let v14: Adt0 = Adt0::C6(std::rc::Rc::new(v13));
            List::cons(Adt6::C1(v14, true), List::<Adt6>::nil())
        }
        Adt0::C7(r13) => {
            let v15 = (*r13).clone();
            let v16: Adt0 = Adt0::C7(std::rc::Rc::new(v15));
            List::cons(Adt6::C1(v16, true), List::<Adt6>::nil())
        }
        Adt0::C8(v17) => {
            let v18: Adt0 = Adt0::C8(v17);
            List::cons(Adt6::C1(v18, true), List::<Adt6>::nil())
        }
        Adt0::C9(r14) => {
            let v19 = (*r14).clone();
            let v20: Adt0 = Adt0::C9(std::rc::Rc::new(v19));
            List::cons(Adt6::C1(v20, true), List::<Adt6>::nil())
        }
        Adt0::C10(r15) => {
            let v21 = (*r15).clone();
            let v22: Adt0 = Adt0::C10(std::rc::Rc::new(v21));
            List::cons(Adt6::C1(v22, true), List::<Adt6>::nil())
        }
        Adt0::C11(r16) => {
            let v23 = (*r16).clone();
            let v24: Adt0 = Adt0::C11(std::rc::Rc::new(v23));
            List::cons(Adt6::C1(v24, true), List::<Adt6>::nil())
        }
        Adt0::C12(r17) => {
            let v25 = (*r17).clone();
            let v26: Adt0 = Adt0::C12(std::rc::Rc::new(v25));
            List::cons(Adt6::C1(v26, true), List::<Adt6>::nil())
        }
        Adt0::C13(r18) => {
            let v27 = (*r18).clone();
            let v28: Adt0 = Adt0::C13(std::rc::Rc::new(v27));
            List::cons(Adt6::C1(v28, true), List::<Adt6>::nil())
        }
        Adt0::C14(v29) => {
            let v30: Adt0 = Adt0::C14(v29);
            List::cons(Adt6::C1(v30, true), List::<Adt6>::nil())
        }
        Adt0::C15(v31) => {
            let v32: Adt0 = Adt0::C15(v31);
            List::cons(Adt6::C1(v32, true), List::<Adt6>::nil())
        }
        Adt0::C16(r19) => {
            let v33 = (*r19).clone();
            let v34: Adt0 = Adt0::C16(std::rc::Rc::new(v33));
            List::cons(Adt6::C1(v34, true), List::<Adt6>::nil())
        }
        Adt0::C17(r20) => {
            let v35 = (*r20).clone();
            let v36: Adt0 = Adt0::C17(std::rc::Rc::new(v35));
            List::cons(Adt6::C1(v36, true), List::<Adt6>::nil())
        }
        Adt0::C18(r21) => {
            let v37 = (*r21).clone();
            let v38: Adt0 = Adt0::C18(std::rc::Rc::new(v37));
            List::cons(Adt6::C1(v38, true), List::<Adt6>::nil())
        }
        Adt0::C19(r22) => {
            let v39 = (*r22).clone();
            let v40: Adt0 = Adt0::C19(std::rc::Rc::new(v39));
            List::cons(Adt6::C1(v40, true), List::<Adt6>::nil())
        }
        Adt0::C20(v41) => {
            let v42: Adt0 = Adt0::C20(v41);
            List::cons(Adt6::C1(v42, true), List::<Adt6>::nil())
        }
        Adt0::C21(r23) => {
            let v43 = (*r23).clone();
            let v44: Adt0 = Adt0::C21(std::rc::Rc::new(v43));
            List::cons(Adt6::C1(v44, true), List::<Adt6>::nil())
        }
        Adt0::C22(r24) => {
            let v45 = (*r24).clone();
            let v46: Adt0 = Adt0::C22(std::rc::Rc::new(v45));
            List::cons(Adt6::C1(v46, true), List::<Adt6>::nil())
        }
        Adt0::C23(r25) => {
            let v47 = (*r25).clone();
            let v48: Adt0 = Adt0::C23(std::rc::Rc::new(v47));
            List::cons(Adt6::C1(v48, true), List::<Adt6>::nil())
        }
        Adt0::C24(r26) => {
            let v49 = (*r26).clone();
            let v50: Adt0 = Adt0::C24(std::rc::Rc::new(v49));
            List::cons(Adt6::C1(v50, true), List::<Adt6>::nil())
        }
        Adt0::C25(v51) => {
            let a6 = {
                let a5 = v51.clone();
                length_list(a5)
            };
            let a7 = 0u64;
            if nat_eq(a6, a7) {
                List::<Adt6>::nil()
            } else {
                let v52: Adt0 = Adt0::C25(v51);
                List::cons(Adt6::C1(v52, true), List::<Adt6>::nil())
            }
        }
        Adt0::C26(r27) => {
            let v53 = (*r27).clone();
            let v54: Adt0 = Adt0::C26(std::rc::Rc::new(v53));
            List::cons(Adt6::C1(v54, true), List::<Adt6>::nil())
        }
        Adt0::C27(r28) => {
            let v55 = (*r28).clone();
            let v56: Adt0 = Adt0::C27(std::rc::Rc::new(v55));
            List::cons(Adt6::C1(v56, true), List::<Adt6>::nil())
        }
        Adt0::C28(v57) => {
            let v58: Adt0 = Adt0::C28(v57);
            List::cons(Adt6::C1(v58, true), List::<Adt6>::nil())
        }
    }
}

pub fn f2(v0: Adt4) -> bool {
    let m29 = v0;
    match m29 {
        Adt4::C0(v1, _) => {
            v1
        }
    }
}

pub fn f3(v0: Adt5) -> List<Adt6> {
    let m30 = v0;
    match m30 {
        Adt5::C0(v1) => {
            v1
        }
    }
}

pub fn f4(v0: Adt4) -> Adt5 {
    let m31 = v0;
    match m31 {
        Adt4::C0(_, v2) => {
            v2
        }
    }
}

pub fn run(p0: Adt0, p1: List<Adt6>) -> Adt0 {
    f0(p0, p1)
}
