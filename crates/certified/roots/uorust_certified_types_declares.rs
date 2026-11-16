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
    C0(List<Adt1>, List<Adt2>, List<Adt5>, List<Adt6>, List<List<Str>>, List<Adt1>, List<Adt1>, bool, List<Adt7>, List<Adt8>, List<Adt9>, List<Adt11>, List<Adt5>),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Str, List<Str>),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Str, List<(Str, Adt3)>),
}

#[derive(Clone)]
pub enum Adt3 {
    C0,
    C1,
    C2(Option<u64>),
    C3(u64),
    C4(Adt4),
    C5(Option<u64>),
    C6(Option<u64>),
    C7,
    C8,
    C9,
    C10,
    C11,
    C12(Str),
    C13(std::rc::Rc<Adt3>),
    C14(std::rc::Rc<Adt3>, std::rc::Rc<Adt3>),
    C15(std::rc::Rc<Adt3>),
    C16(List<Adt3>),
    C17(List<Adt3>),
    C18(std::rc::Rc<Adt3>, std::rc::Rc<Adt3>, bool),
    C19(std::rc::Rc<Adt3>, bool),
    C20(List<Adt3>, std::rc::Rc<Adt3>),
}

#[derive(Clone)]
pub enum Adt4 {
    C0(u64, u64),
}

#[derive(Clone)]
pub enum Adt5 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt6 {
    C0(Str, List<(Str, List<(Option<Str>, Adt3)>)>),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(Str, Adt4),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str, Adt5),
}

#[derive(Clone)]
pub enum Adt9 {
    C0(Str, List<Adt10>),
}

#[derive(Clone)]
pub enum Adt10 {
    C0(u64, Str),
}

#[derive(Clone)]
pub enum Adt11 {
    C0(Str, List<i64>),
}

pub fn f0(v0: Adt0, v1: Str) -> R<bool> {
    let v2: Str = f1(v0.clone(), v1)?;
    if if if f2(match v0.clone() {
        Adt0::C0(h1, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h1
        }
    }, v2.clone()) {
        true
    } else {
        f3(match v0.clone() {
            Adt0::C0(_, h2, _, _, _, _, _, _, _, _, _, _, _) => {
                h2
            }
        }, v2.clone())
    } {
        true
    } else {
        f4(match v0.clone() {
            Adt0::C0(_, _, _, h3, _, _, _, _, _, _, _, _, _) => {
                h3
            }
        }, v2.clone())
    } {
        Ok(true)
    } else {
        Ok(f5(match v0 {
            Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, h4) => {
                h4
            }
        }, v2))
    }
}

pub fn f1(v0: Adt0, v1: Str) -> R<Str> {
    let v2: Str = f6(v1)?;
    let m6 = f7(match v0 {
        Adt0::C0(_, _, h5, _, _, _, _, _, _, _, _, _, _) => {
            h5
        }
    }, v2.clone());
    match m6 {
        None => {
            Ok(v2)
        }
        Some(v3) => {
            Ok(v3)
        }
    }
}

