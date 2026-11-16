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
    pub fn parse_int(text: &Str) -> Option<Option<i64>> { match canonical_decimal(&text.0) { None => Some(None), Some(Some(n)) => match i64::try_from(n) { Ok(value) => Some(Some(value)), Err(_) => None, }, Some(None) => None, } }
    pub fn format_u8(a: u8) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_u8(text: &Str) -> Option<u8> { match canonical_decimal(&text.0) { Some(Some(n)) => u8::try_from(n).ok(), _ => None, } }
    pub fn format_u16(a: u16) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_u16(text: &Str) -> Option<u16> { match canonical_decimal(&text.0) { Some(Some(n)) => u16::try_from(n).ok(), _ => None, } }
    pub fn format_u32(a: u32) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_u32(text: &Str) -> Option<u32> { match canonical_decimal(&text.0) { Some(Some(n)) => u32::try_from(n).ok(), _ => None, } }
    pub fn format_u64(a: u64) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_u64(text: &Str) -> Option<u64> { match canonical_decimal(&text.0) { Some(Some(n)) => u64::try_from(n).ok(), _ => None, } }
    pub fn format_i8(a: i8) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_i8(text: &Str) -> Option<i8> { match canonical_decimal(&text.0) { Some(Some(n)) => i8::try_from(n).ok(), _ => None, } }
    pub fn format_i16(a: i16) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_i16(text: &Str) -> Option<i16> { match canonical_decimal(&text.0) { Some(Some(n)) => i16::try_from(n).ok(), _ => None, } }
    pub fn format_i32(a: i32) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_i32(text: &Str) -> Option<i32> { match canonical_decimal(&text.0) { Some(Some(n)) => i32::try_from(n).ok(), _ => None, } }
    pub fn format_i64(a: i64) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_i64(text: &Str) -> Option<i64> { match canonical_decimal(&text.0) { Some(Some(n)) => i64::try_from(n).ok(), _ => None, } }
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

pub fn format_u8(p0: u8) -> Str {
    Str::format_u8(p0)
}

pub fn parse_u8(p0: Str) -> Option<u8> {
    Str::parse_u8(&p0)
}

pub fn format_u16(p0: u16) -> Str {
    Str::format_u16(p0)
}

pub fn parse_u16(p0: Str) -> Option<u16> {
    Str::parse_u16(&p0)
}

pub fn format_u32(p0: u32) -> Str {
    Str::format_u32(p0)
}

pub fn parse_u32(p0: Str) -> Option<u32> {
    Str::parse_u32(&p0)
}

pub fn format_u64(p0: u64) -> Str {
    Str::format_u64(p0)
}

pub fn parse_u64(p0: Str) -> Option<u64> {
    Str::parse_u64(&p0)
}

pub fn format_i8(p0: i8) -> Str {
    Str::format_i8(p0)
}

pub fn parse_i8(p0: Str) -> Option<i8> {
    Str::parse_i8(&p0)
}

pub fn format_i16(p0: i16) -> Str {
    Str::format_i16(p0)
}

pub fn parse_i16(p0: Str) -> Option<i16> {
    Str::parse_i16(&p0)
}

pub fn format_i32(p0: i32) -> Str {
    Str::format_i32(p0)
}

pub fn parse_i32(p0: Str) -> Option<i32> {
    Str::parse_i32(&p0)
}

pub fn format_i64(p0: i64) -> Str {
    Str::format_i64(p0)
}

pub fn parse_i64(p0: Str) -> Option<i64> {
    Str::parse_i64(&p0)
}

pub fn parse_int(p0: Str) -> R<Option<i64>> {
    match Str::parse_int(&p0) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<Option<i64>, Overflow>(Overflow)
        }
    }
}

pub fn format_int(p0: i64) -> Str {
    Str::format_i64(p0)
}

