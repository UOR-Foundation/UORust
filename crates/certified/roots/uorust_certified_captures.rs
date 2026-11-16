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
    C0(Str, List<Str>, Adt1, Str, List<Adt3>, List<Adt4>, bool, Option<List<Str>>, List<Str>, List<Adt4>, List<Adt5>, Option<Str>, List<Adt4>, List<Adt55>, List<Adt55>, Option<Adt1>, Option<Str>),
}

#[derive(Clone)]
pub enum Adt1 {
    C0,
    C1,
    C2(Option<u64>),
    C3(u64),
    C4(Adt2),
    C5(Option<u64>),
    C6(Option<u64>),
    C7,
    C8,
    C9,
    C10,
    C11,
    C12(Str),
    C13(std::rc::Rc<Adt1>),
    C14(std::rc::Rc<Adt1>, std::rc::Rc<Adt1>),
    C15(std::rc::Rc<Adt1>),
    C16(List<Adt1>),
    C17(List<Adt1>),
    C18(std::rc::Rc<Adt1>, std::rc::Rc<Adt1>, bool),
    C19(std::rc::Rc<Adt1>, bool),
    C20(List<Adt1>, std::rc::Rc<Adt1>),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt3 {
    C0(Str, (Str, Adt1)),
}

#[derive(Clone)]
pub enum Adt4 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt5 {
    C0(Str, Adt6),
}

#[derive(Clone)]
pub enum Adt6 {
    C0(List<Adt7>, Adt22, List<Adt3>),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(std::rc::Rc<Adt8>),
    C1,
    C2(std::rc::Rc<Adt7>),
    C3(std::rc::Rc<Adt9>),
    C4(List<Adt7>),
    C5(Adt16),
    C6(Adt17),
    C7(List<Adt7>),
    C8,
    C9(Adt20),
    C10(std::rc::Rc<Adt21>),
    C11(Adt12),
    C12(List<Adt7>),
    C13(std::rc::Rc<Adt54>),
    C14,
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str, bool, std::rc::Rc<Option<Adt7>>),
}

#[derive(Clone)]
pub enum Adt9 {
    C0(std::rc::Rc<Adt7>, Adt10),
}

#[derive(Clone)]
pub enum Adt10 {
    C0(Str, std::rc::Rc<Adt11>),
}

#[derive(Clone)]
pub enum Adt11 {
    C0(bool, std::rc::Rc<Adt10>),
    C1(std::rc::Rc<Adt10>),
    C2(std::rc::Rc<Adt10>, Str),
    C3(List<Adt10>, std::rc::Rc<Adt10>),
    C4(std::rc::Rc<Adt10>),
    C5(List<Adt10>),
    C6(bool, Adt12),
    C7,
}

#[derive(Clone)]
pub enum Adt12 {
    C0(List<Adt13>, Str),
}

#[derive(Clone)]
pub enum Adt13 {
    C0(Str, Adt14),
}

#[derive(Clone)]
pub enum Adt14 {
    C0,
    C1(List<Adt15>),
    C2,
}

#[derive(Clone)]
pub enum Adt15 {
    C0(Adt10),
    C1(Str),
}

#[derive(Clone)]
pub enum Adt16 {
    C0(Adt12, List<Adt7>),
}

#[derive(Clone)]
pub enum Adt17 {
    C0(Adt12, List<Adt18>, bool),
}

#[derive(Clone)]
pub enum Adt18 {
    C0(Adt19, Adt7),
}

#[derive(Clone)]
pub enum Adt19 {
    C0(Str),
    C1(u64),
}

#[derive(Clone)]
pub enum Adt20 {
    C0(bool),
    C1(Option<u64>, Str),
    C2(Str),
    C3(Str),
    C4(u64),
    C5(List<u64>),
    C6,
}

#[derive(Clone)]
pub enum Adt21 {
    C0(std::rc::Rc<Option<Adt22>>, std::rc::Rc<Option<Adt22>>, bool),
}

