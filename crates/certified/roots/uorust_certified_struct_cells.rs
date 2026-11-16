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
    C0(Adt1, List<Adt7>, bool),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(List<Adt2>, Str),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Str, Adt3),
}

#[derive(Clone)]
pub enum Adt3 {
    C0,
    C1(List<Adt4>),
    C2,
}

#[derive(Clone)]
pub enum Adt4 {
    C0(Adt5),
    C1(Str),
}

#[derive(Clone)]
pub enum Adt5 {
    C0(Str, std::rc::Rc<Adt6>),
}

#[derive(Clone)]
pub enum Adt6 {
    C0(bool, std::rc::Rc<Adt5>),
    C1(std::rc::Rc<Adt5>),
    C2(std::rc::Rc<Adt5>, Str),
    C3(List<Adt5>, std::rc::Rc<Adt5>),
    C4(std::rc::Rc<Adt5>),
    C5(List<Adt5>),
    C6(bool, Adt1),
    C7,
}

#[derive(Clone)]
pub enum Adt7 {
    C0(Adt8, Adt9),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str),
    C1(u64),
}

#[derive(Clone)]
pub enum Adt9 {
    C0(std::rc::Rc<Adt10>),
    C1,
    C2(std::rc::Rc<Adt9>),
    C3(std::rc::Rc<Adt11>),
    C4(List<Adt9>),
    C5(Adt12),
    C6(Adt0),
    C7(List<Adt9>),
    C8,
    C9(Adt13),
    C10(std::rc::Rc<Adt14>),
    C11(Adt1),
    C12(List<Adt9>),
    C13(std::rc::Rc<Adt47>),
    C14,
}

#[derive(Clone)]
pub enum Adt10 {
    C0(Str, bool, std::rc::Rc<Option<Adt9>>),
}

#[derive(Clone)]
pub enum Adt11 {
    C0(std::rc::Rc<Adt9>, Adt5),
}

#[derive(Clone)]
pub enum Adt12 {
    C0(Adt1, List<Adt9>),
}

#[derive(Clone)]
pub enum Adt13 {
    C0(bool),
    C1(Option<u64>, Str),
    C2(Str),
    C3(Str),
    C4(u64),
    C5(List<u64>),
    C6,
}

#[derive(Clone)]
pub enum Adt14 {
    C0(std::rc::Rc<Option<Adt15>>, std::rc::Rc<Option<Adt15>>, bool),
}

#[derive(Clone)]
pub enum Adt15 {
    C0(List<Adt15>),
    C1(std::rc::Rc<Adt16>),
    C2(std::rc::Rc<Adt17>),
    C3(Adt19),
    C4(std::rc::Rc<Adt27>),
    C5(std::rc::Rc<Adt28>),
    C6(std::rc::Rc<Adt29>),
    C7(std::rc::Rc<Adt30>),
    C8(bool),
    C9(std::rc::Rc<Adt31>),
    C10(std::rc::Rc<Adt32>),
    C11(std::rc::Rc<Adt33>),
    C12(std::rc::Rc<Adt34>),
    C13(std::rc::Rc<Adt35>),
    C14(Adt13),
    C15(Adt36),
    C16(std::rc::Rc<Adt24>),
    C17(std::rc::Rc<Adt37>),
    C18(std::rc::Rc<Adt39>),
    C19(std::rc::Rc<Adt15>),
    C20(Adt1),
    C21(std::rc::Rc<Adt40>),
    C22(std::rc::Rc<Option<Adt15>>),
    C23(std::rc::Rc<Adt41>),
    C24(std::rc::Rc<Adt15>),
    C25(List<Adt15>),
    C26(std::rc::Rc<Adt43>),
    C27(std::rc::Rc<Adt45>),
    C28(Adt46),
}

#[derive(Clone)]
pub enum Adt16 {
    C0(std::rc::Rc<Adt15>, std::rc::Rc<Adt15>),
}

#[derive(Clone)]
pub enum Adt17 {
    C0(std::rc::Rc<Adt15>, Adt18, std::rc::Rc<Adt15>),
}

