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
    C0,
    C1,
    C2,
    C3,
}

#[derive(Clone)]
pub enum Adt1 {
    C0(u64, Str),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Str, bool, bool),
}

#[derive(Clone)]
pub enum Adt3 {
    C0(Str, bool),
}

pub fn f0(v0: Str) -> R<Str> {
    let m1 = f1(v0.clone(), 4294967295u64)?;
    match m1 {
        Ok(v1) => {
            let a2 = List::cons(Str::lit("item"), List::cons(f2(v1)?, List::<Str>::nil()));
            let a3 = Str::lit("");
            Ok(join(a2, a3))
        }
        Err(_) => {
            f3(v0)
        }
    }
}

pub fn f1(v0: Str, v1: u64) -> R<Result<u64, Adt0>> {
    let v2: Bytes = {
        let a4 = v0;
        utf8_encode(a4)
    };
    let v3: u64 = {
        let a5 = v2.clone();
        length_bytes(a5)
    };
    let a6 = v3;
    let a7 = 0u64;
    if nat_eq(a6, a7) {
        Ok(Err::<u64, Adt0>(Adt0::C0))
    } else {
        let a8 = f4(v2.clone(), 0u64)?;
        let a9 = 43u64;
        if nat_eq(a8, a9) {
            let a10 = v3;
            let a11 = 1u64;
            if nat_eq(a10, a11) {
                Ok(Err::<u64, Adt0>(Adt0::C1))
            } else {
                f5(v3, v2, 1u64, 0u64, v1)
            }
        } else {
            f5(v3, v2, 0u64, 0u64, v1)
        }
    }
}

pub fn f2(v0: u64) -> R<Str> {
    Ok(f6({
        let a12 = v0;
        let a13 = 1u64;
        nat_add(a12, a13)?
    }, v0))
}

pub fn f3(v0: Str) -> R<Str> {
    let v1: Str = f7(List::cons(f8(v0, Str::lit("r#"))?, List::<Str>::nil()))?;
    let a14 = v1.clone();
    let a15 = Str::lit("");
    if if equal(a14, a15) {
        true
    } else {
        f9(f10(), v1.clone())
    } {
        let a16 = List::cons(v1, List::cons(Str::lit("Value"), List::<Str>::nil()));
        let a17 = Str::lit("");
        Ok(join(a16, a17))
    } else {
        Ok(v1)
    }
}

