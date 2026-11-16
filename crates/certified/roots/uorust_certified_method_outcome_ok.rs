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
    C0(Str),
    C1(Str),
    C2,
    C3(Str),
    C4(Str, List<Adt15>),
    C5(Str),
    C6(Str),
    C7(Str),
    C8(Str),
    C9(Str),
    C10(Adt75),
}

#[derive(Clone)]
pub enum Adt75 {
    C0(Str, Str),
    C1(Str),
    C2(Str),
    C3,
    C4(Str),
    C5(Str, Str),
    C6(Str, List<Adt76>),
}

#[derive(Clone)]
pub enum Adt76 {
    C0(Str, Adt77),
}

#[derive(Clone)]
pub enum Adt77 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt78 {
    C0(Str, Adt69),
}

#[derive(Clone)]
pub enum Adt79 {
    C0,
    C1,
    C2,
    C3,
}

pub fn f0(v0: Adt0, v1: Str, v2: Adt2, v3: Adt16, v4: Adt69) -> R<Result<((Adt71, Adt0), Adt69), Adt74>> {
    let v5: Adt78 = f1(v3.clone(), v2.clone(), v4, Str::lit("value"))?;
    let v6: Str = match v5.clone() {
        Adt78::C0(h1, _) => {
            h1
        }
    };
    let v7: Adt69 = match v5 {
        Adt78::C0(_, h2) => {
            h2
        }
    };
    let v8: Adt78 = f1(v3, v2, v7, Str::lit("error"))?;
    let v9: Str = match v8.clone() {
        Adt78::C0(h3, _) => {
            h3
        }
    };
    let v10: Adt69 = match v8 {
        Adt78::C0(_, h4) => {
            h4
        }
    };
    Ok(Ok::<((Adt71, Adt0), Adt69), Adt74>(((Adt71::C1(v1, List::cons(Adt72::C0({
        let a5 = List::cons(Str::lit("Result.ok "), List::cons(v6.clone(), List::<Str>::nil()));
        let a6 = Str::lit("");
        join(a5, a6)
    }, Adt71::C0(f2(v0.clone(), v6))), List::cons(Adt72::C0({
        let a7 = List::cons(Str::lit("Result.error "), List::cons(v9, List::<Str>::nil()));
        let a8 = Str::lit("");
        join(a7, a8)
    }, Adt71::C0(f3(v0.clone()))), List::<Adt72>::nil()))), f4(v0)), v10)))
}

pub fn f1(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str) -> R<Adt78> {
    let v4: Str = f5(v3.clone(), f6(v0, v1, v2.clone(), v3, List::<Str>::nil()))?;
    let v5: List<Str> = match v2.clone() {
        Adt69::C0(_, _, h9, _, _, _, _) => {
            h9
        }
    };
    let v6: List<Str> = {
        let a10 = v5;
        let a11 = List::cons(v4.clone(), List::<Str>::nil());
        append_list(a10, a11)
    };
    Ok(Adt78::C0(v4, Adt69::C0(match v2.clone() {
        Adt69::C0(h12, _, _, _, _, _, _) => {
            h12
        }
    }, match v2.clone() {
        Adt69::C0(_, h13, _, _, _, _, _) => {
            h13
        }
    }, v6, match v2.clone() {
        Adt69::C0(_, _, _, h14, _, _, _) => {
            h14
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, h15, _, _) => {
            h15
        }
    }, match v2.clone() {
        Adt69::C0(_, _, _, _, _, h16, _) => {
            h16
        }
    }, match v2 {
        Adt69::C0(_, _, _, _, _, _, h17) => {
            h17
        }
    })))
}

pub fn f2(v0: Adt0, v1: Str) -> Str {
    let a18 = List::cons(Str::lit("(ctor Option.some ["), List::cons(f7(v0), List::cons(Str::lit("] "), List::cons(v1, List::cons(Str::lit(")"), List::<Str>::nil())))));
    let a19 = Str::lit("");
    join(a18, a19)
}

