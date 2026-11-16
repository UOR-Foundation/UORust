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
    C6,
    C7,
    C8,
    C9,
    C10,
    C11(Str),
    C12(std::rc::Rc<Adt1>),
    C13(std::rc::Rc<Adt1>, std::rc::Rc<Adt1>),
    C14(std::rc::Rc<Adt1>),
    C15(List<Adt1>),
    C16(List<Adt1>),
    C17(std::rc::Rc<Adt1>, std::rc::Rc<Adt1>, bool),
    C18(std::rc::Rc<Adt1>, bool),
    C19(List<Adt1>, std::rc::Rc<Adt1>),
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

#[derive(Clone)]
pub enum Adt56 {
    C0(Adt57, List<Adt66>, List<Adt68>, List<Str>),
}

#[derive(Clone)]
pub enum Adt57 {
    C0(List<Adt58>, List<Adt59>, List<Adt4>, List<Adt60>, List<List<Str>>, List<Adt58>, List<Adt58>, bool, List<Adt61>, List<Adt62>, List<Adt63>, List<Adt65>),
}

#[derive(Clone)]
pub enum Adt58 {
    C0(Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt59 {
    C0(Str, List<(Str, Adt1)>),
}

#[derive(Clone)]
pub enum Adt60 {
    C0(Str, List<(Str, List<(Option<Str>, Adt1)>)>),
}

#[derive(Clone)]
pub enum Adt61 {
    C0(Str, Adt2),
}

#[derive(Clone)]
pub enum Adt62 {
    C0(Str, Adt4),
}

#[derive(Clone)]
pub enum Adt63 {
    C0(Str, List<Adt64>),
}

#[derive(Clone)]
pub enum Adt64 {
    C0(u64, Str),
}

#[derive(Clone)]
pub enum Adt65 {
    C0(Str, List<i64>),
}

#[derive(Clone)]
pub enum Adt66 {
    C0(Str, Adt67),
}

#[derive(Clone)]
pub enum Adt67 {
    C0(Str, List<Adt1>, Adt1),
}

#[derive(Clone)]
pub enum Adt68 {
    C0(u64, List<Str>),
}

#[derive(Clone)]
pub enum Adt69 {
    C0(List<u64>, List<Str>, List<Str>, List<Adt1>, List<Str>, List<Adt70>, List<Adt1>),
}

#[derive(Clone)]
pub enum Adt70 {
    C0(Str, List<Adt4>, Str, Adt71, List<Str>, List<Str>, bool, bool, Option<(Str, List<Adt73>)>),
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
    C0(Str, List<Adt4>),
    C1(Str, List<Str>),
    C2(Str, List<Adt58>, Option<Str>),
    C3(Str, List<Adt4>, Str, Adt71, List<Str>, Option<Str>, Option<Adt58>, Option<Str>),
    C4(Str, List<Adt4>, Str, Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt74 {
    C0,
    C1,
    C2,
    C3,
}

pub fn f0(v0: Adt0, v1: Adt56, v2: Adt69, v3: Str) -> R<Str> {
    let v4: Str = {
        let a2 = List::cons(match v0.clone() {
            Adt0::C0(_, _, _, h1, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                h1
            }
        }, List::cons(v3, List::<Str>::nil()));
        let a3 = Str::lit("");
        join(a2, a3)
    };
    let v5: List<Str> = f1(match v2.clone() {
        Adt69::C0(_, _, _, _, _, h4, _) => {
            h4
        }
    });
    f2(v4.clone(), f3(v0, v1, v2, v4, v5))
}

pub fn f1(v0: List<Adt70>) -> List<Str> {
    let m5 = v0;
    match m5.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt70::C0(h6, _, _, _, _, _, _, _, _) => {
                    h6
                }
            }, f1(v2))
        }
    }
}

