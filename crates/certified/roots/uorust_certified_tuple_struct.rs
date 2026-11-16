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
    C0(Adt7, List<Adt19>, List<Adt21>, List<Str>),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(List<Adt8>, List<Adt9>, List<Adt12>, List<Adt13>, List<List<Str>>, List<Adt8>, List<Adt8>, bool, List<Adt14>, List<Adt15>, List<Adt16>, List<Adt18>, List<Adt12>),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt9 {
    C0(Str, List<(Str, Adt10)>),
}

#[derive(Clone)]
pub enum Adt10 {
    C0,
    C1,
    C2(Option<u64>),
    C3(u64),
    C4(Adt11),
    C5(Option<u64>),
    C6(Option<u64>),
    C7,
    C8,
    C9,
    C10,
    C11,
    C12(Str),
    C13(std::rc::Rc<Adt10>),
    C14(std::rc::Rc<Adt10>, std::rc::Rc<Adt10>),
    C15(std::rc::Rc<Adt10>),
    C16(List<Adt10>),
    C17(List<Adt10>),
    C18(std::rc::Rc<Adt10>, std::rc::Rc<Adt10>, bool),
    C19(std::rc::Rc<Adt10>, bool),
    C20(List<Adt10>, std::rc::Rc<Adt10>),
}

#[derive(Clone)]
pub enum Adt11 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt12 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt13 {
    C0(Str, List<(Str, List<(Option<Str>, Adt10)>)>),
}

#[derive(Clone)]
pub enum Adt14 {
    C0(Str, Adt11),
}

#[derive(Clone)]
pub enum Adt15 {
    C0(Str, Adt12),
}

#[derive(Clone)]
pub enum Adt16 {
    C0(Str, List<Adt17>),
}

#[derive(Clone)]
pub enum Adt17 {
    C0(u64, Str),
}

#[derive(Clone)]
pub enum Adt18 {
    C0(Str, List<i64>),
}

#[derive(Clone)]
pub enum Adt19 {
    C0(Str, Adt20),
}

#[derive(Clone)]
pub enum Adt20 {
    C0(Str, List<Adt10>, Adt10),
}

#[derive(Clone)]
pub enum Adt21 {
    C0(u64, List<Str>),
}

#[derive(Clone)]
pub enum Adt22 {
    C0(Str, List<Str>, Adt10, Str, List<Adt23>, List<Adt12>, bool, Option<List<Str>>, List<Str>, List<Adt12>, List<Adt24>, Option<Str>, List<Adt12>, List<Adt68>, List<Adt68>, Option<Adt10>, Option<Str>),
}

#[derive(Clone)]
pub enum Adt23 {
    C0(Str, (Str, Adt10)),
}

#[derive(Clone)]
pub enum Adt24 {
    C0(Str, Adt25),
}

#[derive(Clone)]
pub enum Adt25 {
    C0(List<Adt26>, Adt35, List<Adt23>),
}

#[derive(Clone)]
pub enum Adt26 {
    C0(std::rc::Rc<Adt27>),
    C1,
    C2(std::rc::Rc<Adt26>),
    C3(std::rc::Rc<Adt28>),
    C4(List<Adt26>),
    C5(Adt29),
    C6(Adt30),
    C7(List<Adt26>),
    C8,
    C9(Adt33),
    C10(std::rc::Rc<Adt34>),
    C11(Adt0),
    C12(List<Adt26>),
    C13(std::rc::Rc<Adt67>),
    C14,
}

#[derive(Clone)]
pub enum Adt27 {
    C0(Str, bool, std::rc::Rc<Option<Adt26>>),
}

#[derive(Clone)]
pub enum Adt28 {
    C0(std::rc::Rc<Adt26>, Adt4),
}

#[derive(Clone)]
pub enum Adt29 {
    C0(Adt0, List<Adt26>),
}

#[derive(Clone)]
pub enum Adt30 {
    C0(Adt0, List<Adt31>, bool),
}

#[derive(Clone)]
pub enum Adt31 {
    C0(Adt32, Adt26),
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
    C20(Adt0),
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
    C0(Adt26, Option<Adt43>),
}

#[derive(Clone)]
pub enum Adt43 {
    C0(Adt35, Option<Adt35>),
}

#[derive(Clone)]
pub enum Adt44 {
    C0(Adt0, Option<Adt45>, std::rc::Rc<Option<Adt46>>),
}

#[derive(Clone)]
pub enum Adt45 {
    C0(List<Adt35>),
}

#[derive(Clone)]
pub enum Adt46 {
    C0(std::rc::Rc<Adt35>, std::rc::Rc<Adt26>),
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
    C0(std::rc::Rc<Adt35>, Adt4),
}

#[derive(Clone)]
pub enum Adt50 {
    C0(List<Adt26>, std::rc::Rc<Adt35>, bool),
}

#[derive(Clone)]
pub enum Adt51 {
    C0(std::rc::Rc<Adt35>, Adt32),
}

#[derive(Clone)]
pub enum Adt52 {
    C0(bool, std::rc::Rc<Adt26>, std::rc::Rc<Adt35>, Adt40),
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
    C0(std::rc::Rc<Adt26>, std::rc::Rc<Adt35>),
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
    C0(Adt26, Adt35),
}

#[derive(Clone)]
pub enum Adt59 {
    C0(std::rc::Rc<Adt35>, Str, Option<List<Adt3>>, List<Adt35>),
}