pub fn f3(v0: Adt0) -> Str {
    let a20 = List::cons(Str::lit("(ctor Option.none ["), List::cons(f7(v0), List::cons(Str::lit("])"), List::<Str>::nil())));
    let a21 = Str::lit("");
    join(a20, a21)
}

pub fn f4(v0: Adt0) -> Adt0 {
    Adt0::C12(std::rc::Rc::new(v0))
}

pub fn f5(v0: Str, v1: List<Str>) -> R<Str> {
    let v2: u64 = f8(f9(f10(v1.clone(), v0.clone()))?, 0u64);
    let m28 = {
        let a24 = {
            let a22 = v2;
            let a23 = 1u64;
            nat_add(a22, a23)?
        };
        let a25 = 4294967296u64;
        if nat_lt(a24, a25) {
            Some({
                let a26 = v2;
                let a27 = 1u64;
                nat_add(a26, a27)?
            })
        } else {
            None::<u64>
        }
    };
    match m28 {
        None => {
            let m34 = f11(f12(f13(1u64, {
                let a32 = {
                    let a30 = {
                        let a29 = v1.clone();
                        length_list(a29)
                    };
                    let a31 = 2u64;
                    nat_add(a30, a31)?
                };
                let a33 = 1u64;
                nat_sub(a32, a33)
            })?, v0)?, v1);
            match m34 {
                None => {
                    Ok(Str::lit(""))
                }
                Some(v3) => {
                    Ok(v3)
                }
            }
        }
        Some(v4) => {
            let a35 = List::cons(v0, List::cons(f14(v4)?, List::<Str>::nil()));
            let a36 = Str::lit("");
            Ok(join(a35, a36))
        }
    }
}

pub fn f6(v0: Adt16, v1: Adt2, v2: Adt69, v3: Str, v4: List<Str>) -> List<Str> {
    f15({
        let a50 = {
            let a48 = {
                let a45 = {
                    let a42 = {
                        let a39 = f16(f17(match v0.clone() {
                            Adt16::C0(_, _, _, _, h37, _, _, _, _, _, _, _, _, _, _, _, _) => {
                                h37
                            }
                        }));
                        let a40 = match v0 {
                            Adt16::C0(_, _, _, _, _, _, _, _, h38, _, _, _, _, _, _, _, _) => {
                                h38
                            }
                        };
                        append_list(a39, a40)
                    };
                    let a43 = f18(f19(match v1.clone() {
                        Adt2::C0(_, h41, _, _) => {
                            h41
                        }
                    }));
                    append_list(a42, a43)
                };
                let a46 = match v1 {
                    Adt2::C0(_, _, _, h44) => {
                        h44
                    }
                };
                append_list(a45, a46)
            };
            let a49 = match v2 {
                Adt69::C0(_, _, h47, _, _, _, _) => {
                    h47
                }
            };
            append_list(a48, a49)
        };
        let a51 = v4;
        append_list(a50, a51)
    }, v3)
}