#[derive(Clone)]
pub enum Adt22 {
    C0(List<Adt22>),
    C1(std::rc::Rc<Adt23>),
    C2(std::rc::Rc<Adt24>),
    C3(Adt26),
    C4(std::rc::Rc<Adt34>),
    C5(std::rc::Rc<Adt35>),
    C6(std::rc::Rc<Adt36>),
    C7(std::rc::Rc<Adt37>),
    C8(bool),
    C9(std::rc::Rc<Adt38>),
    C10(std::rc::Rc<Adt39>),
    C11(std::rc::Rc<Adt40>),
    C12(std::rc::Rc<Adt41>),
    C13(std::rc::Rc<Adt42>),
    C14(Adt20),
    C15(Adt43),
    C16(std::rc::Rc<Adt31>),
    C17(std::rc::Rc<Adt44>),
    C18(std::rc::Rc<Adt46>),
    C19(std::rc::Rc<Adt22>),
    C20(Adt12),
    C21(std::rc::Rc<Adt47>),
    C22(std::rc::Rc<Option<Adt22>>),
    C23(std::rc::Rc<Adt48>),
    C24(std::rc::Rc<Adt22>),
    C25(List<Adt22>),
    C26(std::rc::Rc<Adt50>),
    C27(std::rc::Rc<Adt52>),
    C28(Adt53),
}

#[derive(Clone)]
pub enum Adt23 {
    C0(std::rc::Rc<Adt22>, std::rc::Rc<Adt22>),
}

#[derive(Clone)]
pub enum Adt24 {
    C0(std::rc::Rc<Adt22>, Adt25, std::rc::Rc<Adt22>),
}

#[derive(Clone)]
pub enum Adt25 {
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
pub enum Adt26 {
    C0(bool, Adt27),
}

#[derive(Clone)]
pub enum Adt27 {
    C0(List<Adt28>),
}

#[derive(Clone)]
pub enum Adt28 {
    C0(Adt29),
    C1(Adt22, bool),
    C2(Adt31),
    C3,
}

#[derive(Clone)]
pub enum Adt29 {
    C0(Adt7, Option<Adt30>),
}

#[derive(Clone)]
pub enum Adt30 {
    C0(Adt22, Option<Adt22>),
}

#[derive(Clone)]
pub enum Adt31 {
    C0(Adt12, Option<Adt32>, std::rc::Rc<Option<Adt33>>),
}

#[derive(Clone)]
pub enum Adt32 {
    C0(List<Adt22>),
}

#[derive(Clone)]
pub enum Adt33 {
    C0(std::rc::Rc<Adt22>, std::rc::Rc<Adt7>),
}

#[derive(Clone)]
pub enum Adt34 {
    C0(bool, std::rc::Rc<Option<Adt22>>),
}

#[derive(Clone)]
pub enum Adt35 {
    C0(std::rc::Rc<Adt22>, List<Adt22>),
}

#[derive(Clone)]
pub enum Adt36 {
    C0(std::rc::Rc<Adt22>, Adt10),
}

#[derive(Clone)]
pub enum Adt37 {
    C0(List<Adt7>, std::rc::Rc<Adt22>, bool),
}

#[derive(Clone)]
pub enum Adt38 {
    C0(std::rc::Rc<Adt22>, Adt19),
}

#[derive(Clone)]
pub enum Adt39 {
    C0(bool, std::rc::Rc<Adt7>, std::rc::Rc<Adt22>, Adt27),
}

#[derive(Clone)]
pub enum Adt40 {
    C0(std::rc::Rc<Adt22>, Adt27, std::rc::Rc<Option<Adt22>>),
}

#[derive(Clone)]
pub enum Adt41 {
    C0(std::rc::Rc<Adt22>, std::rc::Rc<Adt22>),
}

#[derive(Clone)]
pub enum Adt42 {
    C0(std::rc::Rc<Adt7>, std::rc::Rc<Adt22>),
}

#[derive(Clone)]
pub enum Adt43 {
    C0(bool, Adt27),
}

#[derive(Clone)]
pub enum Adt44 {
    C0(std::rc::Rc<Adt22>, List<Adt45>),
}

#[derive(Clone)]
pub enum Adt45 {
    C0(Adt7, Adt22),
}

#[derive(Clone)]
pub enum Adt46 {
    C0(std::rc::Rc<Adt22>, Str, Option<List<Adt15>>, List<Adt22>),
}

#[derive(Clone)]
pub enum Adt47 {
    C0(bool, std::rc::Rc<Adt22>),
}

#[derive(Clone)]
pub enum Adt48 {
    C0(bool, Adt12, List<Adt49>, std::rc::Rc<Option<Adt22>>),
}

#[derive(Clone)]
pub enum Adt49 {
    C0(Adt19, Adt22),
}

#[derive(Clone)]
pub enum Adt50 {
    C0(Adt51, std::rc::Rc<Adt22>),
}

#[derive(Clone)]
pub enum Adt51 {
    C0,
    C1,
    C2,
}

#[derive(Clone)]
pub enum Adt52 {
    C0(bool, std::rc::Rc<Adt22>, Adt27),
}

#[derive(Clone)]
pub enum Adt53 {
    C0(Str, List<Adt22>, List<Adt27>),
}

#[derive(Clone)]
pub enum Adt54 {
    C0(std::rc::Rc<Adt7>, std::rc::Rc<Adt22>),
}

#[derive(Clone)]
pub enum Adt55 {
    C0(Str, u64),
}

pub fn f0(v0: Adt0, v1: List<Str>, v2: List<Str>) -> R<List<(Str, Adt1)>> {
    let v3: List<(Str, Adt1)> = f1(f2(match v0 {
        Adt0::C0(_, _, _, _, h1, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h1
        }
    }), v2, v1)?;
    Ok(v3)
}

pub fn f1(v0: List<(Str, Adt1)>, v1: List<Str>, v2: List<Str>) -> R<List<(Str, Adt1)>> {
    let m2 = v0;
    match m2.uncons() {
        None => {
            Ok(List::<(Str, Adt1)>::nil())
        }
        Some((v3, v4)) => {
            let v5: Str = {
                let (h3, _) = v3.clone();
                h3
            };
            let _: Adt1 = {
                let (_, h4) = v3.clone();
                h4
            };
            let a5 = f3(v1.clone(), v5.clone());
            if if bool_not(a5) {
                f4(v2.clone(), v5)?
            } else {
                false
            } {
                Ok(List::cons(v3, f1(v4, v1, v2)?))
            } else {
                f1(v4, v1, v2)
            }
        }
    }
}

pub fn f2(v0: List<Adt3>) -> List<(Str, Adt1)> {
    let m6 = v0;
    match m6.uncons() {
        None => {
            List::<(Str, Adt1)>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt3::C0(_, h7) => {
                    h7
                }
            }, f2(v2))
        }
    }
}

