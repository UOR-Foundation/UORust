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
    C0(u64, Str),
}

pub fn f0(v0: Adt0, v1: List<Str>) -> R<Str> {
    let m1 = v0;
    match m1 {
        Adt0::C0 => {
            let v2: List<Str> = f1(f2(v1, 0u64)?)?;
            let a4 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C0), List::cons(Str::lit(" {"), List::cons({
                let a2 = v2;
                let a3 = Str::lit(", ");
                join(a2, a3)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a5 = Str::lit("");
            Ok(join(a4, a5))
        }
        Adt0::C1 => {
            let v3: List<Str> = f4(f5(v1, 0u64)?)?;
            let a8 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C1), List::cons(Str::lit(" {"), List::cons({
                let a6 = v3;
                let a7 = Str::lit(", ");
                join(a6, a7)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a9 = Str::lit("");
            Ok(join(a8, a9))
        }
        Adt0::C2(v4) => {
            let v5: List<Str> = f6(f7(v1, 0u64)?)?;
            let a12 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C2(v4)), List::cons(Str::lit(" {"), List::cons({
                let a10 = v5;
                let a11 = Str::lit(", ");
                join(a10, a11)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a13 = Str::lit("");
            Ok(join(a12, a13))
        }
        Adt0::C3(v6) => {
            let v7: List<Str> = f8(f9(v1, 0u64)?)?;
            let a16 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C3(v6)), List::cons(Str::lit(" {"), List::cons({
                let a14 = v7;
                let a15 = Str::lit(", ");
                join(a14, a15)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a17 = Str::lit("");
            Ok(join(a16, a17))
        }
        Adt0::C4(v8) => {
            let v9: List<Str> = f10(f11(v1, 0u64)?)?;
            let a20 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C4(v8)), List::cons(Str::lit(" {"), List::cons({
                let a18 = v9;
                let a19 = Str::lit(", ");
                join(a18, a19)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a21 = Str::lit("");
            Ok(join(a20, a21))
        }
        Adt0::C5(v10) => {
            let v11: List<Str> = f12(f13(v1, 0u64)?)?;
            let a24 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C5(v10)), List::cons(Str::lit(" {"), List::cons({
                let a22 = v11;
                let a23 = Str::lit(", ");
                join(a22, a23)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a25 = Str::lit("");
            Ok(join(a24, a25))
        }
        Adt0::C6 => {
            let v12: List<Str> = f14(f15(v1, 0u64)?)?;
            let a28 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C6), List::cons(Str::lit(" {"), List::cons({
                let a26 = v12;
                let a27 = Str::lit(", ");
                join(a26, a27)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a29 = Str::lit("");
            Ok(join(a28, a29))
        }
        Adt0::C7 => {
            let v13: List<Str> = f16(f17(v1, 0u64)?)?;
            let a32 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C7), List::cons(Str::lit(" {"), List::cons({
                let a30 = v13;
                let a31 = Str::lit(", ");
                join(a30, a31)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a33 = Str::lit("");
            Ok(join(a32, a33))
        }
        Adt0::C8 => {
            let v14: List<Str> = f18(f19(v1, 0u64)?)?;
            let a36 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C8), List::cons(Str::lit(" {"), List::cons({
                let a34 = v14;
                let a35 = Str::lit(", ");
                join(a34, a35)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a37 = Str::lit("");
            Ok(join(a36, a37))
        }
        Adt0::C9 => {
            let v15: List<Str> = f20(f21(v1, 0u64)?)?;
            let a40 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C9), List::cons(Str::lit(" {"), List::cons({
                let a38 = v15;
                let a39 = Str::lit(", ");
                join(a38, a39)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a41 = Str::lit("");
            Ok(join(a40, a41))
        }
        Adt0::C10 => {
            let v16: List<Str> = f22(f23(v1, 0u64)?)?;
            let a44 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C10), List::cons(Str::lit(" {"), List::cons({
                let a42 = v16;
                let a43 = Str::lit(", ");
                join(a42, a43)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a45 = Str::lit("");
            Ok(join(a44, a45))
        }
        Adt0::C11(v17) => {
            let v18: List<Str> = f24(f25(v1, 0u64)?)?;
            let a48 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C11(v17)), List::cons(Str::lit(" {"), List::cons({
                let a46 = v18;
                let a47 = Str::lit(", ");
                join(a46, a47)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a49 = Str::lit("");
            Ok(join(a48, a49))
        }
        Adt0::C12(r80) => {
            let v19 = (*r80).clone();
            let v20: List<Str> = f26(f27(v1, 0u64)?)?;
            let a52 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C12(std::rc::Rc::new(v19))), List::cons(Str::lit(" {"), List::cons({
                let a50 = v20;
                let a51 = Str::lit(", ");
                join(a50, a51)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a53 = Str::lit("");
            Ok(join(a52, a53))
        }
        Adt0::C13(r81, r82) => {
            let v21 = (*r81).clone();
            let v22 = (*r82).clone();
            let v23: List<Str> = f28(f29(v1, 0u64)?)?;
            let a56 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C13(std::rc::Rc::new(v21), std::rc::Rc::new(v22))), List::cons(Str::lit(" {"), List::cons({
                let a54 = v23;
                let a55 = Str::lit(", ");
                join(a54, a55)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a57 = Str::lit("");
            Ok(join(a56, a57))
        }
        Adt0::C14(r83) => {
            let v24 = (*r83).clone();
            let v25: List<Str> = f30(f31(v1, 0u64)?)?;
            let a60 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C14(std::rc::Rc::new(v24))), List::cons(Str::lit(" {"), List::cons({
                let a58 = v25;
                let a59 = Str::lit(", ");
                join(a58, a59)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a61 = Str::lit("");
            Ok(join(a60, a61))
        }
        Adt0::C15(v26) => {
            let v27: List<Str> = f32(f33(v1, 0u64)?)?;
            let a64 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C15(v26)), List::cons(Str::lit(" {"), List::cons({
                let a62 = v27;
                let a63 = Str::lit(", ");
                join(a62, a63)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a65 = Str::lit("");
            Ok(join(a64, a65))
        }
        Adt0::C16(_) => {
            let a66 = List::cons(Str::lit("("), List::cons(f34(v1.clone()), List::cons(Str::lit(", "), List::cons(f35(v1), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a67 = Str::lit("");
            Ok(join(a66, a67))
        }
        Adt0::C17(r84, r85, v31) => {
            let v29 = (*r84).clone();
            let v30 = (*r85).clone();
            let v32: List<Str> = f36(f37(v1, 0u64)?)?;
            let a70 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C17(std::rc::Rc::new(v29), std::rc::Rc::new(v30), v31)), List::cons(Str::lit(" {"), List::cons({
                let a68 = v32;
                let a69 = Str::lit(", ");
                join(a68, a69)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a71 = Str::lit("");
            Ok(join(a70, a71))
        }
        Adt0::C18(r86, v34) => {
            let v33 = (*r86).clone();
            let v35: List<Str> = f38(f39(v1, 0u64)?)?;
            let a74 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C18(std::rc::Rc::new(v33), v34)), List::cons(Str::lit(" {"), List::cons({
                let a72 = v35;
                let a73 = Str::lit(", ");
                join(a72, a73)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a75 = Str::lit("");
            Ok(join(a74, a75))
        }
        Adt0::C19(v36, r87) => {
            let v37 = (*r87).clone();
            let v38: List<Str> = f40(f41(v1, 0u64)?)?;
            let a78 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C19(v36, std::rc::Rc::new(v37))), List::cons(Str::lit(" {"), List::cons({
                let a76 = v38;
                let a77 = Str::lit(", ");
                join(a76, a77)
            }, List::cons(Str::lit("})"), List::<Str>::nil())))));
            let a79 = Str::lit("");
            Ok(join(a78, a79))
        }
    }
}

pub fn f1(v0: List<Adt2>) -> R<List<Str>> {
    let m88 = v0;
    match m88.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h89, _) => {
                    h89
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h90) => {
                    h90
                }
            };
            Ok(List::cons({
                let a91 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a92 = Str::lit("");
                join(a91, a92)
            }, f1(v2)?))
        }
    }
}

pub fn f2(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m93 = v0;
    match m93.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f2(v3, {
                let a94 = v1;
                let a95 = 1u64;
                nat_add(a94, a95)?
            })?))
        }
    }
}

pub fn f3(v0: Adt0) -> Str {
    let m96 = v0;
    match m96 {
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
            let a97 = List::cons(f43(), List::cons(Str::lit("::ParseIntError"), List::<Str>::nil()));
            let a98 = Str::lit("");
            join(a97, a98)
        }
        Adt0::C10 => {
            let a99 = List::cons(f43(), List::cons(Str::lit("::FilePath"), List::<Str>::nil()));
            let a100 = Str::lit("");
            join(a99, a100)
        }
        Adt0::C11(v5) => {
            v5
        }
        Adt0::C12(r123) => {
            let v6 = (*r123).clone();
            let a101 = List::cons(Str::lit("(Option "), List::cons(f3(v6), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a102 = Str::lit("");
            join(a101, a102)
        }
        Adt0::C13(r124, r125) => {
            let v7 = (*r124).clone();
            let v8 = (*r125).clone();
            let a103 = List::cons(Str::lit("(Result "), List::cons(f3(v7), List::cons(Str::lit(" "), List::cons(f3(v8), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a104 = Str::lit("");
            join(a103, a104)
        }
        Adt0::C14(r126) => {
            let v9 = (*r126).clone();
            let a105 = List::cons(Str::lit("(List "), List::cons(f3(v9), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a106 = Str::lit("");
            join(a105, a106)
        }
        Adt0::C15(v10) => {
            let a109 = List::cons(Str::lit("Tuple"), List::cons({
                let a107 = f44(v10);
                let a108 = Str::lit("");
                join(a107, a108)
            }, List::<Str>::nil()));
            let a110 = Str::lit("");
            join(a109, a110)
        }
        Adt0::C16(v11) => {
            let a113 = List::cons(Str::lit("("), List::cons({
                let a111 = f45(v11);
                let a112 = Str::lit(" * ");
                join(a111, a112)
            }, List::cons(Str::lit(")"), List::<Str>::nil())));
            let a114 = Str::lit("");
            join(a113, a114)
        }
        Adt0::C17(r127, r128, _) => {
            let v12 = (*r127).clone();
            let v13 = (*r128).clone();
            let a115 = List::cons(Str::lit("(List Tuple"), List::cons(f46(v12), List::cons(f46(v13), List::cons(Str::lit(")"), List::<Str>::nil()))));
            let a116 = Str::lit("");
            join(a115, a116)
        }
        Adt0::C18(r129, _) => {
            let v15 = (*r129).clone();
            let a117 = List::cons(Str::lit("(List "), List::cons(f3(v15), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a118 = Str::lit("");
            join(a117, a118)
        }
        Adt0::C19(v17, r130) => {
            let v18 = (*r130).clone();
            let a121 = List::cons(Str::lit("(Fn ("), List::cons({
                let a119 = f47(v17);
                let a120 = Str::lit(" ");
                join(a119, a120)
            }, List::cons(Str::lit(") "), List::cons(f3(v18), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a122 = Str::lit("");
            join(a121, a122)
        }
    }
}

pub fn f4(v0: List<Adt2>) -> R<List<Str>> {
    let m131 = v0;
    match m131.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h132, _) => {
                    h132
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h133) => {
                    h133
                }
            };
            Ok(List::cons({
                let a134 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a135 = Str::lit("");
                join(a134, a135)
            }, f4(v2)?))
        }
    }
}

pub fn f5(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m136 = v0;
    match m136.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f5(v3, {
                let a137 = v1;
                let a138 = 1u64;
                nat_add(a137, a138)?
            })?))
        }
    }
}

pub fn f6(v0: List<Adt2>) -> R<List<Str>> {
    let m139 = v0;
    match m139.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h140, _) => {
                    h140
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h141) => {
                    h141
                }
            };
            Ok(List::cons({
                let a142 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a143 = Str::lit("");
                join(a142, a143)
            }, f6(v2)?))
        }
    }
}

pub fn f7(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m144 = v0;
    match m144.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f7(v3, {
                let a145 = v1;
                let a146 = 1u64;
                nat_add(a145, a146)?
            })?))
        }
    }
}

pub fn f8(v0: List<Adt2>) -> R<List<Str>> {
    let m147 = v0;
    match m147.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h148, _) => {
                    h148
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h149) => {
                    h149
                }
            };
            Ok(List::cons({
                let a150 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a151 = Str::lit("");
                join(a150, a151)
            }, f8(v2)?))
        }
    }
}

pub fn f9(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m152 = v0;
    match m152.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f9(v3, {
                let a153 = v1;
                let a154 = 1u64;
                nat_add(a153, a154)?
            })?))
        }
    }
}

pub fn f10(v0: List<Adt2>) -> R<List<Str>> {
    let m155 = v0;
    match m155.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h156, _) => {
                    h156
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h157) => {
                    h157
                }
            };
            Ok(List::cons({
                let a158 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a159 = Str::lit("");
                join(a158, a159)
            }, f10(v2)?))
        }
    }
}

pub fn f11(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m160 = v0;
    match m160.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f11(v3, {
                let a161 = v1;
                let a162 = 1u64;
                nat_add(a161, a162)?
            })?))
        }
    }
}

pub fn f12(v0: List<Adt2>) -> R<List<Str>> {
    let m163 = v0;
    match m163.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h164, _) => {
                    h164
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h165) => {
                    h165
                }
            };
            Ok(List::cons({
                let a166 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a167 = Str::lit("");
                join(a166, a167)
            }, f12(v2)?))
        }
    }
}

pub fn f13(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m168 = v0;
    match m168.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f13(v3, {
                let a169 = v1;
                let a170 = 1u64;
                nat_add(a169, a170)?
            })?))
        }
    }
}

pub fn f14(v0: List<Adt2>) -> R<List<Str>> {
    let m171 = v0;
    match m171.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h172, _) => {
                    h172
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h173) => {
                    h173
                }
            };
            Ok(List::cons({
                let a174 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a175 = Str::lit("");
                join(a174, a175)
            }, f14(v2)?))
        }
    }
}

