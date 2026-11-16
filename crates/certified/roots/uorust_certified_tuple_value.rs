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
    C0(u64, Str),
}

pub fn f0(v0: Adt0, v1: List<Str>) -> R<Str> {
    let m1 = v0;
    match m1 {
        Adt0::C0 => {
            let v2: List<Str> = f1(f2(v1, 0u64)?)?;
            let a6 = {
                let a4 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C0), List::<Str>::nil()));
                let a5 = List::cons(Str::lit(" {"), List::cons({
                    let a2 = v2;
                    let a3 = Str::lit(", ");
                    join(a2, a3)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a4, a5)
            };
            let a7 = Str::lit("");
            Ok(join(a6, a7))
        }
        Adt0::C1 => {
            let v3: List<Str> = f4(f5(v1, 0u64)?)?;
            let a12 = {
                let a10 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C1), List::<Str>::nil()));
                let a11 = List::cons(Str::lit(" {"), List::cons({
                    let a8 = v3;
                    let a9 = Str::lit(", ");
                    join(a8, a9)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a10, a11)
            };
            let a13 = Str::lit("");
            Ok(join(a12, a13))
        }
        Adt0::C2(v4) => {
            let v5: List<Str> = f6(f7(v1, 0u64)?)?;
            let a18 = {
                let a16 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C2(v4)), List::<Str>::nil()));
                let a17 = List::cons(Str::lit(" {"), List::cons({
                    let a14 = v5;
                    let a15 = Str::lit(", ");
                    join(a14, a15)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a16, a17)
            };
            let a19 = Str::lit("");
            Ok(join(a18, a19))
        }
        Adt0::C3(v6) => {
            let v7: List<Str> = f8(f9(v1, 0u64)?)?;
            let a24 = {
                let a22 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C3(v6)), List::<Str>::nil()));
                let a23 = List::cons(Str::lit(" {"), List::cons({
                    let a20 = v7;
                    let a21 = Str::lit(", ");
                    join(a20, a21)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a22, a23)
            };
            let a25 = Str::lit("");
            Ok(join(a24, a25))
        }
        Adt0::C4(v8) => {
            let v9: List<Str> = f10(f11(v1, 0u64)?)?;
            let a30 = {
                let a28 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C4(v8)), List::<Str>::nil()));
                let a29 = List::cons(Str::lit(" {"), List::cons({
                    let a26 = v9;
                    let a27 = Str::lit(", ");
                    join(a26, a27)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a28, a29)
            };
            let a31 = Str::lit("");
            Ok(join(a30, a31))
        }
        Adt0::C5(v10) => {
            let v11: List<Str> = f12(f13(v1, 0u64)?)?;
            let a36 = {
                let a34 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C5(v10)), List::<Str>::nil()));
                let a35 = List::cons(Str::lit(" {"), List::cons({
                    let a32 = v11;
                    let a33 = Str::lit(", ");
                    join(a32, a33)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a34, a35)
            };
            let a37 = Str::lit("");
            Ok(join(a36, a37))
        }
        Adt0::C6(v12) => {
            let v13: List<Str> = f14(f15(v1, 0u64)?)?;
            let a42 = {
                let a40 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C6(v12)), List::<Str>::nil()));
                let a41 = List::cons(Str::lit(" {"), List::cons({
                    let a38 = v13;
                    let a39 = Str::lit(", ");
                    join(a38, a39)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a40, a41)
            };
            let a43 = Str::lit("");
            Ok(join(a42, a43))
        }
        Adt0::C7 => {
            let v14: List<Str> = f16(f17(v1, 0u64)?)?;
            let a48 = {
                let a46 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C7), List::<Str>::nil()));
                let a47 = List::cons(Str::lit(" {"), List::cons({
                    let a44 = v14;
                    let a45 = Str::lit(", ");
                    join(a44, a45)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a46, a47)
            };
            let a49 = Str::lit("");
            Ok(join(a48, a49))
        }
        Adt0::C8 => {
            let v15: List<Str> = f18(f19(v1, 0u64)?)?;
            let a54 = {
                let a52 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C8), List::<Str>::nil()));
                let a53 = List::cons(Str::lit(" {"), List::cons({
                    let a50 = v15;
                    let a51 = Str::lit(", ");
                    join(a50, a51)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a52, a53)
            };
            let a55 = Str::lit("");
            Ok(join(a54, a55))
        }
        Adt0::C9 => {
            let v16: List<Str> = f20(f21(v1, 0u64)?)?;
            let a60 = {
                let a58 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C9), List::<Str>::nil()));
                let a59 = List::cons(Str::lit(" {"), List::cons({
                    let a56 = v16;
                    let a57 = Str::lit(", ");
                    join(a56, a57)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a58, a59)
            };
            let a61 = Str::lit("");
            Ok(join(a60, a61))
        }
        Adt0::C10 => {
            let v17: List<Str> = f22(f23(v1, 0u64)?)?;
            let a66 = {
                let a64 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C10), List::<Str>::nil()));
                let a65 = List::cons(Str::lit(" {"), List::cons({
                    let a62 = v17;
                    let a63 = Str::lit(", ");
                    join(a62, a63)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a64, a65)
            };
            let a67 = Str::lit("");
            Ok(join(a66, a67))
        }
        Adt0::C11 => {
            let v18: List<Str> = f24(f25(v1, 0u64)?)?;
            let a72 = {
                let a70 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C11), List::<Str>::nil()));
                let a71 = List::cons(Str::lit(" {"), List::cons({
                    let a68 = v18;
                    let a69 = Str::lit(", ");
                    join(a68, a69)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a70, a71)
            };
            let a73 = Str::lit("");
            Ok(join(a72, a73))
        }
        Adt0::C12(v19) => {
            let v20: List<Str> = f26(f27(v1, 0u64)?)?;
            let a78 = {
                let a76 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C12(v19)), List::<Str>::nil()));
                let a77 = List::cons(Str::lit(" {"), List::cons({
                    let a74 = v20;
                    let a75 = Str::lit(", ");
                    join(a74, a75)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a76, a77)
            };
            let a79 = Str::lit("");
            Ok(join(a78, a79))
        }
        Adt0::C13(r126) => {
            let v21 = (*r126).clone();
            let v22: List<Str> = f28(f29(v1, 0u64)?)?;
            let a84 = {
                let a82 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C13(std::rc::Rc::new(v21))), List::<Str>::nil()));
                let a83 = List::cons(Str::lit(" {"), List::cons({
                    let a80 = v22;
                    let a81 = Str::lit(", ");
                    join(a80, a81)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a82, a83)
            };
            let a85 = Str::lit("");
            Ok(join(a84, a85))
        }
        Adt0::C14(r127, r128) => {
            let v23 = (*r127).clone();
            let v24 = (*r128).clone();
            let v25: List<Str> = f30(f31(v1, 0u64)?)?;
            let a90 = {
                let a88 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C14(std::rc::Rc::new(v23), std::rc::Rc::new(v24))), List::<Str>::nil()));
                let a89 = List::cons(Str::lit(" {"), List::cons({
                    let a86 = v25;
                    let a87 = Str::lit(", ");
                    join(a86, a87)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a88, a89)
            };
            let a91 = Str::lit("");
            Ok(join(a90, a91))
        }
        Adt0::C15(r129) => {
            let v26 = (*r129).clone();
            let v27: List<Str> = f32(f33(v1, 0u64)?)?;
            let a96 = {
                let a94 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C15(std::rc::Rc::new(v26))), List::<Str>::nil()));
                let a95 = List::cons(Str::lit(" {"), List::cons({
                    let a92 = v27;
                    let a93 = Str::lit(", ");
                    join(a92, a93)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a94, a95)
            };
            let a97 = Str::lit("");
            Ok(join(a96, a97))
        }
        Adt0::C16(v28) => {
            let v29: List<Str> = f34(f35(v1, 0u64)?)?;
            let a102 = {
                let a100 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C16(v28)), List::<Str>::nil()));
                let a101 = List::cons(Str::lit(" {"), List::cons({
                    let a98 = v29;
                    let a99 = Str::lit(", ");
                    join(a98, a99)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a100, a101)
            };
            let a103 = Str::lit("");
            Ok(join(a102, a103))
        }
        Adt0::C17(_) => {
            let a106 = {
                let a104 = List::cons(Str::lit("("), List::cons(f36(v1.clone()), List::<Str>::nil()));
                let a105 = List::cons(Str::lit(", "), List::cons(f37(v1), List::cons(Str::lit(")"), List::<Str>::nil())));
                append_list(a104, a105)
            };
            let a107 = Str::lit("");
            Ok(join(a106, a107))
        }
        Adt0::C18(r130, r131, v33) => {
            let v31 = (*r130).clone();
            let v32 = (*r131).clone();
            let v34: List<Str> = f38(f39(v1, 0u64)?)?;
            let a112 = {
                let a110 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C18(std::rc::Rc::new(v31), std::rc::Rc::new(v32), v33)), List::<Str>::nil()));
                let a111 = List::cons(Str::lit(" {"), List::cons({
                    let a108 = v34;
                    let a109 = Str::lit(", ");
                    join(a108, a109)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a110, a111)
            };
            let a113 = Str::lit("");
            Ok(join(a112, a113))
        }
        Adt0::C19(r132, v36) => {
            let v35 = (*r132).clone();
            let v37: List<Str> = f40(f41(v1, 0u64)?)?;
            let a118 = {
                let a116 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C19(std::rc::Rc::new(v35), v36)), List::<Str>::nil()));
                let a117 = List::cons(Str::lit(" {"), List::cons({
                    let a114 = v37;
                    let a115 = Str::lit(", ");
                    join(a114, a115)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a116, a117)
            };
            let a119 = Str::lit("");
            Ok(join(a118, a119))
        }
        Adt0::C20(v38, r133) => {
            let v39 = (*r133).clone();
            let v40: List<Str> = f42(f43(v1, 0u64)?)?;
            let a124 = {
                let a122 = List::cons(Str::lit("(record "), List::cons(f3(Adt0::C20(v38, std::rc::Rc::new(v39))), List::<Str>::nil()));
                let a123 = List::cons(Str::lit(" {"), List::cons({
                    let a120 = v40;
                    let a121 = Str::lit(", ");
                    join(a120, a121)
                }, List::cons(Str::lit("})"), List::<Str>::nil())));
                append_list(a122, a123)
            };
            let a125 = Str::lit("");
            Ok(join(a124, a125))
        }
    }
}

