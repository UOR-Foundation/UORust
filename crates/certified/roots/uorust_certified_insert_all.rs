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
    C2(Option<u64>),
    C3(u64),
    C4(Adt1),
    C5(Option<u64>),
    C6(Option<u64>),
    C7,
    C8,
    C9,
    C10,
    C11,
    C12(Str),
    C13(std::rc::Rc<Adt0>),
    C14(std::rc::Rc<Adt0>, std::rc::Rc<Adt0>),
    C15(std::rc::Rc<Adt0>),
    C16(List<Adt0>),
    C17(List<Adt0>),
    C18(std::rc::Rc<Adt0>, std::rc::Rc<Adt0>, bool),
    C19(std::rc::Rc<Adt0>, bool),
    C20(List<Adt0>, std::rc::Rc<Adt0>),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Adt3, List<Adt13>, List<Adt15>, List<Str>),
}

#[derive(Clone)]
pub enum Adt3 {
    C0(List<Adt4>, List<Adt5>, List<Adt6>, List<Adt7>, List<List<Str>>, List<Adt4>, List<Adt4>, bool, List<Adt8>, List<Adt9>, List<Adt10>, List<Adt12>, List<Adt6>),
}

#[derive(Clone)]
pub enum Adt4 {
    C0(Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt5 {
    C0(Str, List<(Str, Adt0)>),
}

#[derive(Clone)]
pub enum Adt6 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(Str, List<(Str, List<(Option<Str>, Adt0)>)>),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str, Adt1),
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
    C0(Str, List<Adt0>, Adt0),
}

#[derive(Clone)]
pub enum Adt15 {
    C0(u64, List<Str>),
}

#[derive(Clone)]
pub enum Adt16 {
    C0(Str, List<Str>, Adt0, Str, List<Adt17>, List<Adt6>, bool, Option<List<Str>>, List<Str>, List<Adt6>, List<Adt18>, Option<Str>, List<Adt6>, List<Adt68>, List<Adt68>, Option<Adt0>, Option<Str>),
}