pub fn f7(v0: Adt0) -> Str {
    let m52 = v0;
    match m52 {
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
            let a53 = List::cons(f20(), List::cons(Str::lit("::ParseIntError"), List::<Str>::nil()));
            let a54 = Str::lit("");
            join(a53, a54)
        }
        Adt0::C10 => {
            let a55 = List::cons(f20(), List::cons(Str::lit("::FilePath"), List::<Str>::nil()));
            let a56 = Str::lit("");
            join(a55, a56)
        }
        Adt0::C11(v5) => {
            v5
        }
        Adt0::C12(r79) => {
            let v6 = (*r79).clone();
            let a57 = List::cons(Str::lit("(Option "), List::cons(f7(v6), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a58 = Str::lit("");
            join(a57, a58)
        }
        Adt0::C13(r80, r81) => {
            let v7 = (*r80).clone();
            let v8 = (*r81).clone();
            let a59 = List::cons(Str::lit("(Result "), List::cons(f7(v7), List::cons(Str::lit(" "), List::cons(f7(v8), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a60 = Str::lit("");
            join(a59, a60)
        }
        Adt0::C14(r82) => {
            let v9 = (*r82).clone();
            let a61 = List::cons(Str::lit("(List "), List::cons(f7(v9), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a62 = Str::lit("");
            join(a61, a62)
        }
        Adt0::C15(v10) => {
            let a65 = List::cons(Str::lit("Tuple"), List::cons({
                let a63 = f21(v10);
                let a64 = Str::lit("");
                join(a63, a64)
            }, List::<Str>::nil()));
            let a66 = Str::lit("");
            join(a65, a66)
        }
        Adt0::C16(v11) => {
            let a69 = List::cons(Str::lit("("), List::cons({
                let a67 = f22(v11);
                let a68 = Str::lit(" * ");
                join(a67, a68)
            }, List::cons(Str::lit(")"), List::<Str>::nil())));
            let a70 = Str::lit("");
            join(a69, a70)
        }
        Adt0::C17(r83, r84, _) => {
            let v12 = (*r83).clone();
            let v13 = (*r84).clone();
            let a71 = List::cons(Str::lit("(List Tuple"), List::cons(f23(v12), List::cons(f23(v13), List::cons(Str::lit(")"), List::<Str>::nil()))));
            let a72 = Str::lit("");
            join(a71, a72)
        }
        Adt0::C18(r85, _) => {
            let v15 = (*r85).clone();
            let a73 = List::cons(Str::lit("(List "), List::cons(f7(v15), List::cons(Str::lit(")"), List::<Str>::nil())));
            let a74 = Str::lit("");
            join(a73, a74)
        }
        Adt0::C19(v17, r86) => {
            let v18 = (*r86).clone();
            let a77 = List::cons(Str::lit("(Fn ("), List::cons({
                let a75 = f24(v17);
                let a76 = Str::lit(" ");
                join(a75, a76)
            }, List::cons(Str::lit(") "), List::cons(f7(v18), List::cons(Str::lit(")"), List::<Str>::nil())))));
            let a78 = Str::lit("");
            join(a77, a78)
        }
    }
}

pub fn f8(p91: List<u64>, p92: u64) -> u64 {
    let m90 = (p91, p92);
    {
        let mut m90 = m90;
        loop {
            let step = {
                let (v0, v1) = m90;
                let m87 = v0;
                match m87.uncons() {
                    None => {
                        Err::<(List<u64>, u64), u64>(v1)
                    }
                    Some((v2, v3)) => {
                        Ok::<(List<u64>, u64), u64>((v3, {
                            let a88 = v1;
                            let a89 = v2;
                            if nat_le(a88, a89) {
                                v2
                            } else {
                                v1
                            }
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m90 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f9(v0: List<Str>) -> R<List<u64>> {
    let m93 = v0;
    match m93.uncons() {
        None => {
            Ok(List::<u64>::nil())
        }
        Some((v1, v2)) => {
            let m94 = f25(v1)?;
            match m94 {
                None => {
                    f9(v2)
                }
                Some(v3) => {
                    Ok(List::cons(v3, f9(v2)?))
                }
            }
        }
    }
}

pub fn f10(v0: List<Str>, v1: Str) -> List<Str> {
    let m95 = v0;
    match m95.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            let m96 = f26(v2, v1.clone());
            match m96 {
                None => {
                    f10(v3, v1)
                }
                Some(v4) => {
                    List::cons(v4, f10(v3, v1))
                }
            }
        }
    }
}

pub fn f11(p100: List<Str>, v1: List<Str>) -> Option<Str> {
    let m99 = p100;
    {
        let mut m99 = m99;
        loop {
            let step = {
                let v0 = m99;
                let m97 = v0;
                match m97.uncons() {
                    None => {
                        Err::<List<Str>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a98 = f27(v1.clone(), v2.clone());
                        if bool_not(a98) {
                            Err::<List<Str>, Option<Str>>(Some(v2))
                        } else {
                            Ok::<List<Str>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m99 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f12(v0: List<u64>, v1: Str) -> R<List<Str>> {
    let m101 = v0;
    match m101.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons({
                let a102 = List::cons(v1.clone(), List::cons(f14(v2)?, List::<Str>::nil()));
                let a103 = Str::lit("");
                join(a102, a103)
            }, f12(v3, v1)?))
        }
    }
}

pub fn f13(v0: u64, v1: u64) -> R<List<u64>> {
    let m104 = v1;
    if m104 == 0 {
        Ok(List::<u64>::nil())
    } else {
        let v2 = m104 - 1;
        Ok(List::cons(v0, f13({
            let a105 = v0;
            let a106 = 1u64;
            nat_add(a105, a106)?
        }, v2)?))
    }
}

pub fn f14(v0: u64) -> R<Str> {
    Ok(f28({
        let a107 = v0;
        let a108 = 1u64;
        nat_add(a107, a108)?
    }, v0))
}

pub fn f15(v0: List<Str>, v1: Str) -> List<Str> {
    let m109 = v0;
    match m109.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            if f29(v2.clone(), v1.clone()) {
                List::cons(v2, f15(v3, v1))
            } else {
                f15(v3, v1)
            }
        }
    }
}

pub fn f16(v0: List<(Str, Adt0)>) -> List<Str> {
    let m110 = v0;
    match m110.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            let v3: Str = {
                let (h111, _) = v1;
                h111
            };
            let _: Adt0 = {
                let (_, h112) = v1;
                h112
            };
            List::cons(v3, f16(v2))
        }
    }
}

pub fn f17(v0: List<Adt17>) -> List<(Str, Adt0)> {
    let m113 = v0;
    match m113.uncons() {
        None => {
            List::<(Str, Adt0)>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt17::C0(_, h114) => {
                    h114
                }
            }, f17(v2))
        }
    }
}

pub fn f18(v0: List<Adt14>) -> List<Str> {
    let m115 = v0;
    match m115.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt14::C0(h116, _, _) => {
                    h116
                }
            }, f18(v2))
        }
    }
}

pub fn f19(v0: List<Adt13>) -> List<Adt14> {
    let m117 = v0;
    match m117.uncons() {
        None => {
            List::<Adt14>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt13::C0(_, h118) => {
                    h118
                }
            }, f19(v2))
        }
    }
}

pub fn f20() -> Str {
    Str::lit("Prelude")
}

pub fn f21(v0: List<Adt0>) -> List<Str> {
    let m119 = v0;
    match m119.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f23(v1), f21(v2))
        }
    }
}

pub fn f22(v0: List<Adt0>) -> List<Str> {
    let m120 = v0;
    match m120.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f7(v1), f22(v2))
        }
    }
}

