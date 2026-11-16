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
    C0(Adt3, List<Adt13>, List<Adt15>, List<Str>),
}

#[derive(Clone)]
pub enum Adt3 {
    C0(List<Adt4>, List<Adt5>, List<Adt6>, List<Adt7>, List<List<Str>>, List<Adt4>, List<Adt4>, bool, List<Adt8>, List<Adt9>, List<Adt10>, List<Adt12>),
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
        let a9 = List::cons(Str::lit("List.cons "), List::cons(v13.clone(), List::cons(Str::lit(" "), List::cons(v16.clone(), List::<Str>::nil()))));
        let a10 = Str::lit("");
        join(a9, a10)
    }, Adt71::C0({
        let a11 = List::cons(Str::lit("("), List::cons(v18.clone(), List::cons(Str::lit(" "), List::cons(v16, List::cons(Str::lit(" ("), List::cons(v2.clone(), List::cons(Str::lit(" "), List::cons(v7.clone(), List::cons(Str::lit(" "), List::cons(v13, List::cons(Str::lit("))"), List::<Str>::nil())))))))))));
        let a12 = Str::lit("");
        join(a11, a12)
    })), List::<Adt72>::nil())));
    let v20: List<Str> = f3(v17.clone(), v2.clone());
    let v21: Adt69 = f4(v17, Adt70::C0(v18.clone(), List::cons(Adt6::C0(v10, f5(v1)), List::cons(Adt6::C0(v7, f5(v0.clone())), List::<Adt6>::nil())), f5(v0), v19, f6(List::cons({
        let a13 = List::cons(Str::lit("#"), List::cons(v2, List::<Str>::nil()));
        let a14 = Str::lit("");
        join(a13, a14)
    }, List::<Str>::nil()), List::<Str>::nil()), v20, true, false, None::<(Str, List<Adt73>)>));
    let v22: Adt69 = f7(v21, {
        let a15 = List::cons(Str::lit("#"), List::cons(v18.clone(), List::<Str>::nil()));
        let a16 = Str::lit("");
        join(a15, a16)
    });
    Ok(Adt74::C0(v18, v22))
}

pub fn f1(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str) -> R<Adt74> {
    let v4: Str = f8(v3.clone(), f9(v0, v1, v2.clone(), v3, List::<Str>::nil()))?;
    let v5: List<Str> = match v2.clone() {
        Adt69::C0(_, _, h17, _, _, _, _) => {
            h17
        }
    };
    let v6: List<Str> = {
        let a18 = v5;
        let a19 = List::cons(v4.clone(), List::<Str>::nil());
        append_list(a18, a19)
    };
    Ok(Adt74::C0(v4, Adt69::C0(match v2.clone() {
        Adt69::C0(h20, _, _, _, _, _, _) => {
            h20
        }
    }, match v2.clone() {
        Adt69::C0(_, h21, _, _, _, _, _) => {
            h21
        }
    }, v6, match v2.clone() {
        Adt69::C0(_, _, _, h22, _, _, _) => {
            h22
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, h23, _, _) => {
            h23
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, _, h24, _) => {
            h24
        }
    }, match v2 {
        Adt69::C0(_, _, _, _, _, _, h25) => {
            h25
        }
    })))
}

pub fn f2(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str) -> R<Str> {
    let v4: Str = {
        let a27 = List::cons(match v0.clone() {
            Adt16::C0(_, _, _, h26, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                h26
            }
        }, List::cons(v3, List::<Str>::nil()));
        let a28 = Str::lit("");
        join(a27, a28)
    };
    let v5: List<Str> = f10(match v2.clone() {
        Adt69::C0(_, _, _, _, _, h29, _) => {
            h29
        }
    });
    f8(v4.clone(), f9(v0, v1, v2, v4, v5))
}