#[derive(Clone)]
pub enum Adt17 {
    C0(Str, (Str, Adt0)),
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
    C0(List<u64>, List<Str>, List<Str>, List<Adt0>, List<Str>, List<Adt70>, List<Adt0>),
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
    C2(Str, List<Adt4>, Option<Str>),
    C3(Str, List<Adt6>, Str, Adt71, List<Str>, Option<Str>, Option<Adt4>, Option<Str>),
    C4(Str, List<Adt6>, Str, Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt74 {
    C0(Str, Adt69),
}

#[derive(Clone)]
pub enum Adt75 {
    C0,
    C1,
    C2,
    C3,
}

pub fn f0(v0: Adt0, v1: Adt0, v2: Str, v3: Adt2, v4: Adt16, v5: Adt69) -> R<Adt74> {
    let v6: Adt74 = f1(v4.clone(), v3.clone(), v5, Str::lit("set"))?;
    let v7: Str = match v6.clone() {
        Adt74::C0(h1, _) => {
            h1
        }
    };
    let v8: Adt69 = match v6 {
        Adt74::C0(_, h2) => {
            h2
        }
    };
    let v9: Adt74 = f1(v4.clone(), v3.clone(), v8, Str::lit("items"))?;
    let v10: Str = match v9.clone() {
        Adt74::C0(h3, _) => {
            h3
        }
    };
    let v11: Adt69 = match v9 {
        Adt74::C0(_, h4) => {
            h4
        }
    };
    let v12: Adt74 = f1(v4.clone(), v3.clone(), v11, Str::lit("head"))?;
    let v13: Str = match v12.clone() {
        Adt74::C0(h5, _) => {
            h5
        }
    };
    let v14: Adt69 = match v12 {
        Adt74::C0(_, h6) => {
            h6
        }
    };
    let v15: Adt74 = f1(v4.clone(), v3.clone(), v14, Str::lit("rest"))?;
    let v16: Str = match v15.clone() {
        Adt74::C0(h7, _) => {
            h7
        }
    };
    let v17: Adt69 = match v15 {
        Adt74::C0(_, h8) => {
            h8
        }
    };
    let v18: Str = f2(v4, v3, v17.clone(), Str::lit("InsertAll"))?;
    let v19: Adt71 = Adt71::C1(v10.clone(), List::cons(Adt72::C0(Str::lit("List.nil"), Adt71::C0(v7.clone())), List::cons(Adt72::C0({
        let a11 = {
            let a9 = List::cons(Str::lit("List.cons "), List::cons(v13.clone(), List::<Str>::nil()));
            let a10 = List::cons(Str::lit(" "), List::cons(v16.clone(), List::<Str>::nil()));
            append_list(a9, a10)
        };
        let a12 = Str::lit("");
        join(a11, a12)
    }, Adt71::C0({
        let a19 = {
            let a17 = {
                let a13 = List::cons(Str::lit("("), List::cons(v18.clone(), List::<Str>::nil()));
                let a14 = List::cons(Str::lit(" "), List::cons(v16, List::cons(Str::lit(" ("), List::<Str>::nil())));
                append_list(a13, a14)
            };
            let a18 = {
                let a15 = List::cons(v2.clone(), List::cons(Str::lit(" "), List::cons(v7.clone(), List::<Str>::nil())));
                let a16 = List::cons(Str::lit(" "), List::cons(v13, List::cons(Str::lit("))"), List::<Str>::nil())));
                append_list(a15, a16)
            };
            append_list(a17, a18)
        };
        let a20 = Str::lit("");
        join(a19, a20)
    })), List::<Adt72>::nil())));
    let v20: List<Str> = f3(v17.clone(), v2.clone());
    let v21: Adt69 = f4(v17, Adt70::C0(v18.clone(), List::cons(Adt6::C0(v10, f5(v1)), List::cons(Adt6::C0(v7, f5(v0.clone())), List::<Adt6>::nil())), f5(v0), v19, f6(List::cons({
        let a21 = List::cons(Str::lit("#"), List::cons(v2, List::<Str>::nil()));
        let a22 = Str::lit("");
        join(a21, a22)
    }, List::<Str>::nil()), List::<Str>::nil()), v20, true, false, None::<(Str, List<Adt73>)>));
    let v22: Adt69 = f7(v21, {
        let a23 = List::cons(Str::lit("#"), List::cons(v18.clone(), List::<Str>::nil()));
        let a24 = Str::lit("");
        join(a23, a24)
    });
    Ok(Adt74::C0(v18, v22))
}

pub fn f1(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str) -> R<Adt74> {
    let v4: Str = f8(v3.clone(), f9(v0, v1, v2.clone(), v3, List::<Str>::nil()))?;
    let v5: List<Str> = match v2.clone() {
        Adt69::C0(_, _, h25, _, _, _, _) => {
            h25
        }
    };
    let v6: List<Str> = {
        let a26 = v5;
        let a27 = List::cons(v4.clone(), List::<Str>::nil());
        append_list(a26, a27)
    };
    Ok(Adt74::C0(v4, Adt69::C0(match v2.clone() {
        Adt69::C0(h28, _, _, _, _, _, _) => {
            h28
        }
    }, match v2.clone() {
        Adt69::C0(_, h29, _, _, _, _, _) => {
            h29
        }
    }, v6, match v2.clone() {
        Adt69::C0(_, _, _, h30, _, _, _) => {
            h30
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, h31, _, _) => {
            h31
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, _, h32, _) => {
            h32
        }
    }, match v2 {
        Adt69::C0(_, _, _, _, _, _, h33) => {
            h33
        }
    })))
}

pub fn f2(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str) -> R<Str> {
    let v4: Str = {
        let a35 = List::cons(match v0.clone() {
            Adt16::C0(_, _, _, h34, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                h34
            }
        }, List::cons(v3, List::<Str>::nil()));
        let a36 = Str::lit("");
        join(a35, a36)
    };
    let v5: List<Str> = f10(match v2.clone() {
        Adt69::C0(_, _, _, _, _, h37, _) => {
            h37
        }
    });
    f8(v4.clone(), f9(v0, v1, v2, v4, v5))
}

pub fn f3(v0: Adt69, v1: Str) -> List<Str> {
    let m38 = f11(v0, v1);
    match m38 {
        None => {
            List::<Str>::nil()
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f4(v0: Adt69, v1: Adt70) -> Adt69 {
    let v2: List<Adt70> = match v0.clone() {
        Adt69::C0(_, _, _, _, _, h39, _) => {
            h39
        }
    };
    let v3: List<Adt70> = {
        let a40 = v2;
        let a41 = List::cons(v1, List::<Adt70>::nil());
        append_list(a40, a41)
    };
    Adt69::C0(match v0.clone() {
        Adt69::C0(h42, _, _, _, _, _, _) => {
            h42
        }
    }, match v0.clone() {
        Adt69::C0(_, h43, _, _, _, _, _) => {
            h43
        }
    }, match v0.clone() {
        Adt69::C0(_, _, h44, _, _, _, _) => {
            h44
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, h45, _, _, _) => {
            h45
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, h46, _, _) => {
            h46
        }
    }, v3, match v0 {
        Adt69::C0(_, _, _, _, _, _, h47) => {
            h47
        }
    })
}

pub fn f5(v0: Adt0) -> Str {
    let m48 = v0;
    match m48 {
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
            Str::lit("Int")
        }
        Adt0::C6(_) => {
            Str::lit("Nat")
        }
        Adt0::C7 => {
            Str::lit("String")
        }
        Adt0::C8 => {
            Str::lit("String")
        }
        Adt0::C9 => {
            Str::lit("Nat")
        }
        Adt0::C10 => {
            let a49 = List::cons(f12(), List::cons(Str::lit("::ParseIntError"), List::<Str>::nil()));
            let a50 = Str::lit("");
            join(a49, a50)
        }
        Adt0::C11 => {
            let a51 = List::cons(f12(), List::cons(Str::lit("::FilePath"), List::<Str>::nil()));
            let a52 = Str::lit("");
            join(a51, a52)
        }
        Adt0::C12(v6) => {
            v6
        }
        Adt0::C13(r89) => {
            let v7 = (*r89).clone();
            let a55 = {
                let a53 = List::cons(Str::lit("(Option "), List::<Str>::nil());
                let a54 = List::cons(f5(v7), List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a53, a54)
            };
            let a56 = Str::lit("");
            join(a55, a56)
        }
        Adt0::C14(r90, r91) => {
            let v8 = (*r90).clone();
            let v9 = (*r91).clone();
            let a59 = {
                let a57 = List::cons(Str::lit("(Result "), List::cons(f5(v8), List::<Str>::nil()));
                let a58 = List::cons(Str::lit(" "), List::cons(f5(v9), List::cons(Str::lit(")"), List::<Str>::nil())));
                append_list(a57, a58)
            };
            let a60 = Str::lit("");
            join(a59, a60)
        }
        Adt0::C15(r92) => {
            let v10 = (*r92).clone();
            let a63 = {
                let a61 = List::cons(Str::lit("(List "), List::<Str>::nil());
                let a62 = List::cons(f5(v10), List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a61, a62)
            };
            let a64 = Str::lit("");
            join(a63, a64)
        }
        Adt0::C16(v11) => {
            let a67 = List::cons(Str::lit("Tuple"), List::cons({
                let a65 = f13(v11);
                let a66 = Str::lit("");
                join(a65, a66)
            }, List::<Str>::nil()));
            let a68 = Str::lit("");
            join(a67, a68)
        }
        Adt0::C17(v12) => {
            let a73 = {
                let a71 = List::cons(Str::lit("("), List::<Str>::nil());
                let a72 = List::cons({
                    let a69 = f14(v12);
                    let a70 = Str::lit(" * ");
                    join(a69, a70)
                }, List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a71, a72)
            };
            let a74 = Str::lit("");
            join(a73, a74)
        }
        Adt0::C18(r93, r94, _) => {
            let v13 = (*r93).clone();
            let v14 = (*r94).clone();
            let a77 = {
                let a75 = List::cons(Str::lit("(List Tuple"), List::cons(f15(v13), List::<Str>::nil()));
                let a76 = List::cons(f15(v14), List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a75, a76)
            };
            let a78 = Str::lit("");
            join(a77, a78)
        }
        Adt0::C19(r95, _) => {
            let v16 = (*r95).clone();
            let a81 = {
                let a79 = List::cons(Str::lit("(List "), List::<Str>::nil());
                let a80 = List::cons(f5(v16), List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a79, a80)
            };
            let a82 = Str::lit("");
            join(a81, a82)
        }
        Adt0::C20(v18, r96) => {
            let v19 = (*r96).clone();
            let a87 = {
                let a85 = List::cons(Str::lit("(Fn ("), List::cons({
                    let a83 = f16(v18);
                    let a84 = Str::lit(" ");
                    join(a83, a84)
                }, List::<Str>::nil()));
                let a86 = List::cons(Str::lit(") "), List::cons(f5(v19), List::cons(Str::lit(")"), List::<Str>::nil())));
                append_list(a85, a86)
            };
            let a88 = Str::lit("");
            join(a87, a88)
        }
    }
}

pub fn f6(p99: List<Str>, p100: List<Str>) -> List<Str> {
    let m98 = (p99, p100);
    {
        let mut m98 = m98;
        loop {
            let step = {
                let (v0, v1) = m98;
                let m97 = v0;
                match m97.uncons() {
                    None => {
                        Err::<(List<Str>, List<Str>), List<Str>>(v1)
                    }
                    Some((v2, v3)) => {
                        Ok::<(List<Str>, List<Str>), List<Str>>((v3, f17(v1, v2)))
                    }
                }
            };
            match step {
                Ok(next) => { m98 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f7(v0: Adt69, v1: Str) -> Adt69 {
    let v2: List<Str> = match v0.clone() {
        Adt69::C0(_, h101, _, _, _, _, _) => {
            h101
        }
    };
    let v3: List<Str> = f18(v2, v1);
    Adt69::C0(match v0.clone() {
        Adt69::C0(h102, _, _, _, _, _, _) => {
            h102
        }
    }, v3, match v0.clone() {
        Adt69::C0(_, _, h103, _, _, _, _) => {
            h103
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, h104, _, _, _) => {
            h104
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, h105, _, _) => {
            h105
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, _, h106, _) => {
            h106
        }
    }, match v0 {
        Adt69::C0(_, _, _, _, _, _, h107) => {
            h107
        }
    })
}

pub fn f8(v0: Str, v1: List<Str>) -> R<Str> {
    let v2: u64 = f19(f20(f21(v1.clone(), v0.clone()))?, 0u64);
    let m114 = {
        let a110 = {
            let a108 = v2;
            let a109 = 1u64;
            nat_add(a108, a109)?
        };
        let a111 = 4294967296u64;
        if nat_lt(a110, a111) {
            Some({
                let a112 = v2;
                let a113 = 1u64;
                nat_add(a112, a113)?
            })
        } else {
            None::<u64>
        }
    };
    match m114 {
        None => {
            let m120 = f22(f23(f24(1u64, {
                let a118 = {
                    let a116 = {
                        let a115 = v1.clone();
                        length_list(a115)
                    };
                    let a117 = 2u64;
                    nat_add(a116, a117)?
                };
                let a119 = 1u64;
                nat_sub(a118, a119)
            })?, v0)?, v1);
            match m120 {
                None => {
                    Ok(Str::lit(""))
                }
                Some(v3) => {
                    Ok(v3)
                }
            }
        }
        Some(v4) => {
            let a121 = List::cons(v0, List::cons(f25(v4)?, List::<Str>::nil()));
            let a122 = Str::lit("");
            Ok(join(a121, a122))
        }
    }
}

pub fn f9(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str, v4: List<Str>) -> List<Str> {
    f26({
        let a136 = {
            let a134 = {
                let a131 = {
                    let a128 = {
                        let a125 = f27(f28(match v0.clone() {
                            Adt16::C0(_, _, _, _, h123, _, _, _, _, _, _, _, _, _, _, _, _) => {
                                h123
                            }
                        }));
                        let a126 = match v0 {
                            Adt16::C0(_, _, _, _, _, _, _, _, h124, _, _, _, _, _, _, _, _) => {
                                h124
                            }
                        };
                        append_list(a125, a126)
                    };
                    let a129 = f29(f30(match v1.clone() {
                        Adt2::C0(_, h127, _, _) => {
                            h127
                        }
                    }));
                    append_list(a128, a129)
                };
                let a132 = match v1 {
                    Adt2::C0(_, _, _, h130) => {
                        h130
                    }
                };
                append_list(a131, a132)
            };
            let a135 = match v2 {
                Adt69::C0(_, _, h133, _, _, _, _) => {
                    h133
                }
            };
            append_list(a134, a135)
        };
        let a137 = v4;
        append_list(a136, a137)
    }, v3)
}

pub fn f10(v0: List<Adt70>) -> List<Str> {
    let m138 = v0;
    match m138.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt70::C0(h139, _, _, _, _, _, _, _, _) => {
                    h139
                }
            }, f10(v2))
        }
    }
}

pub fn f11(v0: Adt69, v1: Str) -> Option<List<Str>> {
    let m141 = f31(match v0 {
        Adt69::C0(_, _, _, _, _, h140, _) => {
            h140
        }
    }, v1);
    match m141 {
        None => {
            None::<List<Str>>
        }
        Some(v2) => {
            Some(match v2 {
                Adt70::C0(_, _, _, _, _, h142, _, _, _) => {
                    h142
                }
            })
        }
    }
}

pub fn f12() -> Str {
    Str::lit("Prelude")
}

pub fn f13(v0: List<Adt0>) -> List<Str> {
    let m143 = v0;
    match m143.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f13(v2))
        }
    }
}

pub fn f14(v0: List<Adt0>) -> List<Str> {
    let m144 = v0;
    match m144.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f5(v1), f14(v2))
        }
    }
}