pub fn f4(v0: Bytes, v1: u64) -> R<u64> {
    let m20 = {
        let a18 = v0;
        let a19 = v1;
        index_bytes(a18, a19)
    };
    match m20 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a65 = {
                let a41 = {
                    let a29 = {
                        let a23 = {
                            let a21 = v3;
                            let a22 = 128u8;
                            fixed_u8::bit_and(a21, a22)
                        };
                        let a24 = 128u8;
                        if equal(a23, a24) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a30 = {
                        let a27 = {
                            let a25 = v3;
                            let a26 = 64u8;
                            fixed_u8::bit_and(a25, a26)
                        };
                        let a28 = 64u8;
                        if equal(a27, a28) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a29, a30)?
                };
                let a42 = {
                    let a39 = {
                        let a33 = {
                            let a31 = v3;
                            let a32 = 32u8;
                            fixed_u8::bit_and(a31, a32)
                        };
                        let a34 = 32u8;
                        if equal(a33, a34) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a40 = {
                        let a37 = {
                            let a35 = v3;
                            let a36 = 16u8;
                            fixed_u8::bit_and(a35, a36)
                        };
                        let a38 = 16u8;
                        if equal(a37, a38) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a39, a40)?
                };
                nat_add(a41, a42)?
            };
            let a66 = {
                let a63 = {
                    let a51 = {
                        let a45 = {
                            let a43 = v3;
                            let a44 = 8u8;
                            fixed_u8::bit_and(a43, a44)
                        };
                        let a46 = 8u8;
                        if equal(a45, a46) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a52 = {
                        let a49 = {
                            let a47 = v3;
                            let a48 = 4u8;
                            fixed_u8::bit_and(a47, a48)
                        };
                        let a50 = 4u8;
                        if equal(a49, a50) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a51, a52)?
                };
                let a64 = {
                    let a61 = {
                        let a55 = {
                            let a53 = v3;
                            let a54 = 2u8;
                            fixed_u8::bit_and(a53, a54)
                        };
                        let a56 = 2u8;
                        if equal(a55, a56) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a62 = {
                        let a59 = {
                            let a57 = v3;
                            let a58 = 1u8;
                            fixed_u8::bit_and(a57, a58)
                        };
                        let a60 = 1u8;
                        if equal(a59, a60) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a61, a62)?
                };
                nat_add(a63, a64)?
            };
            nat_add(a65, a66)
        }
    }
}

pub fn f5(p86: u64, v1: Bytes, p87: u64, p88: u64, v4: u64) -> R<Result<u64, Adt0>> {
    let m85 = (p86, (p87, p88));
    {
        let mut m85 = m85;
        loop {
            let step = {
                let (v0, (v2, v3)) = m85;
                let m67 = v0;
                if m67 == 0 {
                    Err::<(u64, (u64, u64)), R<Result<u64, Adt0>>>(Ok(Ok::<u64, Adt0>(v3)))
                } else {
                    let v5 = m67 - 1;
                    let a69 = {
                        let a68 = v1.clone();
                        length_bytes(a68)
                    };
                    let a70 = v2;
                    if nat_le(a69, a70) {
                        Err::<(u64, (u64, u64)), R<Result<u64, Adt0>>>(Ok(Ok::<u64, Adt0>(v3)))
                    } else {
                        let v6: u64 = f4(v1.clone(), v2)?;
                        let a71 = 48u64;
                        let a72 = v6;
                        if if nat_le(a71, a72) {
                            let a73 = v6;
                            let a74 = 57u64;
                            nat_le(a73, a74)
                        } else {
                            false
                        } {
                            let v7: u64 = {
                                let a79 = {
                                    let a75 = v3;
                                    let a76 = 10u64;
                                    nat_mul(a75, a76)?
                                };
                                let a80 = {
                                    let a77 = v6;
                                    let a78 = 48u64;
                                    nat_sub(a77, a78)
                                };
                                nat_add(a79, a80)?
                            };
                            let a81 = v4;
                            let a82 = v7;
                            if nat_lt(a81, a82) {
                                Err::<(u64, (u64, u64)), R<Result<u64, Adt0>>>(Ok(Err::<u64, Adt0>(Adt0::C2)))
                            } else {
                                Ok::<(u64, (u64, u64)), R<Result<u64, Adt0>>>((v5, ({
                                    let a83 = v2;
                                    let a84 = 1u64;
                                    nat_add(a83, a84)?
                                }, v7)))
                            }
                        } else {
                            Err::<(u64, (u64, u64)), R<Result<u64, Adt0>>>(Ok(Err::<u64, Adt0>(Adt0::C1)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m85 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f6(v0: u64, v1: u64) -> Str {
    let m89 = v0;
    if m89 == 0 {
        Str::lit("")
    } else {
        let v2 = m89 - 1;
        let a90 = v1;
        let a91 = 10u64;
        if nat_lt(a90, a91) {
            f11(v1)
        } else {
            let a98 = List::cons(f6(v2, {
                let a92 = v1;
                let a93 = 10u64;
                let a94 = 0u64;
                nat_quot(a92, a93, a94)
            }), List::cons(f11({
                let a95 = v1;
                let a96 = 10u64;
                let a97 = 0u64;
                nat_rem(a95, a96, a97)
            }), List::<Str>::nil()));
            let a99 = Str::lit("");
            join(a98, a99)
        }
    }
}

pub fn f7(v0: List<Str>) -> R<Str> {
    let a100 = f12(f13(f14(f15(v0)?)?, 0u64)?)?;
    let a101 = Str::lit("");
    Ok(join(a100, a101))
}

pub fn f8(v0: Str, v1: Str) -> R<Str> {
    let v2: Bytes = {
        let a102 = v0.clone();
        utf8_encode(a102)
    };
    let v3: Bytes = {
        let a103 = v1;
        utf8_encode(a103)
    };
    let a105 = {
        let a104 = v3.clone();
        length_bytes(a104)
    };
    let a106 = 0u64;
    if nat_eq(a105, a106) {
        Ok(v0)
    } else {
        Ok(f16(v2.clone(), f17({
            let a107 = v2.clone();
            length_bytes(a107)
        }, v2.clone(), v3, 0u64)?, {
            let a108 = v2;
            length_bytes(a108)
        }))
    }
}

pub fn f9(p113: List<Str>, v1: Str) -> bool {
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

pub fn f10() -> List<Str> {
    let a142 = {
        let a126 = {
            let a118 = {
                let a114 = List::cons(Str::lit("at"), List::cons(Str::lit("by"), List::cons(Str::lit("calc"), List::cons(Str::lit("def"), List::cons(Str::lit("do"), List::cons(Str::lit("else"), List::<Str>::nil()))))));
                let a115 = List::cons(Str::lit("end"), List::cons(Str::lit("example"), List::cons(Str::lit("fun"), List::cons(Str::lit("have"), List::cons(Str::lit("if"), List::cons(Str::lit("in"), List::<Str>::nil()))))));
                append_list(a114, a115)
            };
            let a119 = {
                let a116 = List::cons(Str::lit("instance"), List::cons(Str::lit("let"), List::cons(Str::lit("match"), List::cons(Str::lit("mut"), List::cons(Str::lit("nomatch"), List::cons(Str::lit("open"), List::<Str>::nil()))))));
                let a117 = List::cons(Str::lit("prop"), List::cons(Str::lit("return"), List::cons(Str::lit("show"), List::cons(Str::lit("then"), List::cons(Str::lit("theorem"), List::cons(Str::lit("type"), List::cons(Str::lit("where"), List::<Str>::nil())))))));
                append_list(a116, a117)
            };
            append_list(a118, a119)
        };
        let a127 = {
            let a124 = {
                let a120 = List::cons(Str::lit("with"), List::cons(Str::lit("from"), List::cons(Str::lit("for"), List::cons(Str::lit("try"), List::cons(Str::lit("catch"), List::cons(Str::lit("break"), List::<Str>::nil()))))));
                let a121 = List::cons(Str::lit("continue"), List::cons(Str::lit("unless"), List::cons(Str::lit("structure"), List::cons(Str::lit("inductive"), List::cons(Str::lit("class"), List::cons(Str::lit("namespace"), List::cons(Str::lit("section"), List::<Str>::nil())))))));
                append_list(a120, a121)
            };
            let a125 = {
                let a122 = List::cons(Str::lit("local"), List::cons(Str::lit("private"), List::cons(Str::lit("protected"), List::cons(Str::lit("partial"), List::cons(Str::lit("unsafe"), List::cons(Str::lit("macro"), List::<Str>::nil()))))));
                let a123 = List::cons(Str::lit("syntax"), List::cons(Str::lit("import"), List::cons(Str::lit("export"), List::cons(Str::lit("axiom"), List::cons(Str::lit("abbrev"), List::cons(Str::lit("opaque"), List::cons(Str::lit("mutual"), List::<Str>::nil())))))));
                append_list(a122, a123)
            };
            append_list(a124, a125)
        };
        append_list(a126, a127)
    };
    let a143 = {
        let a140 = {
            let a132 = {
                let a128 = List::cons(Str::lit("variable"), List::cons(Str::lit("universe"), List::cons(Str::lit("deriving"), List::cons(Str::lit("extends"), List::cons(Str::lit("suffices"), List::cons(Str::lit("obtain"), List::<Str>::nil()))))));
                let a129 = List::cons(Str::lit("record"), List::cons(Str::lit("ctor"), List::cons(Str::lit("prim"), List::cons(Str::lit("project"), List::cons(Str::lit("cons"), List::cons(Str::lit("nil"), List::<Str>::nil()))))));
                append_list(a128, a129)
            };
            let a133 = {
                let a130 = List::cons(Str::lit("not"), List::cons(Str::lit("true"), List::cons(Str::lit("false"), List::cons(Str::lit("forall"), List::cons(Str::lit("first"), List::cons(Str::lit("second"), List::<Str>::nil()))))));
                let a131 = List::cons(Str::lit("apply"), List::cons(Str::lit("ref"), List::cons(Str::lit("checked"), List::cons(Str::lit("self"), List::cons(Str::lit("noncomputable"), List::cons(Str::lit("termination_by"), List::cons(Str::lit("and_congr"), List::<Str>::nil())))))));
                append_list(a130, a131)
            };
            append_list(a132, a133)
        };
        let a141 = {
            let a138 = {
                let a134 = List::cons(Str::lit("and_true"), List::cons(Str::lit("beq_iff_eq"), List::cons(Str::lit("congr"), List::cons(Str::lit("decide"), List::cons(Str::lit("decide_eq_false_iff_not"), List::cons(Str::lit("decide_eq_true_eq"), List::<Str>::nil()))))));
                let a135 = List::cons(Str::lit("id"), List::cons(Str::lit("reduceIte"), List::cons(Str::lit("rfl"), List::cons(Str::lit("true_and"), List::cons(Str::lit("isFalse"), List::cons(Str::lit("isTrue"), List::cons(Str::lit("none"), List::<Str>::nil())))))));
                append_list(a134, a135)
            };
            let a139 = {
                let a136 = List::cons(Str::lit("some"), List::cons(Str::lit("below"), List::cons(Str::lit("brecOn"), List::cons(Str::lit("casesOn"), List::cons(Str::lit("ctorElim"), List::cons(Str::lit("ctorElimType"), List::<Str>::nil()))))));
                let a137 = List::cons(Str::lit("ctorIdx"), List::cons(Str::lit("mk"), List::cons(Str::lit("noConfusion"), List::cons(Str::lit("noConfusionType"), List::cons(Str::lit("rec"), List::cons(Str::lit("recOn"), List::cons(Str::lit("toCtorIdx"), List::<Str>::nil())))))));
                append_list(a136, a137)
            };
            append_list(a138, a139)
        };
        append_list(a140, a141)
    };
    append_list(a142, a143)
}

pub fn f11(v0: u64) -> Str {
    let a144 = v0;
    let a145 = 0u64;
    if nat_eq(a144, a145) {
        Str::lit("0")
    } else {
        let a146 = v0;
        let a147 = 1u64;
        if nat_eq(a146, a147) {
            Str::lit("1")
        } else {
            let a148 = v0;
            let a149 = 2u64;
            if nat_eq(a148, a149) {
                Str::lit("2")
            } else {
                let a150 = v0;
                let a151 = 3u64;
                if nat_eq(a150, a151) {
                    Str::lit("3")
                } else {
                    let a152 = v0;
                    let a153 = 4u64;
                    if nat_eq(a152, a153) {
                        Str::lit("4")
                    } else {
                        let a154 = v0;
                        let a155 = 5u64;
                        if nat_eq(a154, a155) {
                            Str::lit("5")
                        } else {
                            let a156 = v0;
                            let a157 = 6u64;
                            if nat_eq(a156, a157) {
                                Str::lit("6")
                            } else {
                                let a158 = v0;
                                let a159 = 7u64;
                                if nat_eq(a158, a159) {
                                    Str::lit("7")
                                } else {
                                    let a160 = v0;
                                    let a161 = 8u64;
                                    if nat_eq(a160, a161) {
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

pub fn f12(v0: List<Adt1>) -> R<List<Str>> {
    let m162 = v0;
    match m162.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt1::C0(h163, _) => {
                    h163
                }
            };
            let v4: Str = match v1 {
                Adt1::C0(_, h164) => {
                    h164
                }
            };
            Ok(List::cons({
                let a165 = v3;
                let a166 = 0u64;
                if nat_eq(a165, a166) {
                    f18(v4)?
                } else {
                    v4
                }
            }, f12(v2)?))
        }
    }
}

pub fn f13(v0: List<Str>, v1: u64) -> R<List<Adt1>> {
    let m167 = v0;
    match m167.uncons() {
        None => {
            Ok(List::<Adt1>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt1::C0(v1, v2), f13(v3, {
                let a168 = v1;
                let a169 = 1u64;
                nat_add(a168, a169)?
            })?))
        }
    }
}

pub fn f14(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a170 = v0;
        utf8_encode(a170)
    };
    f19({
        let a171 = v1.clone();
        length_bytes(a171)
    }, v1, 0u64)
}

pub fn f15(v0: List<Str>) -> R<Str> {
    let v1: Str = Str::lit("");
    let v2: Str = f20(v0, v1)?;
    if f21(v2.clone())? {
        let a172 = List::cons(Str::lit("N"), List::cons(v2, List::<Str>::nil()));
        let a173 = Str::lit("");
        Ok(join(a172, a173))
    } else {
        Ok(v2)
    }
}

pub fn f16(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m179 = {
        let a176 = v0;
        let a177 = v1;
        let a178 = {
            let a174 = v2;
            let a175 = v1;
            nat_sub(a174, a175)
        };
        slice_bytes(a176, a177, a178)
    };
    match m179 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m181 = {
                let a180 = v3;
                utf8_decode(a180)
            };
            match m181 {
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

pub fn f17(p187: u64, v1: Bytes, v2: Bytes, p188: u64) -> R<u64> {
    let m186 = (p187, p188);
    {
        let mut m186 = m186;
        loop {
            let step = {
                let (v0, v3) = m186;
                let m182 = v0;
                if m182 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v3))
                } else {
                    let v4 = m182 - 1;
                    if f22(v1.clone(), v3, v2.clone()) {
                        Ok::<(u64, u64), R<u64>>((v4, {
                            let a184 = v3;
                            let a185 = {
                                let a183 = v2.clone();
                                length_bytes(a183)
                            };
                            nat_add(a184, a185)?
                        }))
                    } else {
                        Err::<(u64, u64), R<u64>>(Ok(v3))
                    }
                }
            };
            match step {
                Ok(next) => { m186 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f18(v0: Str) -> R<Str> {
    f23(v0, 65u64, Bytes::lit(&[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a]))
}

pub fn f19(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m189 = v0;
    if m189 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m189 - 1;
        let a191 = {
            let a190 = v1.clone();
            length_bytes(a190)
        };
        let a192 = v2;
        if nat_le(a191, a192) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f24(f4(v1.clone(), v2)?);
            Ok(List::cons(f16(v1.clone(), v2, {
                let a193 = v2;
                let a194 = v4;
                nat_add(a193, a194)?
            }), f19(v3, v1, {
                let a195 = v2;
                let a196 = v4;
                nat_add(a195, a196)?
            })?))
        }
    }
}

pub fn f20(p202: List<Str>, p203: Str) -> R<Str> {
    let m201 = (p202, p203);
    {
        let mut m201 = m201;
        loop {
            let step = {
                let (v0, v1) = m201;
                let m197 = v0;
                match m197.uncons() {
                    None => {
                        Err::<(List<Str>, Str), R<Str>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: bool = true;
                        let v5: bool = false;
                        let v6: Adt2 = f25(f14(v2)?, Adt2::C0(v1, v5, v4))?;
                        let v7: Str = match v6.clone() {
                            Adt2::C0(h198, _, _) => {
                                h198
                            }
                        };
                        let _: bool = match v6.clone() {
                            Adt2::C0(_, h199, _) => {
                                h199
                            }
                        };
                        let _: bool = match v6 {
                            Adt2::C0(_, _, h200) => {
                                h200
                            }
                        };
                        Ok::<(List<Str>, Str), R<Str>>((v3, v7))
                    }
                }
            };
            match step {
                Ok(next) => { m201 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f21(v0: Str) -> R<bool> {
    let m206 = {
        let a204 = f14(v0)?;
        let a205 = 0u64;
        index_list(a204, a205)
    };
    match m206 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            f26(v1)
        }
    }
}

pub fn f22(v0: Bytes, v1: u64, v2: Bytes) -> bool {
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

pub fn f23(v0: Str, v1: u64, v2: Bytes) -> R<Str> {
    let v3: Bytes = {
        let a214 = v0.clone();
        utf8_encode(a214)
    };
    let m218 = {
        let a217 = f27({
            let a215 = v3.clone();
            length_bytes(a215)
        }, v3, 0u64, v1, v2, {
            let a216 = Str::lit("");
            utf8_encode(a216)
        })?;
        utf8_decode(a217)
    };
    match m218 {
        None => {
            Ok(v0)
        }
        Some(v4) => {
            Ok(v4)
        }
    }
}

pub fn f24(v0: u64) -> u64 {
    let a219 = v0;
    let a220 = 128u64;
    if nat_lt(a219, a220) {
        1u64
    } else {
        let a221 = v0;
        let a222 = 224u64;
        if nat_lt(a221, a222) {
            2u64
        } else {
            let a223 = v0;
            let a224 = 240u64;
            if nat_lt(a223, a224) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f25(p240: List<Str>, p241: Adt2) -> R<Adt2> {
    let m239 = (p240, p241);
    {
        let mut m239 = m239;
        loop {
            let step = {
                let (v0, v1) = m239;
                let m225 = v0;
                match m225.uncons() {
                    None => {
                        Err::<(List<Str>, Adt2), R<Adt2>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: Str = match v1.clone() {
                            Adt2::C0(h226, _, _) => {
                                h226
                            }
                        };
                        let v5: bool = match v1.clone() {
                            Adt2::C0(_, h227, _) => {
                                h227
                            }
                        };
                        let v6: bool = match v1 {
                            Adt2::C0(_, _, h228) => {
                                h228
                            }
                        };
                        let a229 = f28(v2.clone())?;
                        if bool_not(a229) {
                            let v7: bool = true;
                            let v8: bool = false;
                            Ok::<(List<Str>, Adt2), R<Adt2>>((v3, Adt2::C0(v4, v8, v7)))
                        } else {
                            let v14: Adt3 = if v6 {
                                let v9: Str = {
                                    let a230 = List::cons(v4, List::cons(f29(v2.clone())?, List::<Str>::nil()));
                                    let a231 = Str::lit("");
                                    join(a230, a231)
                                };
                                let v10: bool = false;
                                Adt3::C0(v9, v10)
                            } else {
                                let v13: Str = if if f30(v2.clone())? {
                                    let a232 = v5;
                                    bool_not(a232)
                                } else {
                                    false
                                } {
                                    let v11: Str = {
                                        let a233 = List::cons(v4, List::cons(f18(v2.clone())?, List::<Str>::nil()));
                                        let a234 = Str::lit("");
                                        join(a233, a234)
                                    };
                                    v11
                                } else {
                                    let v12: Str = {
                                        let a235 = List::cons(v4, List::cons(v2.clone(), List::<Str>::nil()));
                                        let a236 = Str::lit("");
                                        join(a235, a236)
                                    };
                                    v12
                                };
                                Adt3::C0(v13, v6)
                            };
                            let v15: Str = match v14.clone() {
                                Adt3::C0(h237, _) => {
                                    h237
                                }
                            };
                            let v16: bool = match v14 {
                                Adt3::C0(_, h238) => {
                                    h238
                                }
                            };
                            let v17: bool = if f31(v2.clone())? {
                                true
                            } else {
                                f26(v2)?
                            };
                            Ok::<(List<Str>, Adt2), R<Adt2>>((v3, Adt2::C0(v15, v17, v16)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m239 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f26(v0: Str) -> R<bool> {
    let v1: u64 = f32(v0)?;
    let a242 = 48u64;
    let a243 = v1;
    if nat_le(a242, a243) {
        let a244 = v1;
        let a245 = 57u64;
        Ok(nat_le(a244, a245))
    } else {
        Ok(false)
    }
}

pub fn f27(p268: u64, v1: Bytes, p269: u64, v3: u64, v4: Bytes, p270: Bytes) -> R<Bytes> {
    let m267 = (p268, (p269, p270));
    {
        let mut m267 = m267;
        loop {
            let step = {
                let (v0, (v2, v5)) = m267;
                let m246 = v0;
                if m246 == 0 {
                    Err::<(u64, (u64, Bytes)), R<Bytes>>(Ok(v5))
                } else {
                    let v6 = m246 - 1;
                    let m250 = {
                        let a247 = v1.clone();
                        let a248 = v2;
                        let a249 = 1u64;
                        slice_bytes(a247, a248, a249)
                    };
                    match m250 {
                        None => {
                            Err::<(u64, (u64, Bytes)), R<Bytes>>(Ok(v5))
                        }
                        Some(v7) => {
                            let v8: u64 = f4(v1.clone(), v2)?;
                            let v10: Bytes = {
                                let a251 = v3;
                                let a252 = v8;
                                if if nat_le(a251, a252) {
                                    let a255 = v8;
                                    let a256 = {
                                        let a253 = v3;
                                        let a254 = 26u64;
                                        nat_add(a253, a254)?
                                    };
                                    nat_lt(a255, a256)
                                } else {
                                    false
                                } {
                                    let m262 = {
                                        let a259 = v4.clone();
                                        let a260 = {
                                            let a257 = v8;
                                            let a258 = v3;
                                            nat_sub(a257, a258)
                                        };
                                        let a261 = 1u64;
                                        slice_bytes(a259, a260, a261)
                                    };
                                    match m262 {
                                        None => {
                                            v7
                                        }
                                        Some(v9) => {
                                            v9
                                        }
                                    }
                                } else {
                                    v7
                                }
                            };
                            Ok::<(u64, (u64, Bytes)), R<Bytes>>((v6, ({
                                let a263 = v2;
                                let a264 = 1u64;
                                nat_add(a263, a264)?
                            }, {
                                let a265 = v5;
                                let a266 = v10;
                                append_bytes(a265, a266)
                            })))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m267 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f28(v0: Str) -> R<bool> {
    if f33(v0.clone())? {
        Ok(true)
    } else {
        f26(v0)
    }
}

pub fn f29(v0: Str) -> R<Str> {
    f23(v0, 97u64, Bytes::lit(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a]))
}

pub fn f30(v0: Str) -> R<bool> {
    let v1: u64 = f32(v0)?;
    let a271 = 65u64;
    let a272 = v1;
    if nat_le(a271, a272) {
        let a273 = v1;
        let a274 = 90u64;
        Ok(nat_le(a273, a274))
    } else {
        Ok(false)
    }
}

pub fn f31(v0: Str) -> R<bool> {
    let v1: u64 = f32(v0)?;
    let a275 = 97u64;
    let a276 = v1;
    if nat_le(a275, a276) {
        let a277 = v1;
        let a278 = 122u64;
        Ok(nat_le(a277, a278))
    } else {
        Ok(false)
    }
}

pub fn f32(v0: Str) -> R<u64> {
    f4({
        let a279 = v0;
        utf8_encode(a279)
    }, 0u64)
}

pub fn f33(v0: Str) -> R<bool> {
    if f30(v0.clone())? {
        Ok(true)
    } else {
        f31(v0)
    }
}

pub fn run(p0: Str) -> R<Str> {
    f0(p0)
}