pub fn f15(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m176 = v0;
    match m176.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f15(v3, {
                let a177 = v1;
                let a178 = 1u64;
                nat_add(a177, a178)?
            })?))
        }
    }
}

pub fn f16(v0: List<Adt2>) -> R<List<Str>> {
    let m179 = v0;
    match m179.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h180, _) => {
                    h180
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h181) => {
                    h181
                }
            };
            Ok(List::cons({
                let a182 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a183 = Str::lit("");
                join(a182, a183)
            }, f16(v2)?))
        }
    }
}

pub fn f17(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m184 = v0;
    match m184.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f17(v3, {
                let a185 = v1;
                let a186 = 1u64;
                nat_add(a185, a186)?
            })?))
        }
    }
}

pub fn f18(v0: List<Adt2>) -> R<List<Str>> {
    let m187 = v0;
    match m187.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h188, _) => {
                    h188
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h189) => {
                    h189
                }
            };
            Ok(List::cons({
                let a190 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a191 = Str::lit("");
                join(a190, a191)
            }, f18(v2)?))
        }
    }
}

pub fn f19(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m192 = v0;
    match m192.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f19(v3, {
                let a193 = v1;
                let a194 = 1u64;
                nat_add(a193, a194)?
            })?))
        }
    }
}

pub fn f20(v0: List<Adt2>) -> R<List<Str>> {
    let m195 = v0;
    match m195.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h196, _) => {
                    h196
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h197) => {
                    h197
                }
            };
            Ok(List::cons({
                let a198 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a199 = Str::lit("");
                join(a198, a199)
            }, f20(v2)?))
        }
    }
}