pub fn f23(v0: Adt0) -> Str {
    let m121 = v0;
    match m121 {
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
        Adt0::C12(r144) => {
            let v6 = (*r144).clone();
            let a122 = List::cons(Str::lit("Option"), List::cons(f23(v6), List::<Str>::nil()));
            let a123 = Str::lit("");
            join(a122, a123)
        }
        Adt0::C13(r145, r146) => {
            let v7 = (*r145).clone();
            let v8 = (*r146).clone();
            let a124 = List::cons(Str::lit("Result"), List::cons(f23(v7), List::cons(f23(v8), List::<Str>::nil())));
            let a125 = Str::lit("");
            join(a124, a125)
        }
        Adt0::C14(r147) => {
            let v9 = (*r147).clone();
            let a126 = List::cons(Str::lit("List"), List::cons(f23(v9), List::<Str>::nil()));
            let a127 = Str::lit("");
            join(a126, a127)
        }
        Adt0::C15(v10) => {
            let a130 = List::cons(Str::lit("Tuple"), List::cons({
                let a128 = f30(v10);
                let a129 = Str::lit("");
                join(a128, a129)
            }, List::<Str>::nil()));
            let a131 = Str::lit("");
            join(a130, a131)
        }
        Adt0::C16(v11) => {
            let a134 = List::cons(Str::lit("Pair"), List::cons({
                let a132 = f31(v11);
                let a133 = Str::lit("");
                join(a132, a133)
            }, List::<Str>::nil()));
            let a135 = Str::lit("");
            join(a134, a135)
        }
        Adt0::C17(r148, r149, _) => {
            let v12 = (*r148).clone();
            let v13 = (*r149).clone();
            let a136 = List::cons(Str::lit("ListTuple"), List::cons(f23(v12), List::cons(f23(v13), List::<Str>::nil())));
            let a137 = Str::lit("");
            join(a136, a137)
        }
        Adt0::C18(r150, _) => {
            let v15 = (*r150).clone();
            let a138 = List::cons(Str::lit("List"), List::cons(f23(v15), List::<Str>::nil()));
            let a139 = Str::lit("");
            join(a138, a139)
        }
        Adt0::C19(v17, r151) => {
            let v18 = (*r151).clone();
            let a142 = List::cons(Str::lit("Fn"), List::cons({
                let a140 = f32(v17);
                let a141 = Str::lit("");
                join(a140, a141)
            }, List::cons(Str::lit("To"), List::cons(f23(v18), List::<Str>::nil()))));
            let a143 = Str::lit("");
            join(a142, a143)
        }
    }
}

