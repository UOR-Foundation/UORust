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
    C0(List<Adt1>, Str),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Str, Adt2),
}

#[derive(Clone)]
pub enum Adt2 {
    C0,
    C1(List<Adt3>),
    C2,
}

#[derive(Clone)]
pub enum Adt3 {
    C0(Adt4),
    C1(Str),
}

#[derive(Clone)]
pub enum Adt4 {
    C0(Str, std::rc::Rc<Adt5>),
}

#[derive(Clone)]
pub enum Adt5 {
    C0(bool, std::rc::Rc<Adt4>),
    C1(std::rc::Rc<Adt4>),
    C2(std::rc::Rc<Adt4>, Str),
    C3(List<Adt4>, std::rc::Rc<Adt4>),
    C4(std::rc::Rc<Adt4>),
    C5(List<Adt4>),
    C6(bool, Adt0),
    C7,
}

#[derive(Clone)]
pub enum Adt6 {
    C0(List<Adt7>, List<Adt8>, List<Adt11>, List<Adt12>, List<List<Str>>, List<Adt7>, List<Adt7>, bool, List<Adt13>, List<Adt14>, List<Adt15>, List<Adt17>, List<Adt11>),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str, List<(Str, Adt9)>),
}

#[derive(Clone)]
pub enum Adt9 {
    C0,
    C1,
    C2(Option<u64>),
    C3(u64),
    C4(Adt10),
    C5(Option<u64>),
    C6(Option<u64>),
    C7,
    C8,
    C9,
    C10,
    C11,
    C12(Str),
    C13(std::rc::Rc<Adt9>),
    C14(std::rc::Rc<Adt9>, std::rc::Rc<Adt9>),
    C15(std::rc::Rc<Adt9>),
    C16(List<Adt9>),
    C17(List<Adt9>),
    C18(std::rc::Rc<Adt9>, std::rc::Rc<Adt9>, bool),
    C19(std::rc::Rc<Adt9>, bool),
    C20(List<Adt9>, std::rc::Rc<Adt9>),
}

#[derive(Clone)]
pub enum Adt10 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt11 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt12 {
    C0(Str, List<(Str, List<(Option<Str>, Adt9)>)>),
}

#[derive(Clone)]
pub enum Adt13 {
    C0(Str, Adt10),
}

#[derive(Clone)]
pub enum Adt14 {
    C0(Str, Adt11),
}

#[derive(Clone)]
pub enum Adt15 {
    C0(Str, List<Adt16>),
}

#[derive(Clone)]
pub enum Adt16 {
    C0(u64, Str),
}

#[derive(Clone)]
pub enum Adt17 {
    C0(Str, List<i64>),
}

pub fn f0(v0: Adt0, v1: Adt6) -> R<Option<Adt11>> {
    let v2: u64 = {
        let a1 = f1(v0.clone());
        length_list(a1)
    };
    let a2 = v2;
    let a3 = 1u64;
    if nat_eq(a2, a3) {
        let v3: Str = f2(v1.clone(), f3(f4(v0)))?;
        let v4: bool = f5(v3.clone(), v1);
        if v4 {
            Ok(Some(Adt11::C0(v3, f6())))
        } else {
            Ok(None::<Adt11>)
        }
    } else {
        let a4 = v2;
        let a5 = 2u64;
        if nat_lt(a4, a5) {
            Ok(None::<Adt11>)
        } else {
            let v5: Str = f2(v1.clone(), f3(f7(v2, v0.clone())))?;
            let v6: Str = f3(f8(v2, v0));
            let v7: bool = if f9(v5.clone(), v1.clone(), v6.clone()) {
                true
            } else {
                f10(v5.clone(), v1, v6.clone())
            };
            if v7 {
                Ok(Some(Adt11::C0(v5, v6)))
            } else {
                Ok(None::<Adt11>)
            }
        }
    }
}

pub fn f1(v0: Adt0) -> List<Adt1> {
    let m6 = v0;
    match m6 {
        Adt0::C0(v1, _) => {
            v1
        }
    }
}

pub fn f2(v0: Adt6, v1: Str) -> R<Str> {
    let v2: Str = f11(v1)?;
    let m8 = f12(match v0 {
        Adt6::C0(_, _, h7, _, _, _, _, _, _, _, _, _, _) => {
            h7
        }
    }, v2.clone());
    match m8 {
        None => {
            Ok(v2)
        }
        Some(v3) => {
            Ok(v3)
        }
    }
}

