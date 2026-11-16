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

pub fn f0(v0: Adt0, v1: Str) -> R<Str> {
    let v2: Str = f1(v1)?;
    let m2 = f2(match v0 {
        Adt0::C0(_, _, h1, _, _, _, _, _, _, _, _, _, _) => {
            h1
        }
    }, v2.clone());
    match m2 {
        None => {
            Ok(v2)
        }
        Some(v3) => {
            Ok(v3)
        }
    }
}

pub fn f1(v0: Str) -> R<Str> {
    let m3 = f3(v0.clone())?;
    match m3 {
        None => {
            Ok(v0)
        }
        Some(v1) => {
            let v2: Str = match v1.clone() {
                Adt5::C0(h4, _) => {
                    h4
                }
            };
            let v3: Str = match v1 {
                Adt5::C0(_, h5) => {
                    h5
                }
            };
            let m6 = f4(v3, Str::lit(">"));
            match m6 {
                None => {
                    Ok(v0)
                }
                Some(v4) => {
                    if f5(f6(v4, Str::lit(","))?)? {
                        f7(v2)
                    } else {
                        Ok(v0)
                    }
                }
            }
        }
    }
}

pub fn f2(p13: List<Adt5>, v1: Str) -> Option<Str> {
    let m12 = p13;
    {
        let mut m12 = m12;
        loop {
            let step = {
                let v0 = m12;
                let m7 = v0;
                match m7.uncons() {
                    None => {
                        Err::<List<Adt5>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a9 = match v2.clone() {
                            Adt5::C0(h8, _) => {
                                h8
                            }
                        };
                        let a10 = v1.clone();
                        if equal(a9, a10) {
                            Err::<List<Adt5>, Option<Str>>(Some(match v2 {
                                Adt5::C0(_, h11) => {
                                    h11
                                }
                            }))
                        } else {
                            Ok::<List<Adt5>, Option<Str>>(v3)
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

pub fn f3(v0: Str) -> R<Option<Adt5>> {
    let m14 = f8(v0.clone(), Str::lit("<"))?;
    match m14 {
        None => {
            Ok(None::<Adt5>)
        }
        Some(v1) => {
            Ok(Some(Adt5::C0(f9(v0.clone(), 0u64, v1), f9(v0.clone(), {
                let a15 = v1;
                let a16 = f10(Str::lit("<"));
                nat_add(a15, a16)?
            }, f10(v0)))))
        }
    }
}

pub fn f4(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a17 = v0.clone();
        utf8_encode(a17)
    };
    let v3: u64 = {
        let a18 = v2.clone();
        length_bytes(a18)
    };
    let v4: u64 = f10(v1.clone());
    if f11(v0, v1) {
        Some(f12(v2, 0u64, {
            let a19 = v3;
            let a20 = v4;
            nat_sub(a19, a20)
        }))
    } else {
        None::<Str>
    }
}

pub fn f5(p23: List<Str>) -> R<bool> {
    let m22 = p23;
    {
        let mut m22 = m22;
        loop {
            let step = {
                let v0 = m22;
                let m21 = v0;
                match m21.uncons() {
                    None => {
                        Err::<List<Str>, R<bool>>(Ok(true))
                    }
                    Some((v1, v2)) => {
                        if f13(v1)? {
                            Ok::<List<Str>, R<bool>>(v2)
                        } else {
                            Err::<List<Str>, R<bool>>(Ok(false))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m22 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f6(v0: Str, v1: Str) -> R<List<Str>> {
    let v2: Bytes = {
        let a24 = v0.clone();
        utf8_encode(a24)
    };
    let v3: Bytes = {
        let a25 = v1;
        utf8_encode(a25)
    };
    let a27 = {
        let a26 = v3.clone();
        length_bytes(a26)
    };
    let a28 = 0u64;
    if nat_eq(a27, a28) {
        Ok(List::cons(Str::lit(""), {
            let a29 = f14(v0)?;
            let a30 = List::cons(Str::lit(""), List::<Str>::nil());
            append_list(a29, a30)
        }))
    } else {
        f15({
            let a32 = {
                let a31 = v2.clone();
                length_bytes(a31)
            };
            let a33 = 1u64;
            nat_add(a32, a33)?
        }, v2, v3, 0u64, 0u64)
    }
}

pub fn f7(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a34 = v0;
        utf8_encode(a34)
    };
    let v2: u64 = {
        let a35 = v1.clone();
        length_bytes(a35)
    };
    Ok(f12(v1.clone(), 0u64, f16(v2, v1, v2)?))
}

pub fn f8(v0: Str, v1: Str) -> R<Option<u64>> {
    let v2: Bytes = {
        let a36 = v0;
        utf8_encode(a36)
    };
    f17({
        let a38 = {
            let a37 = v2.clone();
            length_bytes(a37)
        };
        let a39 = 1u64;
        nat_add(a38, a39)?
    }, v2, {
        let a40 = v1;
        utf8_encode(a40)
    }, 0u64)
}

pub fn f9(v0: Str, v1: u64, v2: u64) -> Str {
    f12({
        let a41 = v0;
        utf8_encode(a41)
    }, v1, v2)
}

pub fn f10(v0: Str) -> u64 {
    let a43 = {
        let a42 = v0;
        utf8_encode(a42)
    };
    length_bytes(a43)
}

pub fn f11(v0: Str, v1: Str) -> bool {
    let v2: Bytes = {
        let a44 = v0;
        utf8_encode(a44)
    };
    let v3: Bytes = {
        let a45 = v1;
        utf8_encode(a45)
    };
    let v4: u64 = {
        let a46 = v2.clone();
        length_bytes(a46)
    };
    let v5: u64 = {
        let a47 = v3.clone();
        length_bytes(a47)
    };
    let a48 = v5;
    let a49 = v4;
    if nat_le(a48, a49) {
        f18(v2, {
            let a50 = v4;
            let a51 = v5;
            nat_sub(a50, a51)
        }, v3)
    } else {
        false
    }
}

pub fn f12(v0: Bytes, v1: u64, v2: u64) -> Str {
    let m57 = {
        let a54 = v0;
        let a55 = v1;
        let a56 = {
            let a52 = v2;
            let a53 = v1;
            nat_sub(a52, a53)
        };
        slice_bytes(a54, a55, a56)
    };
    match m57 {
        None => {
            Str::lit("")
        }
        Some(v3) => {
            let m59 = {
                let a58 = v3;
                utf8_decode(a58)
            };
            match m59 {
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

pub fn f13(v0: Str) -> R<bool> {
    let m60 = f19(f20(v0)?, Str::lit("'"));
    match m60 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            let a61 = f21(v1, Str::lit("'"))?;
            Ok(bool_not(a61))
        }
    }
}

pub fn f14(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a62 = v0;
        utf8_encode(a62)
    };
    f22({
        let a63 = v1.clone();
        length_bytes(a63)
    }, v1, 0u64)
}

pub fn f15(v0: u64, v1: Bytes, v2: Bytes, v3: u64, v4: u64) -> R<List<Str>> {
    let m64 = v0;
    if m64 == 0 {
        Ok(List::cons(f12(v1.clone(), v3, {
            let a65 = v1;
            length_bytes(a65)
        }), List::<Str>::nil()))
    } else {
        let v5 = m64 - 1;
        let a70 = {
            let a66 = v1.clone();
            length_bytes(a66)
        };
        let a71 = {
            let a68 = v4;
            let a69 = {
                let a67 = v2.clone();
                length_bytes(a67)
            };
            nat_add(a68, a69)?
        };
        if nat_lt(a70, a71) {
            Ok(List::cons(f12(v1.clone(), v3, {
                let a72 = v1;
                length_bytes(a72)
            }), List::<Str>::nil()))
        } else {
            if f18(v1.clone(), v4, v2.clone()) {
                Ok(List::cons(f12(v1.clone(), v3, v4), f15(v5, v1, v2.clone(), {
                    let a74 = v4;
                    let a75 = {
                        let a73 = v2.clone();
                        length_bytes(a73)
                    };
                    nat_add(a74, a75)?
                }, {
                    let a77 = v4;
                    let a78 = {
                        let a76 = v2;
                        length_bytes(a76)
                    };
                    nat_add(a77, a78)?
                })?))
            } else {
                f15(v5, v1, v2, v3, {
                    let a79 = v4;
                    let a80 = 1u64;
                    nat_add(a79, a80)?
                })
            }
        }
    }
}

pub fn f16(p87: u64, v1: Bytes, p88: u64) -> R<u64> {
    let m86 = (p87, p88);
    {
        let mut m86 = m86;
        loop {
            let step = {
                let (v0, v2) = m86;
                let m81 = v0;
                if m81 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m81 - 1;
                    let v4: u64 = f23(v1.clone(), v2)?;
                    let a82 = v4;
                    let a83 = 0u64;
                    if nat_eq(a82, a83) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a84 = v2;
                            let a85 = v4;
                            nat_sub(a84, a85)
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m86 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f17(p99: u64, v1: Bytes, v2: Bytes, p100: u64) -> R<Option<u64>> {
    let m98 = (p99, p100);
    {
        let mut m98 = m98;
        loop {
            let step = {
                let (v0, v3) = m98;
                let m89 = v0;
                if m89 == 0 {
                    Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                } else {
                    let v4 = m89 - 1;
                    let a94 = {
                        let a90 = v1.clone();
                        length_bytes(a90)
                    };
                    let a95 = {
                        let a92 = v3;
                        let a93 = {
                            let a91 = v2.clone();
                            length_bytes(a91)
                        };
                        nat_add(a92, a93)?
                    };
                    if nat_lt(a94, a95) {
                        Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                    } else {
                        if f18(v1.clone(), v3, v2.clone()) {
                            Err::<(u64, u64), R<Option<u64>>>(Ok(Some(v3)))
                        } else {
                            Ok::<(u64, u64), R<Option<u64>>>((v4, {
                                let a96 = v3;
                                let a97 = 1u64;
                                nat_add(a96, a97)?
                            }))
                        }
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

pub fn f18(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m105 = {
        let a102 = v0;
        let a103 = v1;
        let a104 = {
            let a101 = v2.clone();
            length_bytes(a101)
        };
        slice_bytes(a102, a103, a104)
    };
    match m105 {
        None => {
            false
        }
        Some(v3) => {
            let a106 = v3;
            let a107 = v2;
            equal(a106, a107)
        }
    }
}

pub fn f19(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a108 = v0;
        utf8_encode(a108)
    };
    let v3: Bytes = {
        let a109 = v1;
        utf8_encode(a109)
    };
    if f18(v2.clone(), 0u64, v3.clone()) {
        Some(f12(v2.clone(), {
            let a110 = v3;
            length_bytes(a110)
        }, {
            let a111 = v2;
            length_bytes(a111)
        }))
    } else {
        None::<Str>
    }
}

pub fn f20(v0: Str) -> R<Str> {
    f7(f24(v0)?)
}

pub fn f21(v0: Str, v1: Str) -> R<bool> {
    let m112 = f8(v0, v1)?;
    match m112 {
        None => {
            Ok(false)
        }
        Some(_) => {
            Ok(true)
        }
    }
}

pub fn f22(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m113 = v0;
    if m113 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m113 - 1;
        let a115 = {
            let a114 = v1.clone();
            length_bytes(a114)
        };
        let a116 = v2;
        if nat_le(a115, a116) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f25(f26(v1.clone(), v2)?);
            Ok(List::cons(f12(v1.clone(), v2, {
                let a117 = v2;
                let a118 = v4;
                nat_add(a117, a118)?
            }), f22(v3, v1, {
                let a119 = v2;
                let a120 = v4;
                nat_add(a119, a120)?
            })?))
        }
    }
}

pub fn f23(v0: Bytes, v1: u64) -> R<u64> {
    let a121 = v1;
    let a122 = 0u64;
    if nat_eq(a121, a122) {
        Ok(0u64)
    } else {
        if f27(f26(v0.clone(), {
            let a123 = v1;
            let a124 = 1u64;
            nat_sub(a123, a124)
        })?) {
            Ok(1u64)
        } else {
            let a125 = 2u64;
            let a126 = v1;
            if if nat_le(a125, a126) {
                let a129 = f28(v0.clone(), {
                    let a127 = v1;
                    let a128 = 2u64;
                    nat_sub(a127, a128)
                })?;
                let a130 = 2u64;
                nat_eq(a129, a130)
            } else {
                false
            } {
                Ok(2u64)
            } else {
                let a131 = 3u64;
                let a132 = v1;
                if if nat_le(a131, a132) {
                    let a135 = f28(v0, {
                        let a133 = v1;
                        let a134 = 3u64;
                        nat_sub(a133, a134)
                    })?;
                    let a136 = 3u64;
                    nat_eq(a135, a136)
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

pub fn f24(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a137 = v0;
        utf8_encode(a137)
    };
    let v2: u64 = {
        let a138 = v1.clone();
        length_bytes(a138)
    };
    Ok(f12(v1.clone(), f29(v2, v1, 0u64)?, v2))
}

pub fn f25(v0: u64) -> u64 {
    let a139 = v0;
    let a140 = 128u64;
    if nat_lt(a139, a140) {
        1u64
    } else {
        let a141 = v0;
        let a142 = 224u64;
        if nat_lt(a141, a142) {
            2u64
        } else {
            let a143 = v0;
            let a144 = 240u64;
            if nat_lt(a143, a144) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f26(v0: Bytes, v1: u64) -> R<u64> {
    let m147 = {
        let a145 = v0;
        let a146 = v1;
        index_bytes(a145, a146)
    };
    match m147 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a192 = {
                let a168 = {
                    let a156 = {
                        let a150 = {
                            let a148 = v3;
                            let a149 = 128u8;
                            fixed_u8::bit_and(a148, a149)
                        };
                        let a151 = 128u8;
                        if equal(a150, a151) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a157 = {
                        let a154 = {
                            let a152 = v3;
                            let a153 = 64u8;
                            fixed_u8::bit_and(a152, a153)
                        };
                        let a155 = 64u8;
                        if equal(a154, a155) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a156, a157)?
                };
                let a169 = {
                    let a166 = {
                        let a160 = {
                            let a158 = v3;
                            let a159 = 32u8;
                            fixed_u8::bit_and(a158, a159)
                        };
                        let a161 = 32u8;
                        if equal(a160, a161) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a167 = {
                        let a164 = {
                            let a162 = v3;
                            let a163 = 16u8;
                            fixed_u8::bit_and(a162, a163)
                        };
                        let a165 = 16u8;
                        if equal(a164, a165) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a166, a167)?
                };
                nat_add(a168, a169)?
            };
            let a193 = {
                let a190 = {
                    let a178 = {
                        let a172 = {
                            let a170 = v3;
                            let a171 = 8u8;
                            fixed_u8::bit_and(a170, a171)
                        };
                        let a173 = 8u8;
                        if equal(a172, a173) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a179 = {
                        let a176 = {
                            let a174 = v3;
                            let a175 = 4u8;
                            fixed_u8::bit_and(a174, a175)
                        };
                        let a177 = 4u8;
                        if equal(a176, a177) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a178, a179)?
                };
                let a191 = {
                    let a188 = {
                        let a182 = {
                            let a180 = v3;
                            let a181 = 2u8;
                            fixed_u8::bit_and(a180, a181)
                        };
                        let a183 = 2u8;
                        if equal(a182, a183) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a189 = {
                        let a186 = {
                            let a184 = v3;
                            let a185 = 1u8;
                            fixed_u8::bit_and(a184, a185)
                        };
                        let a187 = 1u8;
                        if equal(a186, a187) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a188, a189)?
                };
                nat_add(a190, a191)?
            };
            nat_add(a192, a193)
        }
    }
}

pub fn f27(v0: u64) -> bool {
    let a194 = v0;
    let a195 = 32u64;
    if nat_eq(a194, a195) {
        true
    } else {
        let a196 = 9u64;
        let a197 = v0;
        if nat_le(a196, a197) {
            let a198 = v0;
            let a199 = 13u64;
            nat_le(a198, a199)
        } else {
            false
        }
    }
}

pub fn f28(v0: Bytes, v1: u64) -> R<u64> {
    let v2: u64 = f26(v0.clone(), v1)?;
    let v3: u64 = f26(v0.clone(), {
        let a200 = v1;
        let a201 = 1u64;
        nat_add(a200, a201)?
    })?;
    let v4: u64 = f26(v0, {
        let a202 = v1;
        let a203 = 2u64;
        nat_add(a202, a203)?
    })?;
    if f27(v2) {
        Ok(1u64)
    } else {
        let a204 = v2;
        let a205 = 194u64;
        if nat_eq(a204, a205) {
            let a206 = v3;
            let a207 = 133u64;
            if if nat_eq(a206, a207) {
                true
            } else {
                let a208 = v3;
                let a209 = 160u64;
                nat_eq(a208, a209)
            } {
                Ok(2u64)
            } else {
                Ok(0u64)
            }
        } else {
            let a210 = v2;
            let a211 = 225u64;
            if nat_eq(a210, a211) {
                let a212 = v3;
                let a213 = 154u64;
                if if nat_eq(a212, a213) {
                    let a214 = v4;
                    let a215 = 128u64;
                    nat_eq(a214, a215)
                } else {
                    false
                } {
                    Ok(3u64)
                } else {
                    Ok(0u64)
                }
            } else {
                let a216 = v2;
                let a217 = 226u64;
                if nat_eq(a216, a217) {
                    let a218 = v3;
                    let a219 = 128u64;
                    if nat_eq(a218, a219) {
                        let a220 = 128u64;
                        let a221 = v4;
                        if if if nat_le(a220, a221) {
                            let a222 = v4;
                            let a223 = 138u64;
                            nat_le(a222, a223)
                        } else {
                            false
                        } {
                            true
                        } else {
                            let a224 = v4;
                            let a225 = 168u64;
                            if nat_eq(a224, a225) {
                                true
                            } else {
                                let a226 = v4;
                                let a227 = 169u64;
                                if nat_eq(a226, a227) {
                                    true
                                } else {
                                    let a228 = v4;
                                    let a229 = 175u64;
                                    nat_eq(a228, a229)
                                }
                            }
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    } else {
                        let a230 = v3;
                        let a231 = 129u64;
                        if if nat_eq(a230, a231) {
                            let a232 = v4;
                            let a233 = 159u64;
                            nat_eq(a232, a233)
                        } else {
                            false
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    }
                } else {
                    let a234 = v2;
                    let a235 = 227u64;
                    if if nat_eq(a234, a235) {
                        let a236 = v3;
                        let a237 = 128u64;
                        if nat_eq(a236, a237) {
                            let a238 = v4;
                            let a239 = 128u64;
                            nat_eq(a238, a239)
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

pub fn f29(p246: u64, v1: Bytes, p247: u64) -> R<u64> {
    let m245 = (p246, p247);
    {
        let mut m245 = m245;
        loop {
            let step = {
                let (v0, v2) = m245;
                let m240 = v0;
                if m240 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m240 - 1;
                    let v4: u64 = f28(v1.clone(), v2)?;
                    let a241 = v4;
                    let a242 = 0u64;
                    if nat_eq(a241, a242) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a243 = v2;
                            let a244 = v4;
                            nat_add(a243, a244)?
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m245 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Str) -> R<Str> {
    f0(p0, p1)
}