pub fn f1(v0: List<Adt2>) -> R<List<Str>> {
    let m134 = v0;
    match m134.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h135, _) => {
                    h135
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h136) => {
                    h136
                }
            };
            Ok(List::cons({
                let a139 = {
                    let a137 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a138 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a137, a138)
                };
                let a140 = Str::lit("");
                join(a139, a140)
            }, f1(v2)?))
        }
    }
}

pub fn f2(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m141 = v0;
    match m141.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f2(v3, {
                let a142 = v1;
                let a143 = 1u64;
                nat_add(a142, a143)?
            })?))
        }
    }
}

pub fn f3(v0: Adt0) -> Str {
    let m144 = v0;
    match m144 {
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
            let a145 = List::cons(f45(), List::cons(Str::lit("::ParseIntError"), List::<Str>::nil()));
            let a146 = Str::lit("");
            join(a145, a146)
        }
        Adt0::C11 => {
            let a147 = List::cons(f45(), List::cons(Str::lit("::FilePath"), List::<Str>::nil()));
            let a148 = Str::lit("");
            join(a147, a148)
        }
        Adt0::C12(v6) => {
            v6
        }
        Adt0::C13(r185) => {
            let v7 = (*r185).clone();
            let a151 = {
                let a149 = List::cons(Str::lit("(Option "), List::<Str>::nil());
                let a150 = List::cons(f3(v7), List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a149, a150)
            };
            let a152 = Str::lit("");
            join(a151, a152)
        }
        Adt0::C14(r186, r187) => {
            let v8 = (*r186).clone();
            let v9 = (*r187).clone();
            let a155 = {
                let a153 = List::cons(Str::lit("(Result "), List::cons(f3(v8), List::<Str>::nil()));
                let a154 = List::cons(Str::lit(" "), List::cons(f3(v9), List::cons(Str::lit(")"), List::<Str>::nil())));
                append_list(a153, a154)
            };
            let a156 = Str::lit("");
            join(a155, a156)
        }
        Adt0::C15(r188) => {
            let v10 = (*r188).clone();
            let a159 = {
                let a157 = List::cons(Str::lit("(List "), List::<Str>::nil());
                let a158 = List::cons(f3(v10), List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a157, a158)
            };
            let a160 = Str::lit("");
            join(a159, a160)
        }
        Adt0::C16(v11) => {
            let a163 = List::cons(Str::lit("Tuple"), List::cons({
                let a161 = f46(v11);
                let a162 = Str::lit("");
                join(a161, a162)
            }, List::<Str>::nil()));
            let a164 = Str::lit("");
            join(a163, a164)
        }
        Adt0::C17(v12) => {
            let a169 = {
                let a167 = List::cons(Str::lit("("), List::<Str>::nil());
                let a168 = List::cons({
                    let a165 = f47(v12);
                    let a166 = Str::lit(" * ");
                    join(a165, a166)
                }, List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a167, a168)
            };
            let a170 = Str::lit("");
            join(a169, a170)
        }
        Adt0::C18(r189, r190, _) => {
            let v13 = (*r189).clone();
            let v14 = (*r190).clone();
            let a173 = {
                let a171 = List::cons(Str::lit("(List Tuple"), List::cons(f48(v13), List::<Str>::nil()));
                let a172 = List::cons(f48(v14), List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a171, a172)
            };
            let a174 = Str::lit("");
            join(a173, a174)
        }
        Adt0::C19(r191, _) => {
            let v16 = (*r191).clone();
            let a177 = {
                let a175 = List::cons(Str::lit("(List "), List::<Str>::nil());
                let a176 = List::cons(f3(v16), List::cons(Str::lit(")"), List::<Str>::nil()));
                append_list(a175, a176)
            };
            let a178 = Str::lit("");
            join(a177, a178)
        }
        Adt0::C20(v18, r192) => {
            let v19 = (*r192).clone();
            let a183 = {
                let a181 = List::cons(Str::lit("(Fn ("), List::cons({
                    let a179 = f49(v18);
                    let a180 = Str::lit(" ");
                    join(a179, a180)
                }, List::<Str>::nil()));
                let a182 = List::cons(Str::lit(") "), List::cons(f3(v19), List::cons(Str::lit(")"), List::<Str>::nil())));
                append_list(a181, a182)
            };
            let a184 = Str::lit("");
            join(a183, a184)
        }
    }
}

