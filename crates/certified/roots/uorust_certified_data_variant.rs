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

#[derive(Clone)]
pub enum Adt18 {
    C0(Str, Str, List<(Option<Str>, Adt9)>),
}

pub fn f0(v0: Adt0, v1: Adt6) -> R<Option<Adt18>> {
    let v2: u64 = {
        let a1 = f1(v0.clone());
        length_list(a1)
    };
    let a2 = v2;
    let a3 = 2u64;
    if nat_lt(a2, a3) {
        Ok(None::<Adt18>)
    } else {
        let v3: Str = f2(v1.clone(), f3(f4(v2, v0.clone())))?;
        let v4: Str = f3(f5(v2, v0));
        let m5 = f6(match v1 {
            Adt6::C0(_, _, _, h4, _, _, _, _, _, _, _, _, _) => {
                h4
            }
        }, v3.clone());
        match m5 {
            None => {
                Ok(None::<Adt18>)
            }
            Some(v5) => {
                let m6 = f7(v5, v4.clone());
                match m6 {
                    None => {
                        Ok(None::<Adt18>)
                    }
                    Some(v6) => {
                        let v7: (Str, List<(Option<Str>, Adt9)>) = v6;
                        let _: Str = {
                            let (h7, _) = v7;
                            h7
                        };
                        let v9: List<(Option<Str>, Adt9)> = {
                            let (_, h8) = v7;
                            h8
                        };
                        Ok(Some(Adt18::C0(v3, v4, v9)))
                    }
                }
            }
        }
    }
}

pub fn f1(v0: Adt0) -> List<Adt1> {
    let m9 = v0;
    match m9 {
        Adt0::C0(v1, _) => {
            v1
        }
    }
}

pub fn f2(v0: Adt6, v1: Str) -> R<Str> {
    let v2: Str = f8(v1)?;
    let m11 = f9(match v0 {
        Adt6::C0(_, _, h10, _, _, _, _, _, _, _, _, _, _) => {
            h10
        }
    }, v2.clone());
    match m11 {
        None => {
            Ok(v2)
        }
        Some(v3) => {
            Ok(v3)
        }
    }
}

pub fn f3(v0: Adt1) -> Str {
    let m12 = v0;
    match m12 {
        Adt1::C0(v1, _) => {
            v1
        }
    }
}