pub fn f21(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m200 = v0;
    match m200.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f21(v3, {
                let a201 = v1;
                let a202 = 1u64;
                nat_add(a201, a202)?
            })?))
        }
    }
}

pub fn f22(v0: List<Adt2>) -> R<List<Str>> {
    let m203 = v0;
    match m203.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h204, _) => {
                    h204
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h205) => {
                    h205
                }
            };
            Ok(List::cons({
                let a206 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a207 = Str::lit("");
                join(a206, a207)
            }, f22(v2)?))
        }
    }
}

pub fn f23(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m208 = v0;
    match m208.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f23(v3, {
                let a209 = v1;
                let a210 = 1u64;
                nat_add(a209, a210)?
            })?))
        }
    }
}

pub fn f24(v0: List<Adt2>) -> R<List<Str>> {
    let m211 = v0;
    match m211.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h212, _) => {
                    h212
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h213) => {
                    h213
                }
            };
            Ok(List::cons({
                let a214 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a215 = Str::lit("");
                join(a214, a215)
            }, f24(v2)?))
        }
    }
}

pub fn f25(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m216 = v0;
    match m216.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f25(v3, {
                let a217 = v1;
                let a218 = 1u64;
                nat_add(a217, a218)?
            })?))
        }
    }
}

pub fn f26(v0: List<Adt2>) -> R<List<Str>> {
    let m219 = v0;
    match m219.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h220, _) => {
                    h220
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h221) => {
                    h221
                }
            };
            Ok(List::cons({
                let a222 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a223 = Str::lit("");
                join(a222, a223)
            }, f26(v2)?))
        }
    }
}

