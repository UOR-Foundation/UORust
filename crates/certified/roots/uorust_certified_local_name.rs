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
    C0(u64, Str),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Str, bool, bool),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Str, bool),
}

pub fn f0(v0: Str) -> R<Str> {
    let v1: Str = f1(List::cons(f2(v0, Str::lit("r#"))?, List::<Str>::nil()))?;
    let a1 = v1.clone();
    let a2 = Str::lit("");
    if if equal(a1, a2) {
        true
    } else {
        f3(f4(), v1.clone())
    } {
        let a3 = List::cons(v1, List::cons(Str::lit("Value"), List::<Str>::nil()));
        let a4 = Str::lit("");
        Ok(join(a3, a4))
    } else {
        Ok(v1)
    }
}

pub fn f1(v0: List<Str>) -> R<Str> {
    let a5 = f5(f6(f7(f8(v0)?)?, 0u64)?)?;
    let a6 = Str::lit("");
    Ok(join(a5, a6))
}

pub fn f2(v0: Str, v1: Str) -> R<Str> {
    let v2: Bytes = {
        let a7 = v0.clone();
        utf8_encode(a7)
    };
    let v3: Bytes = {
        let a8 = v1;
        utf8_encode(a8)
    };
    let a10 = {
        let a9 = v3.clone();
        length_bytes(a9)
    };
    let a11 = 0u64;
    if nat_eq(a10, a11) {
        Ok(v0)
    } else {
        Ok(f9(v2.clone(), f10({
            let a12 = v2.clone();
            length_bytes(a12)
        }, v2.clone(), v3, 0u64)?, {
            let a13 = v2;
            length_bytes(a13)
        }))
    }
}