pub fn f4(v0: List<Adt2>) -> R<List<Str>> {
    let m193 = v0;
    match m193.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h194, _) => {
                    h194
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h195) => {
                    h195
                }
            };
            Ok(List::cons({
                let a198 = {
                    let a196 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a197 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a196, a197)
                };
                let a199 = Str::lit("");
                join(a198, a199)
            }, f4(v2)?))
        }
    }
}

pub fn f5(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m200 = v0;
    match m200.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f5(v3, {
                let a201 = v1;
                let a202 = 1u64;
                nat_add(a201, a202)?
            })?))
        }
    }
}

pub fn f6(v0: List<Adt2>) -> R<List<Str>> {
    let m203 = v0;
    match m203.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h204, _) => {
                    h204
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h205) => {
                    h205
                }
            };
            Ok(List::cons({
                let a208 = {
                    let a206 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a207 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a206, a207)
                };
                let a209 = Str::lit("");
                join(a208, a209)
            }, f6(v2)?))
        }
    }
}

pub fn f7(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m210 = v0;
    match m210.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f7(v3, {
                let a211 = v1;
                let a212 = 1u64;
                nat_add(a211, a212)?
            })?))
        }
    }
}

pub fn f8(v0: List<Adt2>) -> R<List<Str>> {
    let m213 = v0;
    match m213.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h214, _) => {
                    h214
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h215) => {
                    h215
                }
            };
            Ok(List::cons({
                let a218 = {
                    let a216 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a217 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a216, a217)
                };
                let a219 = Str::lit("");
                join(a218, a219)
            }, f8(v2)?))
        }
    }
}