pub fn f27(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m224 = v0;
    match m224.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f27(v3, {
                let a225 = v1;
                let a226 = 1u64;
                nat_add(a225, a226)?
            })?))
        }
    }
}

pub fn f28(v0: List<Adt2>) -> R<List<Str>> {
    let m227 = v0;
    match m227.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h228, _) => {
                    h228
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h229) => {
                    h229
                }
            };
            Ok(List::cons({
                let a230 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a231 = Str::lit("");
                join(a230, a231)
            }, f28(v2)?))
        }
    }
}

pub fn f29(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m232 = v0;
    match m232.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f29(v3, {
                let a233 = v1;
                let a234 = 1u64;
                nat_add(a233, a234)?
            })?))
        }
    }
}

pub fn f30(v0: List<Adt2>) -> R<List<Str>> {
    let m235 = v0;
    match m235.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h236, _) => {
                    h236
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h237) => {
                    h237
                }
            };
            Ok(List::cons({
                let a238 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a239 = Str::lit("");
                join(a238, a239)
            }, f30(v2)?))
        }
    }
}

pub fn f31(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m240 = v0;
    match m240.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f31(v3, {
                let a241 = v1;
                let a242 = 1u64;
                nat_add(a241, a242)?
            })?))
        }
    }
}

pub fn f32(v0: List<Adt2>) -> R<List<Str>> {
    let m243 = v0;
    match m243.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h244, _) => {
                    h244
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h245) => {
                    h245
                }
            };
            Ok(List::cons({
                let a246 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a247 = Str::lit("");
                join(a246, a247)
            }, f32(v2)?))
        }
    }
}