#[derive(Clone)]
pub enum Adt60 {
    C0(bool, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt61 {
    C0(bool, Adt0, List<Adt62>, std::rc::Rc<Option<Adt35>>),
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
    C0(std::rc::Rc<Adt26>, std::rc::Rc<Adt35>),
}

#[derive(Clone)]
pub enum Adt68 {
    C0(Str, u64),
}

#[derive(Clone)]
pub enum Adt69 {
    C0,
    C1,
    C2,
    C3,
}

#[derive(Clone)]
pub enum Adt70 {
    C0(Str, bool, bool),
}

#[derive(Clone)]
pub enum Adt71 {
    C0(Str, bool),
}

pub fn f0(v0: Adt0, v1: u64, v2: Adt6, v3: Adt22) -> R<Option<(Str, List<(Str, Adt10)>)>> {
    let m1 = f1(v0);
    match m1 {
        None => {
            Ok(None::<(Str, List<(Str, Adt10)>)>)
        }
        Some(v4) => {
            let v5: Option<Str> = f2(v3)?;
            let v6: Str = f3(v2.clone(), v5, v4)?;
            let m5 = f4({
                let m4 = match v2 {
                    Adt6::C0(h3, _, _, _) => {
                        h3
                    }
                };
                match m4 {
                    Adt7::C0(_, h2, _, _, _, _, _, _, _, _, _, _, _) => {
                        h2
                    }
                }
            }, v6.clone());
            match m5 {
                None => {
                    Ok(None::<(Str, List<(Str, Adt10)>)>)
                }
                Some(v7) => {
                    let v8: bool = f5(f6(v7.clone(), 0u64)?)?;
                    if if v8 {
                        let a7 = {
                            let a6 = v7.clone();
                            length_list(a6)
                        };
                        let a8 = v1;
                        nat_eq(a7, a8)
                    } else {
                        false
                    } {
                        Ok(Some((v6, v7)))
                    } else {
                        Ok(None::<(Str, List<(Str, Adt10)>)>)
                    }
                }
            }
        }
    }
}

pub fn f1(v0: Adt0) -> Option<Str> {
    let a12 = {
        let a10 = {
            let a9 = f7(v0.clone());
            length_list(a9)
        };
        let a11 = 1u64;
        nat_eq(a10, a11)
    };
    if bool_not(a12) {
        None::<Str>
    } else {
        let m15 = {
            let a13 = f7(v0);
            let a14 = 0u64;
            index_list(a13, a14)
        };
        match m15 {
            None => {
                None::<Str>
            }
            Some(v1) => {
                if f8(v1.clone()) {
                    Some(f9(v1))
                } else {
                    None::<Str>
                }
            }
        }
    }
}

pub fn f2(v0: Adt22) -> R<Option<Str>> {
    let m16 = f10(v0)?;
    match m16 {
        None => {
            Ok(None::<Str>)
        }
        Some(v1) => {
            let v2: Str = match v1.clone() {
                Adt12::C0(h17, _) => {
                    h17
                }
            };
            let _: Str = match v1 {
                Adt12::C0(_, h18) => {
                    h18
                }
            };
            Ok(Some(v2))
        }
    }
}

pub fn f3(v0: Adt6, v1: Option<Str>, v2: Str) -> R<Str> {
    let a19 = v2.clone();
    let a20 = Str::lit("Self");
    if equal(a19, a20) {
        let m21 = v1;
        match m21 {
            None => {
                f11(match v0 {
                    Adt6::C0(h22, _, _, _) => {
                        h22
                    }
                }, v2)
            }
            Some(v3) => {
                f11(match v0 {
                    Adt6::C0(h23, _, _, _) => {
                        h23
                    }
                }, v3)
            }
        }
    } else {
        f11(match v0 {
            Adt6::C0(h24, _, _, _) => {
                h24
            }
        }, v2)
    }
}

pub fn f4(p31: List<Adt9>, v1: Str) -> Option<List<(Str, Adt10)>> {
    let m30 = p31;
    {
        let mut m30 = m30;
        loop {
            let step = {
                let v0 = m30;
                let m25 = v0;
                match m25.uncons() {
                    None => {
                        Err::<List<Adt9>, Option<List<(Str, Adt10)>>>(None::<List<(Str, Adt10)>>)
                    }
                    Some((v2, v3)) => {
                        let a27 = match v2.clone() {
                            Adt9::C0(h26, _) => {
                                h26
                            }
                        };
                        let a28 = v1.clone();
                        if equal(a27, a28) {
                            Err::<List<Adt9>, Option<List<(Str, Adt10)>>>(Some(match v2 {
                                Adt9::C0(_, h29) => {
                                    h29
                                }
                            }))
                        } else {
                            Ok::<List<Adt9>, Option<List<(Str, Adt10)>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m30 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f5(p42: List<(u64, (Str, Adt10))>) -> R<bool> {
    let m41 = p42;
    {
        let mut m41 = m41;
        loop {
            let step = {
                let v0 = m41;
                let m32 = v0;
                match m32.uncons() {
                    None => {
                        Err::<List<(u64, (Str, Adt10))>, R<bool>>(Ok(true))
                    }
                    Some((v1, v2)) => {
                        let v3: u64 = {
                            let (h33, _) = v1;
                            h33
                        };
                        let v4: (Str, Adt10) = {
                            let (_, h34) = v1;
                            h34
                        };
                        let v5: Str = {
                            let (h35, _) = v4;
                            h35
                        };
                        let _: Adt10 = {
                            let (_, h36) = v4;
                            h36
                        };
                        let a39 = v5;
                        let a40 = f12({
                            let a37 = List::cons(f13(v3)?, List::<Str>::nil());
                            let a38 = Str::lit("");
                            join(a37, a38)
                        })?;
                        if equal(a39, a40) {
                            Ok::<List<(u64, (Str, Adt10))>, R<bool>>(v2)
                        } else {
                            Err::<List<(u64, (Str, Adt10))>, R<bool>>(Ok(false))
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

pub fn f6(v0: List<(Str, Adt10)>, v1: u64) -> R<List<(u64, (Str, Adt10))>> {
    let m43 = v0;
    match m43.uncons() {
        None => {
            Ok(List::<(u64, (Str, Adt10))>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons((v1, v2), f6(v3, {
                let a44 = v1;
                let a45 = 1u64;
                nat_add(a44, a45)?
            })?))
        }
    }
}

pub fn f7(v0: Adt0) -> List<Adt1> {
    let m46 = v0;
    match m46 {
        Adt0::C0(v1, _) => {
            v1
        }
    }
}

pub fn f8(v0: Adt1) -> bool {
    let m47 = f14(v0);
    match m47 {
        Adt2::C0 => {
            true
        }
        Adt2::C1(_) => {
            false
        }
        Adt2::C2 => {
            false
        }
    }
}

pub fn f9(v0: Adt1) -> Str {
    let m48 = v0;
    match m48 {
        Adt1::C0(v1, _) => {
            v1
        }
    }
}

pub fn f10(v0: Adt22) -> R<Option<Adt12>> {
    let m50 = f15(match v0.clone() {
        Adt22::C0(h49, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h49
        }
    }, Str::lit("::"))?;
    match m50 {
        None => {
            Ok(None::<Adt12>)
        }
        Some(v1) => {
            Ok(Some(Adt12::C0(f16(match v0.clone() {
                Adt22::C0(h51, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                    h51
                }
            }, 0u64, v1), f16(match v0.clone() {
                Adt22::C0(h52, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                    h52
                }
            }, {
                let a53 = v1;
                let a54 = f17(Str::lit("::"));
                nat_add(a53, a54)?
            }, f17(match v0 {
                Adt22::C0(h55, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                    h55
                }
            })))))
        }
    }
}

pub fn f11(v0: Adt7, v1: Str) -> R<Str> {
    let v2: Str = f18(v1)?;
    let m57 = f19(match v0 {
        Adt7::C0(_, _, h56, _, _, _, _, _, _, _, _, _, _) => {
            h56
        }
    }, v2.clone());
    match m57 {
        None => {
            Ok(v2)
        }
        Some(v3) => {
            Ok(v3)
        }
    }
}

pub fn f12(v0: Str) -> R<Str> {
    let m58 = f20(v0.clone(), 4294967295u64)?;
    match m58 {
        Ok(v1) => {
            let a59 = List::cons(Str::lit("item"), List::cons(f13(v1)?, List::<Str>::nil()));
            let a60 = Str::lit("");
            Ok(join(a59, a60))
        }
        Err(_) => {
            f21(v0)
        }
    }
}

pub fn f13(v0: u64) -> R<Str> {
    Ok(f22({
        let a61 = v0;
        let a62 = 1u64;
        nat_add(a61, a62)?
    }, v0))
}

pub fn f14(v0: Adt1) -> Adt2 {
    let m63 = v0;
    match m63 {
        Adt1::C0(_, v2) => {
            v2
        }
    }
}

pub fn f15(v0: Str, v1: Str) -> R<Option<u64>> {
    let v2: Bytes = {
        let a64 = v0;
        utf8_encode(a64)
    };
    f23({
        let a66 = {
            let a65 = v2.clone();
            length_bytes(a65)
        };
        let a67 = 1u64;
        nat_add(a66, a67)?
    }, v2, {
        let a68 = v1;
        utf8_encode(a68)
    }, 0u64)
}

pub fn f16(v0: Str, v1: u64, v2: u64) -> Str {
    f24({
        let a69 = v0;
        utf8_encode(a69)
    }, v1, v2)
}

pub fn f17(v0: Str) -> u64 {
    let a71 = {
        let a70 = v0;
        utf8_encode(a70)
    };
    length_bytes(a71)
}

pub fn f18(v0: Str) -> R<Str> {
    let m72 = f25(v0.clone())?;
    match m72 {
        None => {
            Ok(v0)
        }
        Some(v1) => {
            let v2: Str = match v1.clone() {
                Adt12::C0(h73, _) => {
                    h73
                }
            };
            let v3: Str = match v1 {
                Adt12::C0(_, h74) => {
                    h74
                }
            };
            let m75 = f26(v3, Str::lit(">"));
            match m75 {
                None => {
                    Ok(v0)
                }
                Some(v4) => {
                    if f27(f28(v4, Str::lit(","))?)? {
                        f29(v2)
                    } else {
                        Ok(v0)
                    }
                }
            }
        }
    }
}

pub fn f19(p82: List<Adt12>, v1: Str) -> Option<Str> {
    let m81 = p82;
    {
        let mut m81 = m81;
        loop {
            let step = {
                let v0 = m81;
                let m76 = v0;
                match m76.uncons() {
                    None => {
                        Err::<List<Adt12>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a78 = match v2.clone() {
                            Adt12::C0(h77, _) => {
                                h77
                            }
                        };
                        let a79 = v1.clone();
                        if equal(a78, a79) {
                            Err::<List<Adt12>, Option<Str>>(Some(match v2 {
                                Adt12::C0(_, h80) => {
                                    h80
                                }
                            }))
                        } else {
                            Ok::<List<Adt12>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m81 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f20(v0: Str, v1: u64) -> R<Result<u64, Adt69>> {
    let v2: Bytes = {
        let a83 = v0;
        utf8_encode(a83)
    };
    let v3: u64 = {
        let a84 = v2.clone();
        length_bytes(a84)
    };
    let a85 = v3;
    let a86 = 0u64;
    if nat_eq(a85, a86) {
        Ok(Err::<u64, Adt69>(Adt69::C0))
    } else {
        let a87 = f30(v2.clone(), 0u64)?;
        let a88 = 43u64;
        if nat_eq(a87, a88) {
            let a89 = v3;
            let a90 = 1u64;
            if nat_eq(a89, a90) {
                Ok(Err::<u64, Adt69>(Adt69::C1))
            } else {
                f31(v3, v2, 1u64, 0u64, v1)
            }
        } else {
            f31(v3, v2, 0u64, 0u64, v1)
        }
    }
}

pub fn f21(v0: Str) -> R<Str> {
    let v1: Str = f32(List::cons(f33(v0, Str::lit("r#"))?, List::<Str>::nil()))?;
    let a91 = v1.clone();
    let a92 = Str::lit("");
    if if equal(a91, a92) {
        true
    } else {
        f34(f35(), v1.clone())
    } {
        let a93 = List::cons(v1, List::cons(Str::lit("Value"), List::<Str>::nil()));
        let a94 = Str::lit("");
        Ok(join(a93, a94))
    } else {
        Ok(v1)
    }
}

pub fn f22(v0: u64, v1: u64) -> Str {
    let m95 = v0;
    if m95 == 0 {
        Str::lit("")
    } else {
        let v2 = m95 - 1;
        let a96 = v1;
        let a97 = 10u64;
        if nat_lt(a96, a97) {
            f36(v1)
        } else {
            let a104 = List::cons(f22(v2, {
                let a98 = v1;
                let a99 = 10u64;
                let a100 = 0u64;
                nat_quot(a98, a99, a100)
            }), List::cons(f36({
                let a101 = v1;
                let a102 = 10u64;
                let a103 = 0u64;
                nat_rem(a101, a102, a103)
            }), List::<Str>::nil()));
            let a105 = Str::lit("");
            join(a104, a105)
        }
    }
}

pub fn f23(p116: u64, v1: Bytes, v2: Bytes, p117: u64) -> R<Option<u64>> {
    let m115 = (p116, p117);
    {
        let mut m115 = m115;
        loop {
            let step = {
                let (v0, v3) = m115;
                let m106 = v0;
                if m106 == 0 {
                    Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                } else {
                    let v4 = m106 - 1;
                    let a111 = {
                        let a107 = v1.clone();
                        length_bytes(a107)
                    };
                    let a112 = {
                        let a109 = v3;
                        let a110 = {
                            let a108 = v2.clone();
                            length_bytes(a108)
                        };
                        nat_add(a109, a110)?
                    };
                    if nat_lt(a111, a112) {
                        Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                    } else {
                        if f37(v1.clone(), v3, v2.clone()) {
                            Err::<(u64, u64), R<Option<u64>>>(Ok(Some(v3)))
                        } else {
                            Ok::<(u64, u64), R<Option<u64>>>((v4, {
                                let a113 = v3;
                                let a114 = 1u64;
                                nat_add(a113, a114)?
                            }))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m115 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f24(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m123 = {
        let a120 = v0;
        let a121 = v1;
        let a122 = {
            let a118 = v2;
            let a119 = v1;
            nat_sub(a118, a119)
        };
        slice_bytes(a120, a121, a122)
    };
    match m123 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m125 = {
                let a124 = v3;
                utf8_decode(a124)
            };
            match m125 {
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

pub fn f25(v0: Str) -> R<Option<Adt12>> {
    let m126 = f15(v0.clone(), Str::lit("<"))?;
    match m126 {
        None => {
            Ok(None::<Adt12>)
        }
        Some(v1) => {
            Ok(Some(Adt12::C0(f16(v0.clone(), 0u64, v1), f16(v0.clone(), {
                let a127 = v1;
                let a128 = f17(Str::lit("<"));
                nat_add(a127, a128)?
            }, f17(v0)))))
        }
    }
}

pub fn f26(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a129 = v0.clone();
        utf8_encode(a129)
    };
    let v3: u64 = {
        let a130 = v2.clone();
        length_bytes(a130)
    };
    let v4: u64 = f17(v1.clone());
    if f38(v0, v1) {
        Some(f24(v2, 0u64, {
            let a131 = v3;
            let a132 = v4;
            nat_sub(a131, a132)
        }))
    } else {
        None::<Str>
    }
}

pub fn f27(p135: List<Str>) -> R<bool> {
    let m134 = p135;
    {
        let mut m134 = m134;
        loop {
            let step = {
                let v0 = m134;
                let m133 = v0;
                match m133.uncons() {
                    None => {
                        Err::<List<Str>, R<bool>>(Ok(true))
                    }
                    Some((v1, v2)) => {
                        if f39(v1)? {
                            Ok::<List<Str>, R<bool>>(v2)
                        } else {
                            Err::<List<Str>, R<bool>>(Ok(false))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m134 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f28(v0: Str, v1: Str) -> R<List<Str>> {
    let v2: Bytes = {
        let a136 = v0.clone();
        utf8_encode(a136)
    };
    let v3: Bytes = {
        let a137 = v1;
        utf8_encode(a137)
    };
    let a139 = {
        let a138 = v3.clone();
        length_bytes(a138)
    };
    let a140 = 0u64;
    if nat_eq(a139, a140) {
        Ok(List::cons(Str::lit(""), {
            let a141 = f40(v0)?;
            let a142 = List::cons(Str::lit(""), List::<Str>::nil());
            append_list(a141, a142)
        }))
    } else {
        f41({
            let a144 = {
                let a143 = v2.clone();
                length_bytes(a143)
            };
            let a145 = 1u64;
            nat_add(a144, a145)?
        }, v2, v3, 0u64, 0u64)
    }
}

pub fn f29(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a146 = v0;
        utf8_encode(a146)
    };
    let v2: u64 = {
        let a147 = v1.clone();
        length_bytes(a147)
    };
    Ok(f24(v1.clone(), 0u64, f42(v2, v1, v2)?))
}

pub fn f30(v0: Bytes, v1: u64) -> R<u64> {
    let m150 = {
        let a148 = v0;
        let a149 = v1;
        index_bytes(a148, a149)
    };
    match m150 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a195 = {
                let a171 = {
                    let a159 = {
                        let a153 = {
                            let a151 = v3;
                            let a152 = 128u8;
                            fixed_u8::bit_and(a151, a152)
                        };
                        let a154 = 128u8;
                        if equal(a153, a154) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a160 = {
                        let a157 = {
                            let a155 = v3;
                            let a156 = 64u8;
                            fixed_u8::bit_and(a155, a156)
                        };
                        let a158 = 64u8;
                        if equal(a157, a158) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a159, a160)?
                };
                let a172 = {
                    let a169 = {
                        let a163 = {
                            let a161 = v3;
                            let a162 = 32u8;
                            fixed_u8::bit_and(a161, a162)
                        };
                        let a164 = 32u8;
                        if equal(a163, a164) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a170 = {
                        let a167 = {
                            let a165 = v3;
                            let a166 = 16u8;
                            fixed_u8::bit_and(a165, a166)
                        };
                        let a168 = 16u8;
                        if equal(a167, a168) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a169, a170)?
                };
                nat_add(a171, a172)?
            };
            let a196 = {
                let a193 = {
                    let a181 = {
                        let a175 = {
                            let a173 = v3;
                            let a174 = 8u8;
                            fixed_u8::bit_and(a173, a174)
                        };
                        let a176 = 8u8;
                        if equal(a175, a176) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a182 = {
                        let a179 = {
                            let a177 = v3;
                            let a178 = 4u8;
                            fixed_u8::bit_and(a177, a178)
                        };
                        let a180 = 4u8;
                        if equal(a179, a180) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a181, a182)?
                };
                let a194 = {
                    let a191 = {
                        let a185 = {
                            let a183 = v3;
                            let a184 = 2u8;
                            fixed_u8::bit_and(a183, a184)
                        };
                        let a186 = 2u8;
                        if equal(a185, a186) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a192 = {
                        let a189 = {
                            let a187 = v3;
                            let a188 = 1u8;
                            fixed_u8::bit_and(a187, a188)
                        };
                        let a190 = 1u8;
                        if equal(a189, a190) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a191, a192)?
                };
                nat_add(a193, a194)?
            };
            nat_add(a195, a196)
        }
    }
}

pub fn f31(p216: u64, v1: Bytes, p217: u64, p218: u64, v4: u64) -> R<Result<u64, Adt69>> {
    let m215 = (p216, (p217, p218));
    {
        let mut m215 = m215;
        loop {
            let step = {
                let (v0, (v2, v3)) = m215;
                let m197 = v0;
                if m197 == 0 {
                    Err::<(u64, (u64, u64)), R<Result<u64, Adt69>>>(Ok(Ok::<u64, Adt69>(v3)))
                } else {
                    let v5 = m197 - 1;
                    let a199 = {
                        let a198 = v1.clone();
                        length_bytes(a198)
                    };
                    let a200 = v2;
                    if nat_le(a199, a200) {
                        Err::<(u64, (u64, u64)), R<Result<u64, Adt69>>>(Ok(Ok::<u64, Adt69>(v3)))
                    } else {
                        let v6: u64 = f30(v1.clone(), v2)?;
                        let a201 = 48u64;
                        let a202 = v6;
                        if if nat_le(a201, a202) {
                            let a203 = v6;
                            let a204 = 57u64;
                            nat_le(a203, a204)
                        } else {
                            false
                        } {
                            let v7: u64 = {
                                let a209 = {
                                    let a205 = v3;
                                    let a206 = 10u64;
                                    nat_mul(a205, a206)?
                                };
                                let a210 = {
                                    let a207 = v6;
                                    let a208 = 48u64;
                                    nat_sub(a207, a208)
                                };
                                nat_add(a209, a210)?
                            };
                            let a211 = v4;
                            let a212 = v7;
                            if nat_lt(a211, a212) {
                                Err::<(u64, (u64, u64)), R<Result<u64, Adt69>>>(Ok(Err::<u64, Adt69>(Adt69::C2)))
                            } else {
                                Ok::<(u64, (u64, u64)), R<Result<u64, Adt69>>>((v5, ({
                                    let a213 = v2;
                                    let a214 = 1u64;
                                    nat_add(a213, a214)?
                                }, v7)))
                            }
                        } else {
                            Err::<(u64, (u64, u64)), R<Result<u64, Adt69>>>(Ok(Err::<u64, Adt69>(Adt69::C1)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m215 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f32(v0: List<Str>) -> R<Str> {
    let a219 = f43(f44(f40(f45(v0)?)?, 0u64)?)?;
    let a220 = Str::lit("");
    Ok(join(a219, a220))
}

pub fn f33(v0: Str, v1: Str) -> R<Str> {
    let v2: Bytes = {
        let a221 = v0.clone();
        utf8_encode(a221)
    };
    let v3: Bytes = {
        let a222 = v1;
        utf8_encode(a222)
    };
    let a224 = {
        let a223 = v3.clone();
        length_bytes(a223)
    };
    let a225 = 0u64;
    if nat_eq(a224, a225) {
        Ok(v0)
    } else {
        Ok(f24(v2.clone(), f46({
            let a226 = v2.clone();
            length_bytes(a226)
        }, v2.clone(), v3, 0u64)?, {
            let a227 = v2;
            length_bytes(a227)
        }))
    }
}

pub fn f34(p232: List<Str>, v1: Str) -> bool {
    let m231 = p232;
    {
        let mut m231 = m231;
        loop {
            let step = {
                let v0 = m231;
                let m228 = v0;
                match m228.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a229 = v2;
                        let a230 = v1.clone();
                        if equal(a229, a230) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
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

pub fn f35() -> List<Str> {
    let a261 = {
        let a245 = {
            let a237 = {
                let a233 = List::cons(Str::lit("at"), List::cons(Str::lit("by"), List::cons(Str::lit("calc"), List::cons(Str::lit("def"), List::cons(Str::lit("do"), List::cons(Str::lit("else"), List::<Str>::nil()))))));
                let a234 = List::cons(Str::lit("end"), List::cons(Str::lit("example"), List::cons(Str::lit("fun"), List::cons(Str::lit("have"), List::cons(Str::lit("if"), List::cons(Str::lit("in"), List::<Str>::nil()))))));
                append_list(a233, a234)
            };
            let a238 = {
                let a235 = List::cons(Str::lit("instance"), List::cons(Str::lit("let"), List::cons(Str::lit("match"), List::cons(Str::lit("mut"), List::cons(Str::lit("nomatch"), List::cons(Str::lit("open"), List::<Str>::nil()))))));
                let a236 = List::cons(Str::lit("prop"), List::cons(Str::lit("return"), List::cons(Str::lit("show"), List::cons(Str::lit("then"), List::cons(Str::lit("theorem"), List::cons(Str::lit("type"), List::cons(Str::lit("where"), List::<Str>::nil())))))));
                append_list(a235, a236)
            };
            append_list(a237, a238)
        };
        let a246 = {
            let a243 = {
                let a239 = List::cons(Str::lit("with"), List::cons(Str::lit("from"), List::cons(Str::lit("for"), List::cons(Str::lit("try"), List::cons(Str::lit("catch"), List::cons(Str::lit("break"), List::<Str>::nil()))))));
                let a240 = List::cons(Str::lit("continue"), List::cons(Str::lit("unless"), List::cons(Str::lit("structure"), List::cons(Str::lit("inductive"), List::cons(Str::lit("class"), List::cons(Str::lit("namespace"), List::cons(Str::lit("section"), List::<Str>::nil())))))));
                append_list(a239, a240)
            };
            let a244 = {
                let a241 = List::cons(Str::lit("local"), List::cons(Str::lit("private"), List::cons(Str::lit("protected"), List::cons(Str::lit("partial"), List::cons(Str::lit("unsafe"), List::cons(Str::lit("macro"), List::<Str>::nil()))))));
                let a242 = List::cons(Str::lit("syntax"), List::cons(Str::lit("import"), List::cons(Str::lit("export"), List::cons(Str::lit("axiom"), List::cons(Str::lit("abbrev"), List::cons(Str::lit("opaque"), List::cons(Str::lit("mutual"), List::<Str>::nil())))))));
                append_list(a241, a242)
            };
            append_list(a243, a244)
        };
        append_list(a245, a246)
    };
    let a262 = {
        let a259 = {
            let a251 = {
                let a247 = List::cons(Str::lit("variable"), List::cons(Str::lit("universe"), List::cons(Str::lit("deriving"), List::cons(Str::lit("extends"), List::cons(Str::lit("suffices"), List::cons(Str::lit("obtain"), List::<Str>::nil()))))));
                let a248 = List::cons(Str::lit("record"), List::cons(Str::lit("ctor"), List::cons(Str::lit("prim"), List::cons(Str::lit("project"), List::cons(Str::lit("cons"), List::cons(Str::lit("nil"), List::<Str>::nil()))))));
                append_list(a247, a248)
            };
            let a252 = {
                let a249 = List::cons(Str::lit("not"), List::cons(Str::lit("true"), List::cons(Str::lit("false"), List::cons(Str::lit("forall"), List::cons(Str::lit("first"), List::cons(Str::lit("second"), List::<Str>::nil()))))));
                let a250 = List::cons(Str::lit("apply"), List::cons(Str::lit("ref"), List::cons(Str::lit("checked"), List::cons(Str::lit("self"), List::cons(Str::lit("noncomputable"), List::cons(Str::lit("termination_by"), List::cons(Str::lit("and_congr"), List::<Str>::nil())))))));
                append_list(a249, a250)
            };
            append_list(a251, a252)
        };
        let a260 = {
            let a257 = {
                let a253 = List::cons(Str::lit("and_true"), List::cons(Str::lit("beq_iff_eq"), List::cons(Str::lit("congr"), List::cons(Str::lit("decide"), List::cons(Str::lit("decide_eq_false_iff_not"), List::cons(Str::lit("decide_eq_true_eq"), List::<Str>::nil()))))));
                let a254 = List::cons(Str::lit("id"), List::cons(Str::lit("reduceIte"), List::cons(Str::lit("rfl"), List::cons(Str::lit("true_and"), List::cons(Str::lit("isFalse"), List::cons(Str::lit("isTrue"), List::cons(Str::lit("none"), List::<Str>::nil())))))));
                append_list(a253, a254)
            };
            let a258 = {
                let a255 = List::cons(Str::lit("some"), List::cons(Str::lit("below"), List::cons(Str::lit("brecOn"), List::cons(Str::lit("casesOn"), List::cons(Str::lit("ctorElim"), List::cons(Str::lit("ctorElimType"), List::<Str>::nil()))))));
                let a256 = List::cons(Str::lit("ctorIdx"), List::cons(Str::lit("mk"), List::cons(Str::lit("noConfusion"), List::cons(Str::lit("noConfusionType"), List::cons(Str::lit("rec"), List::cons(Str::lit("recOn"), List::cons(Str::lit("toCtorIdx"), List::<Str>::nil())))))));
                append_list(a255, a256)
            };
            append_list(a257, a258)
        };
        append_list(a259, a260)
    };
    append_list(a261, a262)
}

pub fn f36(v0: u64) -> Str {
    let a263 = v0;
    let a264 = 0u64;
    if nat_eq(a263, a264) {
        Str::lit("0")
    } else {
        let a265 = v0;
        let a266 = 1u64;
        if nat_eq(a265, a266) {
            Str::lit("1")
        } else {
            let a267 = v0;
            let a268 = 2u64;
            if nat_eq(a267, a268) {
                Str::lit("2")
            } else {
                let a269 = v0;
                let a270 = 3u64;
                if nat_eq(a269, a270) {
                    Str::lit("3")
                } else {
                    let a271 = v0;
                    let a272 = 4u64;
                    if nat_eq(a271, a272) {
                        Str::lit("4")
                    } else {
                        let a273 = v0;
                        let a274 = 5u64;
                        if nat_eq(a273, a274) {
                            Str::lit("5")
                        } else {
                            let a275 = v0;
                            let a276 = 6u64;
                            if nat_eq(a275, a276) {
                                Str::lit("6")
                            } else {
                                let a277 = v0;
                                let a278 = 7u64;
                                if nat_eq(a277, a278) {
                                    Str::lit("7")
                                } else {
                                    let a279 = v0;
                                    let a280 = 8u64;
                                    if nat_eq(a279, a280) {
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

pub fn f37(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m285 = {
        let a282 = v0;
        let a283 = v1;
        let a284 = {
            let a281 = v2.clone();
            length_bytes(a281)
        };
        slice_bytes(a282, a283, a284)
    };
    match m285 {
        None => {
            false
        }
        Some(v3) => {
            let a286 = v3;
            let a287 = v2;
            equal(a286, a287)
        }
    }
}

pub fn f38(v0: Str, v1: Str) -> bool {
    let v2: Bytes = {
        let a288 = v0;
        utf8_encode(a288)
    };
    let v3: Bytes = {
        let a289 = v1;
        utf8_encode(a289)
    };
    let v4: u64 = {
        let a290 = v2.clone();
        length_bytes(a290)
    };
    let v5: u64 = {
        let a291 = v3.clone();
        length_bytes(a291)
    };
    let a292 = v5;
    let a293 = v4;
    if nat_le(a292, a293) {
        f37(v2, {
            let a294 = v4;
            let a295 = v5;
            nat_sub(a294, a295)
        }, v3)
    } else {
        false
    }
}

pub fn f39(v0: Str) -> R<bool> {
    let m296 = f47(f48(v0)?, Str::lit("'"));
    match m296 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            let a297 = f49(v1, Str::lit("'"))?;
            Ok(bool_not(a297))
        }
    }
}

pub fn f40(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a298 = v0;
        utf8_encode(a298)
    };
    f50({
        let a299 = v1.clone();
        length_bytes(a299)
    }, v1, 0u64)
}

pub fn f41(v0: u64, v1: Bytes, v2: Bytes, v3: u64, v4: u64) -> R<List<Str>> {
    let m300 = v0;
    if m300 == 0 {
        Ok(List::cons(f24(v1.clone(), v3, {
            let a301 = v1;
            length_bytes(a301)
        }), List::<Str>::nil()))
    } else {
        let v5 = m300 - 1;
        let a306 = {
            let a302 = v1.clone();
            length_bytes(a302)
        };
        let a307 = {
            let a304 = v4;
            let a305 = {
                let a303 = v2.clone();
                length_bytes(a303)
            };
            nat_add(a304, a305)?
        };
        if nat_lt(a306, a307) {
            Ok(List::cons(f24(v1.clone(), v3, {
                let a308 = v1;
                length_bytes(a308)
            }), List::<Str>::nil()))
        } else {
            if f37(v1.clone(), v4, v2.clone()) {
                Ok(List::cons(f24(v1.clone(), v3, v4), f41(v5, v1, v2.clone(), {
                    let a310 = v4;
                    let a311 = {
                        let a309 = v2.clone();
                        length_bytes(a309)
                    };
                    nat_add(a310, a311)?
                }, {
                    let a313 = v4;
                    let a314 = {
                        let a312 = v2;
                        length_bytes(a312)
                    };
                    nat_add(a313, a314)?
                })?))
            } else {
                f41(v5, v1, v2, v3, {
                    let a315 = v4;
                    let a316 = 1u64;
                    nat_add(a315, a316)?
                })
            }
        }
    }
}

pub fn f42(p323: u64, v1: Bytes, p324: u64) -> R<u64> {
    let m322 = (p323, p324);
    {
        let mut m322 = m322;
        loop {
            let step = {
                let (v0, v2) = m322;
                let m317 = v0;
                if m317 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m317 - 1;
                    let v4: u64 = f51(v1.clone(), v2)?;
                    let a318 = v4;
                    let a319 = 0u64;
                    if nat_eq(a318, a319) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a320 = v2;
                            let a321 = v4;
                            nat_sub(a320, a321)
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m322 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f43(v0: List<Adt17>) -> R<List<Str>> {
    let m325 = v0;
    match m325.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt17::C0(h326, _) => {
                    h326
                }
            };
            let v4: Str = match v1 {
                Adt17::C0(_, h327) => {
                    h327
                }
            };
            Ok(List::cons({
                let a328 = v3;
                let a329 = 0u64;
                if nat_eq(a328, a329) {
                    f52(v4)?
                } else {
                    v4
                }
            }, f43(v2)?))
        }
    }
}

pub fn f44(v0: List<Str>, v1: u64) -> R<List<Adt17>> {
    let m330 = v0;
    match m330.uncons() {
        None => {
            Ok(List::<Adt17>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt17::C0(v1, v2), f44(v3, {
                let a331 = v1;
                let a332 = 1u64;
                nat_add(a331, a332)?
            })?))
        }
    }
}

pub fn f45(v0: List<Str>) -> R<Str> {
    let v1: Str = Str::lit("");
    let v2: Str = f53(v0, v1)?;
    if f54(v2.clone())? {
        let a333 = List::cons(Str::lit("N"), List::cons(v2, List::<Str>::nil()));
        let a334 = Str::lit("");
        Ok(join(a333, a334))
    } else {
        Ok(v2)
    }
}

pub fn f46(p340: u64, v1: Bytes, v2: Bytes, p341: u64) -> R<u64> {
    let m339 = (p340, p341);
    {
        let mut m339 = m339;
        loop {
            let step = {
                let (v0, v3) = m339;
                let m335 = v0;
                if m335 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v3))
                } else {
                    let v4 = m335 - 1;
                    if f37(v1.clone(), v3, v2.clone()) {
                        Ok::<(u64, u64), R<u64>>((v4, {
                            let a337 = v3;
                            let a338 = {
                                let a336 = v2.clone();
                                length_bytes(a336)
                            };
                            nat_add(a337, a338)?
                        }))
                    } else {
                        Err::<(u64, u64), R<u64>>(Ok(v3))
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

pub fn f47(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a342 = v0;
        utf8_encode(a342)
    };
    let v3: Bytes = {
        let a343 = v1;
        utf8_encode(a343)
    };
    if f37(v2.clone(), 0u64, v3.clone()) {
        Some(f24(v2.clone(), {
            let a344 = v3;
            length_bytes(a344)
        }, {
            let a345 = v2;
            length_bytes(a345)
        }))
    } else {
        None::<Str>
    }
}

pub fn f48(v0: Str) -> R<Str> {
    f29(f55(v0)?)
}

pub fn f49(v0: Str, v1: Str) -> R<bool> {
    let m346 = f15(v0, v1)?;
    match m346 {
        None => {
            Ok(false)
        }
        Some(_) => {
            Ok(true)
        }
    }
}

pub fn f50(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m347 = v0;
    if m347 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m347 - 1;
        let a349 = {
            let a348 = v1.clone();
            length_bytes(a348)
        };
        let a350 = v2;
        if nat_le(a349, a350) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f56(f30(v1.clone(), v2)?);
            Ok(List::cons(f24(v1.clone(), v2, {
                let a351 = v2;
                let a352 = v4;
                nat_add(a351, a352)?
            }), f50(v3, v1, {
                let a353 = v2;
                let a354 = v4;
                nat_add(a353, a354)?
            })?))
        }
    }
}

pub fn f51(v0: Bytes, v1: u64) -> R<u64> {
    let a355 = v1;
    let a356 = 0u64;
    if nat_eq(a355, a356) {
        Ok(0u64)
    } else {
        if f57(f30(v0.clone(), {
            let a357 = v1;
            let a358 = 1u64;
            nat_sub(a357, a358)
        })?) {
            Ok(1u64)
        } else {
            let a359 = 2u64;
            let a360 = v1;
            if if nat_le(a359, a360) {
                let a363 = f58(v0.clone(), {
                    let a361 = v1;
                    let a362 = 2u64;
                    nat_sub(a361, a362)
                })?;
                let a364 = 2u64;
                nat_eq(a363, a364)
            } else {
                false
            } {
                Ok(2u64)
            } else {
                let a365 = 3u64;
                let a366 = v1;
                if if nat_le(a365, a366) {
                    let a369 = f58(v0, {
                        let a367 = v1;
                        let a368 = 3u64;
                        nat_sub(a367, a368)
                    })?;
                    let a370 = 3u64;
                    nat_eq(a369, a370)
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

pub fn f52(v0: Str) -> R<Str> {
    f59(v0, 65u64, Bytes::lit(&[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a]))
}

pub fn f53(p376: List<Str>, p377: Str) -> R<Str> {
    let m375 = (p376, p377);
    {
        let mut m375 = m375;
        loop {
            let step = {
                let (v0, v1) = m375;
                let m371 = v0;
                match m371.uncons() {
                    None => {
                        Err::<(List<Str>, Str), R<Str>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: bool = true;
                        let v5: bool = false;
                        let v6: Adt70 = f60(f40(v2)?, Adt70::C0(v1, v5, v4))?;
                        let v7: Str = match v6.clone() {
                            Adt70::C0(h372, _, _) => {
                                h372
                            }
                        };
                        let _: bool = match v6.clone() {
                            Adt70::C0(_, h373, _) => {
                                h373
                            }
                        };
                        let _: bool = match v6 {
                            Adt70::C0(_, _, h374) => {
                                h374
                            }
                        };
                        Ok::<(List<Str>, Str), R<Str>>((v3, v7))
                    }
                }
            };
            match step {
                Ok(next) => { m375 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f54(v0: Str) -> R<bool> {
    let m380 = {
        let a378 = f40(v0)?;
        let a379 = 0u64;
        index_list(a378, a379)
    };
    match m380 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            f61(v1)
        }
    }
}

pub fn f55(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a381 = v0;
        utf8_encode(a381)
    };
    let v2: u64 = {
        let a382 = v1.clone();
        length_bytes(a382)
    };
    Ok(f24(v1.clone(), f62(v2, v1, 0u64)?, v2))
}

pub fn f56(v0: u64) -> u64 {
    let a383 = v0;
    let a384 = 128u64;
    if nat_lt(a383, a384) {
        1u64
    } else {
        let a385 = v0;
        let a386 = 224u64;
        if nat_lt(a385, a386) {
            2u64
        } else {
            let a387 = v0;
            let a388 = 240u64;
            if nat_lt(a387, a388) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f57(v0: u64) -> bool {
    let a389 = v0;
    let a390 = 32u64;
    if nat_eq(a389, a390) {
        true
    } else {
        let a391 = 9u64;
        let a392 = v0;
        if nat_le(a391, a392) {
            let a393 = v0;
            let a394 = 13u64;
            nat_le(a393, a394)
        } else {
            false
        }
    }
}

pub fn f58(v0: Bytes, v1: u64) -> R<u64> {
    let v2: u64 = f30(v0.clone(), v1)?;
    let v3: u64 = f30(v0.clone(), {
        let a395 = v1;
        let a396 = 1u64;
        nat_add(a395, a396)?
    })?;
    let v4: u64 = f30(v0, {
        let a397 = v1;
        let a398 = 2u64;
        nat_add(a397, a398)?
    })?;
    if f57(v2) {
        Ok(1u64)
    } else {
        let a399 = v2;
        let a400 = 194u64;
        if nat_eq(a399, a400) {
            let a401 = v3;
            let a402 = 133u64;
            if if nat_eq(a401, a402) {
                true
            } else {
                let a403 = v3;
                let a404 = 160u64;
                nat_eq(a403, a404)
            } {
                Ok(2u64)
            } else {
                Ok(0u64)
            }
        } else {
            let a405 = v2;
            let a406 = 225u64;
            if nat_eq(a405, a406) {
                let a407 = v3;
                let a408 = 154u64;
                if if nat_eq(a407, a408) {
                    let a409 = v4;
                    let a410 = 128u64;
                    nat_eq(a409, a410)
                } else {
                    false
                } {
                    Ok(3u64)
                } else {
                    Ok(0u64)
                }
            } else {
                let a411 = v2;
                let a412 = 226u64;
                if nat_eq(a411, a412) {
                    let a413 = v3;
                    let a414 = 128u64;
                    if nat_eq(a413, a414) {
                        let a415 = 128u64;
                        let a416 = v4;
                        if if if nat_le(a415, a416) {
                            let a417 = v4;
                            let a418 = 138u64;
                            nat_le(a417, a418)
                        } else {
                            false
                        } {
                            true
                        } else {
                            let a419 = v4;
                            let a420 = 168u64;
                            if nat_eq(a419, a420) {
                                true
                            } else {
                                let a421 = v4;
                                let a422 = 169u64;
                                if nat_eq(a421, a422) {
                                    true
                                } else {
                                    let a423 = v4;
                                    let a424 = 175u64;
                                    nat_eq(a423, a424)
                                }
                            }
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    } else {
                        let a425 = v3;
                        let a426 = 129u64;
                        if if nat_eq(a425, a426) {
                            let a427 = v4;
                            let a428 = 159u64;
                            nat_eq(a427, a428)
                        } else {
                            false
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    }
                } else {
                    let a429 = v2;
                    let a430 = 227u64;
                    if if nat_eq(a429, a430) {
                        let a431 = v3;
                        let a432 = 128u64;
                        if nat_eq(a431, a432) {
                            let a433 = v4;
                            let a434 = 128u64;
                            nat_eq(a433, a434)
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

pub fn f59(v0: Str, v1: u64, v2: Bytes) -> R<Str> {
    let v3: Bytes = {
        let a435 = v0.clone();
        utf8_encode(a435)
    };
    let m439 = {
        let a438 = f63({
            let a436 = v3.clone();
            length_bytes(a436)
        }, v3, 0u64, v1, v2, {
            let a437 = Str::lit("");
            utf8_encode(a437)
        })?;
        utf8_decode(a438)
    };
    match m439 {
        None => {
            Ok(v0)
        }
        Some(v4) => {
            Ok(v4)
        }
    }
}

pub fn f60(p455: List<Str>, p456: Adt70) -> R<Adt70> {
    let m454 = (p455, p456);
    {
        let mut m454 = m454;
        loop {
            let step = {
                let (v0, v1) = m454;
                let m440 = v0;
                match m440.uncons() {
                    None => {
                        Err::<(List<Str>, Adt70), R<Adt70>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        let v4: Str = match v1.clone() {
                            Adt70::C0(h441, _, _) => {
                                h441
                            }
                        };
                        let v5: bool = match v1.clone() {
                            Adt70::C0(_, h442, _) => {
                                h442
                            }
                        };
                        let v6: bool = match v1 {
                            Adt70::C0(_, _, h443) => {
                                h443
                            }
                        };
                        let a444 = f64(v2.clone())?;
                        if bool_not(a444) {
                            let v7: bool = true;
                            let v8: bool = false;
                            Ok::<(List<Str>, Adt70), R<Adt70>>((v3, Adt70::C0(v4, v8, v7)))
                        } else {
                            let v14: Adt71 = if v6 {
                                let v9: Str = {
                                    let a445 = List::cons(v4, List::cons(f65(v2.clone())?, List::<Str>::nil()));
                                    let a446 = Str::lit("");
                                    join(a445, a446)
                                };
                                let v10: bool = false;
                                Adt71::C0(v9, v10)
                            } else {
                                let v13: Str = if if f66(v2.clone())? {
                                    let a447 = v5;
                                    bool_not(a447)
                                } else {
                                    false
                                } {
                                    let v11: Str = {
                                        let a448 = List::cons(v4, List::cons(f52(v2.clone())?, List::<Str>::nil()));
                                        let a449 = Str::lit("");
                                        join(a448, a449)
                                    };
                                    v11
                                } else {
                                    let v12: Str = {
                                        let a450 = List::cons(v4, List::cons(v2.clone(), List::<Str>::nil()));
                                        let a451 = Str::lit("");
                                        join(a450, a451)
                                    };
                                    v12
                                };
                                Adt71::C0(v13, v6)
                            };
                            let v15: Str = match v14.clone() {
                                Adt71::C0(h452, _) => {
                                    h452
                                }
                            };
                            let v16: bool = match v14 {
                                Adt71::C0(_, h453) => {
                                    h453
                                }
                            };
                            let v17: bool = if f67(v2.clone())? {
                                true
                            } else {
                                f61(v2)?
                            };
                            Ok::<(List<Str>, Adt70), R<Adt70>>((v3, Adt70::C0(v15, v17, v16)))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m454 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f61(v0: Str) -> R<bool> {
    let v1: u64 = f68(v0)?;
    let a457 = 48u64;
    let a458 = v1;
    if nat_le(a457, a458) {
        let a459 = v1;
        let a460 = 57u64;
        Ok(nat_le(a459, a460))
    } else {
        Ok(false)
    }
}

pub fn f62(p467: u64, v1: Bytes, p468: u64) -> R<u64> {
    let m466 = (p467, p468);
    {
        let mut m466 = m466;
        loop {
            let step = {
                let (v0, v2) = m466;
                let m461 = v0;
                if m461 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m461 - 1;
                    let v4: u64 = f58(v1.clone(), v2)?;
                    let a462 = v4;
                    let a463 = 0u64;
                    if nat_eq(a462, a463) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a464 = v2;
                            let a465 = v4;
                            nat_add(a464, a465)?
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m466 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f63(p491: u64, v1: Bytes, p492: u64, v3: u64, v4: Bytes, p493: Bytes) -> R<Bytes> {
    let m490 = (p491, (p492, p493));
    {
        let mut m490 = m490;
        loop {
            let step = {
                let (v0, (v2, v5)) = m490;
                let m469 = v0;
                if m469 == 0 {
                    Err::<(u64, (u64, Bytes)), R<Bytes>>(Ok(v5))
                } else {
                    let v6 = m469 - 1;
                    let m473 = {
                        let a470 = v1.clone();
                        let a471 = v2;
                        let a472 = 1u64;
                        slice_bytes(a470, a471, a472)
                    };
                    match m473 {
                        None => {
                            Err::<(u64, (u64, Bytes)), R<Bytes>>(Ok(v5))
                        }
                        Some(v7) => {
                            let v8: u64 = f30(v1.clone(), v2)?;
                            let v10: Bytes = {
                                let a474 = v3;
                                let a475 = v8;
                                if if nat_le(a474, a475) {
                                    let a478 = v8;
                                    let a479 = {
                                        let a476 = v3;
                                        let a477 = 26u64;
                                        nat_add(a476, a477)?
                                    };
                                    nat_lt(a478, a479)
                                } else {
                                    false
                                } {
                                    let m485 = {
                                        let a482 = v4.clone();
                                        let a483 = {
                                            let a480 = v8;
                                            let a481 = v3;
                                            nat_sub(a480, a481)
                                        };
                                        let a484 = 1u64;
                                        slice_bytes(a482, a483, a484)
                                    };
                                    match m485 {
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
                                let a486 = v2;
                                let a487 = 1u64;
                                nat_add(a486, a487)?
                            }, {
                                let a488 = v5;
                                let a489 = v10;
                                append_bytes(a488, a489)
                            })))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m490 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f64(v0: Str) -> R<bool> {
    if f69(v0.clone())? {
        Ok(true)
    } else {
        f61(v0)
    }
}

pub fn f65(v0: Str) -> R<Str> {
    f59(v0, 97u64, Bytes::lit(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a]))
}

pub fn f66(v0: Str) -> R<bool> {
    let v1: u64 = f68(v0)?;
    let a494 = 65u64;
    let a495 = v1;
    if nat_le(a494, a495) {
        let a496 = v1;
        let a497 = 90u64;
        Ok(nat_le(a496, a497))
    } else {
        Ok(false)
    }
}

pub fn f67(v0: Str) -> R<bool> {
    let v1: u64 = f68(v0)?;
    let a498 = 97u64;
    let a499 = v1;
    if nat_le(a498, a499) {
        let a500 = v1;
        let a501 = 122u64;
        Ok(nat_le(a500, a501))
    } else {
        Ok(false)
    }
}

pub fn f68(v0: Str) -> R<u64> {
    f30({
        let a502 = v0;
        utf8_encode(a502)
    }, 0u64)
}

pub fn f69(v0: Str) -> R<bool> {
    if f66(v0.clone())? {
        Ok(true)
    } else {
        f67(v0)
    }
}

pub fn run(p0: Adt0, p1: u64, p2: Adt6, p3: Adt22) -> R<Option<(Str, List<(Str, Adt10)>)>> {
    f0(p0, p1, p2, p3)
}