pub fn f9(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m220 = v0;
    match m220.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f9(v3, {
                let a221 = v1;
                let a222 = 1u64;
                nat_add(a221, a222)?
            })?))
        }
    }
}

pub fn f10(v0: List<Adt2>) -> R<List<Str>> {
    let m223 = v0;
    match m223.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h224, _) => {
                    h224
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h225) => {
                    h225
                }
            };
            Ok(List::cons({
                let a228 = {
                    let a226 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a227 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a226, a227)
                };
                let a229 = Str::lit("");
                join(a228, a229)
            }, f10(v2)?))
        }
    }
}

pub fn f11(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m230 = v0;
    match m230.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f11(v3, {
                let a231 = v1;
                let a232 = 1u64;
                nat_add(a231, a232)?
            })?))
        }
    }
}

pub fn f12(v0: List<Adt2>) -> R<List<Str>> {
    let m233 = v0;
    match m233.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h234, _) => {
                    h234
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h235) => {
                    h235
                }
            };
            Ok(List::cons({
                let a238 = {
                    let a236 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a237 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a236, a237)
                };
                let a239 = Str::lit("");
                join(a238, a239)
            }, f12(v2)?))
        }
    }
}

pub fn f13(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m240 = v0;
    match m240.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f13(v3, {
                let a241 = v1;
                let a242 = 1u64;
                nat_add(a241, a242)?
            })?))
        }
    }
}