pub fn f15(v0: Adt0) -> Str {
    let m145 = v0;
    match m145 {
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
            f32()
        }
        Adt0::C6(_) => {
            Str::lit("Nat")
        }
        Adt0::C7 => {
            Str::lit("String")
        }
        Adt0::C8 => {
            Str::lit("String")
        }
        Adt0::C9 => {
            Str::lit("Nat")
        }
        Adt0::C10 => {
            Str::lit("ParseIntError")
        }
        Adt0::C11 => {
            Str::lit("FilePath")
        }
        Adt0::C12(v6) => {
            v6
        }
        Adt0::C13(r174) => {
            let v7 = (*r174).clone();
            let a146 = List::cons(Str::lit("Option"), List::cons(f15(v7), List::<Str>::nil()));
            let a147 = Str::lit("");
            join(a146, a147)
        }
        Adt0::C14(r175, r176) => {
            let v8 = (*r175).clone();
            let v9 = (*r176).clone();
            let a150 = {
                let a148 = List::cons(Str::lit("Result"), List::<Str>::nil());
                let a149 = List::cons(f15(v8), List::cons(f15(v9), List::<Str>::nil()));
                append_list(a148, a149)
            };
            let a151 = Str::lit("");
            join(a150, a151)
        }
        Adt0::C15(r177) => {
            let v10 = (*r177).clone();
            let a152 = List::cons(Str::lit("List"), List::cons(f15(v10), List::<Str>::nil()));
            let a153 = Str::lit("");
            join(a152, a153)
        }
        Adt0::C16(v11) => {
            let a156 = List::cons(Str::lit("Tuple"), List::cons({
                let a154 = f33(v11);
                let a155 = Str::lit("");
                join(a154, a155)
            }, List::<Str>::nil()));
            let a157 = Str::lit("");
            join(a156, a157)
        }
        Adt0::C17(v12) => {
            let a160 = List::cons(Str::lit("Pair"), List::cons({
                let a158 = f34(v12);
                let a159 = Str::lit("");
                join(a158, a159)
            }, List::<Str>::nil()));
            let a161 = Str::lit("");
            join(a160, a161)
        }
        Adt0::C18(r178, r179, _) => {
            let v13 = (*r178).clone();
            let v14 = (*r179).clone();
            let a164 = {
                let a162 = List::cons(Str::lit("ListTuple"), List::<Str>::nil());
                let a163 = List::cons(f15(v13), List::cons(f15(v14), List::<Str>::nil()));
                append_list(a162, a163)
            };
            let a165 = Str::lit("");
            join(a164, a165)
        }
        Adt0::C19(r180, _) => {
            let v16 = (*r180).clone();
            let a166 = List::cons(Str::lit("List"), List::cons(f15(v16), List::<Str>::nil()));
            let a167 = Str::lit("");
            join(a166, a167)
        }
        Adt0::C20(v18, r181) => {
            let v19 = (*r181).clone();
            let a172 = {
                let a170 = List::cons(Str::lit("Fn"), List::cons({
                    let a168 = f35(v18);
                    let a169 = Str::lit("");
                    join(a168, a169)
                }, List::<Str>::nil()));
                let a171 = List::cons(Str::lit("To"), List::cons(f15(v19), List::<Str>::nil()));
                append_list(a170, a171)
            };
            let a173 = Str::lit("");
            join(a172, a173)
        }
    }
}