pub fn f3(p18: List<Str>, v1: Str) -> bool {
    let m17 = p18;
    {
        let mut m17 = m17;
        loop {
            let step = {
                let v0 = m17;
                let m14 = v0;
                match m14.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a15 = v2;
                        let a16 = v1.clone();
                        if equal(a15, a16) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m17 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f4() -> List<Str> {
    let a47 = {
        let a31 = {
            let a23 = {
                let a19 = List::cons(Str::lit("at"), List::cons(Str::lit("by"), List::cons(Str::lit("calc"), List::cons(Str::lit("def"), List::cons(Str::lit("do"), List::cons(Str::lit("else"), List::<Str>::nil()))))));
                let a20 = List::cons(Str::lit("end"), List::cons(Str::lit("example"), List::cons(Str::lit("fun"), List::cons(Str::lit("have"), List::cons(Str::lit("if"), List::cons(Str::lit("in"), List::<Str>::nil()))))));
                append_list(a19, a20)
            };
            let a24 = {
                let a21 = List::cons(Str::lit("instance"), List::cons(Str::lit("let"), List::cons(Str::lit("match"), List::cons(Str::lit("mut"), List::cons(Str::lit("nomatch"), List::cons(Str::lit("open"), List::<Str>::nil()))))));
                let a22 = List::cons(Str::lit("prop"), List::cons(Str::lit("return"), List::cons(Str::lit("show"), List::cons(Str::lit("then"), List::cons(Str::lit("theorem"), List::cons(Str::lit("type"), List::cons(Str::lit("where"), List::<Str>::nil())))))));
                append_list(a21, a22)
            };
            append_list(a23, a24)
        };
        let a32 = {
            let a29 = {
                let a25 = List::cons(Str::lit("with"), List::cons(Str::lit("from"), List::cons(Str::lit("for"), List::cons(Str::lit("try"), List::cons(Str::lit("catch"), List::cons(Str::lit("break"), List::<Str>::nil()))))));
                let a26 = List::cons(Str::lit("continue"), List::cons(Str::lit("unless"), List::cons(Str::lit("structure"), List::cons(Str::lit("inductive"), List::cons(Str::lit("class"), List::cons(Str::lit("namespace"), List::cons(Str::lit("section"), List::<Str>::nil())))))));
                append_list(a25, a26)
            };
            let a30 = {
                let a27 = List::cons(Str::lit("local"), List::cons(Str::lit("private"), List::cons(Str::lit("protected"), List::cons(Str::lit("partial"), List::cons(Str::lit("unsafe"), List::cons(Str::lit("macro"), List::<Str>::nil()))))));
                let a28 = List::cons(Str::lit("syntax"), List::cons(Str::lit("import"), List::cons(Str::lit("export"), List::cons(Str::lit("axiom"), List::cons(Str::lit("abbrev"), List::cons(Str::lit("opaque"), List::cons(Str::lit("mutual"), List::<Str>::nil())))))));
                append_list(a27, a28)
            };
            append_list(a29, a30)
        };
        append_list(a31, a32)
    };
    let a48 = {
        let a45 = {
            let a37 = {
                let a33 = List::cons(Str::lit("variable"), List::cons(Str::lit("universe"), List::cons(Str::lit("deriving"), List::cons(Str::lit("extends"), List::cons(Str::lit("suffices"), List::cons(Str::lit("obtain"), List::<Str>::nil()))))));
                let a34 = List::cons(Str::lit("record"), List::cons(Str::lit("ctor"), List::cons(Str::lit("prim"), List::cons(Str::lit("project"), List::cons(Str::lit("cons"), List::cons(Str::lit("nil"), List::<Str>::nil()))))));
                append_list(a33, a34)
            };
            let a38 = {
                let a35 = List::cons(Str::lit("not"), List::cons(Str::lit("true"), List::cons(Str::lit("false"), List::cons(Str::lit("forall"), List::cons(Str::lit("first"), List::cons(Str::lit("second"), List::<Str>::nil()))))));
                let a36 = List::cons(Str::lit("apply"), List::cons(Str::lit("ref"), List::cons(Str::lit("checked"), List::cons(Str::lit("self"), List::cons(Str::lit("noncomputable"), List::cons(Str::lit("termination_by"), List::cons(Str::lit("and_congr"), List::<Str>::nil())))))));
                append_list(a35, a36)
            };
            append_list(a37, a38)
        };
        let a46 = {
            let a43 = {
                let a39 = List::cons(Str::lit("and_true"), List::cons(Str::lit("beq_iff_eq"), List::cons(Str::lit("congr"), List::cons(Str::lit("decide"), List::cons(Str::lit("decide_eq_false_iff_not"), List::cons(Str::lit("decide_eq_true_eq"), List::<Str>::nil()))))));
                let a40 = List::cons(Str::lit("id"), List::cons(Str::lit("reduceIte"), List::cons(Str::lit("rfl"), List::cons(Str::lit("true_and"), List::cons(Str::lit("isFalse"), List::cons(Str::lit("isTrue"), List::cons(Str::lit("none"), List::<Str>::nil())))))));
                append_list(a39, a40)
            };
            let a44 = {
                let a41 = List::cons(Str::lit("some"), List::cons(Str::lit("below"), List::cons(Str::lit("brecOn"), List::cons(Str::lit("casesOn"), List::cons(Str::lit("ctorElim"), List::cons(Str::lit("ctorElimType"), List::<Str>::nil()))))));
                let a42 = List::cons(Str::lit("ctorIdx"), List::cons(Str::lit("mk"), List::cons(Str::lit("noConfusion"), List::cons(Str::lit("noConfusionType"), List::cons(Str::lit("rec"), List::cons(Str::lit("recOn"), List::cons(Str::lit("toCtorIdx"), List::<Str>::nil())))))));
                append_list(a41, a42)
            };
            append_list(a43, a44)
        };
        append_list(a45, a46)
    };
    append_list(a47, a48)
}

pub fn f5(v0: List<Adt0>) -> R<List<Str>> {
    let m49 = v0;
    match m49.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt0::C0(h50, _) => {
                    h50
                }
            };
            let v4: Str = match v1 {
                Adt0::C0(_, h51) => {
                    h51
                }
            };
            Ok(List::cons({
                let a52 = v3;
                let a53 = 0u64;
                if nat_eq(a52, a53) {
                    f11(v4)?
                } else {
                    v4
                }
            }, f5(v2)?))
        }
    }
}

pub fn f6(v0: List<Str>, v1: u64) -> R<List<Adt0>> {
    let m54 = v0;
    match m54.uncons() {
        None => {
            Ok(List::<Adt0>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt0::C0(v1, v2), f6(v3, {
                let a55 = v1;
                let a56 = 1u64;
                nat_add(a55, a56)?
            })?))
        }
    }
}

pub fn f7(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a57 = v0;
        utf8_encode(a57)
    };
    f12({
        let a58 = v1.clone();
        length_bytes(a58)
    }, v1, 0u64)
}