pub fn f14(v0: List<Adt2>) -> R<List<Str>> {
    let m243 = v0;
    match m243.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h244, _) => {
                    h244
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h245) => {
                    h245
                }
            };
            Ok(List::cons({
                let a248 = {
                    let a246 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a247 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a246, a247)
                };
                let a249 = Str::lit("");
                join(a248, a249)
            }, f14(v2)?))
        }
    }
}

pub fn f15(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m250 = v0;
    match m250.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f15(v3, {
                let a251 = v1;
                let a252 = 1u64;
                nat_add(a251, a252)?
            })?))
        }
    }
}

pub fn f16(v0: List<Adt2>) -> R<List<Str>> {
    let m253 = v0;
    match m253.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h254, _) => {
                    h254
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h255) => {
                    h255
                }
            };
            Ok(List::cons({
                let a258 = {
                    let a256 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a257 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a256, a257)
                };
                let a259 = Str::lit("");
                join(a258, a259)
            }, f16(v2)?))
        }
    }
}

pub fn f17(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m260 = v0;
    match m260.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f17(v3, {
                let a261 = v1;
                let a262 = 1u64;
                nat_add(a261, a262)?
            })?))
        }
    }
}

pub fn f18(v0: List<Adt2>) -> R<List<Str>> {
    let m263 = v0;
    match m263.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h264, _) => {
                    h264
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h265) => {
                    h265
                }
            };
            Ok(List::cons({
                let a268 = {
                    let a266 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a267 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a266, a267)
                };
                let a269 = Str::lit("");
                join(a268, a269)
            }, f18(v2)?))
        }
    }
}

pub fn f19(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m270 = v0;
    match m270.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f19(v3, {
                let a271 = v1;
                let a272 = 1u64;
                nat_add(a271, a272)?
            })?))
        }
    }
}

pub fn f20(v0: List<Adt2>) -> R<List<Str>> {
    let m273 = v0;
    match m273.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h274, _) => {
                    h274
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h275) => {
                    h275
                }
            };
            Ok(List::cons({
                let a278 = {
                    let a276 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a277 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a276, a277)
                };
                let a279 = Str::lit("");
                join(a278, a279)
            }, f20(v2)?))
        }
    }
}

pub fn f21(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m280 = v0;
    match m280.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f21(v3, {
                let a281 = v1;
                let a282 = 1u64;
                nat_add(a281, a282)?
            })?))
        }
    }
}

pub fn f22(v0: List<Adt2>) -> R<List<Str>> {
    let m283 = v0;
    match m283.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h284, _) => {
                    h284
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h285) => {
                    h285
                }
            };
            Ok(List::cons({
                let a288 = {
                    let a286 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a287 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a286, a287)
                };
                let a289 = Str::lit("");
                join(a288, a289)
            }, f22(v2)?))
        }
    }
}

pub fn f23(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m290 = v0;
    match m290.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f23(v3, {
                let a291 = v1;
                let a292 = 1u64;
                nat_add(a291, a292)?
            })?))
        }
    }
}