pub fn f33(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m248 = v0;
    match m248.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f33(v3, {
                let a249 = v1;
                let a250 = 1u64;
                nat_add(a249, a250)?
            })?))
        }
    }
}

pub fn f34(v0: List<Str>) -> Str {
    let m253 = {
        let a251 = v0;
        let a252 = 0u64;
        index_list(a251, a252)
    };
    match m253 {
        None => {
            Str::lit("")
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f35(v0: List<Str>) -> Str {
    let m256 = {
        let a254 = v0;
        let a255 = 1u64;
        index_list(a254, a255)
    };
    match m256 {
        None => {
            Str::lit("")
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f36(v0: List<Adt2>) -> R<List<Str>> {
    let m257 = v0;
    match m257.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h258, _) => {
                    h258
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h259) => {
                    h259
                }
            };
            Ok(List::cons({
                let a260 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a261 = Str::lit("");
                join(a260, a261)
            }, f36(v2)?))
        }
    }
}

pub fn f37(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m262 = v0;
    match m262.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f37(v3, {
                let a263 = v1;
                let a264 = 1u64;
                nat_add(a263, a264)?
            })?))
        }
    }
}

pub fn f38(v0: List<Adt2>) -> R<List<Str>> {
    let m265 = v0;
    match m265.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h266, _) => {
                    h266
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h267) => {
                    h267
                }
            };
            Ok(List::cons({
                let a268 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a269 = Str::lit("");
                join(a268, a269)
            }, f38(v2)?))
        }
    }
}