pub fn f2(v0: Str, v1: List<Str>) -> R<Str> {
    let v2: u64 = f4(f5(f6(v1.clone(), v0.clone()))?, 0u64);
    let m13 = {
        let a9 = {
            let a7 = v2;
            let a8 = 1u64;
            nat_add(a7, a8)?
        };
        let a10 = 4294967296u64;
        if nat_lt(a9, a10) {
            Some({
                let a11 = v2;
                let a12 = 1u64;
                nat_add(a11, a12)?
            })
        } else {
            None::<u64>
        }
    };
    match m13 {
        None => {
            let m19 = f7(f8(f9(1u64, {
                let a17 = {
                    let a15 = {
                        let a14 = v1.clone();
                        length_list(a14)
                    };
                    let a16 = 2u64;
                    nat_add(a15, a16)?
                };
                let a18 = 1u64;
                nat_sub(a17, a18)
            })?, v0)?, v1);
            match m19 {
                None => {
                    Ok(Str::lit(""))
                }
                Some(v3) => {
                    Ok(v3)
                }
            }
        }
        Some(v4) => {
            let a20 = List::cons(v0, List::cons(f10(v4)?, List::<Str>::nil()));
            let a21 = Str::lit("");
            Ok(join(a20, a21))
        }
    }
}

pub fn f3(v0: Adt0, v1: Adt56, v2: Adt69, v3: Str, v4: List<Str>) -> List<Str> {
    f11({
        let a35 = {
            let a33 = {
                let a30 = {
                    let a27 = {
                        let a24 = f12(f13(match v0.clone() {
                            Adt0::C0(_, _, _, _, h22, _, _, _, _, _, _, _, _, _, _, _, _) => {
                                h22
                            }
                        }));
                        let a25 = match v0 {
                            Adt0::C0(_, _, _, _, _, _, _, _, h23, _, _, _, _, _, _, _, _) => {
                                h23
                            }
                        };
                        append_list(a24, a25)
                    };
                    let a28 = f14(f15(match v1.clone() {
                        Adt56::C0(_, h26, _, _) => {
                            h26
                        }
                    }));
                    append_list(a27, a28)
                };
                let a31 = match v1 {
                    Adt56::C0(_, _, _, h29) => {
                        h29
                    }
                };
                append_list(a30, a31)
            };
            let a34 = match v2 {
                Adt69::C0(_, _, h32, _, _, _, _) => {
                    h32
                }
            };
            append_list(a33, a34)
        };
        let a36 = v4;
        append_list(a35, a36)
    }, v3)
}