pub fn f24(v0: List<Adt0>) -> List<Str> {
    let m152 = v0;
    match m152.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f7(v1), f24(v2))
        }
    }
}

pub fn f25(v0: Str) -> R<Option<u64>> {
    let m153 = f33(v0, 4294967295u64)?;
    match m153 {
        Ok(v1) => {
            Ok(Some(v1))
        }
        Err(_) => {
            Ok(None::<u64>)
        }
    }
}

pub fn f26(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a154 = v0;
        utf8_encode(a154)
    };
    let v3: Bytes = {
        let a155 = v1;
        utf8_encode(a155)
    };
    if f34(v2.clone(), 0u64, v3.clone()) {
        Some(f35(v2.clone(), {
            let a156 = v3;
            length_bytes(a156)
        }, {
            let a157 = v2;
            length_bytes(a157)
        }))
    } else {
        None::<Str>
    }
}

pub fn f27(p162: List<Str>, v1: Str) -> bool {
    let m161 = p162;
    {
        let mut m161 = m161;
        loop {
            let step = {
                let v0 = m161;
                let m158 = v0;
                match m158.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a159 = v2;
                        let a160 = v1.clone();
                        if equal(a159, a160) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m161 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f28(v0: u64, v1: u64) -> Str {
    let m163 = v0;
    if m163 == 0 {
        Str::lit("")
    } else {
        let v2 = m163 - 1;
        let a164 = v1;
        let a165 = 10u64;
        if nat_lt(a164, a165) {
            f36(v1)
        } else {
            let a172 = List::cons(f28(v2, {
                let a166 = v1;
                let a167 = 10u64;
                let a168 = 0u64;
                nat_quot(a166, a167, a168)
            }), List::cons(f36({
                let a169 = v1;
                let a170 = 10u64;
                let a171 = 0u64;
                nat_rem(a169, a170, a171)
            }), List::<Str>::nil()));
            let a173 = Str::lit("");
            join(a172, a173)
        }
    }
}

pub fn f29(v0: Str, v1: Str) -> bool {
    f34({
        let a174 = v0;
        utf8_encode(a174)
    }, 0u64, {
        let a175 = v1;
        utf8_encode(a175)
    })
}

pub fn f30(v0: List<Adt0>) -> List<Str> {
    let m176 = v0;
    match m176.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f23(v1), f30(v2))
        }
    }
}

pub fn f31(v0: List<Adt0>) -> List<Str> {
    let m177 = v0;
    match m177.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f23(v1), f31(v2))
        }
    }
}