pub fn f3(p12: List<Str>, v1: Str) -> bool {
    let m11 = p12;
    {
        let mut m11 = m11;
        loop {
            let step = {
                let v0 = m11;
                let m8 = v0;
                match m8.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a9 = v2;
                        let a10 = v1.clone();
                        if equal(a9, a10) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m11 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f4(p15: List<Str>, v1: Str) -> R<bool> {
    let m14 = p15;
    {
        let mut m14 = m14;
        loop {
            let step = {
                let v0 = m14;
                let m13 = v0;
                match m13.uncons() {
                    None => {
                        Err::<List<Str>, R<bool>>(Ok(false))
                    }
                    Some((v2, v3)) => {
                        if f5(v2, v1.clone())? {
                            Err::<List<Str>, R<bool>>(Ok(true))
                        } else {
                            Ok::<List<Str>, R<bool>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m14 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f5(v0: Str, v1: Str) -> R<bool> {
    let a18 = {
        let a16 = v1.clone();
        let a17 = Str::lit("");
        equal(a16, a17)
    };
    if if if bool_not(a18) {
        f6(f7(v1.clone())?)?
    } else {
        false
    } {
        f8(v0.clone(), v1.clone())?
    } else {
        false
    } {
        f9(f10(v0, v1)?)
    } else {
        Ok(false)
    }
}

pub fn f6(p21: List<Str>) -> R<bool> {
    let m20 = p21;
    {
        let mut m20 = m20;
        loop {
            let step = {
                let v0 = m20;
                let m19 = v0;
                match m19.uncons() {
                    None => {
                        Err::<List<Str>, R<bool>>(Ok(true))
                    }
                    Some((v1, v2)) => {
                        if f11(v1)? {
                            Ok::<List<Str>, R<bool>>(v2)
                        } else {
                            Err::<List<Str>, R<bool>>(Ok(false))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m20 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f7(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a22 = v0;
        utf8_encode(a22)
    };
    f12({
        let a23 = v1.clone();
        length_bytes(a23)
    }, v1, 0u64)
}

pub fn f8(v0: Str, v1: Str) -> R<bool> {
    let m24 = f13(v0, v1)?;
    match m24 {
        None => {
            Ok(false)
        }
        Some(_) => {
            Ok(true)
        }
    }
}

pub fn f9(v0: List<Str>) -> R<bool> {
    f14(f15(v0.clone(), f16(v0, 1u64)))
}

pub fn f10(v0: Str, v1: Str) -> R<List<Str>> {
    let v2: Bytes = {
        let a25 = v0.clone();
        utf8_encode(a25)
    };
    let v3: Bytes = {
        let a26 = v1;
        utf8_encode(a26)
    };
    let a28 = {
        let a27 = v3.clone();
        length_bytes(a27)
    };
    let a29 = 0u64;
    if nat_eq(a28, a29) {
        Ok(List::cons(Str::lit(""), {
            let a30 = f7(v0)?;
            let a31 = List::cons(Str::lit(""), List::<Str>::nil());
            append_list(a30, a31)
        }))
    } else {
        f17({
            let a33 = {
                let a32 = v2.clone();
                length_bytes(a32)
            };
            let a34 = 1u64;
            nat_add(a33, a34)?
        }, v2, v3, 0u64, 0u64)
    }
}

pub fn f11(v0: Str) -> R<bool> {
    if f18(v0.clone())? {
        Ok(true)
    } else {
        let a35 = v0;
        let a36 = Str::lit("_");
        Ok(equal(a35, a36))
    }
}

pub fn f12(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m37 = v0;
    if m37 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m37 - 1;
        let a39 = {
            let a38 = v1.clone();
            length_bytes(a38)
        };
        let a40 = v2;
        if nat_le(a39, a40) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f19(f20(v1.clone(), v2)?);
            Ok(List::cons(f21(v1.clone(), v2, {
                let a41 = v2;
                let a42 = v4;
                nat_add(a41, a42)?
            }), f12(v3, v1, {
                let a43 = v2;
                let a44 = v4;
                nat_add(a43, a44)?
            })?))
        }
    }
}

pub fn f13(v0: Str, v1: Str) -> R<Option<u64>> {
    let v2: Bytes = {
        let a45 = v0;
        utf8_encode(a45)
    };
    f22({
        let a47 = {
            let a46 = v2.clone();
            length_bytes(a46)
        };
        let a48 = 1u64;
        nat_add(a47, a48)?
    }, v2, {
        let a49 = v1;
        utf8_encode(a49)
    }, 0u64)
}

pub fn f14(p56: List<Adt4>) -> R<bool> {
    let m55 = p56;
    {
        let mut m55 = m55;
        loop {
            let step = {
                let v0 = m55;
                let m50 = v0;
                match m50.uncons() {
                    None => {
                        Err::<List<Adt4>, R<bool>>(Ok(false))
                    }
                    Some((v1, v2)) => {
                        let v3: Str = match v1.clone() {
                            Adt4::C0(h51, _) => {
                                h51
                            }
                        };
                        let v4: Str = match v1 {
                            Adt4::C0(_, h52) => {
                                h52
                            }
                        };
                        let a53 = f23(v3)?;
                        if if bool_not(a53) {
                            let a54 = f24(v4)?;
                            bool_not(a54)
                        } else {
                            false
                        } {
                            Err::<List<Adt4>, R<bool>>(Ok(true))
                        } else {
                            Ok::<List<Adt4>, R<bool>>(v2)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m55 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f15(v0: List<Str>, v1: List<Str>) -> List<Adt4> {
    let m57 = v0;
    match m57.uncons() {
        None => {
            List::<Adt4>::nil()
        }
        Some((v2, v3)) => {
            let m58 = v1;
            match m58.uncons() {
                None => {
                    List::<Adt4>::nil()
                }
                Some((v4, v5)) => {
                    List::cons(Adt4::C0(v2, v4), f15(v3, v5))
                }
            }
        }
    }
}

pub fn f16(v0: List<Str>, v1: u64) -> List<Str> {
    let m59 = v0;
    match m59.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            let a60 = v1;
            let a61 = 0u64;
            if nat_eq(a60, a61) {
                List::cons(v2, f16(v3, v1))
            } else {
                f16(v3, {
                    let a62 = v1;
                    let a63 = 1u64;
                    nat_sub(a62, a63)
                })
            }
        }
    }
}

pub fn f17(v0: u64, v1: Bytes, v2: Bytes, v3: u64, v4: u64) -> R<List<Str>> {
    let m64 = v0;
    if m64 == 0 {
        Ok(List::cons(f21(v1.clone(), v3, {
            let a65 = v1;
            length_bytes(a65)
        }), List::<Str>::nil()))
    } else {
        let v5 = m64 - 1;
        let a70 = {
            let a66 = v1.clone();
            length_bytes(a66)
        };
        let a71 = {
            let a68 = v4;
            let a69 = {
                let a67 = v2.clone();
                length_bytes(a67)
            };
            nat_add(a68, a69)?
        };
        if nat_lt(a70, a71) {
            Ok(List::cons(f21(v1.clone(), v3, {
                let a72 = v1;
                length_bytes(a72)
            }), List::<Str>::nil()))
        } else {
            if f25(v1.clone(), v4, v2.clone()) {
                Ok(List::cons(f21(v1.clone(), v3, v4), f17(v5, v1, v2.clone(), {
                    let a74 = v4;
                    let a75 = {
                        let a73 = v2.clone();
                        length_bytes(a73)
                    };
                    nat_add(a74, a75)?
                }, {
                    let a77 = v4;
                    let a78 = {
                        let a76 = v2;
                        length_bytes(a76)
                    };
                    nat_add(a77, a78)?
                })?))
            } else {
                f17(v5, v1, v2, v3, {
                    let a79 = v4;
                    let a80 = 1u64;
                    nat_add(a79, a80)?
                })
            }
        }
    }
}

pub fn f18(v0: Str) -> R<bool> {
    if f26(v0.clone())? {
        Ok(true)
    } else {
        f27(v0)
    }
}

pub fn f19(v0: u64) -> u64 {
    let a81 = v0;
    let a82 = 128u64;
    if nat_lt(a81, a82) {
        1u64
    } else {
        let a83 = v0;
        let a84 = 224u64;
        if nat_lt(a83, a84) {
            2u64
        } else {
            let a85 = v0;
            let a86 = 240u64;
            if nat_lt(a85, a86) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f20(v0: Bytes, v1: u64) -> R<u64> {
    let m89 = {
        let a87 = v0;
        let a88 = v1;
        index_bytes(a87, a88)
    };
    match m89 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a134 = {
                let a110 = {
                    let a98 = {
                        let a92 = {
                            let a90 = v3;
                            let a91 = 128u8;
                            fixed_u8::bit_and(a90, a91)
                        };
                        let a93 = 128u8;
                        if equal(a92, a93) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a99 = {
                        let a96 = {
                            let a94 = v3;
                            let a95 = 64u8;
                            fixed_u8::bit_and(a94, a95)
                        };
                        let a97 = 64u8;
                        if equal(a96, a97) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a98, a99)?
                };
                let a111 = {
                    let a108 = {
                        let a102 = {
                            let a100 = v3;
                            let a101 = 32u8;
                            fixed_u8::bit_and(a100, a101)
                        };
                        let a103 = 32u8;
                        if equal(a102, a103) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a109 = {
                        let a106 = {
                            let a104 = v3;
                            let a105 = 16u8;
                            fixed_u8::bit_and(a104, a105)
                        };
                        let a107 = 16u8;
                        if equal(a106, a107) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a108, a109)?
                };
                nat_add(a110, a111)?
            };
            let a135 = {
                let a132 = {
                    let a120 = {
                        let a114 = {
                            let a112 = v3;
                            let a113 = 8u8;
                            fixed_u8::bit_and(a112, a113)
                        };
                        let a115 = 8u8;
                        if equal(a114, a115) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a121 = {
                        let a118 = {
                            let a116 = v3;
                            let a117 = 4u8;
                            fixed_u8::bit_and(a116, a117)
                        };
                        let a119 = 4u8;
                        if equal(a118, a119) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a120, a121)?
                };
                let a133 = {
                    let a130 = {
                        let a124 = {
                            let a122 = v3;
                            let a123 = 2u8;
                            fixed_u8::bit_and(a122, a123)
                        };
                        let a125 = 2u8;
                        if equal(a124, a125) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a131 = {
                        let a128 = {
                            let a126 = v3;
                            let a127 = 1u8;
                            fixed_u8::bit_and(a126, a127)
                        };
                        let a129 = 1u8;
                        if equal(a128, a129) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a130, a131)?
                };
                nat_add(a132, a133)?
            };
            nat_add(a134, a135)
        }
    }
}

pub fn f21(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m141 = {
        let a138 = v0;
        let a139 = v1;
        let a140 = {
            let a136 = v2;
            let a137 = v1;
            nat_sub(a136, a137)
        };
        slice_bytes(a138, a139, a140)
    };
    match m141 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m143 = {
                let a142 = v3;
                utf8_decode(a142)
            };
            match m143 {
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

pub fn f22(p154: u64, v1: Bytes, v2: Bytes, p155: u64) -> R<Option<u64>> {
    let m153 = (p154, p155);
    {
        let mut m153 = m153;
        loop {
            let step = {
                let (v0, v3) = m153;
                let m144 = v0;
                if m144 == 0 {
                    Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                } else {
                    let v4 = m144 - 1;
                    let a149 = {
                        let a145 = v1.clone();
                        length_bytes(a145)
                    };
                    let a150 = {
                        let a147 = v3;
                        let a148 = {
                            let a146 = v2.clone();
                            length_bytes(a146)
                        };
                        nat_add(a147, a148)?
                    };
                    if nat_lt(a149, a150) {
                        Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                    } else {
                        if f25(v1.clone(), v3, v2.clone()) {
                            Err::<(u64, u64), R<Option<u64>>>(Ok(Some(v3)))
                        } else {
                            Ok::<(u64, u64), R<Option<u64>>>((v4, {
                                let a151 = v3;
                                let a152 = 1u64;
                                nat_add(a151, a152)?
                            }))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m153 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f23(v0: Str) -> R<bool> {
    let m161 = {
        let a159 = f7(v0.clone())?;
        let a160 = {
            let a157 = {
                let a156 = f7(v0)?;
                length_list(a156)
            };
            let a158 = 1u64;
            nat_sub(a157, a158)
        };
        index_list(a159, a160)
    };
    match m161 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            f11(v1)
        }
    }
}

pub fn f24(v0: Str) -> R<bool> {
    let m164 = {
        let a162 = f7(v0)?;
        let a163 = 0u64;
        index_list(a162, a163)
    };
    match m164 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            f11(v1)
        }
    }
}

pub fn f25(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m169 = {
        let a166 = v0;
        let a167 = v1;
        let a168 = {
            let a165 = v2.clone();
            length_bytes(a165)
        };
        slice_bytes(a166, a167, a168)
    };
    match m169 {
        None => {
            false
        }
        Some(v3) => {
            let a170 = v3;
            let a171 = v2;
            equal(a170, a171)
        }
    }
}

pub fn f26(v0: Str) -> R<bool> {
    if f28(v0.clone())? {
        Ok(true)
    } else {
        f29(v0)
    }
}

pub fn f27(v0: Str) -> R<bool> {
    let v1: u64 = f30(v0)?;
    let a172 = 48u64;
    let a173 = v1;
    if nat_le(a172, a173) {
        let a174 = v1;
        let a175 = 57u64;
        Ok(nat_le(a174, a175))
    } else {
        Ok(false)
    }
}

pub fn f28(v0: Str) -> R<bool> {
    let v1: u64 = f30(v0)?;
    let a176 = 65u64;
    let a177 = v1;
    if nat_le(a176, a177) {
        let a178 = v1;
        let a179 = 90u64;
        Ok(nat_le(a178, a179))
    } else {
        Ok(false)
    }
}

pub fn f29(v0: Str) -> R<bool> {
    let v1: u64 = f30(v0)?;
    let a180 = 97u64;
    let a181 = v1;
    if nat_le(a180, a181) {
        let a182 = v1;
        let a183 = 122u64;
        Ok(nat_le(a182, a183))
    } else {
        Ok(false)
    }
}

pub fn f30(v0: Str) -> R<u64> {
    f20({
        let a184 = v0;
        utf8_encode(a184)
    }, 0u64)
}

pub fn run(p0: Adt0, p1: List<Str>, p2: List<Str>) -> R<List<(Str, Adt1)>> {
    f0(p0, p1, p2)
}