#[derive(Clone)]
pub enum Adt18 {
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
pub enum Adt19 {
    C0(bool, Adt20),
}

#[derive(Clone)]
pub enum Adt20 {
    C0(List<Adt21>),
}

#[derive(Clone)]
pub enum Adt21 {
    C0(Adt22),
    C1(Adt15, bool),
    C2(Adt24),
    C3,
}

#[derive(Clone)]
pub enum Adt22 {
    C0(Adt9, Option<Adt23>),
}

#[derive(Clone)]
pub enum Adt23 {
    C0(Adt15, Option<Adt15>),
}

#[derive(Clone)]
pub enum Adt24 {
    C0(Adt1, Option<Adt25>, std::rc::Rc<Option<Adt26>>),
}

#[derive(Clone)]
pub enum Adt25 {
    C0(List<Adt15>),
}

#[derive(Clone)]
pub enum Adt26 {
    C0(std::rc::Rc<Adt15>, std::rc::Rc<Adt9>),
}

#[derive(Clone)]
pub enum Adt27 {
    C0(bool, std::rc::Rc<Option<Adt15>>),
}

#[derive(Clone)]
pub enum Adt28 {
    C0(std::rc::Rc<Adt15>, List<Adt15>),
}

#[derive(Clone)]
pub enum Adt29 {
    C0(std::rc::Rc<Adt15>, Adt5),
}

#[derive(Clone)]
pub enum Adt30 {
    C0(List<Adt9>, std::rc::Rc<Adt15>, bool),
}

#[derive(Clone)]
pub enum Adt31 {
    C0(std::rc::Rc<Adt15>, Adt8),
}

#[derive(Clone)]
pub enum Adt32 {
    C0(bool, std::rc::Rc<Adt9>, std::rc::Rc<Adt15>, Adt20),
}

#[derive(Clone)]
pub enum Adt33 {
    C0(std::rc::Rc<Adt15>, Adt20, std::rc::Rc<Option<Adt15>>),
}

#[derive(Clone)]
pub enum Adt34 {
    C0(std::rc::Rc<Adt15>, std::rc::Rc<Adt15>),
}

#[derive(Clone)]
pub enum Adt35 {
    C0(std::rc::Rc<Adt9>, std::rc::Rc<Adt15>),
}

#[derive(Clone)]
pub enum Adt36 {
    C0(bool, Adt20),
}

#[derive(Clone)]
pub enum Adt37 {
    C0(std::rc::Rc<Adt15>, List<Adt38>),
}

#[derive(Clone)]
pub enum Adt38 {
    C0(Adt9, Adt15),
}

#[derive(Clone)]
pub enum Adt39 {
    C0(std::rc::Rc<Adt15>, Str, Option<List<Adt4>>, List<Adt15>),
}

#[derive(Clone)]
pub enum Adt40 {
    C0(bool, std::rc::Rc<Adt15>),
}

#[derive(Clone)]
pub enum Adt41 {
    C0(bool, Adt1, List<Adt42>, std::rc::Rc<Option<Adt15>>),
}

#[derive(Clone)]
pub enum Adt42 {
    C0(Adt8, Adt15),
}

#[derive(Clone)]
pub enum Adt43 {
    C0(Adt44, std::rc::Rc<Adt15>),
}

#[derive(Clone)]
pub enum Adt44 {
    C0,
    C1,
    C2,
}

#[derive(Clone)]
pub enum Adt45 {
    C0(bool, std::rc::Rc<Adt15>, Adt20),
}

#[derive(Clone)]
pub enum Adt46 {
    C0(Str, List<Adt15>, List<Adt20>),
}

#[derive(Clone)]
pub enum Adt47 {
    C0(std::rc::Rc<Adt9>, std::rc::Rc<Adt15>),
}

#[derive(Clone)]
pub enum Adt48 {
    C0(Adt9),
    C1,
    C2(List<Adt9>),
}

#[derive(Clone)]
pub enum Adt49 {
    C0(u64, Str),
}

#[derive(Clone)]
pub enum Adt50 {
    C0(Str, bool, bool),
}

#[derive(Clone)]
pub enum Adt51 {
    C0(Str, bool),
}

pub fn f0(v0: Adt0, v1: List<Str>) -> R<Option<List<Adt48>>> {
    let v2: List<Adt48> = List::<Adt48>::nil();
    let m1 = f1(v1, v2, v0.clone())?;
    match m1 {
        Ok(v3) => {
            let v4: u64 = {
                let a2 = f2(v3.clone());
                length_list(a2)
            };
            let a4 = v4;
            let a5 = {
                let a3 = f3(v0);
                length_list(a3)
            };
            if nat_eq(a4, a5) {
                Ok(Some(v3))
            } else {
                Ok(None::<List<Adt48>>)
            }
        }
        Err(v5) => {
            Ok(v5)
        }
    }
}

pub fn f1(p13: List<Str>, p14: List<Adt48>, v2: Adt0) -> R<Result<List<Adt48>, Option<List<Adt48>>>> {
    let m12 = (p13, p14);
    {
        let mut m12 = m12;
        loop {
            let step = {
                let (v0, v1) = m12;
                let m6 = v0;
                match m6.uncons() {
                    None => {
                        Err::<(List<Str>, List<Adt48>), R<Result<List<Adt48>, Option<List<Adt48>>>>>(Ok(Ok::<List<Adt48>, Option<List<Adt48>>>(v1)))
                    }
                    Some((v3, v4)) => {
                        let v5: Option<Adt7> = f4(f3(v2.clone()), v3)?;
                        let m7 = v5;
                        match m7 {
                            None => {
                                if f5(v2.clone()) {
                                    let v6: List<Adt48> = {
                                        let a8 = v1;
                                        let a9 = List::cons(Adt48::C1, List::<Adt48>::nil());
                                        append_list(a8, a9)
                                    };
                                    Ok::<(List<Str>, List<Adt48>), R<Result<List<Adt48>, Option<List<Adt48>>>>>((v4, v6))
                                } else {
                                    Err::<(List<Str>, List<Adt48>), R<Result<List<Adt48>, Option<List<Adt48>>>>>(Ok(Err::<List<Adt48>, Option<List<Adt48>>>(None::<List<Adt48>>)))
                                }
                            }
                            Some(v7) => {
                                let v8: List<Adt48> = {
                                    let a10 = v1;
                                    let a11 = List::cons(Adt48::C0(f6(v7)), List::<Adt48>::nil());
                                    append_list(a10, a11)
                                };
                                Ok::<(List<Str>, List<Adt48>), R<Result<List<Adt48>, Option<List<Adt48>>>>>((v4, v8))
                            }
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m12 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f2(v0: List<Adt48>) -> List<Adt48> {
    let m15 = v0;
    match m15.uncons() {
        None => {
            List::<Adt48>::nil()
        }
        Some((v1, v2)) => {
            if f7(v1.clone()) {
                List::cons(v1, f2(v2))
            } else {
                f2(v2)
            }
        }
    }
}

pub fn f3(v0: Adt0) -> List<Adt7> {
    let m16 = v0;
    match m16 {
        Adt0::C0(_, v2, _) => {
            v2
        }
    }
}

pub fn f4(p19: List<Adt7>, v1: Str) -> R<Option<Adt7>> {
    let m18 = p19;
    {
        let mut m18 = m18;
        loop {
            let step = {
                let v0 = m18;
                let m17 = v0;
                match m17.uncons() {
                    None => {
                        Err::<List<Adt7>, R<Option<Adt7>>>(Ok(None::<Adt7>))
                    }
                    Some((v2, v3)) => {
                        if f8(v2.clone(), v1.clone())? {
                            Err::<List<Adt7>, R<Option<Adt7>>>(Ok(Some(v2)))
                        } else {
                            Ok::<List<Adt7>, R<Option<Adt7>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m18 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f5(v0: Adt0) -> bool {
    let m20 = v0;
    match m20 {
        Adt0::C0(_, _, v3) => {
            v3
        }
    }
}

pub fn f6(v0: Adt7) -> Adt9 {
    let m21 = v0;
    match m21 {
        Adt7::C0(_, v2) => {
            v2
        }
    }
}

pub fn f7(v0: Adt48) -> bool {
    let m22 = v0;
    match m22 {
        Adt48::C0(_) => {
            true
        }
        Adt48::C1 => {
            false
        }
        Adt48::C2(_) => {
            false
        }
    }
}

pub fn f8(v0: Adt7, v1: Str) -> R<bool> {
    let m23 = f9(v0);
    match m23 {
        Adt8::C0(v2) => {
            let a24 = f10(v2)?;
            let a25 = v1;
            Ok(equal(a24, a25))
        }
        Adt8::C1(_) => {
            Ok(false)
        }
    }
}

pub fn f9(v0: Adt7) -> Adt8 {
    let m26 = v0;
    match m26 {
        Adt7::C0(v1, _) => {
            v1
        }
    }
}

pub fn f10(v0: Str) -> R<Str> {
    let v1: Str = f11(List::cons(f12(v0, Str::lit("r#"))?, List::<Str>::nil()))?;
    let a27 = v1.clone();
    let a28 = Str::lit("");
    if if equal(a27, a28) {
        true
    } else {
        f13(f14(), v1.clone())
    } {
        let a29 = List::cons(v1, List::cons(Str::lit("Value"), List::<Str>::nil()));
        let a30 = Str::lit("");
        Ok(join(a29, a30))
    } else {
        Ok(v1)
    }
}

pub fn f11(v0: List<Str>) -> R<Str> {
    let a31 = f15(f16(f17(f18(v0)?)?, 0u64)?)?;
    let a32 = Str::lit("");
    Ok(join(a31, a32))
}

pub fn f12(v0: Str, v1: Str) -> R<Str> {
    let v2: Bytes = {
        let a33 = v0.clone();
        utf8_encode(a33)
    };
    let v3: Bytes = {
        let a34 = v1;
        utf8_encode(a34)
    };
    let a36 = {
        let a35 = v3.clone();
        length_bytes(a35)
    };
    let a37 = 0u64;
    if nat_eq(a36, a37) {
        Ok(v0)
    } else {
        Ok(f19(v2.clone(), f20({
            let a38 = v2.clone();
            length_bytes(a38)
        }, v2.clone(), v3, 0u64)?, {
            let a39 = v2;
            length_bytes(a39)
        }))
    }
}

pub fn f13(p44: List<Str>, v1: Str) -> bool {
    let m43 = p44;
    {
        let mut m43 = m43;
        loop {
            let step = {
                let v0 = m43;
                let m40 = v0;
                match m40.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a41 = v2;
                        let a42 = v1.clone();
                        if equal(a41, a42) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m43 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f14() -> List<Str> {
    let a73 = {
        let a57 = {
            let a49 = {
                let a45 = List::cons(Str::lit("at"), List::cons(Str::lit("by"), List::cons(Str::lit("calc"), List::cons(Str::lit("def"), List::cons(Str::lit("do"), List::cons(Str::lit("else"), List::<Str>::nil()))))));
                let a46 = List::cons(Str::lit("end"), List::cons(Str::lit("example"), List::cons(Str::lit("fun"), List::cons(Str::lit("have"), List::cons(Str::lit("if"), List::cons(Str::lit("in"), List::<Str>::nil()))))));
                append_list(a45, a46)
            };
            let a50 = {
                let a47 = List::cons(Str::lit("instance"), List::cons(Str::lit("let"), List::cons(Str::lit("match"), List::cons(Str::lit("mut"), List::cons(Str::lit("nomatch"), List::cons(Str::lit("open"), List::<Str>::nil()))))));
                let a48 = List::cons(Str::lit("prop"), List::cons(Str::lit("return"), List::cons(Str::lit("show"), List::cons(Str::lit("then"), List::cons(Str::lit("theorem"), List::cons(Str::lit("type"), List::cons(Str::lit("where"), List::<Str>::nil())))))));
                append_list(a47, a48)
            };
            append_list(a49, a50)
        };
        let a58 = {
            let a55 = {
                let a51 = List::cons(Str::lit("with"), List::cons(Str::lit("from"), List::cons(Str::lit("for"), List::cons(Str::lit("try"), List::cons(Str::lit("catch"), List::cons(Str::lit("break"), List::<Str>::nil()))))));
                let a52 = List::cons(Str::lit("continue"), List::cons(Str::lit("unless"), List::cons(Str::lit("structure"), List::cons(Str::lit("inductive"), List::cons(Str::lit("class"), List::cons(Str::lit("namespace"), List::cons(Str::lit("section"), List::<Str>::nil())))))));
                append_list(a51, a52)
            };
            let a56 = {
                let a53 = List::cons(Str::lit("local"), List::cons(Str::lit("private"), List::cons(Str::lit("protected"), List::cons(Str::lit("partial"), List::cons(Str::lit("unsafe"), List::cons(Str::lit("macro"), List::<Str>::nil()))))));
                let a54 = List::cons(Str::lit("syntax"), List::cons(Str::lit("import"), List::cons(Str::lit("export"), List::cons(Str::lit("axiom"), List::cons(Str::lit("abbrev"), List::cons(Str::lit("opaque"), List::cons(Str::lit("mutual"), List::<Str>::nil())))))));
                append_list(a53, a54)
            };
            append_list(a55, a56)
        };
        append_list(a57, a58)
    };
    let a74 = {
        let a71 = {
            let a63 = {
                let a59 = List::cons(Str::lit("variable"), List::cons(Str::lit("universe"), List::cons(Str::lit("deriving"), List::cons(Str::lit("extends"), List::cons(Str::lit("suffices"), List::cons(Str::lit("obtain"), List::<Str>::nil()))))));
                let a60 = List::cons(Str::lit("record"), List::cons(Str::lit("ctor"), List::cons(Str::lit("prim"), List::cons(Str::lit("project"), List::cons(Str::lit("cons"), List::cons(Str::lit("nil"), List::<Str>::nil()))))));
                append_list(a59, a60)
            };
            let a64 = {
                let a61 = List::cons(Str::lit("not"), List::cons(Str::lit("true"), List::cons(Str::lit("false"), List::cons(Str::lit("forall"), List::cons(Str::lit("first"), List::cons(Str::lit("second"), List::<Str>::nil()))))));
                let a62 = List::cons(Str::lit("apply"), List::cons(Str::lit("ref"), List::cons(Str::lit("checked"), List::cons(Str::lit("self"), List::cons(Str::lit("noncomputable"), List::cons(Str::lit("termination_by"), List::cons(Str::lit("and_congr"), List::<Str>::nil())))))));
                append_list(a61, a62)
            };
            append_list(a63, a64)
        };
        let a72 = {
            let a69 = {
                let a65 = List::cons(Str::lit("and_true"), List::cons(Str::lit("beq_iff_eq"), List::cons(Str::lit("congr"), List::cons(Str::lit("decide"), List::cons(Str::lit("decide_eq_false_iff_not"), List::cons(Str::lit("decide_eq_true_eq"), List::<Str>::nil()))))));
                let a66 = List::cons(Str::lit("id"), List::cons(Str::lit("reduceIte"), List::cons(Str::lit("rfl"), List::cons(Str::lit("true_and"), List::cons(Str::lit("isFalse"), List::cons(Str::lit("isTrue"), List::cons(Str::lit("none"), List::<Str>::nil())))))));
                append_list(a65, a66)
            };
            let a70 = {
                let a67 = List::cons(Str::lit("some"), List::cons(Str::lit("below"), List::cons(Str::lit("brecOn"), List::cons(Str::lit("casesOn"), List::cons(Str::lit("ctorElim"), List::cons(Str::lit("ctorElimType"), List::<Str>::nil()))))));
                let a68 = List::cons(Str::lit("ctorIdx"), List::cons(Str::lit("mk"), List::cons(Str::lit("noConfusion"), List::cons(Str::lit("noConfusionType"), List::cons(Str::lit("rec"), List::cons(Str::lit("recOn"), List::cons(Str::lit("toCtorIdx"), List::<Str>::nil())))))));
                append_list(a67, a68)
            };
            append_list(a69, a70)
        };
        append_list(a71, a72)
    };
    append_list(a73, a74)
}

pub fn f15(v0: List<Adt49>) -> R<List<Str>> {
    let m75 = v0;
    match m75.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt49::C0(h76, _) => {
                    h76
                }
            };
            let v4: Str = match v1 {
                Adt49::C0(_, h77) => {
                    h77
                }
            };
            Ok(List::cons({
                let a78 = v3;
                let a79 = 0u64;
                if nat_eq(a78, a79) {
                    f21(v4)?
                } else {
                    v4
                }
            }, f15(v2)?))
        }
    }
}

pub fn f16(v0: List<Str>, v1: u64) -> R<List<Adt49>> {
    let m80 = v0;
    match m80.uncons() {
        None => {
            Ok(List::<Adt49>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt49::C0(v1, v2), f16(v3, {
                let a81 = v1;
                let a82 = 1u64;
                nat_add(a81, a82)?
            })?))
        }
    }
}

pub fn f17(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a83 = v0;
        utf8_encode(a83)
    };
    f22({
        let a84 = v1.clone();
        length_bytes(a84)
    }, v1, 0u64)
}

pub fn f18(v0: List<Str>) -> R<Str> {
    let v1: Str = Str::lit("");
    let v2: Str = f23(v0, v1)?;
    if f24(v2.clone())? {
        let a85 = List::cons(Str::lit("N"), List::cons(v2, List::<Str>::nil()));
        let a86 = Str::lit("");
        Ok(join(a85, a86))
    } else {
        Ok(v2)
    }
}

pub fn f19(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m92 = {
        let a89 = v0;
        let a90 = v1;
        let a91 = {
            let a87 = v2;
            let a88 = v1;
            nat_sub(a87, a88)
        };
        slice_bytes(a89, a90, a91)
    };
    match m92 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m94 = {
                let a93 = v3;
                utf8_decode(a93)
            };
            match m94 {
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

pub fn f20(p100: u64, v1: Bytes, v2: Bytes, p101: u64) -> R<u64> {
    let m99 = (p100, p101);
    {
        let mut m99 = m99;
        loop {
            let step = {
                let (v0, v3) = m99;
                let m95 = v0;
                if m95 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v3))
                } else {
                    let v4 = m95 - 1;
                    if f25(v1.clone(), v3, v2.clone()) {
                        Ok::<(u64, u64), R<u64>>((v4, {
                            let a97 = v3;
                            let a98 = {
                                let a96 = v2.clone();
                                length_bytes(a96)
                            };
                            nat_add(a97, a98)?
                        }))
                    } else {
                        Err::<(u64, u64), R<u64>>(Ok(v3))
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

pub fn f21(v0: Str) -> R<Str> {
    f26(v0, 65u64, Bytes::lit(&[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a]))
}

pub fn f22(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m102 = v0;
    if m102 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m102 - 1;
        let a104 = {
            let a103 = v1.clone();
            length_bytes(a103)
        };
        let a105 = v2;
        if nat_le(a104, a105) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f27(f28(v1.clone(), v2)?);
            Ok(List::cons(f19(v1.clone(), v2, {
                let a106 = v2;
                let a107 = v4;
                nat_add(a106, a107)?
            }), f22(v3, v1, {
                let a108 = v2;
                let a109 = v4;
                nat_add(a108, a109)?
            })?))
        }
    }
}

pub fn f23(p115: List<Str>, p116: Str) -> R<Str> {
    let m114 = (p115, p116);
    {
        let mut m114 = m114;
        loop {
            let step = {
                let (v0, v1) = m114;
                let m110 = v0;
                match m110.uncons() {
                    None => {
                        Err::<(List<Str>, Str), R<Str>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: bool = true;
                        let v5: bool = false;
                        let v6: Adt50 = f29(f17(v2)?, Adt50::C0(v1, v5, v4))?;
                        let v7: Str = match v6.clone() {
                            Adt50::C0(h111, _, _) => {
                                h111
                            }
                        };
                        let _: bool = match v6.clone() {
                            Adt50::C0(_, h112, _) => {
                                h112
                            }
                        };
                        let _: bool = match v6 {
                            Adt50::C0(_, _, h113) => {
                                h113
                            }
                        };
                        Ok::<(List<Str>, Str), R<Str>>((v3, v7))
                    }
                }
            };
            match step {
                Ok(next) => { m114 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f24(v0: Str) -> R<bool> {
    let m119 = {
        let a117 = f17(v0)?;
        let a118 = 0u64;
        index_list(a117, a118)
    };
    match m119 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            f30(v1)
        }
    }
}

pub fn f25(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m124 = {
        let a121 = v0;
        let a122 = v1;
        let a123 = {
            let a120 = v2.clone();
            length_bytes(a120)
        };
        slice_bytes(a121, a122, a123)
    };
    match m124 {
        None => {
            false
        }
        Some(v3) => {
            let a125 = v3;
            let a126 = v2;
            equal(a125, a126)
        }
    }
}

pub fn f26(v0: Str, v1: u64, v2: Bytes) -> R<Str> {
    let v3: Bytes = {
        let a127 = v0.clone();
        utf8_encode(a127)
    };
    let m131 = {
        let a130 = f31({
            let a128 = v3.clone();
            length_bytes(a128)
        }, v3, 0u64, v1, v2, {
            let a129 = Str::lit("");
            utf8_encode(a129)
        })?;
        utf8_decode(a130)
    };
    match m131 {
        None => {
            Ok(v0)
        }
        Some(v4) => {
            Ok(v4)
        }
    }
}

pub fn f27(v0: u64) -> u64 {
    let a132 = v0;
    let a133 = 128u64;
    if nat_lt(a132, a133) {
        1u64
    } else {
        let a134 = v0;
        let a135 = 224u64;
        if nat_lt(a134, a135) {
            2u64
        } else {
            let a136 = v0;
            let a137 = 240u64;
            if nat_lt(a136, a137) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f28(v0: Bytes, v1: u64) -> R<u64> {
    let m140 = {
        let a138 = v0;
        let a139 = v1;
        index_bytes(a138, a139)
    };
    match m140 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a185 = {
                let a161 = {
                    let a149 = {
                        let a143 = {
                            let a141 = v3;
                            let a142 = 128u8;
                            fixed_u8::bit_and(a141, a142)
                        };
                        let a144 = 128u8;
                        if equal(a143, a144) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a150 = {
                        let a147 = {
                            let a145 = v3;
                            let a146 = 64u8;
                            fixed_u8::bit_and(a145, a146)
                        };
                        let a148 = 64u8;
                        if equal(a147, a148) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a149, a150)?
                };
                let a162 = {
                    let a159 = {
                        let a153 = {
                            let a151 = v3;
                            let a152 = 32u8;
                            fixed_u8::bit_and(a151, a152)
                        };
                        let a154 = 32u8;
                        if equal(a153, a154) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a160 = {
                        let a157 = {
                            let a155 = v3;
                            let a156 = 16u8;
                            fixed_u8::bit_and(a155, a156)
                        };
                        let a158 = 16u8;
                        if equal(a157, a158) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a159, a160)?
                };
                nat_add(a161, a162)?
            };
            let a186 = {
                let a183 = {
                    let a171 = {
                        let a165 = {
                            let a163 = v3;
                            let a164 = 8u8;
                            fixed_u8::bit_and(a163, a164)
                        };
                        let a166 = 8u8;
                        if equal(a165, a166) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a172 = {
                        let a169 = {
                            let a167 = v3;
                            let a168 = 4u8;
                            fixed_u8::bit_and(a167, a168)
                        };
                        let a170 = 4u8;
                        if equal(a169, a170) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a171, a172)?
                };
                let a184 = {
                    let a181 = {
                        let a175 = {
                            let a173 = v3;
                            let a174 = 2u8;
                            fixed_u8::bit_and(a173, a174)
                        };
                        let a176 = 2u8;
                        if equal(a175, a176) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a182 = {
                        let a179 = {
                            let a177 = v3;
                            let a178 = 1u8;
                            fixed_u8::bit_and(a177, a178)
                        };
                        let a180 = 1u8;
                        if equal(a179, a180) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a181, a182)?
                };
                nat_add(a183, a184)?
            };
            nat_add(a185, a186)
        }
    }
}

pub fn f29(p202: List<Str>, p203: Adt50) -> R<Adt50> {
    let m201 = (p202, p203);
    {
        let mut m201 = m201;
        loop {
            let step = {
                let (v0, v1) = m201;
                let m187 = v0;
                match m187.uncons() {
                    None => {
                        Err::<(List<Str>, Adt50), R<Adt50>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: Str = match v1.clone() {
                            Adt50::C0(h188, _, _) => {
                                h188
                            }
                        };
                        let v5: bool = match v1.clone() {
                            Adt50::C0(_, h189, _) => {
                                h189
                            }
                        };
                        let v6: bool = match v1 {
                            Adt50::C0(_, _, h190) => {
                                h190
                            }
                        };
                        let a191 = f32(v2.clone())?;
                        if bool_not(a191) {
                            let v7: bool = true;
                            let v8: bool = false;
                            Ok::<(List<Str>, Adt50), R<Adt50>>((v3, Adt50::C0(v4, v8, v7)))
                        } else {
                            let v14: Adt51 = if v6 {
                                let v9: Str = {
                                    let a192 = List::cons(v4, List::cons(f33(v2.clone())?, List::<Str>::nil()));
                                    let a193 = Str::lit("");
                                    join(a192, a193)
                                };
                                let v10: bool = false;
                                Adt51::C0(v9, v10)
                            } else {
                                let v13: Str = if if f34(v2.clone())? {
                                    let a194 = v5;
                                    bool_not(a194)
                                } else {
                                    false
                                } {
                                    let v11: Str = {
                                        let a195 = List::cons(v4, List::cons(f21(v2.clone())?, List::<Str>::nil()));
                                        let a196 = Str::lit("");
                                        join(a195, a196)
                                    };
                                    v11
                                } else {
                                    let v12: Str = {
                                        let a197 = List::cons(v4, List::cons(v2.clone(), List::<Str>::nil()));
                                        let a198 = Str::lit("");
                                        join(a197, a198)
                                    };
                                    v12
                                };
                                Adt51::C0(v13, v6)
                            };
                            let v15: Str = match v14.clone() {
                                Adt51::C0(h199, _) => {
                                    h199
                                }
                            };
                            let v16: bool = match v14 {
                                Adt51::C0(_, h200) => {
                                    h200
                                }
                            };
                            let v17: bool = if f35(v2.clone())? {
                                true
                            } else {
                                f30(v2)?
                            };
                            Ok::<(List<Str>, Adt50), R<Adt50>>((v3, Adt50::C0(v15, v17, v16)))
                        }
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

pub fn f30(v0: Str) -> R<bool> {
    let v1: u64 = f36(v0)?;
    let a204 = 48u64;
    let a205 = v1;
    if nat_le(a204, a205) {
        let a206 = v1;
        let a207 = 57u64;
        Ok(nat_le(a206, a207))
    } else {
        Ok(false)
    }
}

pub fn f31(p230: u64, v1: Bytes, p231: u64, v3: u64, v4: Bytes, p232: Bytes) -> R<Bytes> {
    let m229 = (p230, (p231, p232));
    {
        let mut m229 = m229;
        loop {
            let step = {
                let (v0, (v2, v5)) = m229;
                let m208 = v0;
                if m208 == 0 {
                    Err::<(u64, (u64, Bytes)), R<Bytes>>(Ok(v5))
                } else {
                    let v6 = m208 - 1;
                    let m212 = {
                        let a209 = v1.clone();
                        let a210 = v2;
                        let a211 = 1u64;
                        slice_bytes(a209, a210, a211)
                    };
                    match m212 {
                        None => {
                            Err::<(u64, (u64, Bytes)), R<Bytes>>(Ok(v5))
                        }
                        Some(v7) => {
                            let v8: u64 = f28(v1.clone(), v2)?;
                            let v10: Bytes = {
                                let a213 = v3;
                                let a214 = v8;
                                if if nat_le(a213, a214) {
                                    let a217 = v8;
                                    let a218 = {
                                        let a215 = v3;
                                        let a216 = 26u64;
                                        nat_add(a215, a216)?
                                    };
                                    nat_lt(a217, a218)
                                } else {
                                    false
                                } {
                                    let m224 = {
                                        let a221 = v4.clone();
                                        let a222 = {
                                            let a219 = v8;
                                            let a220 = v3;
                                            nat_sub(a219, a220)
                                        };
                                        let a223 = 1u64;
                                        slice_bytes(a221, a222, a223)
                                    };
                                    match m224 {
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
                                let a225 = v2;
                                let a226 = 1u64;
                                nat_add(a225, a226)?
                            }, {
                                let a227 = v5;
                                let a228 = v10;
                                append_bytes(a227, a228)
                            })))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m229 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f32(v0: Str) -> R<bool> {
    if f37(v0.clone())? {
        Ok(true)
    } else {
        f30(v0)
    }
}

pub fn f33(v0: Str) -> R<Str> {
    f26(v0, 97u64, Bytes::lit(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a]))
}

pub fn f34(v0: Str) -> R<bool> {
    let v1: u64 = f36(v0)?;
    let a233 = 65u64;
    let a234 = v1;
    if nat_le(a233, a234) {
        let a235 = v1;
        let a236 = 90u64;
        Ok(nat_le(a235, a236))
    } else {
        Ok(false)
    }
}

pub fn f35(v0: Str) -> R<bool> {
    let v1: u64 = f36(v0)?;
    let a237 = 97u64;
    let a238 = v1;
    if nat_le(a237, a238) {
        let a239 = v1;
        let a240 = 122u64;
        Ok(nat_le(a239, a240))
    } else {
        Ok(false)
    }
}

pub fn f36(v0: Str) -> R<u64> {
    f28({
        let a241 = v0;
        utf8_encode(a241)
    }, 0u64)
}

pub fn f37(v0: Str) -> R<bool> {
    if f34(v0.clone())? {
        Ok(true)
    } else {
        f35(v0)
    }
}

pub fn run(p0: Adt0, p1: List<Str>) -> R<Option<List<Adt48>>> {
    f0(p0, p1)
}