pub fn f4(p41: List<u64>, p42: u64) -> u64 {
    let m40 = (p41, p42);
    {
        let mut m40 = m40;
        loop {
            let step = {
                let (v0, v1) = m40;
                let m37 = v0;
                match m37.uncons() {
                    None => {
                        Err::<(List<u64>, u64), u64>(v1)
                    }
                    Some((v2, v3)) => {
                        Ok::<(List<u64>, u64), u64>((v3, {
                            let a38 = v1;
                            let a39 = v2;
                            if nat_le(a38, a39) {
                                v2
                            } else {
                                v1
                            }
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m40 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f5(v0: List<Str>) -> R<List<u64>> {
    let m43 = v0;
    match m43.uncons() {
        None => {
            Ok(List::<u64>::nil())
        }
        Some((v1, v2)) => {
            let m44 = f16(v1)?;
            match m44 {
                None => {
                    f5(v2)
                }
                Some(v3) => {
                    Ok(List::cons(v3, f5(v2)?))
                }
            }
        }
    }
}

pub fn f6(v0: List<Str>, v1: Str) -> List<Str> {
    let m45 = v0;
    match m45.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            let m46 = f17(v2, v1.clone());
            match m46 {
                None => {
                    f6(v3, v1)
                }
                Some(v4) => {
                    List::cons(v4, f6(v3, v1))
                }
            }
        }
    }
}

pub fn f7(p50: List<Str>, v1: List<Str>) -> Option<Str> {
    let m49 = p50;
    {
        let mut m49 = m49;
        loop {
            let step = {
                let v0 = m49;
                let m47 = v0;
                match m47.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a48 = f18(v1.clone(), v2.clone());
                        if bool_not(a48) {
                            Err::<List<Str>, Option<Str>>(Some(v2))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m49 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f8(v0: List<u64>, v1: Str) -> R<List<Str>> {
    let m51 = v0;
    match m51.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons({
                let a52 = List::cons(v1.clone(), List::cons(f10(v2)?, List::<Str>::nil()));
                let a53 = Str::lit("");
                join(a52, a53)
            }, f8(v3, v1)?))
        }
    }
}

pub fn f9(v0: u64, v1: u64) -> R<List<u64>> {
    let m54 = v1;
    if m54 == 0 {
        Ok(List::<u64>::nil())
    } else {
        let v2 = m54 - 1;
        Ok(List::cons(v0, f9({
            let a55 = v0;
            let a56 = 1u64;
            nat_add(a55, a56)?
        }, v2)?))
    }
}

pub fn f10(v0: u64) -> R<Str> {
    Ok(f19({
        let a57 = v0;
        let a58 = 1u64;
        nat_add(a57, a58)?
    }, v0))
}

pub fn f11(v0: List<Str>, v1: Str) -> List<Str> {
    let m59 = v0;
    match m59.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            if f20(v2.clone(), v1.clone()) {
                List::cons(v2, f11(v3, v1))
            } else {
                f11(v3, v1)
            }
        }
    }
}

pub fn f12(v0: List<(Str, Adt1)>) -> List<Str> {
    let m60 = v0;
    match m60.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            let v3: Str = {
                let (h61, _) = v1;
                h61
            };
            let _: Adt1 = {
                let (_, h62) = v1;
                h62
            };
            List::cons(v3, f12(v2))
        }
    }
}

pub fn f13(v0: List<Adt3>) -> List<(Str, Adt1)> {
    let m63 = v0;
    match m63.uncons() {
        None => {
            List::<(Str, Adt1)>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt3::C0(_, h64) => {
                    h64
                }
            }, f13(v2))
        }
    }
}

pub fn f14(v0: List<Adt67>) -> List<Str> {
    let m65 = v0;
    match m65.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt67::C0(h66, _, _) => {
                    h66
                }
            }, f14(v2))
        }
    }
}

pub fn f15(v0: List<Adt66>) -> List<Adt67> {
    let m67 = v0;
    match m67.uncons() {
        None => {
            List::<Adt67>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt66::C0(_, h68) => {
                    h68
                }
            }, f15(v2))
        }
    }
}

pub fn f16(v0: Str) -> R<Option<u64>> {
    let m69 = f21(v0, 4294967295u64)?;
    match m69 {
        Ok(v1) => {
            Ok(Some(v1))
        }
        Err(_) => {
            Ok(None::<u64>)
        }
    }
}

pub fn f17(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a70 = v0;
        utf8_encode(a70)
    };
    let v3: Bytes = {
        let a71 = v1;
        utf8_encode(a71)
    };
    if f22(v2.clone(), 0u64, v3.clone()) {
        Some(f23(v2.clone(), {
            let a72 = v3;
            length_bytes(a72)
        }, {
            let a73 = v2;
            length_bytes(a73)
        }))
    } else {
        None::<Str>
    }
}