pub fn f3(v0: Adt1) -> Str {
    let m9 = v0;
    match m9 {
        Adt1::C0(v1, _) => {
            v1
        }
    }
}

pub fn f4(v0: Adt0) -> Adt1 {
    let m12 = {
        let a10 = f1(v0);
        let a11 = 0u64;
        index_list(a10, a11)
    };
    match m12 {
        None => {
            Adt1::C0(Str::lit(""), Adt2::C0)
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f5(v0: Str, v1: Adt6) -> bool {
    let m14 = f13(match v1 {
        Adt6::C0(h13, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h13
        }
    }, v0);
    match m14 {
        None => {
            false
        }
        Some(v2) => {
            let a16 = {
                let a15 = v2.clone();
                length_list(a15)
            };
            let a17 = 1u64;
            if nat_eq(a16, a17) {
                f14(v2)
            } else {
                false
            }
        }
    }
}

pub fn f6() -> Str {
    Str::lit("mk")
}

pub fn f7(v0: u64, v1: Adt0) -> Adt1 {
    let m22 = {
        let a20 = f1(v1);
        let a21 = {
            let a18 = v0;
            let a19 = 2u64;
            nat_sub(a18, a19)
        };
        index_list(a20, a21)
    };
    match m22 {
        None => {
            Adt1::C0(Str::lit(""), Adt2::C0)
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f8(v0: u64, v1: Adt0) -> Adt1 {
    let m27 = {
        let a25 = f1(v1);
        let a26 = {
            let a23 = v0;
            let a24 = 1u64;
            nat_sub(a23, a24)
        };
        index_list(a25, a26)
    };
    match m27 {
        None => {
            Adt1::C0(Str::lit(""), Adt2::C0)
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f9(v0: Str, v1: Adt6, v2: Str) -> bool {
    let m29 = f15(match v1 {
        Adt6::C0(h28, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h28
        }
    }, v0);
    match m29 {
        None => {
            false
        }
        Some(v3) => {
            f16(v3, v2)
        }
    }
}

pub fn f10(v0: Str, v1: Adt6, v2: Str) -> bool {
    let m31 = f17(match v1 {
        Adt6::C0(_, _, _, h30, _, _, _, _, _, _, _, _, _) => {
            h30
        }
    }, v0);
    match m31 {
        None => {
            false
        }
        Some(v3) => {
            f18(v3, v2)
        }
    }
}

pub fn f11(v0: Str) -> R<Str> {
    let m32 = f19(v0.clone())?;
    match m32 {
        None => {
            Ok(v0)
        }
        Some(v1) => {
            let v2: Str = match v1.clone() {
                Adt11::C0(h33, _) => {
                    h33
                }
            };
            let v3: Str = match v1 {
                Adt11::C0(_, h34) => {
                    h34
                }
            };
            let m35 = f20(v3, Str::lit(">"));
            match m35 {
                None => {
                    Ok(v0)
                }
                Some(v4) => {
                    if f21(f22(v4, Str::lit(","))?)? {
                        f23(v2)
                    } else {
                        Ok(v0)
                    }
                }
            }
        }
    }
}

pub fn f12(p42: List<Adt11>, v1: Str) -> Option<Str> {
    let m41 = p42;
    {
        let mut m41 = m41;
        loop {
            let step = {
                let v0 = m41;
                let m36 = v0;
                match m36.uncons() {
                    None => {
                        Err::<List<Adt11>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a38 = match v2.clone() {
                            Adt11::C0(h37, _) => {
                                h37
                            }
                        };
                        let a39 = v1.clone();
                        if equal(a38, a39) {
                            Err::<List<Adt11>, Option<Str>>(Some(match v2 {
                                Adt11::C0(_, h40) => {
                                    h40
                                }
                            }))
                        } else {
                            Ok::<List<Adt11>, Option<Str>>(v3)
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

pub fn f13(p49: List<Adt7>, v1: Str) -> Option<List<Str>> {
    let m48 = p49;
    {
        let mut m48 = m48;
        loop {
            let step = {
                let v0 = m48;
                let m43 = v0;
                match m43.uncons() {
                    None => {
                        Err::<List<Adt7>, Option<List<Str>>>(None::<List<Str>>)
                    }
                    Some((v2, v3)) => {
                        let a45 = match v2.clone() {
                            Adt7::C0(h44, _) => {
                                h44
                            }
                        };
                        let a46 = v1.clone();
                        if equal(a45, a46) {
                            Err::<List<Adt7>, Option<List<Str>>>(Some(match v2 {
                                Adt7::C0(_, h47) => {
                                    h47
                                }
                            }))
                        } else {
                            Ok::<List<Adt7>, Option<List<Str>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m48 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f14(p54: List<Str>) -> bool {
    let m53 = p54;
    {
        let mut m53 = m53;
        loop {
            let step = {
                let v0 = m53;
                let m50 = v0;
                match m50.uncons() {
                    None => {
                        Err::<List<Str>, bool>(true)
                    }
                    Some((v1, v2)) => {
                        let a51 = v1;
                        let a52 = f6();
                        if equal(a51, a52) {
                            Ok::<List<Str>, bool>(v2)
                        } else {
                            Err::<List<Str>, bool>(false)
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

pub fn f15(p61: List<Adt7>, v1: Str) -> Option<List<Str>> {
    let m60 = p61;
    {
        let mut m60 = m60;
        loop {
            let step = {
                let v0 = m60;
                let m55 = v0;
                match m55.uncons() {
                    None => {
                        Err::<List<Adt7>, Option<List<Str>>>(None::<List<Str>>)
                    }
                    Some((v2, v3)) => {
                        let a57 = match v2.clone() {
                            Adt7::C0(h56, _) => {
                                h56
                            }
                        };
                        let a58 = v1.clone();
                        if equal(a57, a58) {
                            Err::<List<Adt7>, Option<List<Str>>>(Some(match v2 {
                                Adt7::C0(_, h59) => {
                                    h59
                                }
                            }))
                        } else {
                            Ok::<List<Adt7>, Option<List<Str>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m60 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f16(p66: List<Str>, v1: Str) -> bool {
    let m65 = p66;
    {
        let mut m65 = m65;
        loop {
            let step = {
                let v0 = m65;
                let m62 = v0;
                match m62.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a63 = v2;
                        let a64 = v1.clone();
                        if equal(a63, a64) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m65 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f17(p73: List<Adt12>, v1: Str) -> Option<List<(Str, List<(Option<Str>, Adt9)>)>> {
    let m72 = p73;
    {
        let mut m72 = m72;
        loop {
            let step = {
                let v0 = m72;
                let m67 = v0;
                match m67.uncons() {
                    None => {
                        Err::<List<Adt12>, Option<List<(Str, List<(Option<Str>, Adt9)>)>>>(None::<List<(Str, List<(Option<Str>, Adt9)>)>>)
                    }
                    Some((v2, v3)) => {
                        let a69 = match v2.clone() {
                            Adt12::C0(h68, _) => {
                                h68
                            }
                        };
                        let a70 = v1.clone();
                        if equal(a69, a70) {
                            Err::<List<Adt12>, Option<List<(Str, List<(Option<Str>, Adt9)>)>>>(Some(match v2 {
                                Adt12::C0(_, h71) => {
                                    h71
                                }
                            }))
                        } else {
                            Ok::<List<Adt12>, Option<List<(Str, List<(Option<Str>, Adt9)>)>>>(v3)
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

pub fn f18(p83: List<(Str, List<(Option<Str>, Adt9)>)>, v1: Str) -> bool {
    let m82 = p83;
    {
        let mut m82 = m82;
        loop {
            let step = {
                let v0 = m82;
                let m74 = v0;
                match m74.uncons() {
                    None => {
                        Err::<List<(Str, List<(Option<Str>, Adt9)>)>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let v4: Str = {
                            let (h75, _) = v2;
                            h75
                        };
                        let v5: List<(Option<Str>, Adt9)> = {
                            let (_, h76) = v2;
                            h76
                        };
                        let a77 = v4;
                        let a78 = v1.clone();
                        if if equal(a77, a78) {
                            let a80 = {
                                let a79 = v5;
                                length_list(a79)
                            };
                            let a81 = 0u64;
                            nat_eq(a80, a81)
                        } else {
                            false
                        } {
                            Err::<List<(Str, List<(Option<Str>, Adt9)>)>, bool>(true)
                        } else {
                            Ok::<List<(Str, List<(Option<Str>, Adt9)>)>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m82 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f19(v0: Str) -> R<Option<Adt11>> {
    let m84 = f24(v0.clone(), Str::lit("<"))?;
    match m84 {
        None => {
            Ok(None::<Adt11>)
        }
        Some(v1) => {
            Ok(Some(Adt11::C0(f25(v0.clone(), 0u64, v1), f25(v0.clone(), {
                let a85 = v1;
                let a86 = f26(Str::lit("<"));
                nat_add(a85, a86)?
            }, f26(v0)))))
        }
    }
}

pub fn f20(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a87 = v0.clone();
        utf8_encode(a87)
    };
    let v3: u64 = {
        let a88 = v2.clone();
        length_bytes(a88)
    };
    let v4: u64 = f26(v1.clone());
    if f27(v0, v1) {
        Some(f28(v2, 0u64, {
            let a89 = v3;
            let a90 = v4;
            nat_sub(a89, a90)
        }))
    } else {
        None::<Str>
    }
}

pub fn f21(p93: List<Str>) -> R<bool> {
    let m92 = p93;
    {
        let mut m92 = m92;
        loop {
            let step = {
                let v0 = m92;
                let m91 = v0;
                match m91.uncons() {
                    None => {
                        Err::<List<Str>, R<bool>>(Ok(true))
                    }
                    Some((v1, v2)) => {
                        if f29(v1)? {
                            Ok::<List<Str>, R<bool>>(v2)
                        } else {
                            Err::<List<Str>, R<bool>>(Ok(false))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m92 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f22(v0: Str, v1: Str) -> R<List<Str>> {
    let v2: Bytes = {
        let a94 = v0.clone();
        utf8_encode(a94)
    };
    let v3: Bytes = {
        let a95 = v1;
        utf8_encode(a95)
    };
    let a97 = {
        let a96 = v3.clone();
        length_bytes(a96)
    };
    let a98 = 0u64;
    if nat_eq(a97, a98) {
        Ok(List::cons(Str::lit(""), {
            let a99 = f30(v0)?;
            let a100 = List::cons(Str::lit(""), List::<Str>::nil());
            append_list(a99, a100)
        }))
    } else {
        f31({
            let a102 = {
                let a101 = v2.clone();
                length_bytes(a101)
            };
            let a103 = 1u64;
            nat_add(a102, a103)?
        }, v2, v3, 0u64, 0u64)
    }
}

pub fn f23(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a104 = v0;
        utf8_encode(a104)
    };
    let v2: u64 = {
        let a105 = v1.clone();
        length_bytes(a105)
    };
    Ok(f28(v1.clone(), 0u64, f32(v2, v1, v2)?))
}

pub fn f24(v0: Str, v1: Str) -> R<Option<u64>> {
    let v2: Bytes = {
        let a106 = v0;
        utf8_encode(a106)
    };
    f33({
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

pub fn f25(v0: Str, v1: u64, v2: u64) -> Str {
    f28({
        let a111 = v0;
        utf8_encode(a111)
    }, v1, v2)
}

pub fn f26(v0: Str) -> u64 {
    let a113 = {
        let a112 = v0;
        utf8_encode(a112)
    };
    length_bytes(a113)
}

pub fn f27(v0: Str, v1: Str) -> bool {
    let v2: Bytes = {
        let a114 = v0;
        utf8_encode(a114)
    };
    let v3: Bytes = {
        let a115 = v1;
        utf8_encode(a115)
    };
    let v4: u64 = {
        let a116 = v2.clone();
        length_bytes(a116)
    };
    let v5: u64 = {
        let a117 = v3.clone();
        length_bytes(a117)
    };
    let a118 = v5;
    let a119 = v4;
    if nat_le(a118, a119) {
        f34(v2, {
            let a120 = v4;
            let a121 = v5;
            nat_sub(a120, a121)
        }, v3)
    } else {
        false
    }
}

pub fn f28(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m127 = {
        let a124 = v0;
        let a125 = v1;
        let a126 = {
            let a122 = v2;
            let a123 = v1;
            nat_sub(a122, a123)
        };
        slice_bytes(a124, a125, a126)
    };
    match m127 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m129 = {
                let a128 = v3;
                utf8_decode(a128)
            };
            match m129 {
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

pub fn f29(v0: Str) -> R<bool> {
    let m130 = f35(f36(v0)?, Str::lit("'"));
    match m130 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            let a131 = f37(v1, Str::lit("'"))?;
            Ok(bool_not(a131))
        }
    }
}

pub fn f30(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a132 = v0;
        utf8_encode(a132)
    };
    f38({
        let a133 = v1.clone();
        length_bytes(a133)
    }, v1, 0u64)
}

pub fn f31(v0: u64, v1: Bytes, v2: Bytes, v3: u64, v4: u64) -> R<List<Str>> {
    let m134 = v0;
    if m134 == 0 {
        Ok(List::cons(f28(v1.clone(), v3, {
            let a135 = v1;
            length_bytes(a135)
        }), List::<Str>::nil()))
    } else {
        let v5 = m134 - 1;
        let a140 = {
            let a136 = v1.clone();
            length_bytes(a136)
        };
        let a141 = {
            let a138 = v4;
            let a139 = {
                let a137 = v2.clone();
                length_bytes(a137)
            };
            nat_add(a138, a139)?
        };
        if nat_lt(a140, a141) {
            Ok(List::cons(f28(v1.clone(), v3, {
                let a142 = v1;
                length_bytes(a142)
            }), List::<Str>::nil()))
        } else {
            if f34(v1.clone(), v4, v2.clone()) {
                Ok(List::cons(f28(v1.clone(), v3, v4), f31(v5, v1, v2.clone(), {
                    let a144 = v4;
                    let a145 = {
                        let a143 = v2.clone();
                        length_bytes(a143)
                    };
                    nat_add(a144, a145)?
                }, {
                    let a147 = v4;
                    let a148 = {
                        let a146 = v2;
                        length_bytes(a146)
                    };
                    nat_add(a147, a148)?
                })?))
            } else {
                f31(v5, v1, v2, v3, {
                    let a149 = v4;
                    let a150 = 1u64;
                    nat_add(a149, a150)?
                })
            }
        }
    }
}

pub fn f32(p157: u64, v1: Bytes, p158: u64) -> R<u64> {
    let m156 = (p157, p158);
    {
        let mut m156 = m156;
        loop {
            let step = {
                let (v0, v2) = m156;
                let m151 = v0;
                if m151 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m151 - 1;
                    let v4: u64 = f39(v1.clone(), v2)?;
                    let a152 = v4;
                    let a153 = 0u64;
                    if nat_eq(a152, a153) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a154 = v2;
                            let a155 = v4;
                            nat_sub(a154, a155)
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m156 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f33(p169: u64, v1: Bytes, v2: Bytes, p170: u64) -> R<Option<u64>> {
    let m168 = (p169, p170);
    {
        let mut m168 = m168;
        loop {
            let step = {
                let (v0, v3) = m168;
                let m159 = v0;
                if m159 == 0 {
                    Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                } else {
                    let v4 = m159 - 1;
                    let a164 = {
                        let a160 = v1.clone();
                        length_bytes(a160)
                    };
                    let a165 = {
                        let a162 = v3;
                        let a163 = {
                            let a161 = v2.clone();
                            length_bytes(a161)
                        };
                        nat_add(a162, a163)?
                    };
                    if nat_lt(a164, a165) {
                        Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                    } else {
                        if f34(v1.clone(), v3, v2.clone()) {
                            Err::<(u64, u64), R<Option<u64>>>(Ok(Some(v3)))
                        } else {
                            Ok::<(u64, u64), R<Option<u64>>>((v4, {
                                let a166 = v3;
                                let a167 = 1u64;
                                nat_add(a166, a167)?
                            }))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m168 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f34(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m175 = {
        let a172 = v0;
        let a173 = v1;
        let a174 = {
            let a171 = v2.clone();
            length_bytes(a171)
        };
        slice_bytes(a172, a173, a174)
    };
    match m175 {
        None => {
            false
        }
        Some(v3) => {
            let a176 = v3;
            let a177 = v2;
            equal(a176, a177)
        }
    }
}

pub fn f35(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a178 = v0;
        utf8_encode(a178)
    };
    let v3: Bytes = {
        let a179 = v1;
        utf8_encode(a179)
    };
    if f34(v2.clone(), 0u64, v3.clone()) {
        Some(f28(v2.clone(), {
            let a180 = v3;
            length_bytes(a180)
        }, {
            let a181 = v2;
            length_bytes(a181)
        }))
    } else {
        None::<Str>
    }
}

pub fn f36(v0: Str) -> R<Str> {
    f23(f40(v0)?)
}

pub fn f37(v0: Str, v1: Str) -> R<bool> {
    let m182 = f24(v0, v1)?;
    match m182 {
        None => {
            Ok(false)
        }
        Some(_) => {
            Ok(true)
        }
    }
}

pub fn f38(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m183 = v0;
    if m183 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m183 - 1;
        let a185 = {
            let a184 = v1.clone();
            length_bytes(a184)
        };
        let a186 = v2;
        if nat_le(a185, a186) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f41(f42(v1.clone(), v2)?);
            Ok(List::cons(f28(v1.clone(), v2, {
                let a187 = v2;
                let a188 = v4;
                nat_add(a187, a188)?
            }), f38(v3, v1, {
                let a189 = v2;
                let a190 = v4;
                nat_add(a189, a190)?
            })?))
        }
    }
}

pub fn f39(v0: Bytes, v1: u64) -> R<u64> {
    let a191 = v1;
    let a192 = 0u64;
    if nat_eq(a191, a192) {
        Ok(0u64)
    } else {
        if f43(f42(v0.clone(), {
            let a193 = v1;
            let a194 = 1u64;
            nat_sub(a193, a194)
        })?) {
            Ok(1u64)
        } else {
            let a195 = 2u64;
            let a196 = v1;
            if if nat_le(a195, a196) {
                let a199 = f44(v0.clone(), {
                    let a197 = v1;
                    let a198 = 2u64;
                    nat_sub(a197, a198)
                })?;
                let a200 = 2u64;
                nat_eq(a199, a200)
            } else {
                false
            } {
                Ok(2u64)
            } else {
                let a201 = 3u64;
                let a202 = v1;
                if if nat_le(a201, a202) {
                    let a205 = f44(v0, {
                        let a203 = v1;
                        let a204 = 3u64;
                        nat_sub(a203, a204)
                    })?;
                    let a206 = 3u64;
                    nat_eq(a205, a206)
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

pub fn f40(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a207 = v0;
        utf8_encode(a207)
    };
    let v2: u64 = {
        let a208 = v1.clone();
        length_bytes(a208)
    };
    Ok(f28(v1.clone(), f45(v2, v1, 0u64)?, v2))
}

pub fn f41(v0: u64) -> u64 {
    let a209 = v0;
    let a210 = 128u64;
    if nat_lt(a209, a210) {
        1u64
    } else {
        let a211 = v0;
        let a212 = 224u64;
        if nat_lt(a211, a212) {
            2u64
        } else {
            let a213 = v0;
            let a214 = 240u64;
            if nat_lt(a213, a214) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f42(v0: Bytes, v1: u64) -> R<u64> {
    let m217 = {
        let a215 = v0;
        let a216 = v1;
        index_bytes(a215, a216)
    };
    match m217 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a262 = {
                let a238 = {
                    let a226 = {
                        let a220 = {
                            let a218 = v3;
                            let a219 = 128u8;
                            fixed_u8::bit_and(a218, a219)
                        };
                        let a221 = 128u8;
                        if equal(a220, a221) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a227 = {
                        let a224 = {
                            let a222 = v3;
                            let a223 = 64u8;
                            fixed_u8::bit_and(a222, a223)
                        };
                        let a225 = 64u8;
                        if equal(a224, a225) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a226, a227)?
                };
                let a239 = {
                    let a236 = {
                        let a230 = {
                            let a228 = v3;
                            let a229 = 32u8;
                            fixed_u8::bit_and(a228, a229)
                        };
                        let a231 = 32u8;
                        if equal(a230, a231) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a237 = {
                        let a234 = {
                            let a232 = v3;
                            let a233 = 16u8;
                            fixed_u8::bit_and(a232, a233)
                        };
                        let a235 = 16u8;
                        if equal(a234, a235) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a236, a237)?
                };
                nat_add(a238, a239)?
            };
            let a263 = {
                let a260 = {
                    let a248 = {
                        let a242 = {
                            let a240 = v3;
                            let a241 = 8u8;
                            fixed_u8::bit_and(a240, a241)
                        };
                        let a243 = 8u8;
                        if equal(a242, a243) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a249 = {
                        let a246 = {
                            let a244 = v3;
                            let a245 = 4u8;
                            fixed_u8::bit_and(a244, a245)
                        };
                        let a247 = 4u8;
                        if equal(a246, a247) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a248, a249)?
                };
                let a261 = {
                    let a258 = {
                        let a252 = {
                            let a250 = v3;
                            let a251 = 2u8;
                            fixed_u8::bit_and(a250, a251)
                        };
                        let a253 = 2u8;
                        if equal(a252, a253) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a259 = {
                        let a256 = {
                            let a254 = v3;
                            let a255 = 1u8;
                            fixed_u8::bit_and(a254, a255)
                        };
                        let a257 = 1u8;
                        if equal(a256, a257) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a258, a259)?
                };
                nat_add(a260, a261)?
            };
            nat_add(a262, a263)
        }
    }
}

pub fn f43(v0: u64) -> bool {
    let a264 = v0;
    let a265 = 32u64;
    if nat_eq(a264, a265) {
        true
    } else {
        let a266 = 9u64;
        let a267 = v0;
        if nat_le(a266, a267) {
            let a268 = v0;
            let a269 = 13u64;
            nat_le(a268, a269)
        } else {
            false
        }
    }
}

pub fn f44(v0: Bytes, v1: u64) -> R<u64> {
    let v2: u64 = f42(v0.clone(), v1)?;
    let v3: u64 = f42(v0.clone(), {
        let a270 = v1;
        let a271 = 1u64;
        nat_add(a270, a271)?
    })?;
    let v4: u64 = f42(v0, {
        let a272 = v1;
        let a273 = 2u64;
        nat_add(a272, a273)?
    })?;
    if f43(v2) {
        Ok(1u64)
    } else {
        let a274 = v2;
        let a275 = 194u64;
        if nat_eq(a274, a275) {
            let a276 = v3;
            let a277 = 133u64;
            if if nat_eq(a276, a277) {
                true
            } else {
                let a278 = v3;
                let a279 = 160u64;
                nat_eq(a278, a279)
            } {
                Ok(2u64)
            } else {
                Ok(0u64)
            }
        } else {
            let a280 = v2;
            let a281 = 225u64;
            if nat_eq(a280, a281) {
                let a282 = v3;
                let a283 = 154u64;
                if if nat_eq(a282, a283) {
                    let a284 = v4;
                    let a285 = 128u64;
                    nat_eq(a284, a285)
                } else {
                    false
                } {
                    Ok(3u64)
                } else {
                    Ok(0u64)
                }
            } else {
                let a286 = v2;
                let a287 = 226u64;
                if nat_eq(a286, a287) {
                    let a288 = v3;
                    let a289 = 128u64;
                    if nat_eq(a288, a289) {
                        let a290 = 128u64;
                        let a291 = v4;
                        if if if nat_le(a290, a291) {
                            let a292 = v4;
                            let a293 = 138u64;
                            nat_le(a292, a293)
                        } else {
                            false
                        } {
                            true
                        } else {
                            let a294 = v4;
                            let a295 = 168u64;
                            if nat_eq(a294, a295) {
                                true
                            } else {
                                let a296 = v4;
                                let a297 = 169u64;
                                if nat_eq(a296, a297) {
                                    true
                                } else {
                                    let a298 = v4;
                                    let a299 = 175u64;
                                    nat_eq(a298, a299)
                                }
                            }
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    } else {
                        let a300 = v3;
                        let a301 = 129u64;
                        if if nat_eq(a300, a301) {
                            let a302 = v4;
                            let a303 = 159u64;
                            nat_eq(a302, a303)
                        } else {
                            false
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    }
                } else {
                    let a304 = v2;
                    let a305 = 227u64;
                    if if nat_eq(a304, a305) {
                        let a306 = v3;
                        let a307 = 128u64;
                        if nat_eq(a306, a307) {
                            let a308 = v4;
                            let a309 = 128u64;
                            nat_eq(a308, a309)
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

pub fn f45(p316: u64, v1: Bytes, p317: u64) -> R<u64> {
    let m315 = (p316, p317);
    {
        let mut m315 = m315;
        loop {
            let step = {
                let (v0, v2) = m315;
                let m310 = v0;
                if m310 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m310 - 1;
                    let v4: u64 = f44(v1.clone(), v2)?;
                    let a311 = v4;
                    let a312 = 0u64;
                    if nat_eq(a311, a312) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a313 = v2;
                            let a314 = v4;
                            nat_add(a313, a314)?
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m315 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Adt6) -> R<Option<Adt11>> {
    f0(p0, p1)
}