pub fn f16(v0: List<Adt0>) -> List<Str> {
    let m182 = v0;
    match m182.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f5(v1), f16(v2))
        }
    }
}

pub fn f17(v0: List<Str>, v1: Str) -> List<Str> {
    let m183 = v0;
    match m183.uncons() {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some((v2, v3)) => {
            let a184 = v1.clone();
            let a185 = v2.clone();
            if equal(a184, a185) {
                List::cons(v2, v3)
            } else {
                let m188 = {
                    let a186 = v1.clone();
                    let a187 = v2.clone();
                    compare(a186, a187)
                };
                if match m188 {
                    Ordering::Less => {
                        true
                    }
                    Ordering::Equal => {
                        false
                    }
                    Ordering::Greater => {
                        false
                    }
                } {
                    List::cons(v1, List::cons(v2, v3))
                } else {
                    List::cons(v2, f17(v3, v1))
                }
            }
        }
    }
}

pub fn f18(v0: List<Str>, v1: Str) -> List<Str> {
    let m189 = v0;
    match m189.uncons() {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some((v2, v3)) => {
            let a190 = v1.clone();
            let a191 = v2.clone();
            if equal(a190, a191) {
                List::cons(v2, v3)
            } else {
                let m194 = {
                    let a192 = v1.clone();
                    let a193 = v2.clone();
                    compare(a192, a193)
                };
                if match m194 {
                    Ordering::Less => {
                        true
                    }
                    Ordering::Equal => {
                        false
                    }
                    Ordering::Greater => {
                        false
                    }
                } {
                    List::cons(v1, List::cons(v2, v3))
                } else {
                    List::cons(v2, f18(v3, v1))
                }
            }
        }
    }
}