#[derive(Clone)]
pub enum Adt0 {
    C0(Str, Adt1),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Str, std::rc::Rc<Adt2>),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(bool, std::rc::Rc<Adt1>),
    C1(std::rc::Rc<Adt1>),
    C2(std::rc::Rc<Adt1>, Str),
    C3(List<Adt1>, std::rc::Rc<Adt1>),
    C4(std::rc::Rc<Adt1>),
    C5(List<Adt1>),
    C6(bool, Adt3),
    C7,
}

#[derive(Clone)]
pub enum Adt3 {
    C0(List<Adt4>, Str),
}

#[derive(Clone)]
pub enum Adt4 {
    C0(Str, Adt5),
}

#[derive(Clone)]
pub enum Adt5 {
    C0,
    C1(List<Adt6>),
    C2,
}

#[derive(Clone)]
pub enum Adt6 {
    C0(Adt1),
    C1(Str),
}

pub fn f0(v0: Str, v1: bool, v2: List<Str>, v3: List<Adt0>) -> Option<Str> {
    let m1 = v3;
    match m1.uncons() {
        None => {
            if f1(v0.clone(), v1) {
                Some({
                    let a4 = {
                        let a2 = List::cons(v0, List::<Str>::nil());
                        let a3 = List::cons(Str::lit(": "), List::cons(f2(), List::<Str>::nil()));
                        append_list(a2, a3)
                    };
                    let a5 = Str::lit("");
                    join(a4, a5)
                })
            } else {
                let m6 = f3(v2);
                match m6 {
                    None => {
                        None::<Str>
                    }
                    Some(v4) => {
                        Some({
                            let a9 = {
                                let a7 = List::cons(v0, List::cons(Str::lit("::"), List::<Str>::nil()));
                                let a8 = List::cons(v4, List::cons(Str::lit(": "), List::cons(f4(), List::<Str>::nil())));
                                append_list(a7, a8)
                            };
                            let a10 = Str::lit("");
                            join(a9, a10)
                        })
                    }
                }
            }
        }
        Some((v5, v6)) => {
            let m11 = v6;
            match m11.uncons() {
                None => {
                    let v7: Str = match v5.clone() {
                        Adt0::C0(h12, _) => {
                            h12
                        }
                    };
                    let v8: Adt1 = match v5 {
                        Adt0::C0(_, h13) => {
                            h13
                        }
                    };
                    let a14 = v7;
                    let a15 = Str::lit("");
                    if equal(a14, a15) {
                        Some({
                            let a18 = {
                                let a16 = List::cons(v0, List::<Str>::nil());
                                let a17 = List::cons(Str::lit(": "), List::cons(f5(v8), List::<Str>::nil()));
                                append_list(a16, a17)
                            };
                            let a19 = Str::lit("");
                            join(a18, a19)
                        })
                    } else {
                        if f1(v0.clone(), v1) {
                            Some({
                                let a22 = {
                                    let a20 = List::cons(v0, List::<Str>::nil());
                                    let a21 = List::cons(Str::lit(": "), List::cons(f2(), List::<Str>::nil()));
                                    append_list(a20, a21)
                                };
                                let a23 = Str::lit("");
                                join(a22, a23)
                            })
                        } else {
                            let m24 = f6(v2);
                            match m24 {
                                None => {
                                    None::<Str>
                                }
                                Some(v9) => {
                                    Some({
                                        let a27 = {
                                            let a25 = List::cons(v0, List::cons(Str::lit("::"), List::<Str>::nil()));
                                            let a26 = List::cons(v9, List::cons(Str::lit(": "), List::cons(f4(), List::<Str>::nil())));
                                            append_list(a25, a26)
                                        };
                                        let a28 = Str::lit("");
                                        join(a27, a28)
                                    })
                                }
                            }
                        }
                    }
                }
                Some((_, _)) => {
                    if f1(v0.clone(), v1) {
                        Some({
                            let a31 = {
                                let a29 = List::cons(v0, List::<Str>::nil());
                                let a30 = List::cons(Str::lit(": "), List::cons(f2(), List::<Str>::nil()));
                                append_list(a29, a30)
                            };
                            let a32 = Str::lit("");
                            join(a31, a32)
                        })
                    } else {
                        let m33 = f7(v2);
                        match m33 {
                            None => {
                                None::<Str>
                            }
                            Some(v12) => {
                                Some({
                                    let a36 = {
                                        let a34 = List::cons(v0, List::cons(Str::lit("::"), List::<Str>::nil()));
                                        let a35 = List::cons(v12, List::cons(Str::lit(": "), List::cons(f4(), List::<Str>::nil())));
                                        append_list(a34, a35)
                                    };
                                    let a37 = Str::lit("");
                                    join(a36, a37)
                                })
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn f1(v0: Str, v1: bool) -> bool {
    let v2: bool = if v1 {
        f8(v0.clone(), Str::lit("Tuple"))
    } else {
        false
    };
    if if f9(f10(), v0.clone()) {
        true
    } else {
        f8(v0, Str::lit("LexLean"))
    } {
        true
    } else {
        v2
    }
}

pub fn f2() -> Str {
    Str::lit("a type named as LexLean or the projection names one of its own")
}

pub fn f3(p40: List<Str>) -> Option<Str> {
    let m39 = p40;
    {
        let mut m39 = m39;
        loop {
            let step = {
                let v0 = m39;
                let m38 = v0;
                match m38.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v1, v2)) => {
                        if f11(f12(), v1.clone()) {
                            Err::<List<Str>, Option<Str>>(Some(v1))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v2)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m39 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f4() -> Str {
    Str::lit("a variant named as a word LexLean reserves")
}

pub fn f5(v0: Adt1) -> Str {
    let m41 = v0;
    match m41 {
        Adt1::C0(v1, _) => {
            v1
        }
    }
}

pub fn f6(p44: List<Str>) -> Option<Str> {
    let m43 = p44;
    {
        let mut m43 = m43;
        loop {
            let step = {
                let v0 = m43;
                let m42 = v0;
                match m42.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v1, v2)) => {
                        if f13(f12(), v1.clone()) {
                            Err::<List<Str>, Option<Str>>(Some(v1))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v2)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m43 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f7(p47: List<Str>) -> Option<Str> {
    let m46 = p47;
    {
        let mut m46 = m46;
        loop {
            let step = {
                let v0 = m46;
                let m45 = v0;
                match m45.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v1, v2)) => {
                        if f14(f12(), v1.clone()) {
                            Err::<List<Str>, Option<Str>>(Some(v1))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v2)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m46 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f8(v0: Str, v1: Str) -> bool {
    f15({
        let a48 = v0;
        utf8_encode(a48)
    }, 0u64, {
        let a49 = v1;
        utf8_encode(a49)
    })
}

pub fn f9(p54: List<Str>, v1: Str) -> bool {
    let m53 = p54;
    {
        let mut m53 = m53;
        loop {
            let step = {
                let v0 = m53;
                let m50 = v0;
                match m50.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a51 = v2;
                        let a52 = v1.clone();
                        if equal(a51, a52) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
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

pub fn f10() -> List<Str> {
    let a67 = {
        let a59 = {
            let a55 = List::cons(Str::lit("And"), List::cons(Str::lit("BitVec"), List::cons(Str::lit("Bool"), List::<Str>::nil())));
            let a56 = List::cons(Str::lit("ByteArray"), List::cons(Str::lit("ContractViolation"), List::cons(Str::lit("Except"), List::<Str>::nil())));
            append_list(a55, a56)
        };
        let a60 = {
            let a57 = List::cons(Str::lit("Iff"), List::cons(Str::lit("Int"), List::cons(Str::lit("Int16"), List::<Str>::nil())));
            let a58 = List::cons(Str::lit("Int32"), List::cons(Str::lit("Int64"), List::cons(Str::lit("Int8"), List::cons(Str::lit("List"), List::<Str>::nil()))));
            append_list(a57, a58)
        };
        append_list(a59, a60)
    };
    let a68 = {
        let a65 = {
            let a61 = List::cons(Str::lit("Nat"), List::cons(Str::lit("Option"), List::cons(Str::lit("Ordering"), List::<Str>::nil())));
            let a62 = List::cons(Str::lit("Prod"), List::cons(Str::lit("Prop"), List::cons(Str::lit("ReasoningFailure"), List::cons(Str::lit("Result"), List::<Str>::nil()))));
            append_list(a61, a62)
        };
        let a66 = {
            let a63 = List::cons(Str::lit("String"), List::cons(Str::lit("Type"), List::cons(Str::lit("UInt16"), List::<Str>::nil())));
            let a64 = List::cons(Str::lit("UInt32"), List::cons(Str::lit("UInt64"), List::cons(Str::lit("UInt8"), List::cons(Str::lit("Unit"), List::<Str>::nil()))));
            append_list(a63, a64)
        };
        append_list(a65, a66)
    };
    append_list(a67, a68)
}

pub fn f11(p73: List<Str>, v1: Str) -> bool {
    let m72 = p73;
    {
        let mut m72 = m72;
        loop {
            let step = {
                let v0 = m72;
                let m69 = v0;
                match m69.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a70 = v2;
                        let a71 = v1.clone();
                        if equal(a70, a71) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m72 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f12() -> List<Str> {
    let a102 = {
        let a86 = {
            let a78 = {
                let a74 = List::cons(Str::lit("at"), List::cons(Str::lit("by"), List::cons(Str::lit("calc"), List::cons(Str::lit("def"), List::cons(Str::lit("do"), List::cons(Str::lit("else"), List::<Str>::nil()))))));
                let a75 = List::cons(Str::lit("end"), List::cons(Str::lit("example"), List::cons(Str::lit("fun"), List::cons(Str::lit("have"), List::cons(Str::lit("if"), List::cons(Str::lit("in"), List::<Str>::nil()))))));
                append_list(a74, a75)
            };
            let a79 = {
                let a76 = List::cons(Str::lit("instance"), List::cons(Str::lit("let"), List::cons(Str::lit("match"), List::cons(Str::lit("mut"), List::cons(Str::lit("nomatch"), List::cons(Str::lit("open"), List::<Str>::nil()))))));
                let a77 = List::cons(Str::lit("prop"), List::cons(Str::lit("return"), List::cons(Str::lit("show"), List::cons(Str::lit("then"), List::cons(Str::lit("theorem"), List::cons(Str::lit("type"), List::cons(Str::lit("where"), List::<Str>::nil())))))));
                append_list(a76, a77)
            };
            append_list(a78, a79)
        };
        let a87 = {
            let a84 = {
                let a80 = List::cons(Str::lit("with"), List::cons(Str::lit("from"), List::cons(Str::lit("for"), List::cons(Str::lit("try"), List::cons(Str::lit("catch"), List::cons(Str::lit("break"), List::<Str>::nil()))))));
                let a81 = List::cons(Str::lit("continue"), List::cons(Str::lit("unless"), List::cons(Str::lit("structure"), List::cons(Str::lit("inductive"), List::cons(Str::lit("class"), List::cons(Str::lit("namespace"), List::cons(Str::lit("section"), List::<Str>::nil())))))));
                append_list(a80, a81)
            };
            let a85 = {
                let a82 = List::cons(Str::lit("local"), List::cons(Str::lit("private"), List::cons(Str::lit("protected"), List::cons(Str::lit("partial"), List::cons(Str::lit("unsafe"), List::cons(Str::lit("macro"), List::<Str>::nil()))))));
                let a83 = List::cons(Str::lit("syntax"), List::cons(Str::lit("import"), List::cons(Str::lit("export"), List::cons(Str::lit("axiom"), List::cons(Str::lit("abbrev"), List::cons(Str::lit("opaque"), List::cons(Str::lit("mutual"), List::<Str>::nil())))))));
                append_list(a82, a83)
            };
            append_list(a84, a85)
        };
        append_list(a86, a87)
    };
    let a103 = {
        let a100 = {
            let a92 = {
                let a88 = List::cons(Str::lit("variable"), List::cons(Str::lit("universe"), List::cons(Str::lit("deriving"), List::cons(Str::lit("extends"), List::cons(Str::lit("suffices"), List::cons(Str::lit("obtain"), List::<Str>::nil()))))));
                let a89 = List::cons(Str::lit("record"), List::cons(Str::lit("ctor"), List::cons(Str::lit("prim"), List::cons(Str::lit("project"), List::cons(Str::lit("cons"), List::cons(Str::lit("nil"), List::<Str>::nil()))))));
                append_list(a88, a89)
            };
            let a93 = {
                let a90 = List::cons(Str::lit("not"), List::cons(Str::lit("true"), List::cons(Str::lit("false"), List::cons(Str::lit("forall"), List::cons(Str::lit("first"), List::cons(Str::lit("second"), List::<Str>::nil()))))));
                let a91 = List::cons(Str::lit("apply"), List::cons(Str::lit("ref"), List::cons(Str::lit("checked"), List::cons(Str::lit("self"), List::cons(Str::lit("noncomputable"), List::cons(Str::lit("termination_by"), List::cons(Str::lit("and_congr"), List::<Str>::nil())))))));
                append_list(a90, a91)
            };
            append_list(a92, a93)
        };
        let a101 = {
            let a98 = {
                let a94 = List::cons(Str::lit("and_true"), List::cons(Str::lit("beq_iff_eq"), List::cons(Str::lit("congr"), List::cons(Str::lit("decide"), List::cons(Str::lit("decide_eq_false_iff_not"), List::cons(Str::lit("decide_eq_true_eq"), List::<Str>::nil()))))));
                let a95 = List::cons(Str::lit("id"), List::cons(Str::lit("reduceIte"), List::cons(Str::lit("rfl"), List::cons(Str::lit("true_and"), List::cons(Str::lit("isFalse"), List::cons(Str::lit("isTrue"), List::cons(Str::lit("none"), List::<Str>::nil())))))));
                append_list(a94, a95)
            };
            let a99 = {
                let a96 = List::cons(Str::lit("some"), List::cons(Str::lit("below"), List::cons(Str::lit("brecOn"), List::cons(Str::lit("casesOn"), List::cons(Str::lit("ctorElim"), List::cons(Str::lit("ctorElimType"), List::<Str>::nil()))))));
                let a97 = List::cons(Str::lit("ctorIdx"), List::cons(Str::lit("mk"), List::cons(Str::lit("noConfusion"), List::cons(Str::lit("noConfusionType"), List::cons(Str::lit("rec"), List::cons(Str::lit("recOn"), List::cons(Str::lit("toCtorIdx"), List::<Str>::nil())))))));
                append_list(a96, a97)
            };
            append_list(a98, a99)
        };
        append_list(a100, a101)
    };
    append_list(a102, a103)
}

pub fn f13(p108: List<Str>, v1: Str) -> bool {
    let m107 = p108;
    {
        let mut m107 = m107;
        loop {
            let step = {
                let v0 = m107;
                let m104 = v0;
                match m104.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a105 = v2;
                        let a106 = v1.clone();
                        if equal(a105, a106) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m107 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f14(p113: List<Str>, v1: Str) -> bool {
    let m112 = p113;
    {
        let mut m112 = m112;
        loop {
            let step = {
                let v0 = m112;
                let m109 = v0;
                match m109.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a110 = v2;
                        let a111 = v1.clone();
                        if equal(a110, a111) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m112 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f15(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m118 = {
        let a115 = v0;
        let a116 = v1;
        let a117 = {
            let a114 = v2.clone();
            length_bytes(a114)
        };
        slice_bytes(a115, a116, a117)
    };
    match m118 {
        None => {
            false
        }
        Some(v3) => {
            let a119 = v3;
            let a120 = v2;
            equal(a119, a120)
        }
    }
}

pub fn run(p0: Str, p1: bool, p2: List<Str>, p3: List<Adt0>) -> Option<Str> {
    f0(p0, p1, p2, p3)
}
