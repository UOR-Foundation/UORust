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
    C0(u64, List<Str>),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(List<Adt0>, bool),
}

pub fn f0(v0: List<Str>) -> R<List<Adt0>> {
    let v1: List<Adt0> = List::<Adt0>::nil();
    let v2: bool = {
        let a2 = {
            let a1 = v0.clone();
            length_list(a1)
        };
        let a3 = 0u64;
        nat_eq(a2, a3)
    };
    let v3: Adt1 = f1(v0, Adt1::C0(v1, v2))?;
    let v4: List<Adt0> = match v3.clone() {
        Adt1::C0(h4, _) => {
            h4
        }
    };
    let v5: bool = match v3 {
        Adt1::C0(_, h5) => {
            h5
        }
    };
    let v7: List<Adt0> = if v5 {
        let v6: List<Adt0> = f2({
            let a6 = List::cons(16u64, List::<u64>::nil());
            let a7 = List::cons(32u64, List::cons(64u64, List::<u64>::nil()));
            append_list(a6, a7)
        }, v4)?;
        v6
    } else {
        v4
    };
    Ok(v7)
}

pub fn f1(p15: List<Str>, p16: Adt1) -> R<Adt1> {
    let m14 = (p15, p16);
    {
        let mut m14 = m14;
        loop {
            let step = {
                let (v0, v1) = m14;
                let m8 = v0;
                match m8.uncons() {
                    None => {
                        Err::<(List<Str>, Adt1), R<Adt1>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: List<Adt0> = match v1.clone() {
                            Adt1::C0(h9, _) => {
                                h9
                            }
                        };
                        let v5: bool = match v1 {
                            Adt1::C0(_, h10) => {
                                h10
                            }
                        };
                        let m11 = f3(v2.clone())?;
                        match m11 {
                            None => {
                                let v6: bool = true;
                                Ok::<(List<Str>, Adt1), R<Adt1>>((v3, Adt1::C0(v4, v6)))
                            }
                            Some(v7) => {
                                let v8: List<Str> = f4(v4.clone(), v7);
                                let v9: List<Str> = {
                                    let a12 = v8;
                                    let a13 = List::cons(v2, List::<Str>::nil());
                                    append_list(a12, a13)
                                };
                                let v10: List<Adt0> = f5(v4, Adt0::C0(v7, v9));
                                Ok::<(List<Str>, Adt1), R<Adt1>>((v3, Adt1::C0(v10, v5)))
                            }
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

pub fn f2(p25: List<u64>, p26: List<Adt0>) -> R<List<Adt0>> {
    let m24 = (p25, p26);
    {
        let mut m24 = m24;
        loop {
            let step = {
                let (v0, v1) = m24;
                let m17 = v0;
                match m17.uncons() {
                    None => {
                        Err::<(List<u64>, List<Adt0>), R<List<Adt0>>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: List<Str> = f6(v1.clone(), v2);
                        let v5: List<Str> = {
                            let a22 = v4;
                            let a23 = List::cons({
                                let a20 = {
                                    let a18 = List::cons(Str::lit("any "), List::<Str>::nil());
                                    let a19 = List::cons(f7(v2)?, List::cons(Str::lit("-bit target"), List::<Str>::nil()));
                                    append_list(a18, a19)
                                };
                                let a21 = Str::lit("");
                                join(a20, a21)
                            }, List::<Str>::nil());
                            append_list(a22, a23)
                        };
                        let v6: List<Adt0> = f8(v1, Adt0::C0(v2, v5));
                        Ok::<(List<u64>, List<Adt0>), R<List<Adt0>>>((v3, v6))
                    }
                }
            };
            match step {
                Ok(next) => { m24 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f3(v0: Str) -> R<Option<u64>> {
    let m29 = {
        let a27 = f9(v0, Str::lit("-"))?;
        let a28 = 0u64;
        index_list(a27, a28)
    };
    match m29 {
        None => {
            Ok(None::<u64>)
        }
        Some(v1) => {
            let v2: List<Str> = {
                let a34 = {
                    let a30 = List::cons(Str::lit("x86_64"), List::cons(Str::lit("aarch64"), List::cons(Str::lit("arm64"), List::<Str>::nil())));
                    let a31 = List::cons(Str::lit("riscv64"), List::cons(Str::lit("powerpc64"), List::cons(Str::lit("powerpc64le"), List::cons(Str::lit("s390x"), List::<Str>::nil()))));
                    append_list(a30, a31)
                };
                let a35 = {
                    let a32 = List::cons(Str::lit("sparc64"), List::cons(Str::lit("mips64"), List::cons(Str::lit("mips64el"), List::<Str>::nil())));
                    let a33 = List::cons(Str::lit("loongarch64"), List::cons(Str::lit("wasm64"), List::cons(Str::lit("bpfel"), List::cons(Str::lit("bpfeb"), List::<Str>::nil()))));
                    append_list(a32, a33)
                };
                append_list(a34, a35)
            };
            let v3: List<Str> = {
                let a48 = {
                    let a40 = {
                        let a36 = List::cons(Str::lit("i386"), List::cons(Str::lit("i586"), List::cons(Str::lit("i686"), List::cons(Str::lit("x86"), List::<Str>::nil()))));
                        let a37 = List::cons(Str::lit("arm"), List::cons(Str::lit("armv7"), List::cons(Str::lit("armv7a"), List::cons(Str::lit("armv7r"), List::<Str>::nil()))));
                        append_list(a36, a37)
                    };
                    let a41 = {
                        let a38 = List::cons(Str::lit("armebv7r"), List::cons(Str::lit("armv5te"), List::cons(Str::lit("armv4t"), List::cons(Str::lit("armv6"), List::<Str>::nil()))));
                        let a39 = List::cons(Str::lit("thumbv6m"), List::cons(Str::lit("thumbv7m"), List::cons(Str::lit("thumbv7em"), List::cons(Str::lit("thumbv7neon"), List::<Str>::nil()))));
                        append_list(a38, a39)
                    };
                    append_list(a40, a41)
                };
                let a49 = {
                    let a46 = {
                        let a42 = List::cons(Str::lit("thumbv8m.base"), List::cons(Str::lit("thumbv8m.main"), List::cons(Str::lit("riscv32i"), List::cons(Str::lit("riscv32im"), List::<Str>::nil()))));
                        let a43 = List::cons(Str::lit("riscv32imc"), List::cons(Str::lit("riscv32imac"), List::cons(Str::lit("riscv32imafc"), List::cons(Str::lit("powerpc"), List::<Str>::nil()))));
                        append_list(a42, a43)
                    };
                    let a47 = {
                        let a44 = List::cons(Str::lit("mips"), List::cons(Str::lit("mipsel"), List::cons(Str::lit("sparc"), List::cons(Str::lit("wasm32"), List::<Str>::nil()))));
                        let a45 = List::cons(Str::lit("hexagon"), List::cons(Str::lit("csky"), List::cons(Str::lit("xtensa"), List::cons(Str::lit("m68k"), List::<Str>::nil()))));
                        append_list(a44, a45)
                    };
                    append_list(a46, a47)
                };
                append_list(a48, a49)
            };
            if f10(v2, v1.clone()) {
                Ok(Some(64u64))
            } else {
                if if if f11(v3, v1.clone()) {
                    true
                } else {
                    f12(v1.clone(), Str::lit("thumbv"))
                } {
                    true
                } else {
                    f12(v1.clone(), Str::lit("armv"))
                } {
                    Ok(Some(32u64))
                } else {
                    let a50 = v1.clone();
                    let a51 = Str::lit("avr");
                    if if equal(a50, a51) {
                        true
                    } else {
                        let a52 = v1;
                        let a53 = Str::lit("msp430");
                        equal(a52, a53)
                    } {
                        Ok(Some(16u64))
                    } else {
                        Ok(None::<u64>)
                    }
                }
            }
        }
    }
}

pub fn f4(v0: List<Adt0>, v1: u64) -> List<Str> {
    let m54 = f13(v0, v1);
    match m54 {
        None => {
            List::<Str>::nil()
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f5(v0: List<Adt0>, v1: Adt0) -> List<Adt0> {
    let m55 = v0;
    match m55.uncons() {
        None => {
            List::cons(v1, List::<Adt0>::nil())
        }
        Some((v2, v3)) => {
            let a58 = match v1.clone() {
                Adt0::C0(h56, _) => {
                    h56
                }
            };
            let a59 = match v2.clone() {
                Adt0::C0(h57, _) => {
                    h57
                }
            };
            if nat_eq(a58, a59) {
                List::cons(v1, v3)
            } else {
                let a62 = match v1.clone() {
                    Adt0::C0(h60, _) => {
                        h60
                    }
                };
                let a63 = match v2.clone() {
                    Adt0::C0(h61, _) => {
                        h61
                    }
                };
                if nat_lt(a62, a63) {
                    List::cons(v1, List::cons(v2, v3))
                } else {
                    List::cons(v2, f5(v3, v1))
                }
            }
        }
    }
}

pub fn f6(v0: List<Adt0>, v1: u64) -> List<Str> {
    let m64 = f14(v0, v1);
    match m64 {
        None => {
            List::<Str>::nil()
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f7(v0: u64) -> R<Str> {
    Ok(f15({
        let a65 = v0;
        let a66 = 1u64;
        nat_add(a65, a66)?
    }, v0))
}

pub fn f8(v0: List<Adt0>, v1: Adt0) -> List<Adt0> {
    let m67 = v0;
    match m67.uncons() {
        None => {
            List::cons(v1, List::<Adt0>::nil())
        }
        Some((v2, v3)) => {
            let a70 = match v1.clone() {
                Adt0::C0(h68, _) => {
                    h68
                }
            };
            let a71 = match v2.clone() {
                Adt0::C0(h69, _) => {
                    h69
                }
            };
            if nat_eq(a70, a71) {
                List::cons(v1, v3)
            } else {
                let a74 = match v1.clone() {
                    Adt0::C0(h72, _) => {
                        h72
                    }
                };
                let a75 = match v2.clone() {
                    Adt0::C0(h73, _) => {
                        h73
                    }
                };
                if nat_lt(a74, a75) {
                    List::cons(v1, List::cons(v2, v3))
                } else {
                    List::cons(v2, f8(v3, v1))
                }
            }
        }
    }
}

pub fn f9(v0: Str, v1: Str) -> R<List<Str>> {
    let v2: Bytes = {
        let a76 = v0.clone();
        utf8_encode(a76)
    };
    let v3: Bytes = {
        let a77 = v1;
        utf8_encode(a77)
    };
    let a79 = {
        let a78 = v3.clone();
        length_bytes(a78)
    };
    let a80 = 0u64;
    if nat_eq(a79, a80) {
        Ok(List::cons(Str::lit(""), {
            let a81 = f16(v0)?;
            let a82 = List::cons(Str::lit(""), List::<Str>::nil());
            append_list(a81, a82)
        }))
    } else {
        f17({
            let a84 = {
                let a83 = v2.clone();
                length_bytes(a83)
            };
            let a85 = 1u64;
            nat_add(a84, a85)?
        }, v2, v3, 0u64, 0u64)
    }
}

pub fn f10(p90: List<Str>, v1: Str) -> bool {
    let m89 = p90;
    {
        let mut m89 = m89;
        loop {
            let step = {
                let v0 = m89;
                let m86 = v0;
                match m86.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a87 = v2;
                        let a88 = v1.clone();
                        if equal(a87, a88) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m89 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f11(p95: List<Str>, v1: Str) -> bool {
    let m94 = p95;
    {
        let mut m94 = m94;
        loop {
            let step = {
                let v0 = m94;
                let m91 = v0;
                match m91.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a92 = v2;
                        let a93 = v1.clone();
                        if equal(a92, a93) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m94 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f12(v0: Str, v1: Str) -> bool {
    f18({
        let a96 = v0;
        utf8_encode(a96)
    }, 0u64, {
        let a97 = v1;
        utf8_encode(a97)
    })
}

pub fn f13(p104: List<Adt0>, v1: u64) -> Option<List<Str>> {
    let m103 = p104;
    {
        let mut m103 = m103;
        loop {
            let step = {
                let v0 = m103;
                let m98 = v0;
                match m98.uncons() {
                    None => {
                        Err::<List<Adt0>, Option<List<Str>>>(None::<List<Str>>)
                    }
                    Some((v2, v3)) => {
                        let a100 = match v2.clone() {
                            Adt0::C0(h99, _) => {
                                h99
                            }
                        };
                        let a101 = v1;
                        if nat_eq(a100, a101) {
                            Err::<List<Adt0>, Option<List<Str>>>(Some(match v2 {
                                Adt0::C0(_, h102) => {
                                    h102
                                }
                            }))
                        } else {
                            Ok::<List<Adt0>, Option<List<Str>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m103 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f14(p111: List<Adt0>, v1: u64) -> Option<List<Str>> {
    let m110 = p111;
    {
        let mut m110 = m110;
        loop {
            let step = {
                let v0 = m110;
                let m105 = v0;
                match m105.uncons() {
                    None => {
                        Err::<List<Adt0>, Option<List<Str>>>(None::<List<Str>>)
                    }
                    Some((v2, v3)) => {
                        let a107 = match v2.clone() {
                            Adt0::C0(h106, _) => {
                                h106
                            }
                        };
                        let a108 = v1;
                        if nat_eq(a107, a108) {
                            Err::<List<Adt0>, Option<List<Str>>>(Some(match v2 {
                                Adt0::C0(_, h109) => {
                                    h109
                                }
                            }))
                        } else {
                            Ok::<List<Adt0>, Option<List<Str>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m110 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f15(v0: u64, v1: u64) -> Str {
    let m112 = v0;
    if m112 == 0 {
        Str::lit("")
    } else {
        let v2 = m112 - 1;
        let a113 = v1;
        let a114 = 10u64;
        if nat_lt(a113, a114) {
            f19(v1)
        } else {
            let a121 = List::cons(f15(v2, {
                let a115 = v1;
                let a116 = 10u64;
                let a117 = 0u64;
                nat_quot(a115, a116, a117)
            }), List::cons(f19({
                let a118 = v1;
                let a119 = 10u64;
                let a120 = 0u64;
                nat_rem(a118, a119, a120)
            }), List::<Str>::nil()));
            let a122 = Str::lit("");
            join(a121, a122)
        }
    }
}

pub fn f16(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a123 = v0;
        utf8_encode(a123)
    };
    f20({
        let a124 = v1.clone();
        length_bytes(a124)
    }, v1, 0u64)
}

pub fn f17(v0: u64, v1: Bytes, v2: Bytes, v3: u64, v4: u64) -> R<List<Str>> {
    let m125 = v0;
    if m125 == 0 {
        Ok(List::cons(f21(v1.clone(), v3, {
            let a126 = v1;
            length_bytes(a126)
        }), List::<Str>::nil()))
    } else {
        let v5 = m125 - 1;
        let a131 = {
            let a127 = v1.clone();
            length_bytes(a127)
        };
        let a132 = {
            let a129 = v4;
            let a130 = {
                let a128 = v2.clone();
                length_bytes(a128)
            };
            nat_add(a129, a130)?
        };
        if nat_lt(a131, a132) {
            Ok(List::cons(f21(v1.clone(), v3, {
                let a133 = v1;
                length_bytes(a133)
            }), List::<Str>::nil()))
        } else {
            if f18(v1.clone(), v4, v2.clone()) {
                Ok(List::cons(f21(v1.clone(), v3, v4), f17(v5, v1, v2.clone(), {
                    let a135 = v4;
                    let a136 = {
                        let a134 = v2.clone();
                        length_bytes(a134)
                    };
                    nat_add(a135, a136)?
                }, {
                    let a138 = v4;
                    let a139 = {
                        let a137 = v2;
                        length_bytes(a137)
                    };
                    nat_add(a138, a139)?
                })?))
            } else {
                f17(v5, v1, v2, v3, {
                    let a140 = v4;
                    let a141 = 1u64;
                    nat_add(a140, a141)?
                })
            }
        }
    }
}

pub fn f18(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m146 = {
        let a143 = v0;
        let a144 = v1;
        let a145 = {
            let a142 = v2.clone();
            length_bytes(a142)
        };
        slice_bytes(a143, a144, a145)
    };
    match m146 {
        None => {
            false
        }
        Some(v3) => {
            let a147 = v3;
            let a148 = v2;
            equal(a147, a148)
        }
    }
}

pub fn f19(v0: u64) -> Str {
    let a149 = v0;
    let a150 = 0u64;
    if nat_eq(a149, a150) {
        Str::lit("0")
    } else {
        let a151 = v0;
        let a152 = 1u64;
        if nat_eq(a151, a152) {
            Str::lit("1")
        } else {
            let a153 = v0;
            let a154 = 2u64;
            if nat_eq(a153, a154) {
                Str::lit("2")
            } else {
                let a155 = v0;
                let a156 = 3u64;
                if nat_eq(a155, a156) {
                    Str::lit("3")
                } else {
                    let a157 = v0;
                    let a158 = 4u64;
                    if nat_eq(a157, a158) {
                        Str::lit("4")
                    } else {
                        let a159 = v0;
                        let a160 = 5u64;
                        if nat_eq(a159, a160) {
                            Str::lit("5")
                        } else {
                            let a161 = v0;
                            let a162 = 6u64;
                            if nat_eq(a161, a162) {
                                Str::lit("6")
                            } else {
                                let a163 = v0;
                                let a164 = 7u64;
                                if nat_eq(a163, a164) {
                                    Str::lit("7")
                                } else {
                                    let a165 = v0;
                                    let a166 = 8u64;
                                    if nat_eq(a165, a166) {
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

pub fn f20(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m167 = v0;
    if m167 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m167 - 1;
        let a169 = {
            let a168 = v1.clone();
            length_bytes(a168)
        };
        let a170 = v2;
        if nat_le(a169, a170) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f22(f23(v1.clone(), v2)?);
            Ok(List::cons(f21(v1.clone(), v2, {
                let a171 = v2;
                let a172 = v4;
                nat_add(a171, a172)?
            }), f20(v3, v1, {
                let a173 = v2;
                let a174 = v4;
                nat_add(a173, a174)?
            })?))
        }
    }
}

pub fn f21(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m180 = {
        let a177 = v0;
        let a178 = v1;
        let a179 = {
            let a175 = v2;
            let a176 = v1;
            nat_sub(a175, a176)
        };
        slice_bytes(a177, a178, a179)
    };
    match m180 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m182 = {
                let a181 = v3;
                utf8_decode(a181)
            };
            match m182 {
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

pub fn f22(v0: u64) -> u64 {
    let a183 = v0;
    let a184 = 128u64;
    if nat_lt(a183, a184) {
        1u64
    } else {
        let a185 = v0;
        let a186 = 224u64;
        if nat_lt(a185, a186) {
            2u64
        } else {
            let a187 = v0;
            let a188 = 240u64;
            if nat_lt(a187, a188) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f23(v0: Bytes, v1: u64) -> R<u64> {
    let m191 = {
        let a189 = v0;
        let a190 = v1;
        index_bytes(a189, a190)
    };
    match m191 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a236 = {
                let a212 = {
                    let a200 = {
                        let a194 = {
                            let a192 = v3;
                            let a193 = 128u8;
                            fixed_u8::bit_and(a192, a193)
                        };
                        let a195 = 128u8;
                        if equal(a194, a195) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a201 = {
                        let a198 = {
                            let a196 = v3;
                            let a197 = 64u8;
                            fixed_u8::bit_and(a196, a197)
                        };
                        let a199 = 64u8;
                        if equal(a198, a199) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a200, a201)?
                };
                let a213 = {
                    let a210 = {
                        let a204 = {
                            let a202 = v3;
                            let a203 = 32u8;
                            fixed_u8::bit_and(a202, a203)
                        };
                        let a205 = 32u8;
                        if equal(a204, a205) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a211 = {
                        let a208 = {
                            let a206 = v3;
                            let a207 = 16u8;
                            fixed_u8::bit_and(a206, a207)
                        };
                        let a209 = 16u8;
                        if equal(a208, a209) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a210, a211)?
                };
                nat_add(a212, a213)?
            };
            let a237 = {
                let a234 = {
                    let a222 = {
                        let a216 = {
                            let a214 = v3;
                            let a215 = 8u8;
                            fixed_u8::bit_and(a214, a215)
                        };
                        let a217 = 8u8;
                        if equal(a216, a217) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a223 = {
                        let a220 = {
                            let a218 = v3;
                            let a219 = 4u8;
                            fixed_u8::bit_and(a218, a219)
                        };
                        let a221 = 4u8;
                        if equal(a220, a221) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a222, a223)?
                };
                let a235 = {
                    let a232 = {
                        let a226 = {
                            let a224 = v3;
                            let a225 = 2u8;
                            fixed_u8::bit_and(a224, a225)
                        };
                        let a227 = 2u8;
                        if equal(a226, a227) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a233 = {
                        let a230 = {
                            let a228 = v3;
                            let a229 = 1u8;
                            fixed_u8::bit_and(a228, a229)
                        };
                        let a231 = 1u8;
                        if equal(a230, a231) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a232, a233)?
                };
                nat_add(a234, a235)?
            };
            nat_add(a236, a237)
        }
    }
}

pub fn run(p0: List<Str>) -> R<List<Adt0>> {
    f0(p0)
}