pub fn f8(v0: List<Str>) -> R<Str> {
    let v1: Str = Str::lit("");
    let v2: Str = f13(v0, v1)?;
    if f14(v2.clone())? {
        let a59 = List::cons(Str::lit("N"), List::cons(v2, List::<Str>::nil()));
        let a60 = Str::lit("");
        Ok(join(a59, a60))
    } else {
        Ok(v2)
    }
}

pub fn f9(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m66 = {
        let a63 = v0;
        let a64 = v1;
        let a65 = {
            let a61 = v2;
            let a62 = v1;
            nat_sub(a61, a62)
        };
        slice_bytes(a63, a64, a65)
    };
    match m66 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m68 = {
                let a67 = v3;
                utf8_decode(a67)
            };
            match m68 {
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

pub fn f10(p74: u64, v1: Bytes, v2: Bytes, p75: u64) -> R<u64> {
    let m73 = (p74, p75);
    {
        let mut m73 = m73;
        loop {
            let step = {
                let (v0, v3) = m73;
                let m69 = v0;
                if m69 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v3))
                } else {
                    let v4 = m69 - 1;
                    if f15(v1.clone(), v3, v2.clone()) {
                        Ok::<(u64, u64), R<u64>>((v4, {
                            let a71 = v3;
                            let a72 = {
                                let a70 = v2.clone();
                                length_bytes(a70)
                            };
                            nat_add(a71, a72)?
                        }))
                    } else {
                        Err::<(u64, u64), R<u64>>(Ok(v3))
                    }
                }
            };
            match step {
                Ok(next) => { m73 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f11(v0: Str) -> R<Str> {
    f16(v0, 65u64, Bytes::lit(&[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a]))
}

pub fn f12(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m76 = v0;
    if m76 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m76 - 1;
        let a78 = {
            let a77 = v1.clone();
            length_bytes(a77)
        };
        let a79 = v2;
        if nat_le(a78, a79) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f17(f18(v1.clone(), v2)?);
            Ok(List::cons(f9(v1.clone(), v2, {
                let a80 = v2;
                let a81 = v4;
                nat_add(a80, a81)?
            }), f12(v3, v1, {
                let a82 = v2;
                let a83 = v4;
                nat_add(a82, a83)?
            })?))
        }
    }
}

pub fn f13(p89: List<Str>, p90: Str) -> R<Str> {
    let m88 = (p89, p90);
    {
        let mut m88 = m88;
        loop {
            let step = {
                let (v0, v1) = m88;
                let m84 = v0;
                match m84.uncons() {
                    None => {
                        Err::<(List<Str>, Str), R<Str>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: bool = true;
                        let v5: bool = false;
                        let v6: Adt1 = f19(f7(v2)?, Adt1::C0(v1, v5, v4))?;
                        let v7: Str = match v6.clone() {
                            Adt1::C0(h85, _, _) => {
                                h85
                            }
                        };
                        let _: bool = match v6.clone() {
                            Adt1::C0(_, h86, _) => {
                                h86
                            }
                        };
                        let _: bool = match v6 {
                            Adt1::C0(_, _, h87) => {
                                h87
                            }
                        };
                        Ok::<(List<Str>, Str), R<Str>>((v3, v7))
                    }
                }
            };
            match step {
                Ok(next) => { m88 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f14(v0: Str) -> R<bool> {
    let m93 = {
        let a91 = f7(v0)?;
        let a92 = 0u64;
        index_list(a91, a92)
    };
    match m93 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            f20(v1)
        }
    }
}

pub fn f15(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m98 = {
        let a95 = v0;
        let a96 = v1;
        let a97 = {
            let a94 = v2.clone();
            length_bytes(a94)
        };
        slice_bytes(a95, a96, a97)
    };
    match m98 {
        None => {
            false
        }
        Some(v3) => {
            let a99 = v3;
            let a100 = v2;
            equal(a99, a100)
        }
    }
}

pub fn f16(v0: Str, v1: u64, v2: Bytes) -> R<Str> {
    let v3: Bytes = {
        let a101 = v0.clone();
        utf8_encode(a101)
    };
    let m105 = {
        let a104 = f21({
            let a102 = v3.clone();
            length_bytes(a102)
        }, v3, 0u64, v1, v2, {
            let a103 = Str::lit("");
            utf8_encode(a103)
        })?;
        utf8_decode(a104)
    };
    match m105 {
        None => {
            Ok(v0)
        }
        Some(v4) => {
            Ok(v4)
        }
    }
}

pub fn f17(v0: u64) -> u64 {
    let a106 = v0;
    let a107 = 128u64;
    if nat_lt(a106, a107) {
        1u64
    } else {
        let a108 = v0;
        let a109 = 224u64;
        if nat_lt(a108, a109) {
            2u64
        } else {
            let a110 = v0;
            let a111 = 240u64;
            if nat_lt(a110, a111) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f18(v0: Bytes, v1: u64) -> R<u64> {
    let m114 = {
        let a112 = v0;
        let a113 = v1;
        index_bytes(a112, a113)
    };
    match m114 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a159 = {
                let a135 = {
                    let a123 = {
                        let a117 = {
                            let a115 = v3;
                            let a116 = 128u8;
                            fixed_u8::bit_and(a115, a116)
                        };
                        let a118 = 128u8;
                        if equal(a117, a118) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a124 = {
                        let a121 = {
                            let a119 = v3;
                            let a120 = 64u8;
                            fixed_u8::bit_and(a119, a120)
                        };
                        let a122 = 64u8;
                        if equal(a121, a122) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a123, a124)?
                };
                let a136 = {
                    let a133 = {
                        let a127 = {
                            let a125 = v3;
                            let a126 = 32u8;
                            fixed_u8::bit_and(a125, a126)
                        };
                        let a128 = 32u8;
                        if equal(a127, a128) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a134 = {
                        let a131 = {
                            let a129 = v3;
                            let a130 = 16u8;
                            fixed_u8::bit_and(a129, a130)
                        };
                        let a132 = 16u8;
                        if equal(a131, a132) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a133, a134)?
                };
                nat_add(a135, a136)?
            };
            let a160 = {
                let a157 = {
                    let a145 = {
                        let a139 = {
                            let a137 = v3;
                            let a138 = 8u8;
                            fixed_u8::bit_and(a137, a138)
                        };
                        let a140 = 8u8;
                        if equal(a139, a140) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a146 = {
                        let a143 = {
                            let a141 = v3;
                            let a142 = 4u8;
                            fixed_u8::bit_and(a141, a142)
                        };
                        let a144 = 4u8;
                        if equal(a143, a144) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a145, a146)?
                };
                let a158 = {
                    let a155 = {
                        let a149 = {
                            let a147 = v3;
                            let a148 = 2u8;
                            fixed_u8::bit_and(a147, a148)
                        };
                        let a150 = 2u8;
                        if equal(a149, a150) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a156 = {
                        let a153 = {
                            let a151 = v3;
                            let a152 = 1u8;
                            fixed_u8::bit_and(a151, a152)
                        };
                        let a154 = 1u8;
                        if equal(a153, a154) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a155, a156)?
                };
                nat_add(a157, a158)?
            };
            nat_add(a159, a160)
        }
    }
}

pub fn f19(p176: List<Str>, p177: Adt1) -> R<Adt1> {
    let m175 = (p176, p177);
    {
        let mut m175 = m175;
        loop {
            let step = {
                let (v0, v1) = m175;
                let m161 = v0;
                match m161.uncons() {
                    None => {
                        Err::<(List<Str>, Adt1), R<Adt1>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: Str = match v1.clone() {
                            Adt1::C0(h162, _, _) => {
                                h162
                            }
                        };
                        let v5: bool = match v1.clone() {
                            Adt1::C0(_, h163, _) => {
                                h163
                            }
                        };
                        let v6: bool = match v1 {
                            Adt1::C0(_, _, h164) => {
                                h164
                            }
                        };
                        let a165 = f22(v2.clone())?;
                        if bool_not(a165) {
                            let v7: bool = true;
                            let v8: bool = false;
                            Ok::<(List<Str>, Adt1), R<Adt1>>((v3, Adt1::C0(v4, v8, v7)))
                        } else {
                            let v14: Adt2 = if v6 {
                                let v9: Str = {
                                    let a166 = List::cons(v4, List::cons(f23(v2.clone())?, List::<Str>::nil()));
                                    let a167 = Str::lit("");
                                    join(a166, a167)
                                };
                                let v10: bool = false;
                                Adt2::C0(v9, v10)
                            } else {
                                let v13: Str = if if f24(v2.clone())? {
                                    let a168 = v5;
                                    bool_not(a168)
                                } else {
                                    false
                                } {
                                    let v11: Str = {
                                        let a169 = List::cons(v4, List::cons(f11(v2.clone())?, List::<Str>::nil()));
                                        let a170 = Str::lit("");
                                        join(a169, a170)
                                    };
                                    v11
                                } else {
                                    let v12: Str = {
                                        let a171 = List::cons(v4, List::cons(v2.clone(), List::<Str>::nil()));
                                        let a172 = Str::lit("");
                                        join(a171, a172)
                                    };
                                    v12
                                };
                                Adt2::C0(v13, v6)
                            };
                            let v15: Str = match v14.clone() {
                                Adt2::C0(h173, _) => {
                                    h173
                                }
                            };
                            let v16: bool = match v14 {
                                Adt2::C0(_, h174) => {
                                    h174
                                }
                            };
                            let v17: bool = if f25(v2.clone())? {
                                true
                            } else {
                                f20(v2)?
                            };
                            Ok::<(List<Str>, Adt1), R<Adt1>>((v3, Adt1::C0(v15, v17, v16)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m175 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f20(v0: Str) -> R<bool> {
    let v1: u64 = f26(v0)?;
    let a178 = 48u64;
    let a179 = v1;
    if nat_le(a178, a179) {
        let a180 = v1;
        let a181 = 57u64;
        Ok(nat_le(a180, a181))
    } else {
        Ok(false)
    }
}

pub fn f21(p204: u64, v1: Bytes, p205: u64, v3: u64, v4: Bytes, p206: Bytes) -> R<Bytes> {
    let m203 = (p204, (p205, p206));
    {
        let mut m203 = m203;
        loop {
            let step = {
                let (v0, (v2, v5)) = m203;
                let m182 = v0;
                if m182 == 0 {
                    Err::<(u64, (u64, Bytes)), R<Bytes>>(Ok(v5))
                } else {
                    let v6 = m182 - 1;
                    let m186 = {
                        let a183 = v1.clone();
                        let a184 = v2;
                        let a185 = 1u64;
                        slice_bytes(a183, a184, a185)
                    };
                    match m186 {
                        None => {
                            Err::<(u64, (u64, Bytes)), R<Bytes>>(Ok(v5))
                        }
                        Some(v7) => {
                            let v8: u64 = f18(v1.clone(), v2)?;
                            let v10: Bytes = {
                                let a187 = v3;
                                let a188 = v8;
                                if if nat_le(a187, a188) {
                                    let a191 = v8;
                                    let a192 = {
                                        let a189 = v3;
                                        let a190 = 26u64;
                                        nat_add(a189, a190)?
                                    };
                                    nat_lt(a191, a192)
                                } else {
                                    false
                                } {
                                    let m198 = {
                                        let a195 = v4.clone();
                                        let a196 = {
                                            let a193 = v8;
                                            let a194 = v3;
                                            nat_sub(a193, a194)
                                        };
                                        let a197 = 1u64;
                                        slice_bytes(a195, a196, a197)
                                    };
                                    match m198 {
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
                                let a199 = v2;
                                let a200 = 1u64;
                                nat_add(a199, a200)?
                            }, {
                                let a201 = v5;
                                let a202 = v10;
                                append_bytes(a201, a202)
                            })))
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

pub fn f22(v0: Str) -> R<bool> {
    if f27(v0.clone())? {
        Ok(true)
    } else {
        f20(v0)
    }
}

pub fn f23(v0: Str) -> R<Str> {
    f16(v0, 97u64, Bytes::lit(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a]))
}

pub fn f24(v0: Str) -> R<bool> {
    let v1: u64 = f26(v0)?;
    let a207 = 65u64;
    let a208 = v1;
    if nat_le(a207, a208) {
        let a209 = v1;
        let a210 = 90u64;
        Ok(nat_le(a209, a210))
    } else {
        Ok(false)
    }
}

pub fn f25(v0: Str) -> R<bool> {
    let v1: u64 = f26(v0)?;
    let a211 = 97u64;
    let a212 = v1;
    if nat_le(a211, a212) {
        let a213 = v1;
        let a214 = 122u64;
        Ok(nat_le(a213, a214))
    } else {
        Ok(false)
    }
}

pub fn f26(v0: Str) -> R<u64> {
    f18({
        let a215 = v0;
        utf8_encode(a215)
    }, 0u64)
}

pub fn f27(v0: Str) -> R<bool> {
    if f24(v0.clone())? {
        Ok(true)
    } else {
        f25(v0)
    }
}

pub fn run(p0: Str) -> R<Str> {
    f0(p0)
}