pub fn f32(v0: List<Adt0>) -> List<Str> {
    let m178 = v0;
    match m178.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f23(v1), f32(v2))
        }
    }
}

pub fn f33(v0: Str, v1: u64) -> R<Result<u64, Adt79>> {
    let v2: Bytes = {
        let a179 = v0;
        utf8_encode(a179)
    };
    let v3: u64 = {
        let a180 = v2.clone();
        length_bytes(a180)
    };
    let a181 = v3;
    let a182 = 0u64;
    if nat_eq(a181, a182) {
        Ok(Err::<u64, Adt79>(Adt79::C0))
    } else {
        let a183 = f37(v2.clone(), 0u64)?;
        let a184 = 43u64;
        if nat_eq(a183, a184) {
            let a185 = v3;
            let a186 = 1u64;
            if nat_eq(a185, a186) {
                Ok(Err::<u64, Adt79>(Adt79::C1))
            } else {
                f38(v3, v2, 1u64, 0u64, v1)
            }
        } else {
            f38(v3, v2, 0u64, 0u64, v1)
        }
    }
}

pub fn f34(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m191 = {
        let a188 = v0;
        let a189 = v1;
        let a190 = {
            let a187 = v2.clone();
            length_bytes(a187)
        };
        slice_bytes(a188, a189, a190)
    };
    match m191 {
        None => {
            false
        }
        Some(v3) => {
            let a192 = v3;
            let a193 = v2;
            equal(a192, a193)
        }
    }
}