pub fn f39(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m270 = v0;
    match m270.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f39(v3, {
                let a271 = v1;
                let a272 = 1u64;
                nat_add(a271, a272)?
            })?))
        }
    }
}

pub fn f40(v0: List<Adt2>) -> R<List<Str>> {
    let m273 = v0;
    match m273.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h274, _) => {
                    h274
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h275) => {
                    h275
                }
            };
            Ok(List::cons({
                let a276 = List::cons(Str::lit("item"), List::cons(f42(v3)?, List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()))));
                let a277 = Str::lit("");
                join(a276, a277)
            }, f40(v2)?))
        }
    }
}

pub fn f41(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m278 = v0;
    match m278.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f41(v3, {
                let a279 = v1;
                let a280 = 1u64;
                nat_add(a279, a280)?
            })?))
        }
    }
}

pub fn f42(v0: u64) -> R<Str> {
    Ok(f48({
        let a281 = v0;
        let a282 = 1u64;
        nat_add(a281, a282)?
    }, v0))
}

pub fn f43() -> Str {
    Str::lit("Prelude")
}

pub fn f44(v0: List<Adt0>) -> List<Str> {
    let m283 = v0;
    match m283.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f46(v1), f44(v2))
        }
    }
}

pub fn f45(v0: List<Adt0>) -> List<Str> {
    let m284 = v0;
    match m284.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f3(v1), f45(v2))
        }
    }
}

pub fn f46(v0: Adt0) -> Str {
    let m285 = v0;
    match m285 {
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
        Adt0::C12(r308) => {
            let v6 = (*r308).clone();
            let a286 = List::cons(Str::lit("Option"), List::cons(f46(v6), List::<Str>::nil()));
            let a287 = Str::lit("");
            join(a286, a287)
        }
        Adt0::C13(r309, r310) => {
            let v7 = (*r309).clone();
            let v8 = (*r310).clone();
            let a288 = List::cons(Str::lit("Result"), List::cons(f46(v7), List::cons(f46(v8), List::<Str>::nil())));
            let a289 = Str::lit("");
            join(a288, a289)
        }
        Adt0::C14(r311) => {
            let v9 = (*r311).clone();
            let a290 = List::cons(Str::lit("List"), List::cons(f46(v9), List::<Str>::nil()));
            let a291 = Str::lit("");
            join(a290, a291)
        }
        Adt0::C15(v10) => {
            let a294 = List::cons(Str::lit("Tuple"), List::cons({
                let a292 = f49(v10);
                let a293 = Str::lit("");
                join(a292, a293)
            }, List::<Str>::nil()));
            let a295 = Str::lit("");
            join(a294, a295)
        }
        Adt0::C16(v11) => {
            let a298 = List::cons(Str::lit("Pair"), List::cons({
                let a296 = f50(v11);
                let a297 = Str::lit("");
                join(a296, a297)
            }, List::<Str>::nil()));
            let a299 = Str::lit("");
            join(a298, a299)
        }
        Adt0::C17(r312, r313, _) => {
            let v12 = (*r312).clone();
            let v13 = (*r313).clone();
            let a300 = List::cons(Str::lit("ListTuple"), List::cons(f46(v12), List::cons(f46(v13), List::<Str>::nil())));
            let a301 = Str::lit("");
            join(a300, a301)
        }
        Adt0::C18(r314, _) => {
            let v15 = (*r314).clone();
            let a302 = List::cons(Str::lit("List"), List::cons(f46(v15), List::<Str>::nil()));
            let a303 = Str::lit("");
            join(a302, a303)
        }
        Adt0::C19(v17, r315) => {
            let v18 = (*r315).clone();
            let a306 = List::cons(Str::lit("Fn"), List::cons({
                let a304 = f51(v17);
                let a305 = Str::lit("");
                join(a304, a305)
            }, List::cons(Str::lit("To"), List::cons(f46(v18), List::<Str>::nil()))));
            let a307 = Str::lit("");
            join(a306, a307)
        }
    }
}