pub fn f18(p78: List<Str>, v1: Str) -> bool {
    let m77 = p78;
    {
        let mut m77 = m77;
        loop {
            let step = {
                let v0 = m77;
                let m74 = v0;
                match m74.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a75 = v2;
                        let a76 = v1.clone();
                        if equal(a75, a76) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m77 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f19(v0: u64, v1: u64) -> Str {
    let m79 = v0;
    if m79 == 0 {
        Str::lit("")
    } else {
        let v2 = m79 - 1;
        let a80 = v1;
        let a81 = 10u64;
        if nat_lt(a80, a81) {
            f24(v1)
        } else {
            let a88 = List::cons(f19(v2, {
                let a82 = v1;
                let a83 = 10u64;
                let a84 = 0u64;
                nat_quot(a82, a83, a84)
            }), List::cons(f24({
                let a85 = v1;
                let a86 = 10u64;
                let a87 = 0u64;
                nat_rem(a85, a86, a87)
            }), List::<Str>::nil()));
            let a89 = Str::lit("");
            join(a88, a89)
        }
    }
}

pub fn f20(v0: Str, v1: Str) -> bool {
    f22({
        let a90 = v0;
        utf8_encode(a90)
    }, 0u64, {
        let a91 = v1;
        utf8_encode(a91)
    })
}

pub fn f21(v0: Str, v1: u64) -> R<Result<u64, Adt74>> {
    let v2: Bytes = {
        let a92 = v0;
        utf8_encode(a92)
    };
    let v3: u64 = {
        let a93 = v2.clone();
        length_bytes(a93)
    };
    let a94 = v3;
    let a95 = 0u64;
    if nat_eq(a94, a95) {
        Ok(Err::<u64, Adt74>(Adt74::C0))
    } else {
        let a96 = f25(v2.clone(), 0u64)?;
        let a97 = 43u64;
        if nat_eq(a96, a97) {
            let a98 = v3;
            let a99 = 1u64;
            if nat_eq(a98, a99) {
                Ok(Err::<u64, Adt74>(Adt74::C1))
            } else {
                f26(v3, v2, 1u64, 0u64, v1)
            }
        } else {
            f26(v3, v2, 0u64, 0u64, v1)
        }
    }
}

pub fn f22(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m104 = {
        let a101 = v0;
        let a102 = v1;
        let a103 = {
            let a100 = v2.clone();
            length_bytes(a100)
        };
        slice_bytes(a101, a102, a103)
    };
    match m104 {
        None => {
            false
        }
        Some(v3) => {
            let a105 = v3;
            let a106 = v2;
            equal(a105, a106)
        }
    }
}

pub fn f23(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m112 = {
        let a109 = v0;
        let a110 = v1;
        let a111 = {
            let a107 = v2;
            let a108 = v1;
            nat_sub(a107, a108)
        };
        slice_bytes(a109, a110, a111)
    };
    match m112 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m114 = {
                let a113 = v3;
                utf8_decode(a113)
            };
            match m114 {
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

pub fn f24(v0: u64) -> Str {
    let a115 = v0;
    let a116 = 0u64;
    if nat_eq(a115, a116) {
        Str::lit("0")
    } else {
        let a117 = v0;
        let a118 = 1u64;
        if nat_eq(a117, a118) {
            Str::lit("1")
        } else {
            let a119 = v0;
            let a120 = 2u64;
            if nat_eq(a119, a120) {
                Str::lit("2")
            } else {
                let a121 = v0;
                let a122 = 3u64;
                if nat_eq(a121, a122) {
                    Str::lit("3")
                } else {
                    let a123 = v0;
                    let a124 = 4u64;
                    if nat_eq(a123, a124) {
                        Str::lit("4")
                    } else {
                        let a125 = v0;
                        let a126 = 5u64;
                        if nat_eq(a125, a126) {
                            Str::lit("5")
                        } else {
                            let a127 = v0;
                            let a128 = 6u64;
                            if nat_eq(a127, a128) {
                                Str::lit("6")
                            } else {
                                let a129 = v0;
                                let a130 = 7u64;
                                if nat_eq(a129, a130) {
                                    Str::lit("7")
                                } else {
                                    let a131 = v0;
                                    let a132 = 8u64;
                                    if nat_eq(a131, a132) {
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

pub fn f25(v0: Bytes, v1: u64) -> R<u64> {
    let m135 = {
        let a133 = v0;
        let a134 = v1;
        index_bytes(a133, a134)
    };
    match m135 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a180 = {
                let a156 = {
                    let a144 = {
                        let a138 = {
                            let a136 = v3;
                            let a137 = 128u8;
                            fixed_u8::bit_and(a136, a137)
                        };
                        let a139 = 128u8;
                        if equal(a138, a139) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a145 = {
                        let a142 = {
                            let a140 = v3;
                            let a141 = 64u8;
                            fixed_u8::bit_and(a140, a141)
                        };
                        let a143 = 64u8;
                        if equal(a142, a143) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a144, a145)?
                };
                let a157 = {
                    let a154 = {
                        let a148 = {
                            let a146 = v3;
                            let a147 = 32u8;
                            fixed_u8::bit_and(a146, a147)
                        };
                        let a149 = 32u8;
                        if equal(a148, a149) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a155 = {
                        let a152 = {
                            let a150 = v3;
                            let a151 = 16u8;
                            fixed_u8::bit_and(a150, a151)
                        };
                        let a153 = 16u8;
                        if equal(a152, a153) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a154, a155)?
                };
                nat_add(a156, a157)?
            };
            let a181 = {
                let a178 = {
                    let a166 = {
                        let a160 = {
                            let a158 = v3;
                            let a159 = 8u8;
                            fixed_u8::bit_and(a158, a159)
                        };
                        let a161 = 8u8;
                        if equal(a160, a161) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a167 = {
                        let a164 = {
                            let a162 = v3;
                            let a163 = 4u8;
                            fixed_u8::bit_and(a162, a163)
                        };
                        let a165 = 4u8;
                        if equal(a164, a165) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a166, a167)?
                };
                let a179 = {
                    let a176 = {
                        let a170 = {
                            let a168 = v3;
                            let a169 = 2u8;
                            fixed_u8::bit_and(a168, a169)
                        };
                        let a171 = 2u8;
                        if equal(a170, a171) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a177 = {
                        let a174 = {
                            let a172 = v3;
                            let a173 = 1u8;
                            fixed_u8::bit_and(a172, a173)
                        };
                        let a175 = 1u8;
                        if equal(a174, a175) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a176, a177)?
                };
                nat_add(a178, a179)?
            };
            nat_add(a180, a181)
        }
    }
}

pub fn f26(p201: u64, v1: Bytes, p202: u64, p203: u64, v4: u64) -> R<Result<u64, Adt74>> {
    let m200 = (p201, (p202, p203));
    {
        let mut m200 = m200;
        loop {
            let step = {
                let (v0, (v2, v3)) = m200;
                let m182 = v0;
                if m182 == 0 {
                    Err::<(u64, (u64, u64)), R<Result<u64, Adt74>>>(Ok(Ok::<u64, Adt74>(v3)))
                } else {
                    let v5 = m182 - 1;
                    let a184 = {
                        let a183 = v1.clone();
                        length_bytes(a183)
                    };
                    let a185 = v2;
                    if nat_le(a184, a185) {
                        Err::<(u64, (u64, u64)), R<Result<u64, Adt74>>>(Ok(Ok::<u64, Adt74>(v3)))
                    } else {
                        let v6: u64 = f25(v1.clone(), v2)?;
                        let a186 = 48u64;
                        let a187 = v6;
                        if if nat_le(a186, a187) {
                            let a188 = v6;
                            let a189 = 57u64;
                            nat_le(a188, a189)
                        } else {
                            false
                        } {
                            let v7: u64 = {
                                let a194 = {
                                    let a190 = v3;
                                    let a191 = 10u64;
                                    nat_mul(a190, a191)?
                                };
                                let a195 = {
                                    let a192 = v6;
                                    let a193 = 48u64;
                                    nat_sub(a192, a193)
                                };
                                nat_add(a194, a195)?
                            };
                            let a196 = v4;
                            let a197 = v7;
                            if nat_lt(a196, a197) {
                                Err::<(u64, (u64, u64)), R<Result<u64, Adt74>>>(Ok(Err::<u64, Adt74>(Adt74::C2)))
                            } else {
                                Ok::<(u64, (u64, u64)), R<Result<u64, Adt74>>>((v5, ({
                                    let a198 = v2;
                                    let a199 = 1u64;
                                    nat_add(a198, a199)?
                                }, v7)))
                            }
                        } else {
                            Err::<(u64, (u64, u64)), R<Result<u64, Adt74>>>(Ok(Err::<u64, Adt74>(Adt74::C1)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m200 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Adt56, p2: Adt69, p3: Str) -> R<Str> {
    f0(p0, p1, p2, p3)
}