pub fn f24(v0: List<Adt2>) -> R<List<Str>> {
    let m293 = v0;
    match m293.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h294, _) => {
                    h294
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h295) => {
                    h295
                }
            };
            Ok(List::cons({
                let a298 = {
                    let a296 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a297 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a296, a297)
                };
                let a299 = Str::lit("");
                join(a298, a299)
            }, f24(v2)?))
        }
    }
}

pub fn f25(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m300 = v0;
    match m300.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f25(v3, {
                let a301 = v1;
                let a302 = 1u64;
                nat_add(a301, a302)?
            })?))
        }
    }
}

pub fn f26(v0: List<Adt2>) -> R<List<Str>> {
    let m303 = v0;
    match m303.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h304, _) => {
                    h304
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h305) => {
                    h305
                }
            };
            Ok(List::cons({
                let a308 = {
                    let a306 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a307 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a306, a307)
                };
                let a309 = Str::lit("");
                join(a308, a309)
            }, f26(v2)?))
        }
    }
}

pub fn f27(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m310 = v0;
    match m310.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f27(v3, {
                let a311 = v1;
                let a312 = 1u64;
                nat_add(a311, a312)?
            })?))
        }
    }
}

pub fn f28(v0: List<Adt2>) -> R<List<Str>> {
    let m313 = v0;
    match m313.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h314, _) => {
                    h314
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h315) => {
                    h315
                }
            };
            Ok(List::cons({
                let a318 = {
                    let a316 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a317 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a316, a317)
                };
                let a319 = Str::lit("");
                join(a318, a319)
            }, f28(v2)?))
        }
    }
}

pub fn f29(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m320 = v0;
    match m320.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f29(v3, {
                let a321 = v1;
                let a322 = 1u64;
                nat_add(a321, a322)?
            })?))
        }
    }
}

pub fn f30(v0: List<Adt2>) -> R<List<Str>> {
    let m323 = v0;
    match m323.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h324, _) => {
                    h324
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h325) => {
                    h325
                }
            };
            Ok(List::cons({
                let a328 = {
                    let a326 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a327 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a326, a327)
                };
                let a329 = Str::lit("");
                join(a328, a329)
            }, f30(v2)?))
        }
    }
}

pub fn f31(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m330 = v0;
    match m330.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f31(v3, {
                let a331 = v1;
                let a332 = 1u64;
                nat_add(a331, a332)?
            })?))
        }
    }
}

pub fn f32(v0: List<Adt2>) -> R<List<Str>> {
    let m333 = v0;
    match m333.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h334, _) => {
                    h334
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h335) => {
                    h335
                }
            };
            Ok(List::cons({
                let a338 = {
                    let a336 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a337 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a336, a337)
                };
                let a339 = Str::lit("");
                join(a338, a339)
            }, f32(v2)?))
        }
    }
}

pub fn f33(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m340 = v0;
    match m340.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f33(v3, {
                let a341 = v1;
                let a342 = 1u64;
                nat_add(a341, a342)?
            })?))
        }
    }
}

pub fn f34(v0: List<Adt2>) -> R<List<Str>> {
    let m343 = v0;
    match m343.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h344, _) => {
                    h344
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h345) => {
                    h345
                }
            };
            Ok(List::cons({
                let a348 = {
                    let a346 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a347 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a346, a347)
                };
                let a349 = Str::lit("");
                join(a348, a349)
            }, f34(v2)?))
        }
    }
}

pub fn f35(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m350 = v0;
    match m350.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f35(v3, {
                let a351 = v1;
                let a352 = 1u64;
                nat_add(a351, a352)?
            })?))
        }
    }
}

pub fn f36(v0: List<Str>) -> Str {
    let m355 = {
        let a353 = v0;
        let a354 = 0u64;
        index_list(a353, a354)
    };
    match m355 {
        None => {
            Str::lit("")
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f37(v0: List<Str>) -> Str {
    let m358 = {
        let a356 = v0;
        let a357 = 1u64;
        index_list(a356, a357)
    };
    match m358 {
        None => {
            Str::lit("")
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f38(v0: List<Adt2>) -> R<List<Str>> {
    let m359 = v0;
    match m359.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h360, _) => {
                    h360
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h361) => {
                    h361
                }
            };
            Ok(List::cons({
                let a364 = {
                    let a362 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a363 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a362, a363)
                };
                let a365 = Str::lit("");
                join(a364, a365)
            }, f38(v2)?))
        }
    }
}

pub fn f39(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m366 = v0;
    match m366.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f39(v3, {
                let a367 = v1;
                let a368 = 1u64;
                nat_add(a367, a368)?
            })?))
        }
    }
}