pub fn f19(p199: List<u64>, p200: u64) -> u64 {
    let m198 = (p199, p200);
    {
        let mut m198 = m198;
        loop {
            let step = {
                let (v0, v1) = m198;
                let m195 = v0;
                match m195.uncons() {
                    None => {
                        Err::<(List<u64>, u64), u64>(v1)
                    }
                    Some((v2, v3)) => {
                        Ok::<(List<u64>, u64), u64>((v3, {
                            let a196 = v1;
                            let a197 = v2;
                            if nat_le(a196, a197) {
                                v2
                            } else {
                                v1
                            }
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m198 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f20(v0: List<Str>) -> R<List<u64>> {
    let m201 = v0;
    match m201.uncons() {
        None => {
            Ok(List::<u64>::nil())
        }
        Some((v1, v2)) => {
            let m202 = f36(v1)?;
            match m202 {
                None => {
                    f20(v2)
                }
                Some(v3) => {
                    Ok(List::cons(v3, f20(v2)?))
                }
            }
        }
    }
}

pub fn f21(v0: List<Str>, v1: Str) -> List<Str> {
    let m203 = v0;
    match m203.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            let m204 = f37(v2, v1.clone());
            match m204 {
                None => {
                    f21(v3, v1)
                }
                Some(v4) => {
                    List::cons(v4, f21(v3, v1))
                }
            }
        }
    }
}

pub fn f22(p208: List<Str>, v1: List<Str>) -> Option<Str> {
    let m207 = p208;
    {
        let mut m207 = m207;
        loop {
            let step = {
                let v0 = m207;
                let m205 = v0;
                match m205.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a206 = f38(v1.clone(), v2.clone());
                        if bool_not(a206) {
                            Err::<List<Str>, Option<Str>>(Some(v2))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m207 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f23(v0: List<u64>, v1: Str) -> R<List<Str>> {
    let m209 = v0;
    match m209.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons({
                let a210 = List::cons(v1.clone(), List::cons(f25(v2)?, List::<Str>::nil()));
                let a211 = Str::lit("");
                join(a210, a211)
            }, f23(v3, v1)?))
        }
    }
}

pub fn f24(v0: u64, v1: u64) -> R<List<u64>> {
    let m212 = v1;
    if m212 == 0 {
        Ok(List::<u64>::nil())
    } else {
        let v2 = m212 - 1;
        Ok(List::cons(v0, f24({
            let a213 = v0;
            let a214 = 1u64;
            nat_add(a213, a214)?
        }, v2)?))
    }
}

pub fn f25(v0: u64) -> R<Str> {
    Ok(f39({
        let a215 = v0;
        let a216 = 1u64;
        nat_add(a215, a216)?
    }, v0))
}

pub fn f26(v0: List<Str>, v1: Str) -> List<Str> {
    let m217 = v0;
    match m217.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            if f40(v2.clone(), v1.clone()) {
                List::cons(v2, f26(v3, v1))
            } else {
                f26(v3, v1)
            }
        }
    }
}

pub fn f27(v0: List<(Str, Adt0)>) -> List<Str> {
    let m218 = v0;
    match m218.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            let v3: Str = {
                let (h219, _) = v1;
                h219
            };
            let _: Adt0 = {
                let (_, h220) = v1;
                h220
            };
            List::cons(v3, f27(v2))
        }
    }
}

pub fn f28(v0: List<Adt17>) -> List<(Str, Adt0)> {
    let m221 = v0;
    match m221.uncons() {
        None => {
            List::<(Str, Adt0)>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt17::C0(_, h222) => {
                    h222
                }
            }, f28(v2))
        }
    }
}

pub fn f29(v0: List<Adt14>) -> List<Str> {
    let m223 = v0;
    match m223.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt14::C0(h224, _, _) => {
                    h224
                }
            }, f29(v2))
        }
    }
}