pub fn f35(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m199 = {
        let a196 = v0;
        let a197 = v1;
        let a198 = {
            let a194 = v2;
            let a195 = v1;
            nat_sub(a194, a195)
        };
        slice_bytes(a196, a197, a198)
    };
    match m199 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m201 = {
                let a200 = v3;
                utf8_decode(a200)
            };
            match m201 {
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

pub fn f36(v0: u64) -> Str {
    let a202 = v0;
    let a203 = 0u64;
    if nat_eq(a202, a203) {
        Str::lit("0")
    } else {
        let a204 = v0;
        let a205 = 1u64;
        if nat_eq(a204, a205) {
            Str::lit("1")
        } else {
            let a206 = v0;
            let a207 = 2u64;
            if nat_eq(a206, a207) {
                Str::lit("2")
            } else {
                let a208 = v0;
                let a209 = 3u64;
                if nat_eq(a208, a209) {
                    Str::lit("3")
                } else {
                    let a210 = v0;
                    let a211 = 4u64;
                    if nat_eq(a210, a211) {
                        Str::lit("4")
                    } else {
                        let a212 = v0;
                        let a213 = 5u64;
                        if nat_eq(a212, a213) {
                            Str::lit("5")
                        } else {
                            let a214 = v0;
                            let a215 = 6u64;
                            if nat_eq(a214, a215) {
                                Str::lit("6")
                            } else {
                                let a216 = v0;
                                let a217 = 7u64;
                                if nat_eq(a216, a217) {
                                    Str::lit("7")
                                } else {
                                    let a218 = v0;
                                    let a219 = 8u64;
                                    if nat_eq(a218, a219) {
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

pub fn f37(v0: Bytes, v1: u64) -> R<u64> {
    let m222 = {
        let a220 = v0;
        let a221 = v1;
        index_bytes(a220, a221)
    };
    match m222 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a267 = {
                let a243 = {
                    let a231 = {
                        let a225 = {
                            let a223 = v3;
                            let a224 = 128u8;
                            fixed_u8::bit_and(a223, a224)
                        };
                        let a226 = 128u8;
                        if equal(a225, a226) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a232 = {
                        let a229 = {
                            let a227 = v3;
                            let a228 = 64u8;
                            fixed_u8::bit_and(a227, a228)
                        };
                        let a230 = 64u8;
                        if equal(a229, a230) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a231, a232)?
                };
                let a244 = {
                    let a241 = {
                        let a235 = {
                            let a233 = v3;
                            let a234 = 32u8;
                            fixed_u8::bit_and(a233, a234)
                        };
                        let a236 = 32u8;
                        if equal(a235, a236) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a242 = {
                        let a239 = {
                            let a237 = v3;
                            let a238 = 16u8;
                            fixed_u8::bit_and(a237, a238)
                        };
                        let a240 = 16u8;
                        if equal(a239, a240) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a241, a242)?
                };
                nat_add(a243, a244)?
            };
            let a268 = {
                let a265 = {
                    let a253 = {
                        let a247 = {
                            let a245 = v3;
                            let a246 = 8u8;
                            fixed_u8::bit_and(a245, a246)
                        };
                        let a248 = 8u8;
                        if equal(a247, a248) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a254 = {
                        let a251 = {
                            let a249 = v3;
                            let a250 = 4u8;
                            fixed_u8::bit_and(a249, a250)
                        };
                        let a252 = 4u8;
                        if equal(a251, a252) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a253, a254)?
                };
                let a266 = {
                    let a263 = {
                        let a257 = {
                            let a255 = v3;
                            let a256 = 2u8;
                            fixed_u8::bit_and(a255, a256)
                        };
                        let a258 = 2u8;
                        if equal(a257, a258) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a264 = {
                        let a261 = {
                            let a259 = v3;
                            let a260 = 1u8;
                            fixed_u8::bit_and(a259, a260)
                        };
                        let a262 = 1u8;
                        if equal(a261, a262) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a263, a264)?
                };
                nat_add(a265, a266)?
            };
            nat_add(a267, a268)
        }
    }
}

pub fn f38(p288: u64, v1: Bytes, p289: u64, p290: u64, v4: u64) -> R<Result<u64, Adt79>> {
    let m287 = (p288, (p289, p290));
    {
        let mut m287 = m287;
        loop {
            let step = {
                let (v0, (v2, v3)) = m287;
                let m269 = v0;
                if m269 == 0 {
                    Err::<(u64, (u64, u64)), R<Result<u64, Adt79>>>(Ok(Ok::<u64, Adt79>(v3)))
                } else {
                    let v5 = m269 - 1;
                    let a271 = {
                        let a270 = v1.clone();
                        length_bytes(a270)
                    };
                    let a272 = v2;
                    if nat_le(a271, a272) {
                        Err::<(u64, (u64, u64)), R<Result<u64, Adt79>>>(Ok(Ok::<u64, Adt79>(v3)))
                    } else {
                        let v6: u64 = f37(v1.clone(), v2)?;
                        let a273 = 48u64;
                        let a274 = v6;
                        if if nat_le(a273, a274) {
                            let a275 = v6;
                            let a276 = 57u64;
                            nat_le(a275, a276)
                        } else {
                            false
                        } {
                            let v7: u64 = {
                                let a281 = {
                                    let a277 = v3;
                                    let a278 = 10u64;
                                    nat_mul(a277, a278)?
                                };
                                let a282 = {
                                    let a279 = v6;
                                    let a280 = 48u64;
                                    nat_sub(a279, a280)
                                };
                                nat_add(a281, a282)?
                            };
                            let a283 = v4;
                            let a284 = v7;
                            if nat_lt(a283, a284) {
                                Err::<(u64, (u64, u64)), R<Result<u64, Adt79>>>(Ok(Err::<u64, Adt79>(Adt79::C2)))
                            } else {
                                Ok::<(u64, (u64, u64)), R<Result<u64, Adt79>>>((v5, ({
                                    let a285 = v2;
                                    let a286 = 1u64;
                                    nat_add(a285, a286)?
                                }, v7)))
                            }
                        } else {
                            Err::<(u64, (u64, u64)), R<Result<u64, Adt79>>>(Ok(Err::<u64, Adt79>(Adt79::C1)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m287 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Str, p2: Adt2, p3: Adt16, p4: Adt69) -> R<Result<((Adt71, Adt0), Adt69), Adt74>> {
    f0(p0, p1, p2, p3, p4)
}