pub fn f40(v0: List<Adt2>) -> R<List<Str>> {
    let m369 = v0;
    match m369.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h370, _) => {
                    h370
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h371) => {
                    h371
                }
            };
            Ok(List::cons({
                let a374 = {
                    let a372 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a373 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a372, a373)
                };
                let a375 = Str::lit("");
                join(a374, a375)
            }, f40(v2)?))
        }
    }
}

pub fn f41(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m376 = v0;
    match m376.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f41(v3, {
                let a377 = v1;
                let a378 = 1u64;
                nat_add(a377, a378)?
            })?))
        }
    }
}

pub fn f42(v0: List<Adt2>) -> R<List<Str>> {
    let m379 = v0;
    match m379.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let v3: u64 = match v1.clone() {
                Adt2::C0(h380, _) => {
                    h380
                }
            };
            let v4: Str = match v1 {
                Adt2::C0(_, h381) => {
                    h381
                }
            };
            Ok(List::cons({
                let a384 = {
                    let a382 = List::cons(Str::lit("item"), List::cons(f44(v3)?, List::<Str>::nil()));
                    let a383 = List::cons(Str::lit(" := "), List::cons(v4, List::<Str>::nil()));
                    append_list(a382, a383)
                };
                let a385 = Str::lit("");
                join(a384, a385)
            }, f42(v2)?))
        }
    }
}

pub fn f43(v0: List<Str>, v1: u64) -> R<List<Adt2>> {
    let m386 = v0;
    match m386.uncons() {
        None => {
            Ok(List::<Adt2>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(Adt2::C0(v1, v2), f43(v3, {
                let a387 = v1;
                let a388 = 1u64;
                nat_add(a387, a388)?
            })?))
        }
    }
}

pub fn f44(v0: u64) -> R<Str> {
    Ok(f50({
        let a389 = v0;
        let a390 = 1u64;
        nat_add(a389, a390)?
    }, v0))
}

pub fn f45() -> Str {
    Str::lit("Prelude")
}

pub fn f46(v0: List<Adt0>) -> List<Str> {
    let m391 = v0;
    match m391.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f48(v1), f46(v2))
        }
    }
}

pub fn f47(v0: List<Adt0>) -> List<Str> {
    let m392 = v0;
    match m392.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f3(v1), f47(v2))
        }
    }
}

pub fn f48(v0: Adt0) -> Str {
    let m393 = v0;
    match m393 {
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
            f51()
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
        Adt0::C13(r422) => {
            let v7 = (*r422).clone();
            let a394 = List::cons(Str::lit("Option"), List::cons(f48(v7), List::<Str>::nil()));
            let a395 = Str::lit("");
            join(a394, a395)
        }
        Adt0::C14(r423, r424) => {
            let v8 = (*r423).clone();
            let v9 = (*r424).clone();
            let a398 = {
                let a396 = List::cons(Str::lit("Result"), List::<Str>::nil());
                let a397 = List::cons(f48(v8), List::cons(f48(v9), List::<Str>::nil()));
                append_list(a396, a397)
            };
            let a399 = Str::lit("");
            join(a398, a399)
        }
        Adt0::C15(r425) => {
            let v10 = (*r425).clone();
            let a400 = List::cons(Str::lit("List"), List::cons(f48(v10), List::<Str>::nil()));
            let a401 = Str::lit("");
            join(a400, a401)
        }
        Adt0::C16(v11) => {
            let a404 = List::cons(Str::lit("Tuple"), List::cons({
                let a402 = f52(v11);
                let a403 = Str::lit("");
                join(a402, a403)
            }, List::<Str>::nil()));
            let a405 = Str::lit("");
            join(a404, a405)
        }
        Adt0::C17(v12) => {
            let a408 = List::cons(Str::lit("Pair"), List::cons({
                let a406 = f53(v12);
                let a407 = Str::lit("");
                join(a406, a407)
            }, List::<Str>::nil()));
            let a409 = Str::lit("");
            join(a408, a409)
        }
        Adt0::C18(r426, r427, _) => {
            let v13 = (*r426).clone();
            let v14 = (*r427).clone();
            let a412 = {
                let a410 = List::cons(Str::lit("ListTuple"), List::<Str>::nil());
                let a411 = List::cons(f48(v13), List::cons(f48(v14), List::<Str>::nil()));
                append_list(a410, a411)
            };
            let a413 = Str::lit("");
            join(a412, a413)
        }
        Adt0::C19(r428, _) => {
            let v16 = (*r428).clone();
            let a414 = List::cons(Str::lit("List"), List::cons(f48(v16), List::<Str>::nil()));
            let a415 = Str::lit("");
            join(a414, a415)
        }
        Adt0::C20(v18, r429) => {
            let v19 = (*r429).clone();
            let a420 = {
                let a418 = List::cons(Str::lit("Fn"), List::cons({
                    let a416 = f54(v18);
                    let a417 = Str::lit("");
                    join(a416, a417)
                }, List::<Str>::nil()));
                let a419 = List::cons(Str::lit("To"), List::cons(f48(v19), List::<Str>::nil()));
                append_list(a418, a419)
            };
            let a421 = Str::lit("");
            join(a420, a421)
        }
    }
}