pub fn f4(v0: u64, v1: Adt0) -> Adt1 {
    let m17 = {
        let a15 = f1(v1);
        let a16 = {
            let a13 = v0;
            let a14 = 2u64;
            nat_sub(a13, a14)
        };
        index_list(a15, a16)
    };
    match m17 {
        None => {
            Adt1::C0(Str::lit(""), Adt2::C0)
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f5(v0: u64, v1: Adt0) -> Adt1 {
    let m22 = {
        let a20 = f1(v1);
        let a21 = {
            let a18 = v0;
            let a19 = 1u64;
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

pub fn f6(p29: List<Adt12>, v1: Str) -> Option<List<(Str, List<(Option<Str>, Adt9)>)>> {
    let m28 = p29;
    {
        let mut m28 = m28;
        loop {
            let step = {
                let v0 = m28;
                let m23 = v0;
                match m23.uncons() {
                    None => {
                        Err::<List<Adt12>, Option<List<(Str, List<(Option<Str>, Adt9)>)>>>(None::<List<(Str, List<(Option<Str>, Adt9)>)>>)
                    }
                    Some((v2, v3)) => {
                        let a25 = match v2.clone() {
                            Adt12::C0(h24, _) => {
                                h24
                            }
                        };
                        let a26 = v1.clone();
                        if equal(a25, a26) {
                            Err::<List<Adt12>, Option<List<(Str, List<(Option<Str>, Adt9)>)>>>(Some(match v2 {
                                Adt12::C0(_, h27) => {
                                    h27
                                }
                            }))
                        } else {
                            Ok::<List<Adt12>, Option<List<(Str, List<(Option<Str>, Adt9)>)>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m28 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f7(p36: List<(Str, List<(Option<Str>, Adt9)>)>, v1: Str) -> Option<(Str, List<(Option<Str>, Adt9)>)> {
    let m35 = p36;
    {
        let mut m35 = m35;
        loop {
            let step = {
                let v0 = m35;
                let m30 = v0;
                match m30.uncons() {
                    None => {
                        Err::<List<(Str, List<(Option<Str>, Adt9)>)>, Option<(Str, List<(Option<Str>, Adt9)>)>>(None::<(Str, List<(Option<Str>, Adt9)>)>)
                    }
                    Some((v2, v3)) => {
                        let v4: Str = {
                            let (h31, _) = v2.clone();
                            h31
                        };
                        let _: List<(Option<Str>, Adt9)> = {
                            let (_, h32) = v2.clone();
                            h32
                        };
                        let a33 = v4;
                        let a34 = v1.clone();
                        if equal(a33, a34) {
                            Err::<List<(Str, List<(Option<Str>, Adt9)>)>, Option<(Str, List<(Option<Str>, Adt9)>)>>(Some(v2))
                        } else {
                            Ok::<List<(Str, List<(Option<Str>, Adt9)>)>, Option<(Str, List<(Option<Str>, Adt9)>)>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m35 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f8(v0: Str) -> R<Str> {
    let m37 = f10(v0.clone())?;
    match m37 {
        None => {
            Ok(v0)
        }
        Some(v1) => {
            let v2: Str = match v1.clone() {
                Adt11::C0(h38, _) => {
                    h38
                }
            };
            let v3: Str = match v1 {
                Adt11::C0(_, h39) => {
                    h39
                }
            };
            let m40 = f11(v3, Str::lit(">"));
            match m40 {
                None => {
                    Ok(v0)
                }
                Some(v4) => {
                    if f12(f13(v4, Str::lit(","))?)? {
                        f14(v2)
                    } else {
                        Ok(v0)
                    }
                }
            }
        }
    }
}

pub fn f9(p47: List<Adt11>, v1: Str) -> Option<Str> {
    let m46 = p47;
    {
        let mut m46 = m46;
        loop {
            let step = {
                let v0 = m46;
                let m41 = v0;
                match m41.uncons() {
                    None => {
                        Err::<List<Adt11>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a43 = match v2.clone() {
                            Adt11::C0(h42, _) => {
                                h42
                            }
                        };
                        let a44 = v1.clone();
                        if equal(a43, a44) {
                            Err::<List<Adt11>, Option<Str>>(Some(match v2 {
                                Adt11::C0(_, h45) => {
                                    h45
                                }
                            }))
                        } else {
                            Ok::<List<Adt11>, Option<Str>>(v3)
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

pub fn f10(v0: Str) -> R<Option<Adt11>> {
    let m48 = f15(v0.clone(), Str::lit("<"))?;
    match m48 {
        None => {
            Ok(None::<Adt11>)
        }
        Some(v1) => {
            Ok(Some(Adt11::C0(f16(v0.clone(), 0u64, v1), f16(v0.clone(), {
                let a49 = v1;
                let a50 = f17(Str::lit("<"));
                nat_add(a49, a50)?
            }, f17(v0)))))
        }
    }
}

pub fn f11(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a51 = v0.clone();
        utf8_encode(a51)
    };
    let v3: u64 = {
        let a52 = v2.clone();
        length_bytes(a52)
    };
    let v4: u64 = f17(v1.clone());
    if f18(v0, v1) {
        Some(f19(v2, 0u64, {
            let a53 = v3;
            let a54 = v4;
            nat_sub(a53, a54)
        }))
    } else {
        None::<Str>
    }
}

pub fn f12(p57: List<Str>) -> R<bool> {
    let m56 = p57;
    {
        let mut m56 = m56;
        loop {
            let step = {
                let v0 = m56;
                let m55 = v0;
                match m55.uncons() {
                    None => {
                        Err::<List<Str>, R<bool>>(Ok(true))
                    }
                    Some((v1, v2)) => {
                        if f20(v1)? {
                            Ok::<List<Str>, R<bool>>(v2)
                        } else {
                            Err::<List<Str>, R<bool>>(Ok(false))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m56 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f13(v0: Str, v1: Str) -> R<List<Str>> {
    let v2: Bytes = {
        let a58 = v0.clone();
        utf8_encode(a58)
    };
    let v3: Bytes = {
        let a59 = v1;
        utf8_encode(a59)
    };
    let a61 = {
        let a60 = v3.clone();
        length_bytes(a60)
    };
    let a62 = 0u64;
    if nat_eq(a61, a62) {
        Ok(List::cons(Str::lit(""), {
            let a63 = f21(v0)?;
            let a64 = List::cons(Str::lit(""), List::<Str>::nil());
            append_list(a63, a64)
        }))
    } else {
        f22({
            let a66 = {
                let a65 = v2.clone();
                length_bytes(a65)
            };
            let a67 = 1u64;
            nat_add(a66, a67)?
        }, v2, v3, 0u64, 0u64)
    }
}

pub fn f14(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a68 = v0;
        utf8_encode(a68)
    };
    let v2: u64 = {
        let a69 = v1.clone();
        length_bytes(a69)
    };
    Ok(f19(v1.clone(), 0u64, f23(v2, v1, v2)?))
}

pub fn f15(v0: Str, v1: Str) -> R<Option<u64>> {
    let v2: Bytes = {
        let a70 = v0;
        utf8_encode(a70)
    };
    f24({
        let a72 = {
            let a71 = v2.clone();
            length_bytes(a71)
        };
        let a73 = 1u64;
        nat_add(a72, a73)?
    }, v2, {
        let a74 = v1;
        utf8_encode(a74)
    }, 0u64)
}

pub fn f16(v0: Str, v1: u64, v2: u64) -> Str {
    f19({
        let a75 = v0;
        utf8_encode(a75)
    }, v1, v2)
}

pub fn f17(v0: Str) -> u64 {
    let a77 = {
        let a76 = v0;
        utf8_encode(a76)
    };
    length_bytes(a77)
}

pub fn f18(v0: Str, v1: Str) -> bool {
    let v2: Bytes = {
        let a78 = v0;
        utf8_encode(a78)
    };
    let v3: Bytes = {
        let a79 = v1;
        utf8_encode(a79)
    };
    let v4: u64 = {
        let a80 = v2.clone();
        length_bytes(a80)
    };
    let v5: u64 = {
        let a81 = v3.clone();
        length_bytes(a81)
    };
    let a82 = v5;
    let a83 = v4;
    if nat_le(a82, a83) {
        f25(v2, {
            let a84 = v4;
            let a85 = v5;
            nat_sub(a84, a85)
        }, v3)
    } else {
        false
    }
}

pub fn f19(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m91 = {
        let a88 = v0;
        let a89 = v1;
        let a90 = {
            let a86 = v2;
            let a87 = v1;
            nat_sub(a86, a87)
        };
        slice_bytes(a88, a89, a90)
    };
    match m91 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m93 = {
                let a92 = v3;
                utf8_decode(a92)
            };
            match m93 {
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

pub fn f20(v0: Str) -> R<bool> {
    let m94 = f26(f27(v0)?, Str::lit("'"));
    match m94 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            let a95 = f28(v1, Str::lit("'"))?;
            Ok(bool_not(a95))
        }
    }
}

pub fn f21(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a96 = v0;
        utf8_encode(a96)
    };
    f29({
        let a97 = v1.clone();
        length_bytes(a97)
    }, v1, 0u64)
}

pub fn f22(v0: u64, v1: Bytes, v2: Bytes, v3: u64, v4: u64) -> R<List<Str>> {
    let m98 = v0;
    if m98 == 0 {
        Ok(List::cons(f19(v1.clone(), v3, {
            let a99 = v1;
            length_bytes(a99)
        }), List::<Str>::nil()))
    } else {
        let v5 = m98 - 1;
        let a104 = {
            let a100 = v1.clone();
            length_bytes(a100)
        };
        let a105 = {
            let a102 = v4;
            let a103 = {
                let a101 = v2.clone();
                length_bytes(a101)
            };
            nat_add(a102, a103)?
        };
        if nat_lt(a104, a105) {
            Ok(List::cons(f19(v1.clone(), v3, {
                let a106 = v1;
                length_bytes(a106)
            }), List::<Str>::nil()))
        } else {
            if f25(v1.clone(), v4, v2.clone()) {
                Ok(List::cons(f19(v1.clone(), v3, v4), f22(v5, v1, v2.clone(), {
                    let a108 = v4;
                    let a109 = {
                        let a107 = v2.clone();
                        length_bytes(a107)
                    };
                    nat_add(a108, a109)?
                }, {
                    let a111 = v4;
                    let a112 = {
                        let a110 = v2;
                        length_bytes(a110)
                    };
                    nat_add(a111, a112)?
                })?))
            } else {
                f22(v5, v1, v2, v3, {
                    let a113 = v4;
                    let a114 = 1u64;
                    nat_add(a113, a114)?
                })
            }
        }
    }
}

pub fn f23(p121: u64, v1: Bytes, p122: u64) -> R<u64> {
    let m120 = (p121, p122);
    {
        let mut m120 = m120;
        loop {
            let step = {
                let (v0, v2) = m120;
                let m115 = v0;
                if m115 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m115 - 1;
                    let v4: u64 = f30(v1.clone(), v2)?;
                    let a116 = v4;
                    let a117 = 0u64;
                    if nat_eq(a116, a117) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a118 = v2;
                            let a119 = v4;
                            nat_sub(a118, a119)
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m120 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f24(p133: u64, v1: Bytes, v2: Bytes, p134: u64) -> R<Option<u64>> {
    let m132 = (p133, p134);
    {
        let mut m132 = m132;
        loop {
            let step = {
                let (v0, v3) = m132;
                let m123 = v0;
                if m123 == 0 {
                    Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                } else {
                    let v4 = m123 - 1;
                    let a128 = {
                        let a124 = v1.clone();
                        length_bytes(a124)
                    };
                    let a129 = {
                        let a126 = v3;
                        let a127 = {
                            let a125 = v2.clone();
                            length_bytes(a125)
                        };
                        nat_add(a126, a127)?
                    };
                    if nat_lt(a128, a129) {
                        Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                    } else {
                        if f25(v1.clone(), v3, v2.clone()) {
                            Err::<(u64, u64), R<Option<u64>>>(Ok(Some(v3)))
                        } else {
                            Ok::<(u64, u64), R<Option<u64>>>((v4, {
                                let a130 = v3;
                                let a131 = 1u64;
                                nat_add(a130, a131)?
                            }))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m132 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f25(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m139 = {
        let a136 = v0;
        let a137 = v1;
        let a138 = {
            let a135 = v2.clone();
            length_bytes(a135)
        };
        slice_bytes(a136, a137, a138)
    };
    match m139 {
        None => {
            false
        }
        Some(v3) => {
            let a140 = v3;
            let a141 = v2;
            equal(a140, a141)
        }
    }
}

pub fn f26(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a142 = v0;
        utf8_encode(a142)
    };
    let v3: Bytes = {
        let a143 = v1;
        utf8_encode(a143)
    };
    if f25(v2.clone(), 0u64, v3.clone()) {
        Some(f19(v2.clone(), {
            let a144 = v3;
            length_bytes(a144)
        }, {
            let a145 = v2;
            length_bytes(a145)
        }))
    } else {
        None::<Str>
    }
}

pub fn f27(v0: Str) -> R<Str> {
    f14(f31(v0)?)
}

pub fn f28(v0: Str, v1: Str) -> R<bool> {
    let m146 = f15(v0, v1)?;
    match m146 {
        None => {
            Ok(false)
        }
        Some(_) => {
            Ok(true)
        }
    }
}

pub fn f29(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m147 = v0;
    if m147 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m147 - 1;
        let a149 = {
            let a148 = v1.clone();
            length_bytes(a148)
        };
        let a150 = v2;
        if nat_le(a149, a150) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f32(f33(v1.clone(), v2)?);
            Ok(List::cons(f19(v1.clone(), v2, {
                let a151 = v2;
                let a152 = v4;
                nat_add(a151, a152)?
            }), f29(v3, v1, {
                let a153 = v2;
                let a154 = v4;
                nat_add(a153, a154)?
            })?))
        }
    }
}

pub fn f30(v0: Bytes, v1: u64) -> R<u64> {
    let a155 = v1;
    let a156 = 0u64;
    if nat_eq(a155, a156) {
        Ok(0u64)
    } else {
        if f34(f33(v0.clone(), {
            let a157 = v1;
            let a158 = 1u64;
            nat_sub(a157, a158)
        })?) {
            Ok(1u64)
        } else {
            let a159 = 2u64;
            let a160 = v1;
            if if nat_le(a159, a160) {
                let a163 = f35(v0.clone(), {
                    let a161 = v1;
                    let a162 = 2u64;
                    nat_sub(a161, a162)
                })?;
                let a164 = 2u64;
                nat_eq(a163, a164)
            } else {
                false
            } {
                Ok(2u64)
            } else {
                let a165 = 3u64;
                let a166 = v1;
                if if nat_le(a165, a166) {
                    let a169 = f35(v0, {
                        let a167 = v1;
                        let a168 = 3u64;
                        nat_sub(a167, a168)
                    })?;
                    let a170 = 3u64;
                    nat_eq(a169, a170)
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

pub fn f31(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a171 = v0;
        utf8_encode(a171)
    };
    let v2: u64 = {
        let a172 = v1.clone();
        length_bytes(a172)
    };
    Ok(f19(v1.clone(), f36(v2, v1, 0u64)?, v2))
}

pub fn f32(v0: u64) -> u64 {
    let a173 = v0;
    let a174 = 128u64;
    if nat_lt(a173, a174) {
        1u64
    } else {
        let a175 = v0;
        let a176 = 224u64;
        if nat_lt(a175, a176) {
            2u64
        } else {
            let a177 = v0;
            let a178 = 240u64;
            if nat_lt(a177, a178) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f33(v0: Bytes, v1: u64) -> R<u64> {
    let m181 = {
        let a179 = v0;
        let a180 = v1;
        index_bytes(a179, a180)
    };
    match m181 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a226 = {
                let a202 = {
                    let a190 = {
                        let a184 = {
                            let a182 = v3;
                            let a183 = 128u8;
                            fixed_u8::bit_and(a182, a183)
                        };
                        let a185 = 128u8;
                        if equal(a184, a185) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a191 = {
                        let a188 = {
                            let a186 = v3;
                            let a187 = 64u8;
                            fixed_u8::bit_and(a186, a187)
                        };
                        let a189 = 64u8;
                        if equal(a188, a189) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a190, a191)?
                };
                let a203 = {
                    let a200 = {
                        let a194 = {
                            let a192 = v3;
                            let a193 = 32u8;
                            fixed_u8::bit_and(a192, a193)
                        };
                        let a195 = 32u8;
                        if equal(a194, a195) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a201 = {
                        let a198 = {
                            let a196 = v3;
                            let a197 = 16u8;
                            fixed_u8::bit_and(a196, a197)
                        };
                        let a199 = 16u8;
                        if equal(a198, a199) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a200, a201)?
                };
                nat_add(a202, a203)?
            };
            let a227 = {
                let a224 = {
                    let a212 = {
                        let a206 = {
                            let a204 = v3;
                            let a205 = 8u8;
                            fixed_u8::bit_and(a204, a205)
                        };
                        let a207 = 8u8;
                        if equal(a206, a207) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a213 = {
                        let a210 = {
                            let a208 = v3;
                            let a209 = 4u8;
                            fixed_u8::bit_and(a208, a209)
                        };
                        let a211 = 4u8;
                        if equal(a210, a211) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a212, a213)?
                };
                let a225 = {
                    let a222 = {
                        let a216 = {
                            let a214 = v3;
                            let a215 = 2u8;
                            fixed_u8::bit_and(a214, a215)
                        };
                        let a217 = 2u8;
                        if equal(a216, a217) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a223 = {
                        let a220 = {
                            let a218 = v3;
                            let a219 = 1u8;
                            fixed_u8::bit_and(a218, a219)
                        };
                        let a221 = 1u8;
                        if equal(a220, a221) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a222, a223)?
                };
                nat_add(a224, a225)?
            };
            nat_add(a226, a227)
        }
    }
}

pub fn f34(v0: u64) -> bool {
    let a228 = v0;
    let a229 = 32u64;
    if nat_eq(a228, a229) {
        true
    } else {
        let a230 = 9u64;
        let a231 = v0;
        if nat_le(a230, a231) {
            let a232 = v0;
            let a233 = 13u64;
            nat_le(a232, a233)
        } else {
            false
        }
    }
}

pub fn f35(v0: Bytes, v1: u64) -> R<u64> {
    let v2: u64 = f33(v0.clone(), v1)?;
    let v3: u64 = f33(v0.clone(), {
        let a234 = v1;
        let a235 = 1u64;
        nat_add(a234, a235)?
    })?;
    let v4: u64 = f33(v0, {
        let a236 = v1;
        let a237 = 2u64;
        nat_add(a236, a237)?
    })?;
    if f34(v2) {
        Ok(1u64)
    } else {
        let a238 = v2;
        let a239 = 194u64;
        if nat_eq(a238, a239) {
            let a240 = v3;
            let a241 = 133u64;
            if if nat_eq(a240, a241) {
                true
            } else {
                let a242 = v3;
                let a243 = 160u64;
                nat_eq(a242, a243)
            } {
                Ok(2u64)
            } else {
                Ok(0u64)
            }
        } else {
            let a244 = v2;
            let a245 = 225u64;
            if nat_eq(a244, a245) {
                let a246 = v3;
                let a247 = 154u64;
                if if nat_eq(a246, a247) {
                    let a248 = v4;
                    let a249 = 128u64;
                    nat_eq(a248, a249)
                } else {
                    false
                } {
                    Ok(3u64)
                } else {
                    Ok(0u64)
                }
            } else {
                let a250 = v2;
                let a251 = 226u64;
                if nat_eq(a250, a251) {
                    let a252 = v3;
                    let a253 = 128u64;
                    if nat_eq(a252, a253) {
                        let a254 = 128u64;
                        let a255 = v4;
                        if if if nat_le(a254, a255) {
                            let a256 = v4;
                            let a257 = 138u64;
                            nat_le(a256, a257)
                        } else {
                            false
                        } {
                            true
                        } else {
                            let a258 = v4;
                            let a259 = 168u64;
                            if nat_eq(a258, a259) {
                                true
                            } else {
                                let a260 = v4;
                                let a261 = 169u64;
                                if nat_eq(a260, a261) {
                                    true
                                } else {
                                    let a262 = v4;
                                    let a263 = 175u64;
                                    nat_eq(a262, a263)
                                }
                            }
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    } else {
                        let a264 = v3;
                        let a265 = 129u64;
                        if if nat_eq(a264, a265) {
                            let a266 = v4;
                            let a267 = 159u64;
                            nat_eq(a266, a267)
                        } else {
                            false
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    }
                } else {
                    let a268 = v2;
                    let a269 = 227u64;
                    if if nat_eq(a268, a269) {
                        let a270 = v3;
                        let a271 = 128u64;
                        if nat_eq(a270, a271) {
                            let a272 = v4;
                            let a273 = 128u64;
                            nat_eq(a272, a273)
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

pub fn f36(p280: u64, v1: Bytes, p281: u64) -> R<u64> {
    let m279 = (p280, p281);
    {
        let mut m279 = m279;
        loop {
            let step = {
                let (v0, v2) = m279;
                let m274 = v0;
                if m274 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m274 - 1;
                    let v4: u64 = f35(v1.clone(), v2)?;
                    let a275 = v4;
                    let a276 = 0u64;
                    if nat_eq(a275, a276) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a277 = v2;
                            let a278 = v4;
                            nat_add(a277, a278)?
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m279 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Adt6) -> R<Option<Adt18>> {
    f0(p0, p1)
}