pub fn f3(v0: Adt69, v1: Str) -> List<Str> {
    let m30 = f11(v0, v1);
    match m30 {
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
        Adt69::C0(_, _, _, _, _, h31, _) => {
            h31
        }
    };
    let v3: List<Adt70> = {
        let a32 = v2;
        let a33 = List::cons(v1, List::<Adt70>::nil());
        append_list(a32, a33)
    };
    Adt69::C0(match v0.clone() {
        Adt69::C0(h34, _, _, _, _, _, _) => {
            h34
        }
    }, match v0.clone() {
        Adt69::C0(_, h35, _, _, _, _, _) => {
            h35
        }
    }, match v0.clone() {
        Adt69::C0(_, _, h36, _, _, _, _) => {
            h36
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, h37, _, _, _) => {
            h37
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, h38, _, _) => {
            h38
        }
    }, v3, match v0 {
        Adt69::C0(_, _, _, _, _, _, h39) => {
            h39
        }
    })
}

pub fn f5(v0: Adt0) -> Str {
    let m40 = v0;
    match m40 {
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
            let a41 = List::cons(f12(), List::cons(Str::lit("::ParseIntError"), List::<Str>::nil()));
            let a42 = Str::lit("");
            join(a41, a42)
        }
        Adt0::C10 => {
            let a43 = List::cons(f12(), List::cons(Str::lit("::FilePath"), List::<Str>::nil()));
            let a44 = Str::lit("");
            join(a43, a44)
        }
        Adt0::C11(v5) => {
            v5
        }
        Adt0::C12(r67) => {
            let v6 = (*r67).clone();
            let a45 = List::cons(Str::lit("(Option "), List::cons(f5(v6), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a46 = Str::lit("");
            join(a45, a46)
        }
        Adt0::C13(r68, r69) => {
            let v7 = (*r68).clone();
            let v8 = (*r69).clone();
            let a47 = List::cons(Str::lit("(Result "), List::cons(f5(v7), List::cons(Str::lit(" "), List::cons(f5(v8), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a48 = Str::lit("");
            join(a47, a48)
        }
        Adt0::C14(r70) => {
            let v9 = (*r70).clone();
            let a49 = List::cons(Str::lit("(List "), List::cons(f5(v9), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a50 = Str::lit("");
            join(a49, a50)
        }
        Adt0::C15(v10) => {
            let a53 = List::cons(Str::lit("Tuple"), List::cons({
                let a51 = f13(v10);
                let a52 = Str::lit("");
                join(a51, a52)
            }, List::<Str>::nil()));
            let a54 = Str::lit("");
            join(a53, a54)
        }
        Adt0::C16(v11) => {
            let a57 = List::cons(Str::lit("("), List::cons({
                let a55 = f14(v11);
                let a56 = Str::lit(" * ");
                join(a55, a56)
            }, List::cons(Str::lit(")"), List::<Str>::nil())));
            let a58 = Str::lit("");
            join(a57, a58)
        }
        Adt0::C17(r71, r72, _) => {
            let v12 = (*r71).clone();
            let v13 = (*r72).clone();
            let a59 = List::cons(Str::lit("(List Tuple"), List::cons(f15(v12), List::cons(f15(v13), List::cons(Str::lit(")"), List::<Str>::nil()))));
            let a60 = Str::lit("");
            join(a59, a60)
        }
        Adt0::C18(r73, _) => {
            let v15 = (*r73).clone();
            let a61 = List::cons(Str::lit("(List "), List::cons(f5(v15), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a62 = Str::lit("");
            join(a61, a62)
        }
        Adt0::C19(v17, r74) => {
            let v18 = (*r74).clone();
            let a65 = List::cons(Str::lit("(Fn ("), List::cons({
                let a63 = f16(v17);
                let a64 = Str::lit(" ");
                join(a63, a64)
            }, List::cons(Str::lit(") "), List::cons(f5(v18), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a66 = Str::lit("");
            join(a65, a66)
        }
    }
}

pub fn f6(p77: List<Str>, p78: List<Str>) -> List<Str> {
    let m76 = (p77, p78);
    {
        let mut m76 = m76;
        loop {
            let step = {
                let (v0, v1) = m76;
                let m75 = v0;
                match m75.uncons() {
                    None => {
                        Err::<(List<Str>, List<Str>), List<Str>>(v1)
                    }
                    Some((v2, v3)) => {
                        Ok::<(List<Str>, List<Str>), List<Str>>((v3, f17(v1, v2)))
                    }
                }
            };
            match step {
                Ok(next) => { m76 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f7(v0: Adt69, v1: Str) -> Adt69 {
    let v2: List<Str> = match v0.clone() {
        Adt69::C0(_, h79, _, _, _, _, _) => {
            h79
        }
    };
    let v3: List<Str> = f18(v2, v1);
    Adt69::C0(match v0.clone() {
        Adt69::C0(h80, _, _, _, _, _, _) => {
            h80
        }
    }, v3, match v0.clone() {
        Adt69::C0(_, _, h81, _, _, _, _) => {
            h81
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, h82, _, _, _) => {
            h82
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, h83, _, _) => {
            h83
        }
    }, match v0.clone() {
        Adt69::C0(_, _, _, _, _, h84, _) => {
            h84
        }
    }, match v0 {
        Adt69::C0(_, _, _, _, _, _, h85) => {
            h85
        }
    })
}

pub fn f8(v0: Str, v1: List<Str>) -> R<Str> {
    let v2: u64 = f19(f20(f21(v1.clone(), v0.clone()))?, 0u64);
    let m92 = {
        let a88 = {
            let a86 = v2;
            let a87 = 1u64;
            nat_add(a86, a87)?
        };
        let a89 = 4294967296u64;
        if nat_lt(a88, a89) {
            Some({
                let a90 = v2;
                let a91 = 1u64;
                nat_add(a90, a91)?
            })
        } else {
            None::<u64>
        }
    };
    match m92 {
        None => {
            let m98 = f22(f23(f24(1u64, {
                let a96 = {
                    let a94 = {
                        let a93 = v1.clone();
                        length_list(a93)
                    };
                    let a95 = 2u64;
                    nat_add(a94, a95)?
                };
                let a97 = 1u64;
                nat_sub(a96, a97)
            })?, v0)?, v1);
            match m98 {
                None => {
                    Ok(Str::lit(""))
                }
                Some(v3) => {
                    Ok(v3)
                }
            }
        }
        Some(v4) => {
            let a99 = List::cons(v0, List::cons(f25(v4)?, List::<Str>::nil()));
            let a100 = Str::lit("");
            Ok(join(a99, a100))
        }
    }
}

pub fn f9(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str, v4: List<Str>) -> List<Str> {
    f26({
        let a114 = {
            let a112 = {
                let a109 = {
                    let a106 = {
                        let a103 = f27(f28(match v0.clone() {
                            Adt16::C0(_, _, _, _, h101, _, _, _, _, _, _, _, _, _, _, _, _) => {
                                h101
                            }
                        }));
                        let a104 = match v0 {
                            Adt16::C0(_, _, _, _, _, _, _, _, h102, _, _, _, _, _, _, _, _) => {
                                h102
                            }
                        };
                        append_list(a103, a104)
                    };
                    let a107 = f29(f30(match v1.clone() {
                        Adt2::C0(_, h105, _, _) => {
                            h105
                        }
                    }));
                    append_list(a106, a107)
                };
                let a110 = match v1 {
                    Adt2::C0(_, _, _, h108) => {
                        h108
                    }
                };
                append_list(a109, a110)
            };
            let a113 = match v2 {
                Adt69::C0(_, _, h111, _, _, _, _) => {
                    h111
                }
            };
            append_list(a112, a113)
        };
        let a115 = v4;
        append_list(a114, a115)
    }, v3)
}

pub fn f10(v0: List<Adt70>) -> List<Str> {
    let m116 = v0;
    match m116.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt70::C0(h117, _, _, _, _, _, _, _, _) => {
                    h117
                }
            }, f10(v2))
        }
    }
}

pub fn f11(v0: Adt69, v1: Str) -> Option<List<Str>> {
    let m119 = f31(match v0 {
        Adt69::C0(_, _, _, _, _, h118, _) => {
            h118
        }
    }, v1);
    match m119 {
        None => {
            None::<List<Str>>
        }
        Some(v2) => {
            Some(match v2 {
                Adt70::C0(_, _, _, _, _, h120, _, _, _) => {
                    h120
                }
            })
        }
    }
}

pub fn f12() -> Str {
    Str::lit("Prelude")
}

pub fn f13(v0: List<Adt0>) -> List<Str> {
    let m121 = v0;
    match m121.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f13(v2))
        }
    }
}

pub fn f14(v0: List<Adt0>) -> List<Str> {
    let m122 = v0;
    match m122.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f5(v1), f14(v2))
        }
    }
}

pub fn f15(v0: Adt0) -> Str {
    let m123 = v0;
    match m123 {
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
        Adt0::C12(r146) => {
            let v6 = (*r146).clone();
            let a124 = List::cons(Str::lit("Option"), List::cons(f15(v6), List::<Str>::nil()));
            let a125 = Str::lit("");
            join(a124, a125)
        }
        Adt0::C13(r147, r148) => {
            let v7 = (*r147).clone();
            let v8 = (*r148).clone();
            let a126 = List::cons(Str::lit("Result"), List::cons(f15(v7), List::cons(f15(v8), List::<Str>::nil())));
            let a127 = Str::lit("");
            join(a126, a127)
        }
        Adt0::C14(r149) => {
            let v9 = (*r149).clone();
            let a128 = List::cons(Str::lit("List"), List::cons(f15(v9), List::<Str>::nil()));
            let a129 = Str::lit("");
            join(a128, a129)
        }
        Adt0::C15(v10) => {
            let a132 = List::cons(Str::lit("Tuple"), List::cons({
                let a130 = f32(v10);
                let a131 = Str::lit("");
                join(a130, a131)
            }, List::<Str>::nil()));
            let a133 = Str::lit("");
            join(a132, a133)
        }
        Adt0::C16(v11) => {
            let a136 = List::cons(Str::lit("Pair"), List::cons({
                let a134 = f33(v11);
                let a135 = Str::lit("");
                join(a134, a135)
            }, List::<Str>::nil()));
            let a137 = Str::lit("");
            join(a136, a137)
        }
        Adt0::C17(r150, r151, _) => {
            let v12 = (*r150).clone();
            let v13 = (*r151).clone();
            let a138 = List::cons(Str::lit("ListTuple"), List::cons(f15(v12), List::cons(f15(v13), List::<Str>::nil())));
            let a139 = Str::lit("");
            join(a138, a139)
        }
        Adt0::C18(r152, _) => {
            let v15 = (*r152).clone();
            let a140 = List::cons(Str::lit("List"), List::cons(f15(v15), List::<Str>::nil()));
            let a141 = Str::lit("");
            join(a140, a141)
        }
        Adt0::C19(v17, r153) => {
            let v18 = (*r153).clone();
            let a144 = List::cons(Str::lit("Fn"), List::cons({
                let a142 = f34(v17);
                let a143 = Str::lit("");
                join(a142, a143)
            }, List::cons(Str::lit("To"), List::cons(f15(v18), List::<Str>::nil()))));
            let a145 = Str::lit("");
            join(a144, a145)
        }
    }
}

pub fn f16(v0: List<Adt0>) -> List<Str> {
    let m154 = v0;
    match m154.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f5(v1), f16(v2))
        }
    }
}

pub fn f17(v0: List<Str>, v1: Str) -> List<Str> {
    let m155 = v0;
    match m155.uncons() {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some((v2, v3)) => {
            let a156 = v1.clone();
            let a157 = v2.clone();
            if equal(a156, a157) {
                List::cons(v2, v3)
            } else {
                let m160 = {
                    let a158 = v1.clone();
                    let a159 = v2.clone();
                    compare(a158, a159)
                };
                if match m160 {
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
    let m161 = v0;
    match m161.uncons() {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some((v2, v3)) => {
            let a162 = v1.clone();
            let a163 = v2.clone();
            if equal(a162, a163) {
                List::cons(v2, v3)
            } else {
                let m166 = {
                    let a164 = v1.clone();
                    let a165 = v2.clone();
                    compare(a164, a165)
                };
                if match m166 {
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

pub fn f19(p171: List<u64>, p172: u64) -> u64 {
    let m170 = (p171, p172);
    {
        let mut m170 = m170;
        loop {
            let step = {
                let (v0, v1) = m170;
                let m167 = v0;
                match m167.uncons() {
                    None => {
                        Err::<(List<u64>, u64), u64>(v1)
                    }
                    Some((v2, v3)) => {
                        Ok::<(List<u64>, u64), u64>((v3, {
                            let a168 = v1;
                            let a169 = v2;
                            if nat_le(a168, a169) {
                                v2
                            } else {
                                v1
                            }
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m170 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f20(v0: List<Str>) -> R<List<u64>> {
    let m173 = v0;
    match m173.uncons() {
        None => {
            Ok(List::<u64>::nil())
        }
        Some((v1, v2)) => {
            let m174 = f35(v1)?;
            match m174 {
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
    let m175 = v0;
    match m175.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            let m176 = f36(v2, v1.clone());
            match m176 {
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

pub fn f22(p180: List<Str>, v1: List<Str>) -> Option<Str> {
    let m179 = p180;
    {
        let mut m179 = m179;
        loop {
            let step = {
                let v0 = m179;
                let m177 = v0;
                match m177.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a178 = f37(v1.clone(), v2.clone());
                        if bool_not(a178) {
                            Err::<List<Str>, Option<Str>>(Some(v2))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m179 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f23(v0: List<u64>, v1: Str) -> R<List<Str>> {
    let m181 = v0;
    match m181.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons({
                let a182 = List::cons(v1.clone(), List::cons(f25(v2)?, List::<Str>::nil()));
                let a183 = Str::lit("");
                join(a182, a183)
            }, f23(v3, v1)?))
        }
    }
}

pub fn f24(v0: u64, v1: u64) -> R<List<u64>> {
    let m184 = v1;
    if m184 == 0 {
        Ok(List::<u64>::nil())
    } else {
        let v2 = m184 - 1;
        Ok(List::cons(v0, f24({
            let a185 = v0;
            let a186 = 1u64;
            nat_add(a185, a186)?
        }, v2)?))
    }
}

pub fn f25(v0: u64) -> R<Str> {
    Ok(f38({
        let a187 = v0;
        let a188 = 1u64;
        nat_add(a187, a188)?
    }, v0))
}

pub fn f26(v0: List<Str>, v1: Str) -> List<Str> {
    let m189 = v0;
    match m189.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            if f39(v2.clone(), v1.clone()) {
                List::cons(v2, f26(v3, v1))
            } else {
                f26(v3, v1)
            }
        }
    }
}

pub fn f27(v0: List<(Str, Adt0)>) -> List<Str> {
    let m190 = v0;
    match m190.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            let v3: Str = {
                let (h191, _) = v1;
                h191
            };
            let _: Adt0 = {
                let (_, h192) = v1;
                h192
            };
            List::cons(v3, f27(v2))
        }
    }
}

pub fn f28(v0: List<Adt17>) -> List<(Str, Adt0)> {
    let m193 = v0;
    match m193.uncons() {
        None => {
            List::<(Str, Adt0)>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt17::C0(_, h194) => {
                    h194
                }
            }, f28(v2))
        }
    }
}

pub fn f29(v0: List<Adt14>) -> List<Str> {
    let m195 = v0;
    match m195.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt14::C0(h196, _, _) => {
                    h196
                }
            }, f29(v2))
        }
    }
}

pub fn f30(v0: List<Adt13>) -> List<Adt14> {
    let m197 = v0;
    match m197.uncons() {
        None => {
            List::<Adt14>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt13::C0(_, h198) => {
                    h198
                }
            }, f30(v2))
        }
    }
}

pub fn f31(p204: List<Adt70>, v1: Str) -> Option<Adt70> {
    let m203 = p204;
    {
        let mut m203 = m203;
        loop {
            let step = {
                let v0 = m203;
                let m199 = v0;
                match m199.uncons() {
                    None => {
                        Err::<List<Adt70>, Option<Adt70>>(None::<Adt70>)
                    }
                    Some((v2, v3)) => {
                        let a201 = match v2.clone() {
                            Adt70::C0(h200, _, _, _, _, _, _, _, _) => {
                                h200
                            }
                        };
                        let a202 = v1.clone();
                        if equal(a201, a202) {
                            Err::<List<Adt70>, Option<Adt70>>(Some(v2))
                        } else {
                            Ok::<List<Adt70>, Option<Adt70>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m203 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f32(v0: List<Adt0>) -> List<Str> {
    let m205 = v0;
    match m205.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f32(v2))
        }
    }
}

pub fn f33(v0: List<Adt0>) -> List<Str> {
    let m206 = v0;
    match m206.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f33(v2))
        }
    }
}

pub fn f34(v0: List<Adt0>) -> List<Str> {
    let m207 = v0;
    match m207.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f15(v1), f34(v2))
        }
    }
}

pub fn f35(v0: Str) -> R<Option<u64>> {
    let m208 = f40(v0, 4294967295u64)?;
    match m208 {
        Ok(v1) => {
            Ok(Some(v1))
        }
        Err(_) => {
            Ok(None::<u64>)
        }
    }
}

pub fn f36(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a209 = v0;
        utf8_encode(a209)
    };
    let v3: Bytes = {
        let a210 = v1;
        utf8_encode(a210)
    };
    if f41(v2.clone(), 0u64, v3.clone()) {
        Some(f42(v2.clone(), {
            let a211 = v3;
            length_bytes(a211)
        }, {
            let a212 = v2;
            length_bytes(a212)
        }))
    } else {
        None::<Str>
    }
}

pub fn f37(p217: List<Str>, v1: Str) -> bool {
    let m216 = p217;
    {
        let mut m216 = m216;
        loop {
            let step = {
                let v0 = m216;
                let m213 = v0;
                match m213.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a214 = v2;
                        let a215 = v1.clone();
                        if equal(a214, a215) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m216 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f38(v0: u64, v1: u64) -> Str {
    let m218 = v0;
    if m218 == 0 {
        Str::lit("")
    } else {
        let v2 = m218 - 1;
        let a219 = v1;
        let a220 = 10u64;
        if nat_lt(a219, a220) {
            f43(v1)
        } else {
            let a227 = List::cons(f38(v2, {
                let a221 = v1;
                let a222 = 10u64;
                let a223 = 0u64;
                nat_quot(a221, a222, a223)
            }), List::cons(f43({
                let a224 = v1;
                let a225 = 10u64;
                let a226 = 0u64;
                nat_rem(a224, a225, a226)
            }), List::<Str>::nil()));
            let a228 = Str::lit("");
            join(a227, a228)
        }
    }
}

pub fn f39(v0: Str, v1: Str) -> bool {
    f41({
        let a229 = v0;
        utf8_encode(a229)
    }, 0u64, {
        let a230 = v1;
        utf8_encode(a230)
    })
}

pub fn f40(v0: Str, v1: u64) -> R<Result<u64, Adt75>> {
    let v2: Bytes = {
        let a231 = v0;
        utf8_encode(a231)
    };
    let v3: u64 = {
        let a232 = v2.clone();
        length_bytes(a232)
    };
    let a233 = v3;
    let a234 = 0u64;
    if nat_eq(a233, a234) {
        Ok(Err::<u64, Adt75>(Adt75::C0))
    } else {
        let a235 = f44(v2.clone(), 0u64)?;
        let a236 = 43u64;
        if nat_eq(a235, a236) {
            let a237 = v3;
            let a238 = 1u64;
            if nat_eq(a237, a238) {
                Ok(Err::<u64, Adt75>(Adt75::C1))
            } else {
                f45(v3, v2, 1u64, 0u64, v1)
            }
        } else {
            f45(v3, v2, 0u64, 0u64, v1)
        }
    }
}

pub fn f41(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m243 = {
        let a240 = v0;
        let a241 = v1;
        let a242 = {
            let a239 = v2.clone();
            length_bytes(a239)
        };
        slice_bytes(a240, a241, a242)
    };
    match m243 {
        None => {
            false
        }
        Some(v3) => {
            let a244 = v3;
            let a245 = v2;
            equal(a244, a245)
        }
    }
}

pub fn f42(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m251 = {
        let a248 = v0;
        let a249 = v1;
        let a250 = {
            let a246 = v2;
            let a247 = v1;
            nat_sub(a246, a247)
        };
        slice_bytes(a248, a249, a250)
    };
    match m251 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m253 = {
                let a252 = v3;
                utf8_decode(a252)
            };
            match m253 {
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

pub fn f43(v0: u64) -> Str {
    let a254 = v0;
    let a255 = 0u64;
    if nat_eq(a254, a255) {
        Str::lit("0")
    } else {
        let a256 = v0;
        let a257 = 1u64;
        if nat_eq(a256, a257) {
            Str::lit("1")
        } else {
            let a258 = v0;
            let a259 = 2u64;
            if nat_eq(a258, a259) {
                Str::lit("2")
            } else {
                let a260 = v0;
                let a261 = 3u64;
                if nat_eq(a260, a261) {
                    Str::lit("3")
                } else {
                    let a262 = v0;
                    let a263 = 4u64;
                    if nat_eq(a262, a263) {
                        Str::lit("4")
                    } else {
                        let a264 = v0;
                        let a265 = 5u64;
                        if nat_eq(a264, a265) {
                            Str::lit("5")
                        } else {
                            let a266 = v0;
                            let a267 = 6u64;
                            if nat_eq(a266, a267) {
                                Str::lit("6")
                            } else {
                                let a268 = v0;
                                let a269 = 7u64;
                                if nat_eq(a268, a269) {
                                    Str::lit("7")
                                } else {
                                    let a270 = v0;
                                    let a271 = 8u64;
                                    if nat_eq(a270, a271) {
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

pub fn f44(v0: Bytes, v1: u64) -> R<u64> {
    let m274 = {
        let a272 = v0;
        let a273 = v1;
        index_bytes(a272, a273)
    };
    match m274 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a319 = {
                let a295 = {
                    let a283 = {
                        let a277 = {
                            let a275 = v3;
                            let a276 = 128u8;
                            fixed_u8::bit_and(a275, a276)
                        };
                        let a278 = 128u8;
                        if equal(a277, a278) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a284 = {
                        let a281 = {
                            let a279 = v3;
                            let a280 = 64u8;
                            fixed_u8::bit_and(a279, a280)
                        };
                        let a282 = 64u8;
                        if equal(a281, a282) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a283, a284)?
                };
                let a296 = {
                    let a293 = {
                        let a287 = {
                            let a285 = v3;
                            let a286 = 32u8;
                            fixed_u8::bit_and(a285, a286)
                        };
                        let a288 = 32u8;
                        if equal(a287, a288) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a294 = {
                        let a291 = {
                            let a289 = v3;
                            let a290 = 16u8;
                            fixed_u8::bit_and(a289, a290)
                        };
                        let a292 = 16u8;
                        if equal(a291, a292) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a293, a294)?
                };
                nat_add(a295, a296)?
            };
            let a320 = {
                let a317 = {
                    let a305 = {
                        let a299 = {
                            let a297 = v3;
                            let a298 = 8u8;
                            fixed_u8::bit_and(a297, a298)
                        };
                        let a300 = 8u8;
                        if equal(a299, a300) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a306 = {
                        let a303 = {
                            let a301 = v3;
                            let a302 = 4u8;
                            fixed_u8::bit_and(a301, a302)
                        };
                        let a304 = 4u8;
                        if equal(a303, a304) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a305, a306)?
                };
                let a318 = {
                    let a315 = {
                        let a309 = {
                            let a307 = v3;
                            let a308 = 2u8;
                            fixed_u8::bit_and(a307, a308)
                        };
                        let a310 = 2u8;
                        if equal(a309, a310) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a316 = {
                        let a313 = {
                            let a311 = v3;
                            let a312 = 1u8;
                            fixed_u8::bit_and(a311, a312)
                        };
                        let a314 = 1u8;
                        if equal(a313, a314) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a315, a316)?
                };
                nat_add(a317, a318)?
            };
            nat_add(a319, a320)
        }
    }
}

pub fn f45(p340: u64, v1: Bytes, p341: u64, p342: u64, v4: u64) -> R<Result<u64, Adt75>> {
    let m339 = (p340, (p341, p342));
    {
        let mut m339 = m339;
        loop {
            let step = {
                let (v0, (v2, v3)) = m339;
                let m321 = v0;
                if m321 == 0 {
                    Err::<(u64, (u64, u64)), R<Result<u64, Adt75>>>(Ok(Ok::<u64, Adt75>(v3)))
                } else {
                    let v5 = m321 - 1;
                    let a323 = {
                        let a322 = v1.clone();
                        length_bytes(a322)
                    };
                    let a324 = v2;
                    if nat_le(a323, a324) {
                        Err::<(u64, (u64, u64)), R<Result<u64, Adt75>>>(Ok(Ok::<u64, Adt75>(v3)))
                    } else {
                        let v6: u64 = f44(v1.clone(), v2)?;
                        let a325 = 48u64;
                        let a326 = v6;
                        if if nat_le(a325, a326) {
                            let a327 = v6;
                            let a328 = 57u64;
                            nat_le(a327, a328)
                        } else {
                            false
                        } {
                            let v7: u64 = {
                                let a333 = {
                                    let a329 = v3;
                                    let a330 = 10u64;
                                    nat_mul(a329, a330)?
                                };
                                let a334 = {
                                    let a331 = v6;
                                    let a332 = 48u64;
                                    nat_sub(a331, a332)
                                };
                                nat_add(a333, a334)?
                            };
                            let a335 = v4;
                            let a336 = v7;
                            if nat_lt(a335, a336) {
                                Err::<(u64, (u64, u64)), R<Result<u64, Adt75>>>(Ok(Err::<u64, Adt75>(Adt75::C2)))
                            } else {
                                Ok::<(u64, (u64, u64)), R<Result<u64, Adt75>>>((v5, ({
                                    let a337 = v2;
                                    let a338 = 1u64;
                                    nat_add(a337, a338)?
                                }, v7)))
                            }
                        } else {
                            Err::<(u64, (u64, u64)), R<Result<u64, Adt75>>>(Ok(Err::<u64, Adt75>(Adt75::C1)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m339 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Adt0, p2: Str, p3: Adt2, p4: Adt16, p5: Adt69) -> R<Adt74> {
    f0(p0, p1, p2, p3, p4, p5)
}