pub fn f49(v0: List<Adt0>) -> List<Str> {
    let m430 = v0;
    match m430.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f3(v1), f49(v2))
        }
    }
}

pub fn f50(v0: u64, v1: u64) -> Str {
    let m431 = v0;
    if m431 == 0 {
        Str::lit("")
    } else {
        let v2 = m431 - 1;
        let a432 = v1;
        let a433 = 10u64;
        if nat_lt(a432, a433) {
            f55(v1)
        } else {
            let a440 = List::cons(f50(v2, {
                let a434 = v1;
                let a435 = 10u64;
                let a436 = 0u64;
                nat_quot(a434, a435, a436)
            }), List::cons(f55({
                let a437 = v1;
                let a438 = 10u64;
                let a439 = 0u64;
                nat_rem(a437, a438, a439)
            }), List::<Str>::nil()));
            let a441 = Str::lit("");
            join(a440, a441)
        }
    }
}

pub fn f51() -> Str {
    Str::lit("LexLeanInt")
}

pub fn f52(v0: List<Adt0>) -> List<Str> {
    let m442 = v0;
    match m442.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f48(v1), f52(v2))
        }
    }
}

pub fn f53(v0: List<Adt0>) -> List<Str> {
    let m443 = v0;
    match m443.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f48(v1), f53(v2))
        }
    }
}

pub fn f54(v0: List<Adt0>) -> List<Str> {
    let m444 = v0;
    match m444.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f48(v1), f54(v2))
        }
    }
}

pub fn f55(v0: u64) -> Str {
    let a445 = v0;
    let a446 = 0u64;
    if nat_eq(a445, a446) {
        Str::lit("0")
    } else {
        let a447 = v0;
        let a448 = 1u64;
        if nat_eq(a447, a448) {
            Str::lit("1")
        } else {
            let a449 = v0;
            let a450 = 2u64;
            if nat_eq(a449, a450) {
                Str::lit("2")
            } else {
                let a451 = v0;
                let a452 = 3u64;
                if nat_eq(a451, a452) {
                    Str::lit("3")
                } else {
                    let a453 = v0;
                    let a454 = 4u64;
                    if nat_eq(a453, a454) {
                        Str::lit("4")
                    } else {
                        let a455 = v0;
                        let a456 = 5u64;
                        if nat_eq(a455, a456) {
                            Str::lit("5")
                        } else {
                            let a457 = v0;
                            let a458 = 6u64;
                            if nat_eq(a457, a458) {
                                Str::lit("6")
                            } else {
                                let a459 = v0;
                                let a460 = 7u64;
                                if nat_eq(a459, a460) {
                                    Str::lit("7")
                                } else {
                                    let a461 = v0;
                                    let a462 = 8u64;
                                    if nat_eq(a461, a462) {
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

pub fn run(p0: Adt0, p1: List<Str>) -> R<Str> {
    f0(p0, p1)
}