pub fn f2(p12: List<Adt1>, v1: Str) -> bool {
    let m11 = p12;
    {
        let mut m11 = m11;
        loop {
            let step = {
                let v0 = m11;
                let m7 = v0;
                match m7.uncons() {
                    None => {
                        Err::<List<Adt1>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a9 = match v2 {
                            Adt1::C0(h8, _) => {
                                h8
                            }
                        };
                        let a10 = v1.clone();
                        if equal(a9, a10) {
                            Err::<List<Adt1>, bool>(true)
                        } else {
                            Ok::<List<Adt1>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m11 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f3(p18: List<Adt2>, v1: Str) -> bool {
    let m17 = p18;
    {
        let mut m17 = m17;
        loop {
            let step = {
                let v0 = m17;
                let m13 = v0;
                match m13.uncons() {
                    None => {
                        Err::<List<Adt2>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a15 = match v2 {
                            Adt2::C0(h14, _) => {
                                h14
                            }
                        };
                        let a16 = v1.clone();
                        if equal(a15, a16) {
                            Err::<List<Adt2>, bool>(true)
                        } else {
                            Ok::<List<Adt2>, bool>(v3)
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

pub fn f4(p24: List<Adt6>, v1: Str) -> bool {
    let m23 = p24;
    {
        let mut m23 = m23;
        loop {
            let step = {
                let v0 = m23;
                let m19 = v0;
                match m19.uncons() {
                    None => {
                        Err::<List<Adt6>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a21 = match v2 {
                            Adt6::C0(h20, _) => {
                                h20
                            }
                        };
                        let a22 = v1.clone();
                        if equal(a21, a22) {
                            Err::<List<Adt6>, bool>(true)
                        } else {
                            Ok::<List<Adt6>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m23 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f5(p30: List<Adt5>, v1: Str) -> bool {
    let m29 = p30;
    {
        let mut m29 = m29;
        loop {
            let step = {
                let v0 = m29;
                let m25 = v0;
                match m25.uncons() {
                    None => {
                        Err::<List<Adt5>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a27 = match v2 {
                            Adt5::C0(h26, _) => {
                                h26
                            }
                        };
                        let a28 = v1.clone();
                        if equal(a27, a28) {
                            Err::<List<Adt5>, bool>(true)
                        } else {
                            Ok::<List<Adt5>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m29 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f6(v0: Str) -> R<Str> {
    let m31 = f8(v0.clone())?;
    match m31 {
        None => {
            Ok(v0)
        }
        Some(v1) => {
            let v2: Str = match v1.clone() {
                Adt5::C0(h32, _) => {
                    h32
                }
            };
            let v3: Str = match v1 {
                Adt5::C0(_, h33) => {
                    h33
                }
            };
            let m34 = f9(v3, Str::lit(">"));
            match m34 {
                None => {
                    Ok(v0)
                }
                Some(v4) => {
                    if f10(f11(v4, Str::lit(","))?)? {
                        f12(v2)
                    } else {
                        Ok(v0)
                    }
                }
            }
        }
    }
}

pub fn f7(p41: List<Adt5>, v1: Str) -> Option<Str> {
    let m40 = p41;
    {
        let mut m40 = m40;
        loop {
            let step = {
                let v0 = m40;
                let m35 = v0;
                match m35.uncons() {
                    None => {
                        Err::<List<Adt5>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a37 = match v2.clone() {
                            Adt5::C0(h36, _) => {
                                h36
                            }
                        };
                        let a38 = v1.clone();
                        if equal(a37, a38) {
                            Err::<List<Adt5>, Option<Str>>(Some(match v2 {
                                Adt5::C0(_, h39) => {
                                    h39
                                }
                            }))
                        } else {
                            Ok::<List<Adt5>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m40 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f8(v0: Str) -> R<Option<Adt5>> {
    let m42 = f13(v0.clone(), Str::lit("<"))?;
    match m42 {
        None => {
            Ok(None::<Adt5>)
        }
        Some(v1) => {
            Ok(Some(Adt5::C0(f14(v0.clone(), 0u64, v1), f14(v0.clone(), {
                let a43 = v1;
                let a44 = f15(Str::lit("<"));
                nat_add(a43, a44)?
            }, f15(v0)))))
        }
    }
}

pub fn f9(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a45 = v0.clone();
        utf8_encode(a45)
    };
    let v3: u64 = {
        let a46 = v2.clone();
        length_bytes(a46)
    };
    let v4: u64 = f15(v1.clone());
    if f16(v0, v1) {
        Some(f17(v2, 0u64, {
            let a47 = v3;
            let a48 = v4;
            nat_sub(a47, a48)
        }))
    } else {
        None::<Str>
    }
}

pub fn f10(p51: List<Str>) -> R<bool> {
    let m50 = p51;
    {
        let mut m50 = m50;
        loop {
            let step = {
                let v0 = m50;
                let m49 = v0;
                match m49.uncons() {
                    None => {
                        Err::<List<Str>, R<bool>>(Ok(true))
                    }
                    Some((v1, v2)) => {
                        if f18(v1)? {
                            Ok::<List<Str>, R<bool>>(v2)
                        } else {
                            Err::<List<Str>, R<bool>>(Ok(false))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m50 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f11(v0: Str, v1: Str) -> R<List<Str>> {
    let v2: Bytes = {
        let a52 = v0.clone();
        utf8_encode(a52)
    };
    let v3: Bytes = {
        let a53 = v1;
        utf8_encode(a53)
    };
    let a55 = {
        let a54 = v3.clone();
        length_bytes(a54)
    };
    let a56 = 0u64;
    if nat_eq(a55, a56) {
        Ok(List::cons(Str::lit(""), {
            let a57 = f19(v0)?;
            let a58 = List::cons(Str::lit(""), List::<Str>::nil());
            append_list(a57, a58)
        }))
    } else {
        f20({
            let a60 = {
                let a59 = v2.clone();
                length_bytes(a59)
            };
            let a61 = 1u64;
            nat_add(a60, a61)?
        }, v2, v3, 0u64, 0u64)
    }
}

pub fn f12(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a62 = v0;
        utf8_encode(a62)
    };
    let v2: u64 = {
        let a63 = v1.clone();
        length_bytes(a63)
    };
    Ok(f17(v1.clone(), 0u64, f21(v2, v1, v2)?))
}

pub fn f13(v0: Str, v1: Str) -> R<Option<u64>> {
    let v2: Bytes = {
        let a64 = v0;
        utf8_encode(a64)
    };
    f22({
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

pub fn f14(v0: Str, v1: u64, v2: u64) -> Str {
    f17({
        let a69 = v0;
        utf8_encode(a69)
    }, v1, v2)
}

pub fn f15(v0: Str) -> u64 {
    let a71 = {
        let a70 = v0;
        utf8_encode(a70)
    };
    length_bytes(a71)
}

pub fn f16(v0: Str, v1: Str) -> bool {
    let v2: Bytes = {
        let a72 = v0;
        utf8_encode(a72)
    };
    let v3: Bytes = {
        let a73 = v1;
        utf8_encode(a73)
    };
    let v4: u64 = {
        let a74 = v2.clone();
        length_bytes(a74)
    };
    let v5: u64 = {
        let a75 = v3.clone();
        length_bytes(a75)
    };
    let a76 = v5;
    let a77 = v4;
    if nat_le(a76, a77) {
        f23(v2, {
            let a78 = v4;
            let a79 = v5;
            nat_sub(a78, a79)
        }, v3)
    } else {
        false
    }
}

pub fn f17(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m85 = {
        let a82 = v0;
        let a83 = v1;
        let a84 = {
            let a80 = v2;
            let a81 = v1;
            nat_sub(a80, a81)
        };
        slice_bytes(a82, a83, a84)
    };
    match m85 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m87 = {
                let a86 = v3;
                utf8_decode(a86)
            };
            match m87 {
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

pub fn f18(v0: Str) -> R<bool> {
    let m88 = f24(f25(v0)?, Str::lit("'"));
    match m88 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            let a89 = f26(v1, Str::lit("'"))?;
            Ok(bool_not(a89))
        }
    }
}

pub fn f19(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a90 = v0;
        utf8_encode(a90)
    };
    f27({
        let a91 = v1.clone();
        length_bytes(a91)
    }, v1, 0u64)
}

pub fn f20(v0: u64, v1: Bytes, v2: Bytes, v3: u64, v4: u64) -> R<List<Str>> {
    let m92 = v0;
    if m92 == 0 {
        Ok(List::cons(f17(v1.clone(), v3, {
            let a93 = v1;
            length_bytes(a93)
        }), List::<Str>::nil()))
    } else {
        let v5 = m92 - 1;
        let a98 = {
            let a94 = v1.clone();
            length_bytes(a94)
        };
        let a99 = {
            let a96 = v4;
            let a97 = {
                let a95 = v2.clone();
                length_bytes(a95)
            };
            nat_add(a96, a97)?
        };
        if nat_lt(a98, a99) {
            Ok(List::cons(f17(v1.clone(), v3, {
                let a100 = v1;
                length_bytes(a100)
            }), List::<Str>::nil()))
        } else {
            if f23(v1.clone(), v4, v2.clone()) {
                Ok(List::cons(f17(v1.clone(), v3, v4), f20(v5, v1, v2.clone(), {
                    let a102 = v4;
                    let a103 = {
                        let a101 = v2.clone();
                        length_bytes(a101)
                    };
                    nat_add(a102, a103)?
                }, {
                    let a105 = v4;
                    let a106 = {
                        let a104 = v2;
                        length_bytes(a104)
                    };
                    nat_add(a105, a106)?
                })?))
            } else {
                f20(v5, v1, v2, v3, {
                    let a107 = v4;
                    let a108 = 1u64;
                    nat_add(a107, a108)?
                })
            }
        }
    }
}

pub fn f21(p115: u64, v1: Bytes, p116: u64) -> R<u64> {
    let m114 = (p115, p116);
    {
        let mut m114 = m114;
        loop {
            let step = {
                let (v0, v2) = m114;
                let m109 = v0;
                if m109 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m109 - 1;
                    let v4: u64 = f28(v1.clone(), v2)?;
                    let a110 = v4;
                    let a111 = 0u64;
                    if nat_eq(a110, a111) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a112 = v2;
                            let a113 = v4;
                            nat_sub(a112, a113)
                        }))
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

pub fn f22(p127: u64, v1: Bytes, v2: Bytes, p128: u64) -> R<Option<u64>> {
    let m126 = (p127, p128);
    {
        let mut m126 = m126;
        loop {
            let step = {
                let (v0, v3) = m126;
                let m117 = v0;
                if m117 == 0 {
                    Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                } else {
                    let v4 = m117 - 1;
                    let a122 = {
                        let a118 = v1.clone();
                        length_bytes(a118)
                    };
                    let a123 = {
                        let a120 = v3;
                        let a121 = {
                            let a119 = v2.clone();
                            length_bytes(a119)
                        };
                        nat_add(a120, a121)?
                    };
                    if nat_lt(a122, a123) {
                        Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                    } else {
                        if f23(v1.clone(), v3, v2.clone()) {
                            Err::<(u64, u64), R<Option<u64>>>(Ok(Some(v3)))
                        } else {
                            Ok::<(u64, u64), R<Option<u64>>>((v4, {
                                let a124 = v3;
                                let a125 = 1u64;
                                nat_add(a124, a125)?
                            }))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m126 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f23(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m133 = {
        let a130 = v0;
        let a131 = v1;
        let a132 = {
            let a129 = v2.clone();
            length_bytes(a129)
        };
        slice_bytes(a130, a131, a132)
    };
    match m133 {
        None => {
            false
        }
        Some(v3) => {
            let a134 = v3;
            let a135 = v2;
            equal(a134, a135)
        }
    }
}

pub fn f24(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a136 = v0;
        utf8_encode(a136)
    };
    let v3: Bytes = {
        let a137 = v1;
        utf8_encode(a137)
    };
    if f23(v2.clone(), 0u64, v3.clone()) {
        Some(f17(v2.clone(), {
            let a138 = v3;
            length_bytes(a138)
        }, {
            let a139 = v2;
            length_bytes(a139)
        }))
    } else {
        None::<Str>
    }
}

pub fn f25(v0: Str) -> R<Str> {
    f12(f29(v0)?)
}

pub fn f26(v0: Str, v1: Str) -> R<bool> {
    let m140 = f13(v0, v1)?;
    match m140 {
        None => {
            Ok(false)
        }
        Some(_) => {
            Ok(true)
        }
    }
}

pub fn f27(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m141 = v0;
    if m141 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m141 - 1;
        let a143 = {
            let a142 = v1.clone();
            length_bytes(a142)
        };
        let a144 = v2;
        if nat_le(a143, a144) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f30(f31(v1.clone(), v2)?);
            Ok(List::cons(f17(v1.clone(), v2, {
                let a145 = v2;
                let a146 = v4;
                nat_add(a145, a146)?
            }), f27(v3, v1, {
                let a147 = v2;
                let a148 = v4;
                nat_add(a147, a148)?
            })?))
        }
    }
}

pub fn f28(v0: Bytes, v1: u64) -> R<u64> {
    let a149 = v1;
    let a150 = 0u64;
    if nat_eq(a149, a150) {
        Ok(0u64)
    } else {
        if f32(f31(v0.clone(), {
            let a151 = v1;
            let a152 = 1u64;
            nat_sub(a151, a152)
        })?) {
            Ok(1u64)
        } else {
            let a153 = 2u64;
            let a154 = v1;
            if if nat_le(a153, a154) {
                let a157 = f33(v0.clone(), {
                    let a155 = v1;
                    let a156 = 2u64;
                    nat_sub(a155, a156)
                })?;
                let a158 = 2u64;
                nat_eq(a157, a158)
            } else {
                false
            } {
                Ok(2u64)
            } else {
                let a159 = 3u64;
                let a160 = v1;
                if if nat_le(a159, a160) {
                    let a163 = f33(v0, {
                        let a161 = v1;
                        let a162 = 3u64;
                        nat_sub(a161, a162)
                    })?;
                    let a164 = 3u64;
                    nat_eq(a163, a164)
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

pub fn f29(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a165 = v0;
        utf8_encode(a165)
    };
    let v2: u64 = {
        let a166 = v1.clone();
        length_bytes(a166)
    };
    Ok(f17(v1.clone(), f34(v2, v1, 0u64)?, v2))
}

pub fn f30(v0: u64) -> u64 {
    let a167 = v0;
    let a168 = 128u64;
    if nat_lt(a167, a168) {
        1u64
    } else {
        let a169 = v0;
        let a170 = 224u64;
        if nat_lt(a169, a170) {
            2u64
        } else {
            let a171 = v0;
            let a172 = 240u64;
            if nat_lt(a171, a172) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f31(v0: Bytes, v1: u64) -> R<u64> {
    let m175 = {
        let a173 = v0;
        let a174 = v1;
        index_bytes(a173, a174)
    };
    match m175 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a220 = {
                let a196 = {
                    let a184 = {
                        let a178 = {
                            let a176 = v3;
                            let a177 = 128u8;
                            fixed_u8::bit_and(a176, a177)
                        };
                        let a179 = 128u8;
                        if equal(a178, a179) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a185 = {
                        let a182 = {
                            let a180 = v3;
                            let a181 = 64u8;
                            fixed_u8::bit_and(a180, a181)
                        };
                        let a183 = 64u8;
                        if equal(a182, a183) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a184, a185)?
                };
                let a197 = {
                    let a194 = {
                        let a188 = {
                            let a186 = v3;
                            let a187 = 32u8;
                            fixed_u8::bit_and(a186, a187)
                        };
                        let a189 = 32u8;
                        if equal(a188, a189) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a195 = {
                        let a192 = {
                            let a190 = v3;
                            let a191 = 16u8;
                            fixed_u8::bit_and(a190, a191)
                        };
                        let a193 = 16u8;
                        if equal(a192, a193) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a194, a195)?
                };
                nat_add(a196, a197)?
            };
            let a221 = {
                let a218 = {
                    let a206 = {
                        let a200 = {
                            let a198 = v3;
                            let a199 = 8u8;
                            fixed_u8::bit_and(a198, a199)
                        };
                        let a201 = 8u8;
                        if equal(a200, a201) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a207 = {
                        let a204 = {
                            let a202 = v3;
                            let a203 = 4u8;
                            fixed_u8::bit_and(a202, a203)
                        };
                        let a205 = 4u8;
                        if equal(a204, a205) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a206, a207)?
                };
                let a219 = {
                    let a216 = {
                        let a210 = {
                            let a208 = v3;
                            let a209 = 2u8;
                            fixed_u8::bit_and(a208, a209)
                        };
                        let a211 = 2u8;
                        if equal(a210, a211) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a217 = {
                        let a214 = {
                            let a212 = v3;
                            let a213 = 1u8;
                            fixed_u8::bit_and(a212, a213)
                        };
                        let a215 = 1u8;
                        if equal(a214, a215) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a216, a217)?
                };
                nat_add(a218, a219)?
            };
            nat_add(a220, a221)
        }
    }
}

pub fn f32(v0: u64) -> bool {
    let a222 = v0;
    let a223 = 32u64;
    if nat_eq(a222, a223) {
        true
    } else {
        let a224 = 9u64;
        let a225 = v0;
        if nat_le(a224, a225) {
            let a226 = v0;
            let a227 = 13u64;
            nat_le(a226, a227)
        } else {
            false
        }
    }
}

pub fn f33(v0: Bytes, v1: u64) -> R<u64> {
    let v2: u64 = f31(v0.clone(), v1)?;
    let v3: u64 = f31(v0.clone(), {
        let a228 = v1;
        let a229 = 1u64;
        nat_add(a228, a229)?
    })?;
    let v4: u64 = f31(v0, {
        let a230 = v1;
        let a231 = 2u64;
        nat_add(a230, a231)?
    })?;
    if f32(v2) {
        Ok(1u64)
    } else {
        let a232 = v2;
        let a233 = 194u64;
        if nat_eq(a232, a233) {
            let a234 = v3;
            let a235 = 133u64;
            if if nat_eq(a234, a235) {
                true
            } else {
                let a236 = v3;
                let a237 = 160u64;
                nat_eq(a236, a237)
            } {
                Ok(2u64)
            } else {
                Ok(0u64)
            }
        } else {
            let a238 = v2;
            let a239 = 225u64;
            if nat_eq(a238, a239) {
                let a240 = v3;
                let a241 = 154u64;
                if if nat_eq(a240, a241) {
                    let a242 = v4;
                    let a243 = 128u64;
                    nat_eq(a242, a243)
                } else {
                    false
                } {
                    Ok(3u64)
                } else {
                    Ok(0u64)
                }
            } else {
                let a244 = v2;
                let a245 = 226u64;
                if nat_eq(a244, a245) {
                    let a246 = v3;
                    let a247 = 128u64;
                    if nat_eq(a246, a247) {
                        let a248 = 128u64;
                        let a249 = v4;
                        if if if nat_le(a248, a249) {
                            let a250 = v4;
                            let a251 = 138u64;
                            nat_le(a250, a251)
                        } else {
                            false
                        } {
                            true
                        } else {
                            let a252 = v4;
                            let a253 = 168u64;
                            if nat_eq(a252, a253) {
                                true
                            } else {
                                let a254 = v4;
                                let a255 = 169u64;
                                if nat_eq(a254, a255) {
                                    true
                                } else {
                                    let a256 = v4;
                                    let a257 = 175u64;
                                    nat_eq(a256, a257)
                                }
                            }
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    } else {
                        let a258 = v3;
                        let a259 = 129u64;
                        if if nat_eq(a258, a259) {
                            let a260 = v4;
                            let a261 = 159u64;
                            nat_eq(a260, a261)
                        } else {
                            false
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    }
                } else {
                    let a262 = v2;
                    let a263 = 227u64;
                    if if nat_eq(a262, a263) {
                        let a264 = v3;
                        let a265 = 128u64;
                        if nat_eq(a264, a265) {
                            let a266 = v4;
                            let a267 = 128u64;
                            nat_eq(a266, a267)
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

pub fn f34(p274: u64, v1: Bytes, p275: u64) -> R<u64> {
    let m273 = (p274, p275);
    {
        let mut m273 = m273;
        loop {
            let step = {
                let (v0, v2) = m273;
                let m268 = v0;
                if m268 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m268 - 1;
                    let v4: u64 = f33(v1.clone(), v2)?;
                    let a269 = v4;
                    let a270 = 0u64;
                    if nat_eq(a269, a270) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a271 = v2;
                            let a272 = v4;
                            nat_add(a271, a272)?
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m273 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Str) -> R<bool> {
    f0(p0, p1)
}
