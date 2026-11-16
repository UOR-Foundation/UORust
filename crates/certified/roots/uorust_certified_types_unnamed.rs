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

#[derive(Clone)]
pub enum Adt12 {
    C0(Str),
    C1(Str),
    C2,
    C3(Str),
    C4(Str, List<Adt13>),
    C5(Str),
    C6(Str),
    C7(Str),
    C8(Str),
    C9(Str),
    C10(Adt14),
}

#[derive(Clone)]
pub enum Adt13 {
    C0(u64, List<Str>),
}

#[derive(Clone)]
pub enum Adt14 {
    C0(Str, Str),
    C1(Str),
    C2(Str),
    C3,
    C4(Str),
    C5(Str, Str),
    C6(Str, List<Adt15>),
}

#[derive(Clone)]
pub enum Adt15 {
    C0(Str, Adt16),
}

#[derive(Clone)]
pub enum Adt16 {
    C0(u64, u64),
}

pub fn f0(v0: Adt0, v1: Str) -> R<Adt12> {
    let v2: Option<Str> = f1(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, h1) => {
            h1
        }
    }, f2(v0, v1.clone())?);
    Ok(Adt12::C1(f3(v2, v1)))
}

pub fn f1(p8: List<Adt5>, v1: Str) -> Option<Str> {
    let m7 = p8;
    {
        let mut m7 = m7;
        loop {
            let step = {
                let v0 = m7;
                let m2 = v0;
                match m2.uncons() {
                    None => {
                        Err::<List<Adt5>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a4 = match v2.clone() {
                            Adt5::C0(h3, _) => {
                                h3
                            }
                        };
                        let a5 = v1.clone();
                        if equal(a4, a5) {
                            Err::<List<Adt5>, Option<Str>>(Some(match v2 {
                                Adt5::C0(_, h6) => {
                                    h6
                                }
                            }))
                        } else {
                            Ok::<List<Adt5>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m7 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f2(v0: Adt0, v1: Str) -> R<Str> {
    let v2: Str = f4(v1)?;
    let m10 = f5(match v0 {
        Adt0::C0(_, _, h9, _, _, _, _, _, _, _, _, _, _) => {
            h9
        }
    }, v2.clone());
    match m10 {
        None => {
            Ok(v2)
        }
        Some(v3) => {
            Ok(v3)
        }
    }
}

pub fn f3(v0: Option<Str>, v1: Str) -> Str {
    let m11 = v0;
    match m11 {
        None => {
            v1
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f4(v0: Str) -> R<Str> {
    let m12 = f6(v0.clone())?;
    match m12 {
        None => {
            Ok(v0)
        }
        Some(v1) => {
            let v2: Str = match v1.clone() {
                Adt5::C0(h13, _) => {
                    h13
                }
            };
            let v3: Str = match v1 {
                Adt5::C0(_, h14) => {
                    h14
                }
            };
            let m15 = f7(v3, Str::lit(">"));
            match m15 {
                None => {
                    Ok(v0)
                }
                Some(v4) => {
                    if f8(f9(v4, Str::lit(","))?)? {
                        f10(v2)
                    } else {
                        Ok(v0)
                    }
                }
            }
        }
    }
}

pub fn f5(p22: List<Adt5>, v1: Str) -> Option<Str> {
    let m21 = p22;
    {
        let mut m21 = m21;
        loop {
            let step = {
                let v0 = m21;
                let m16 = v0;
                match m16.uncons() {
                    None => {
                        Err::<List<Adt5>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a18 = match v2.clone() {
                            Adt5::C0(h17, _) => {
                                h17
                            }
                        };
                        let a19 = v1.clone();
                        if equal(a18, a19) {
                            Err::<List<Adt5>, Option<Str>>(Some(match v2 {
                                Adt5::C0(_, h20) => {
                                    h20
                                }
                            }))
                        } else {
                            Ok::<List<Adt5>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m21 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f6(v0: Str) -> R<Option<Adt5>> {
    let m23 = f11(v0.clone(), Str::lit("<"))?;
    match m23 {
        None => {
            Ok(None::<Adt5>)
        }
        Some(v1) => {
            Ok(Some(Adt5::C0(f12(v0.clone(), 0u64, v1), f12(v0.clone(), {
                let a24 = v1;
                let a25 = f13(Str::lit("<"));
                nat_add(a24, a25)?
            }, f13(v0)))))
        }
    }
}

pub fn f7(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a26 = v0.clone();
        utf8_encode(a26)
    };
    let v3: u64 = {
        let a27 = v2.clone();
        length_bytes(a27)
    };
    let v4: u64 = f13(v1.clone());
    if f14(v0, v1) {
        Some(f15(v2, 0u64, {
            let a28 = v3;
            let a29 = v4;
            nat_sub(a28, a29)
        }))
    } else {
        None::<Str>
    }
}

pub fn f8(p32: List<Str>) -> R<bool> {
    let m31 = p32;
    {
        let mut m31 = m31;
        loop {
            let step = {
                let v0 = m31;
                let m30 = v0;
                match m30.uncons() {
                    None => {
                        Err::<List<Str>, R<bool>>(Ok(true))
                    }
                    Some((v1, v2)) => {
                        if f16(v1)? {
                            Ok::<List<Str>, R<bool>>(v2)
                        } else {
                            Err::<List<Str>, R<bool>>(Ok(false))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m31 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f9(v0: Str, v1: Str) -> R<List<Str>> {
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
        Ok(List::cons(Str::lit(""), {
            let a38 = f17(v0)?;
            let a39 = List::cons(Str::lit(""), List::<Str>::nil());
            append_list(a38, a39)
        }))
    } else {
        f18({
            let a41 = {
                let a40 = v2.clone();
                length_bytes(a40)
            };
            let a42 = 1u64;
            nat_add(a41, a42)?
        }, v2, v3, 0u64, 0u64)
    }
}

pub fn f10(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a43 = v0;
        utf8_encode(a43)
    };
    let v2: u64 = {
        let a44 = v1.clone();
        length_bytes(a44)
    };
    Ok(f15(v1.clone(), 0u64, f19(v2, v1, v2)?))
}

pub fn f11(v0: Str, v1: Str) -> R<Option<u64>> {
    let v2: Bytes = {
        let a45 = v0;
        utf8_encode(a45)
    };
    f20({
        let a47 = {
            let a46 = v2.clone();
            length_bytes(a46)
        };
        let a48 = 1u64;
        nat_add(a47, a48)?
    }, v2, {
        let a49 = v1;
        utf8_encode(a49)
    }, 0u64)
}

pub fn f12(v0: Str, v1: u64, v2: u64) -> Str {
    f15({
        let a50 = v0;
        utf8_encode(a50)
    }, v1, v2)
}

pub fn f13(v0: Str) -> u64 {
    let a52 = {
        let a51 = v0;
        utf8_encode(a51)
    };
    length_bytes(a52)
}

pub fn f14(v0: Str, v1: Str) -> bool {
    let v2: Bytes = {
        let a53 = v0;
        utf8_encode(a53)
    };
    let v3: Bytes = {
        let a54 = v1;
        utf8_encode(a54)
    };
    let v4: u64 = {
        let a55 = v2.clone();
        length_bytes(a55)
    };
    let v5: u64 = {
        let a56 = v3.clone();
        length_bytes(a56)
    };
    let a57 = v5;
    let a58 = v4;
    if nat_le(a57, a58) {
        f21(v2, {
            let a59 = v4;
            let a60 = v5;
            nat_sub(a59, a60)
        }, v3)
    } else {
        false
    }
}

pub fn f15(v0: Bytes, v1: u64, v2: u64) -> Str {
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

pub fn f16(v0: Str) -> R<bool> {
    let m69 = f22(f23(v0)?, Str::lit("'"));
    match m69 {
        None => {
            Ok(false)
        }
        Some(v1) => {
            let a70 = f24(v1, Str::lit("'"))?;
            Ok(bool_not(a70))
        }
    }
}

pub fn f17(v0: Str) -> R<List<Str>> {
    let v1: Bytes = {
        let a71 = v0;
        utf8_encode(a71)
    };
    f25({
        let a72 = v1.clone();
        length_bytes(a72)
    }, v1, 0u64)
}

pub fn f18(v0: u64, v1: Bytes, v2: Bytes, v3: u64, v4: u64) -> R<List<Str>> {
    let m73 = v0;
    if m73 == 0 {
        Ok(List::cons(f15(v1.clone(), v3, {
            let a74 = v1;
            length_bytes(a74)
        }), List::<Str>::nil()))
    } else {
        let v5 = m73 - 1;
        let a79 = {
            let a75 = v1.clone();
            length_bytes(a75)
        };
        let a80 = {
            let a77 = v4;
            let a78 = {
                let a76 = v2.clone();
                length_bytes(a76)
            };
            nat_add(a77, a78)?
        };
        if nat_lt(a79, a80) {
            Ok(List::cons(f15(v1.clone(), v3, {
                let a81 = v1;
                length_bytes(a81)
            }), List::<Str>::nil()))
        } else {
            if f21(v1.clone(), v4, v2.clone()) {
                Ok(List::cons(f15(v1.clone(), v3, v4), f18(v5, v1, v2.clone(), {
                    let a83 = v4;
                    let a84 = {
                        let a82 = v2.clone();
                        length_bytes(a82)
                    };
                    nat_add(a83, a84)?
                }, {
                    let a86 = v4;
                    let a87 = {
                        let a85 = v2;
                        length_bytes(a85)
                    };
                    nat_add(a86, a87)?
                })?))
            } else {
                f18(v5, v1, v2, v3, {
                    let a88 = v4;
                    let a89 = 1u64;
                    nat_add(a88, a89)?
                })
            }
        }
    }
}

pub fn f19(p96: u64, v1: Bytes, p97: u64) -> R<u64> {
    let m95 = (p96, p97);
    {
        let mut m95 = m95;
        loop {
            let step = {
                let (v0, v2) = m95;
                let m90 = v0;
                if m90 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m90 - 1;
                    let v4: u64 = f26(v1.clone(), v2)?;
                    let a91 = v4;
                    let a92 = 0u64;
                    if nat_eq(a91, a92) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a93 = v2;
                            let a94 = v4;
                            nat_sub(a93, a94)
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m95 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f20(p108: u64, v1: Bytes, v2: Bytes, p109: u64) -> R<Option<u64>> {
    let m107 = (p108, p109);
    {
        let mut m107 = m107;
        loop {
            let step = {
                let (v0, v3) = m107;
                let m98 = v0;
                if m98 == 0 {
                    Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                } else {
                    let v4 = m98 - 1;
                    let a103 = {
                        let a99 = v1.clone();
                        length_bytes(a99)
                    };
                    let a104 = {
                        let a101 = v3;
                        let a102 = {
                            let a100 = v2.clone();
                            length_bytes(a100)
                        };
                        nat_add(a101, a102)?
                    };
                    if nat_lt(a103, a104) {
                        Err::<(u64, u64), R<Option<u64>>>(Ok(None::<u64>))
                    } else {
                        if f21(v1.clone(), v3, v2.clone()) {
                            Err::<(u64, u64), R<Option<u64>>>(Ok(Some(v3)))
                        } else {
                            Ok::<(u64, u64), R<Option<u64>>>((v4, {
                                let a105 = v3;
                                let a106 = 1u64;
                                nat_add(a105, a106)?
                            }))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m107 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f21(v0: Bytes, v1: u64, v2: Bytes) -> bool {
    let m114 = {
        let a111 = v0;
        let a112 = v1;
        let a113 = {
            let a110 = v2.clone();
            length_bytes(a110)
        };
        slice_bytes(a111, a112, a113)
    };
    match m114 {
        None => {
            false
        }
        Some(v3) => {
            let a115 = v3;
            let a116 = v2;
            equal(a115, a116)
        }
    }
}

pub fn f22(v0: Str, v1: Str) -> Option<Str> {
    let v2: Bytes = {
        let a117 = v0;
        utf8_encode(a117)
    };
    let v3: Bytes = {
        let a118 = v1;
        utf8_encode(a118)
    };
    if f21(v2.clone(), 0u64, v3.clone()) {
        Some(f15(v2.clone(), {
            let a119 = v3;
            length_bytes(a119)
        }, {
            let a120 = v2;
            length_bytes(a120)
        }))
    } else {
        None::<Str>
    }
}

pub fn f23(v0: Str) -> R<Str> {
    f10(f27(v0)?)
}

pub fn f24(v0: Str, v1: Str) -> R<bool> {
    let m121 = f11(v0, v1)?;
    match m121 {
        None => {
            Ok(false)
        }
        Some(_) => {
            Ok(true)
        }
    }
}

pub fn f25(v0: u64, v1: Bytes, v2: u64) -> R<List<Str>> {
    let m122 = v0;
    if m122 == 0 {
        Ok(List::<Str>::nil())
    } else {
        let v3 = m122 - 1;
        let a124 = {
            let a123 = v1.clone();
            length_bytes(a123)
        };
        let a125 = v2;
        if nat_le(a124, a125) {
            Ok(List::<Str>::nil())
        } else {
            let v4: u64 = f28(f29(v1.clone(), v2)?);
            Ok(List::cons(f15(v1.clone(), v2, {
                let a126 = v2;
                let a127 = v4;
                nat_add(a126, a127)?
            }), f25(v3, v1, {
                let a128 = v2;
                let a129 = v4;
                nat_add(a128, a129)?
            })?))
        }
    }
}

pub fn f26(v0: Bytes, v1: u64) -> R<u64> {
    let a130 = v1;
    let a131 = 0u64;
    if nat_eq(a130, a131) {
        Ok(0u64)
    } else {
        if f30(f29(v0.clone(), {
            let a132 = v1;
            let a133 = 1u64;
            nat_sub(a132, a133)
        })?) {
            Ok(1u64)
        } else {
            let a134 = 2u64;
            let a135 = v1;
            if if nat_le(a134, a135) {
                let a138 = f31(v0.clone(), {
                    let a136 = v1;
                    let a137 = 2u64;
                    nat_sub(a136, a137)
                })?;
                let a139 = 2u64;
                nat_eq(a138, a139)
            } else {
                false
            } {
                Ok(2u64)
            } else {
                let a140 = 3u64;
                let a141 = v1;
                if if nat_le(a140, a141) {
                    let a144 = f31(v0, {
                        let a142 = v1;
                        let a143 = 3u64;
                        nat_sub(a142, a143)
                    })?;
                    let a145 = 3u64;
                    nat_eq(a144, a145)
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

pub fn f27(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a146 = v0;
        utf8_encode(a146)
    };
    let v2: u64 = {
        let a147 = v1.clone();
        length_bytes(a147)
    };
    Ok(f15(v1.clone(), f32(v2, v1, 0u64)?, v2))
}

pub fn f28(v0: u64) -> u64 {
    let a148 = v0;
    let a149 = 128u64;
    if nat_lt(a148, a149) {
        1u64
    } else {
        let a150 = v0;
        let a151 = 224u64;
        if nat_lt(a150, a151) {
            2u64
        } else {
            let a152 = v0;
            let a153 = 240u64;
            if nat_lt(a152, a153) {
                3u64
            } else {
                4u64
            }
        }
    }
}

pub fn f29(v0: Bytes, v1: u64) -> R<u64> {
    let m156 = {
        let a154 = v0;
        let a155 = v1;
        index_bytes(a154, a155)
    };
    match m156 {
        None => {
            Ok(256u64)
        }
        Some(v2) => {
            let v3: u8 = v2;
            let a201 = {
                let a177 = {
                    let a165 = {
                        let a159 = {
                            let a157 = v3;
                            let a158 = 128u8;
                            fixed_u8::bit_and(a157, a158)
                        };
                        let a160 = 128u8;
                        if equal(a159, a160) {
                            128u64
                        } else {
                            0u64
                        }
                    };
                    let a166 = {
                        let a163 = {
                            let a161 = v3;
                            let a162 = 64u8;
                            fixed_u8::bit_and(a161, a162)
                        };
                        let a164 = 64u8;
                        if equal(a163, a164) {
                            64u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a165, a166)?
                };
                let a178 = {
                    let a175 = {
                        let a169 = {
                            let a167 = v3;
                            let a168 = 32u8;
                            fixed_u8::bit_and(a167, a168)
                        };
                        let a170 = 32u8;
                        if equal(a169, a170) {
                            32u64
                        } else {
                            0u64
                        }
                    };
                    let a176 = {
                        let a173 = {
                            let a171 = v3;
                            let a172 = 16u8;
                            fixed_u8::bit_and(a171, a172)
                        };
                        let a174 = 16u8;
                        if equal(a173, a174) {
                            16u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a175, a176)?
                };
                nat_add(a177, a178)?
            };
            let a202 = {
                let a199 = {
                    let a187 = {
                        let a181 = {
                            let a179 = v3;
                            let a180 = 8u8;
                            fixed_u8::bit_and(a179, a180)
                        };
                        let a182 = 8u8;
                        if equal(a181, a182) {
                            8u64
                        } else {
                            0u64
                        }
                    };
                    let a188 = {
                        let a185 = {
                            let a183 = v3;
                            let a184 = 4u8;
                            fixed_u8::bit_and(a183, a184)
                        };
                        let a186 = 4u8;
                        if equal(a185, a186) {
                            4u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a187, a188)?
                };
                let a200 = {
                    let a197 = {
                        let a191 = {
                            let a189 = v3;
                            let a190 = 2u8;
                            fixed_u8::bit_and(a189, a190)
                        };
                        let a192 = 2u8;
                        if equal(a191, a192) {
                            2u64
                        } else {
                            0u64
                        }
                    };
                    let a198 = {
                        let a195 = {
                            let a193 = v3;
                            let a194 = 1u8;
                            fixed_u8::bit_and(a193, a194)
                        };
                        let a196 = 1u8;
                        if equal(a195, a196) {
                            1u64
                        } else {
                            0u64
                        }
                    };
                    nat_add(a197, a198)?
                };
                nat_add(a199, a200)?
            };
            nat_add(a201, a202)
        }
    }
}

pub fn f30(v0: u64) -> bool {
    let a203 = v0;
    let a204 = 32u64;
    if nat_eq(a203, a204) {
        true
    } else {
        let a205 = 9u64;
        let a206 = v0;
        if nat_le(a205, a206) {
            let a207 = v0;
            let a208 = 13u64;
            nat_le(a207, a208)
        } else {
            false
        }
    }
}

pub fn f31(v0: Bytes, v1: u64) -> R<u64> {
    let v2: u64 = f29(v0.clone(), v1)?;
    let v3: u64 = f29(v0.clone(), {
        let a209 = v1;
        let a210 = 1u64;
        nat_add(a209, a210)?
    })?;
    let v4: u64 = f29(v0, {
        let a211 = v1;
        let a212 = 2u64;
        nat_add(a211, a212)?
    })?;
    if f30(v2) {
        Ok(1u64)
    } else {
        let a213 = v2;
        let a214 = 194u64;
        if nat_eq(a213, a214) {
            let a215 = v3;
            let a216 = 133u64;
            if if nat_eq(a215, a216) {
                true
            } else {
                let a217 = v3;
                let a218 = 160u64;
                nat_eq(a217, a218)
            } {
                Ok(2u64)
            } else {
                Ok(0u64)
            }
        } else {
            let a219 = v2;
            let a220 = 225u64;
            if nat_eq(a219, a220) {
                let a221 = v3;
                let a222 = 154u64;
                if if nat_eq(a221, a222) {
                    let a223 = v4;
                    let a224 = 128u64;
                    nat_eq(a223, a224)
                } else {
                    false
                } {
                    Ok(3u64)
                } else {
                    Ok(0u64)
                }
            } else {
                let a225 = v2;
                let a226 = 226u64;
                if nat_eq(a225, a226) {
                    let a227 = v3;
                    let a228 = 128u64;
                    if nat_eq(a227, a228) {
                        let a229 = 128u64;
                        let a230 = v4;
                        if if if nat_le(a229, a230) {
                            let a231 = v4;
                            let a232 = 138u64;
                            nat_le(a231, a232)
                        } else {
                            false
                        } {
                            true
                        } else {
                            let a233 = v4;
                            let a234 = 168u64;
                            if nat_eq(a233, a234) {
                                true
                            } else {
                                let a235 = v4;
                                let a236 = 169u64;
                                if nat_eq(a235, a236) {
                                    true
                                } else {
                                    let a237 = v4;
                                    let a238 = 175u64;
                                    nat_eq(a237, a238)
                                }
                            }
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    } else {
                        let a239 = v3;
                        let a240 = 129u64;
                        if if nat_eq(a239, a240) {
                            let a241 = v4;
                            let a242 = 159u64;
                            nat_eq(a241, a242)
                        } else {
                            false
                        } {
                            Ok(3u64)
                        } else {
                            Ok(0u64)
                        }
                    }
                } else {
                    let a243 = v2;
                    let a244 = 227u64;
                    if if nat_eq(a243, a244) {
                        let a245 = v3;
                        let a246 = 128u64;
                        if nat_eq(a245, a246) {
                            let a247 = v4;
                            let a248 = 128u64;
                            nat_eq(a247, a248)
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

pub fn f32(p255: u64, v1: Bytes, p256: u64) -> R<u64> {
    let m254 = (p255, p256);
    {
        let mut m254 = m254;
        loop {
            let step = {
                let (v0, v2) = m254;
                let m249 = v0;
                if m249 == 0 {
                    Err::<(u64, u64), R<u64>>(Ok(v2))
                } else {
                    let v3 = m249 - 1;
                    let v4: u64 = f31(v1.clone(), v2)?;
                    let a250 = v4;
                    let a251 = 0u64;
                    if nat_eq(a250, a251) {
                        Err::<(u64, u64), R<u64>>(Ok(v2))
                    } else {
                        Ok::<(u64, u64), R<u64>>((v3, {
                            let a252 = v2;
                            let a253 = v4;
                            nat_add(a252, a253)?
                        }))
                    }
                }
            };
            match step {
                Ok(next) => { m254 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0, p1: Str) -> R<Adt12> {
    f0(p0, p1)
}