pub fn f30(v0: List<Adt13>) -> List<Adt14> {
    let m225 = v0;
    match m225.uncons() {
        None => {
            List::<Adt14>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt13::C0(_, h226) => {
                    h226
                }
            }, f30(v2))
        }
    }
}

pub fn f31(p232: List<Adt70>, v1: Str) -> Option<Adt70> {
    let m231 = p232;
    {
        let mut m231 = m231;
        loop {
            let step = {
                let v0 = m231;
                let m227 = v0;
                match m227.uncons() {
                    None => {
                        Err::<List<Adt70>, Option<Adt70>>(None::<Adt70>)
                    }
                    Some((v2, v3)) => {
                        let a229 = match v2.clone() {
                            Adt70::C0(h228, _, _, _, _, _, _, _, _) => {
                                h228
                            }
                        };
                        let a230 = v1.clone();
                        if equal(a229, a230) {
                            Err::<List<Adt70>, Option<Adt70>>(Some(v2))
                        } else {
                            Ok::<List<Adt70>, Option<Adt70>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m231 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f32() -> Str {
    Str::lit("LexLeanInt")
}

pub fn f33(v0: List<Adt0>) -> List<Str> {
    let m233 = v0;
    match m233.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f33(v2))
        }
    }
}

pub fn f34(v0: List<Adt0>) -> List<Str> {
    let m234 = v0;
    match m234.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f34(v2))
        }
    }
}

pub fn f35(v0: List<Adt0>) -> List<Str> {
    let m235 = v0;
    match m235.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f35(v2))
        }
    }
}

pub fn f36(v0: Str) -> R<Option<u64>> {
    let m236 = f41(v0, 4294967295u64)?;
    match m236 {
        Ok(v1) => {
            Ok(Some(v1))
        }
        Err(_) => {
            Ok(None::<u64>)
        }
    }
}