pub fn f47(v0: List<Adt0>) -> List<Str> {
    let m316 = v0;
    match m316.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f3(v1), f47(v2))
        }
    }
}

pub fn f48(v0: u64, v1: u64) -> Str {
    let m317 = v0;
    if m317 == 0 {
        Str::lit("")
    } else {
        let v2 = m317 - 1;
        let a318 = v1;
        let a319 = 10u64;
        if nat_lt(a318, a319) {
            f52(v1)
        } else {
            let a326 = List::cons(f48(v2, {
                let a320 = v1;
                let a321 = 10u64;
                let a322 = 0u64;
                nat_quot(a320, a321, a322)
            }), List::cons(f52({
                let a323 = v1;
                let a324 = 10u64;
                let a325 = 0u64;
                nat_rem(a323, a324, a325)
            }), List::<Str>::nil()));
            let a327 = Str::lit("");
            join(a326, a327)
        }
    }
}

pub fn f49(v0: List<Adt0>) -> List<Str> {
    let m328 = v0;
    match m328.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f46(v1), f49(v2))
        }
    }
}

pub fn f50(v0: List<Adt0>) -> List<Str> {
    let m329 = v0;
    match m329.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f46(v1), f50(v2))
        }
    }
}

pub fn f51(v0: List<Adt0>) -> List<Str> {
    let m330 = v0;
    match m330.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f46(v1), f51(v2))
        }
    }
}

pub fn f52(v0: u64) -> Str {
    let a331 = v0;
    let a332 = 0u64;
    if nat_eq(a331, a332) {
        Str::lit("0")
    } else {
        let a333 = v0;
        let a334 = 1u64;
        if nat_eq(a333, a334) {
            Str::lit("1")
        } else {
            let a335 = v0;
            let a336 = 2u64;
            if nat_eq(a335, a336) {
                Str::lit("2")
            } else {
                let a337 = v0;
                let a338 = 3u64;
                if nat_eq(a337, a338) {
                    Str::lit("3")
                } else {
                    let a339 = v0;
                    let a340 = 4u64;
                    if nat_eq(a339, a340) {
                        Str::lit("4")
                    } else {
                        let a341 = v0;
                        let a342 = 5u64;
                        if nat_eq(a341, a342) {
                            Str::lit("5")
                        } else {
                            let a343 = v0;
                            let a344 = 6u64;
                            if nat_eq(a343, a344) {
                                Str::lit("6")
                            } else {
                                let a345 = v0;
                                let a346 = 7u64;
                                if nat_eq(a345, a346) {
                                    Str::lit("7")
                                } else {
                                    let a347 = v0;
                                    let a348 = 8u64;
                                    if nat_eq(a347, a348) {
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

pub fn run(p0: Adt0, p1: List<Str>) -> R<Str> {
    f0(p0, p1)
}