pub fn f37(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a237 = v0;
        utf8_encode(a237)
    };
    let v3: Bytes = {
        let a238 = v1;
        utf8_encode(a238)
    };
    if f42(v2.clone(), 0u64, v3.clone()) {
        Some(f43(v2.clone(), {
            let a239 = v3;
            length_bytes(a239)
        }, {
            let a240 = v2;
            length_bytes(a240)
        }))
    } else {
        None::<Str>
    }
}

pub fn f38(p245: List<Str>, v1: Str) -> bool {
    let m244 = p245;
    {
        let mut m244 = m244;
        loop {
            let step = {
                let v0 = m244;
                let m241 = v0;
                match m241.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a242 = v2;
                        let a243 = v1.clone();
                        if equal(a242, a243) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m244 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f39(v0: u64, v1: u64) -> Str {
    let m246 = v0;
    if m246 == 0 {
        Str::lit("")
    } else {
        let v2 = m246 - 1;
        let a247 = v1;
        let a248 = 10u64;
        if nat_lt(a247, a248) {
            f44(v1)
        } else {
            let a255 = List::cons(f39(v2, {
                let a249 = v1;
                let a250 = 10u64;
                let a251 = 0u64;
                nat_quot(a249, a250, a251)
            }), List::cons(f44({
                let a252 = v1;
                let a253 = 10u64;
                let a254 = 0u64;
                nat_rem(a252, a253, a254)
            }), List::<Str>::nil()));
            let a256 = Str::lit("");
            join(a255, a256)
        }
    }
}

pub fn f40(v0: Str, v1: Str) -> bool {
    f42({
        let a257 = v0;
        utf8_encode(a257)
    }, 0u64, {
        let a258 = v1;
        utf8_encode(a258)
    })
}

pub fn f41(v0: Str, v1: u64) -> R<Result<u64, Adt75>> {
    let v2: Bytes = {
        let a259 = v0;
        utf8_encode(a259)
    };
    let v3: u64 = {
        let a260 = v2.clone();
        length_bytes(a260)
    };
    let a261 = v3;
    let a262 = 0u64;
    if nat_eq(a261, a262) {
        Ok(Err::<u64, Adt75>(Adt75::C0))
    } else {
        let a263 = f45(v2.clone(), 0u64)?;
        let a264 = 43u64;
        if nat_eq(a263, a264) {
            let a265 = v3;
            let a266 = 1u64;
            if nat_eq(a265, a266) {
                Ok(Err::<u64, Adt75>(Adt75::C1))
            } else {
                f46(v3, v2, 1u64, 0u64, v1)
            }
        } else {
            f46(v3, v2, 0u64, 0u64, v1)
        }
    }
}

pub fn f42(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m271 = {
        let a268 = v0;
        let a269 = v1;
        let a270 = {
            let a267 = v2.clone();
            length_bytes(a267)
        };
        slice_bytes(a268, a269, a270)
    };
    match m271 {
        None => {
            false
        }
        Some(v3) => {
            let a272 = v3;
            let a273 = v2;
            equal(a272, a273)
        }
    }
}

pub fn f43(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m279 = {
        let a276 = v0;
        let a277 = v1;
        let a278 = {
            let a274 = v2;
            let a275 = v1;
            nat_sub(a274, a275)
        };
        slice_bytes(a276, a277, a278)
    };
    match m279 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m281 = {
                let a280 = v3;
                utf8_decode(a280)
            };
            match m281 {
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

pub fn f44(v0: u64) -> Str {
    let a282 = v0;
    let a283 = 0u64;
    if nat_eq(a282, a283) {
        Str::lit("0")
    } else {
        let a284 = v0;
        let a285 = 1u64;
        if nat_eq(a284, a285) {
            Str::lit("1")
        } else {
            let a286 = v0;
            let a287 = 2u64;
            if nat_eq(a286, a287) {
                Str::lit("2")
            } else {
                let a288 = v0;
                let a289 = 3u64;
                if nat_eq(a288, a289) {
                    Str::lit("3")
                } else {
                    let a290 = v0;
                    let a291 = 4u64;
                    if nat_eq(a290, a291) {
                        Str::lit("4")
                    } else {
                        let a292 = v0;
                        let a293 = 5u64;
                        if nat_eq(a292, a293) {
                            Str::lit("5")
                        } else {
                            let a294 = v0;
                            let a295 = 6u64;
                            if nat_eq(a294, a295) {
                                Str::lit("6")
                            } else {
                                let a296 = v0;
                                let a297 = 7u64;
                                if nat_eq(a296, a297) {
                                    Str::lit("7")
                                } else {
                                    let a298 = v0;
                                    let a299 = 8u64;
                                    if nat_eq(a298, a299) {
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

pub fn f45(v0: Bytes, v1: u64) -> R<u64> {
    let m302 = {
        let a300 = v0;
        let a301 = v1;
        index_bytes(a300, a301)
    };
    match m302 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a347 = {
                let a323 = {
                    let a311 = {
                        let a305 = {
                            let a303 = v3;
                            let a304 = 128u8;
                            fixed_u8::bit_and(a303, a304)
                        };
                        let a306 = 128u8;
                        if equal(a305, a306) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a312 = {
                        let a309 = {
                            let a307 = v3;
                            let a308 = 64u8;
                            fixed_u8::bit_and(a307, a308)
                        };
                        let a310 = 64u8;
                        if equal(a309, a310) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a311, a312)?
                };
                let a324 = {
                    let a321 = {
                        let a315 = {
                            let a313 = v3;
                            let a314 = 32u8;
                            fixed_u8::bit_and(a313, a314)
                        };
                        let a316 = 32u8;
                        if equal(a315, a316) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a322 = {
                        let a319 = {
                            let a317 = v3;
                            let a318 = 16u8;
                            fixed_u8::bit_and(a317, a318)
                        };
                        let a320 = 16u8;
                        if equal(a319, a320) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a321, a322)?
                };
                nat_add(a323, a324)?
            };
            let a348 = {
                let a345 = {
                    let a333 = {
                        let a327 = {
                            let a325 = v3;
                            let a326 = 8u8;
                            fixed_u8::bit_and(a325, a326)
                        };
                        let a328 = 8u8;
                        if equal(a327, a328) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a334 = {
                        let a331 = {
                            let a329 = v3;
                            let a330 = 4u8;
                            fixed_u8::bit_and(a329, a330)
                        };
                        let a332 = 4u8;
                        if equal(a331, a332) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a333, a334)?
                };
                let a346 = {
                    let a343 = {
                        let a337 = {
                            let a335 = v3;
                            let a336 = 2u8;
                            fixed_u8::bit_and(a335, a336)
                        };
                        let a338 = 2u8;
                        if equal(a337, a338) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a344 = {
                        let a341 = {
                            let a339 = v3;
                            let a340 = 1u8;
                            fixed_u8::bit_and(a339, a340)
                        };
                        let a342 = 1u8;
                        if equal(a341, a342) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a343, a344)?
                };
                nat_add(a345, a346)?
            };
            nat_add(a347, a348)
        }
    }
}

pub fn f46(p368: u64, v1: Bytes, p369: u64, p370: u64, v4: u64) -> R<Result<u64, Adt75>> {
    let m367 = (p368, (p369, p370));
    {
        let mut m367 = m367;
        loop {
            let step = {
                let (v0, (v2, v3)) = m367;
                let m349 = v0;
                if m349 == 0 {
                    Err::<(u64, (u64, u64)), R<Result<u64, Adt75>>>(Ok(Ok::<u64, Adt75>(v3)))
                } else {
                    let v5 = m349 - 1;
                    let a351 = {
                        let a350 = v1.clone();
                        length_bytes(a350)
                    };
                    let a352 = v2;
                    if nat_le(a351, a352) {
                        Err::<(u64, (u64, u64)), R<Result<u64, Adt75>>>(Ok(Ok::<u64, Adt75>(v3)))
                    } else {
                        let v6: u64 = f45(v1.clone(), v2)?;
                        let a353 = 48u64;
                        let a354 = v6;
                        if if nat_le(a353, a354) {
                            let a355 = v6;
                            let a356 = 57u64;
                            nat_le(a355, a356)
                        } else {
                            false
                        } {
                            let v7: u64 = {
                                let a361 = {
                                    let a357 = v3;
                                    let a358 = 10u64;
                                    nat_mul(a357, a358)?
                                };
                                let a362 = {
                                    let a359 = v6;
                                    let a360 = 48u64;
                                    nat_sub(a359, a360)
                                };
                                nat_add(a361, a362)?
                            };
                            let a363 = v4;
                            let a364 = v7;
                            if nat_lt(a363, a364) {
                                Err::<(u64, (u64, u64)), R<Result<u64, Adt75>>>(Ok(Err::<u64, Adt75>(Adt75::C2)))
                            } else {
                                Ok::<(u64, (u64, u64)), R<Result<u64, Adt75>>>((v5, ({
                                    let a365 = v2;
                                    let a366 = 1u64;
                                    nat_add(a365, a366)?
                                }, v7)))
                            }
                        } else {
                            Err::<(u64, (u64, u64)), R<Result<u64, Adt75>>>(Ok(Err::<u64, Adt75>(Adt75::C1)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m367 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Adt0, p2: Str, p3: Adt2, p4: Adt16, p5: Adt69) -> R<Adt74> {
    f0(p0, p1, p2, p3, p4, p5)
}
