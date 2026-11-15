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
    C0(u32, u32, u32, u32, u32, u32, u32, u32),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Adt0, List<u32>, u64),
}

pub fn f0(v0: List<u64>) -> R<Str> {
    Ok(f1(f2(v0)?))
}

pub fn f1(v0: List<u64>) -> Str {
    let m2 = {
        let a1 = f3(v0);
        utf8_decode(a1)
    };
    match m2 {
        None => {
            Str::lit("")
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f2(v0: List<u64>) -> R<List<u64>> {
    let v1: Bytes = f4(f5(v0))?;
    let v2: Bytes = f6(f7({
        let a4 = {
            let a3 = v1.clone();
            length_bytes(a3)
        };
        let a5 = f8();
        let a6 = 0u64;
        nat_quot(a4, a5, a6)
    }, f9(), v1, 0u64)?)?;
    f10({
        let a7 = v2.clone();
        length_bytes(a7)
    }, v2, 0u64)
}

pub fn f3(v0: List<u64>) -> Bytes {
    let m8 = v0;
    match m8.uncons() {
        None => {
            Bytes::lit(&[])
        }
        Some((v1, v2)) => {
            let a17 = {
                let a15 = f11({
                    let a9 = v1;
                    let a10 = 16u64;
                    let a11 = 0u64;
                    nat_quot(a9, a10, a11)
                });
                let a16 = f11({
                    let a12 = v1;
                    let a13 = 16u64;
                    let a14 = 0u64;
                    nat_rem(a12, a13, a14)
                });
                append_bytes(a15, a16)
            };
            let a18 = f3(v2);
            append_bytes(a17, a18)
        }
    }
}

pub fn f4(v0: Bytes) -> R<Bytes> {
    let v1: u64 = {
        let a19 = v0.clone();
        length_bytes(a19)
    };
    let v2: u64 = {
        let a20 = f8();
        let a21 = f12();
        nat_sub(a20, a21)
    };
    let v3: u64 = {
        let a24 = {
            let a22 = v1;
            let a23 = 1u64;
            nat_add(a22, a23)?
        };
        let a25 = f8();
        let a26 = 0u64;
        nat_rem(a24, a25, a26)
    };
    let a41 = {
        let a37 = {
            let a27 = v0;
            let a28 = Bytes::lit(&[0x80]);
            append_bytes(a27, a28)
        };
        let a38 = f13({
            let a29 = v3;
            let a30 = v2;
            if nat_le(a29, a30) {
                let a31 = v2;
                let a32 = v3;
                nat_sub(a31, a32)
            } else {
                let a35 = {
                    let a33 = f8();
                    let a34 = v2;
                    nat_add(a33, a34)?
                };
                let a36 = v3;
                nat_sub(a35, a36)
            }
        });
        append_bytes(a37, a38)
    };
    let a42 = f14(f12(), {
        let a39 = v1;
        let a40 = 8u64;
        nat_mul(a39, a40)?
    });
    Ok(append_bytes(a41, a42))
}

pub fn f5(v0: List<u64>) -> Bytes {
    let m43 = v0;
    match m43.uncons() {
        None => {
            Bytes::lit(&[])
        }
        Some((v1, v2)) => {
            let a44 = f15(v1);
            let a45 = f5(v2);
            append_bytes(a44, a45)
        }
    }
}

pub fn f6(v0: Adt0) -> R<Bytes> {
    let a1588 = {
        let a1395 = {
            let a1202 = {
                let a1009 = {
                    let a816 = {
                        let a623 = {
                            let a430 = {
                                let a237 = Bytes::lit(&[]);
                                let a238 = f14(4u64, {
                                    let v1: u32 = match v0.clone() {
                                        Adt0::C0(h46, _, _, _, _, _, _, _) => {
                                            h46
                                        }
                                    };
                                    let a235 = {
                                        let a139 = {
                                            let a91 = {
                                                let a67 = {
                                                    let a55 = {
                                                        let a49 = {
                                                            let a47 = v1;
                                                            let a48 = 2147483648u32;
                                                            fixed_u32::bit_and(a47, a48)
                                                        };
                                                        let a50 = 2147483648u32;
                                                        if equal(a49, a50) {
                                                            2147483648u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a56 = {
                                                        let a53 = {
                                                            let a51 = v1;
                                                            let a52 = 1073741824u32;
                                                            fixed_u32::bit_and(a51, a52)
                                                        };
                                                        let a54 = 1073741824u32;
                                                        if equal(a53, a54) {
                                                            1073741824u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a55, a56)?
                                                };
                                                let a68 = {
                                                    let a65 = {
                                                        let a59 = {
                                                            let a57 = v1;
                                                            let a58 = 536870912u32;
                                                            fixed_u32::bit_and(a57, a58)
                                                        };
                                                        let a60 = 536870912u32;
                                                        if equal(a59, a60) {
                                                            536870912u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a66 = {
                                                        let a63 = {
                                                            let a61 = v1;
                                                            let a62 = 268435456u32;
                                                            fixed_u32::bit_and(a61, a62)
                                                        };
                                                        let a64 = 268435456u32;
                                                        if equal(a63, a64) {
                                                            268435456u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a65, a66)?
                                                };
                                                nat_add(a67, a68)?
                                            };
                                            let a92 = {
                                                let a89 = {
                                                    let a77 = {
                                                        let a71 = {
                                                            let a69 = v1;
                                                            let a70 = 134217728u32;
                                                            fixed_u32::bit_and(a69, a70)
                                                        };
                                                        let a72 = 134217728u32;
                                                        if equal(a71, a72) {
                                                            134217728u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a78 = {
                                                        let a75 = {
                                                            let a73 = v1;
                                                            let a74 = 67108864u32;
                                                            fixed_u32::bit_and(a73, a74)
                                                        };
                                                        let a76 = 67108864u32;
                                                        if equal(a75, a76) {
                                                            67108864u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a77, a78)?
                                                };
                                                let a90 = {
                                                    let a87 = {
                                                        let a81 = {
                                                            let a79 = v1;
                                                            let a80 = 33554432u32;
                                                            fixed_u32::bit_and(a79, a80)
                                                        };
                                                        let a82 = 33554432u32;
                                                        if equal(a81, a82) {
                                                            33554432u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a88 = {
                                                        let a85 = {
                                                            let a83 = v1;
                                                            let a84 = 16777216u32;
                                                            fixed_u32::bit_and(a83, a84)
                                                        };
                                                        let a86 = 16777216u32;
                                                        if equal(a85, a86) {
                                                            16777216u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a87, a88)?
                                                };
                                                nat_add(a89, a90)?
                                            };
                                            nat_add(a91, a92)?
                                        };
                                        let a140 = {
                                            let a137 = {
                                                let a113 = {
                                                    let a101 = {
                                                        let a95 = {
                                                            let a93 = v1;
                                                            let a94 = 8388608u32;
                                                            fixed_u32::bit_and(a93, a94)
                                                        };
                                                        let a96 = 8388608u32;
                                                        if equal(a95, a96) {
                                                            8388608u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a102 = {
                                                        let a99 = {
                                                            let a97 = v1;
                                                            let a98 = 4194304u32;
                                                            fixed_u32::bit_and(a97, a98)
                                                        };
                                                        let a100 = 4194304u32;
                                                        if equal(a99, a100) {
                                                            4194304u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a101, a102)?
                                                };
                                                let a114 = {
                                                    let a111 = {
                                                        let a105 = {
                                                            let a103 = v1;
                                                            let a104 = 2097152u32;
                                                            fixed_u32::bit_and(a103, a104)
                                                        };
                                                        let a106 = 2097152u32;
                                                        if equal(a105, a106) {
                                                            2097152u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a112 = {
                                                        let a109 = {
                                                            let a107 = v1;
                                                            let a108 = 1048576u32;
                                                            fixed_u32::bit_and(a107, a108)
                                                        };
                                                        let a110 = 1048576u32;
                                                        if equal(a109, a110) {
                                                            1048576u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a111, a112)?
                                                };
                                                nat_add(a113, a114)?
                                            };
                                            let a138 = {
                                                let a135 = {
                                                    let a123 = {
                                                        let a117 = {
                                                            let a115 = v1;
                                                            let a116 = 524288u32;
                                                            fixed_u32::bit_and(a115, a116)
                                                        };
                                                        let a118 = 524288u32;
                                                        if equal(a117, a118) {
                                                            524288u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a124 = {
                                                        let a121 = {
                                                            let a119 = v1;
                                                            let a120 = 262144u32;
                                                            fixed_u32::bit_and(a119, a120)
                                                        };
                                                        let a122 = 262144u32;
                                                        if equal(a121, a122) {
                                                            262144u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a123, a124)?
                                                };
                                                let a136 = {
                                                    let a133 = {
                                                        let a127 = {
                                                            let a125 = v1;
                                                            let a126 = 131072u32;
                                                            fixed_u32::bit_and(a125, a126)
                                                        };
                                                        let a128 = 131072u32;
                                                        if equal(a127, a128) {
                                                            131072u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a134 = {
                                                        let a131 = {
                                                            let a129 = v1;
                                                            let a130 = 65536u32;
                                                            fixed_u32::bit_and(a129, a130)
                                                        };
                                                        let a132 = 65536u32;
                                                        if equal(a131, a132) {
                                                            65536u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a133, a134)?
                                                };
                                                nat_add(a135, a136)?
                                            };
                                            nat_add(a137, a138)?
                                        };
                                        nat_add(a139, a140)?
                                    };
                                    let a236 = {
                                        let a233 = {
                                            let a185 = {
                                                let a161 = {
                                                    let a149 = {
                                                        let a143 = {
                                                            let a141 = v1;
                                                            let a142 = 32768u32;
                                                            fixed_u32::bit_and(a141, a142)
                                                        };
                                                        let a144 = 32768u32;
                                                        if equal(a143, a144) {
                                                            32768u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a150 = {
                                                        let a147 = {
                                                            let a145 = v1;
                                                            let a146 = 16384u32;
                                                            fixed_u32::bit_and(a145, a146)
                                                        };
                                                        let a148 = 16384u32;
                                                        if equal(a147, a148) {
                                                            16384u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a149, a150)?
                                                };
                                                let a162 = {
                                                    let a159 = {
                                                        let a153 = {
                                                            let a151 = v1;
                                                            let a152 = 8192u32;
                                                            fixed_u32::bit_and(a151, a152)
                                                        };
                                                        let a154 = 8192u32;
                                                        if equal(a153, a154) {
                                                            8192u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a160 = {
                                                        let a157 = {
                                                            let a155 = v1;
                                                            let a156 = 4096u32;
                                                            fixed_u32::bit_and(a155, a156)
                                                        };
                                                        let a158 = 4096u32;
                                                        if equal(a157, a158) {
                                                            4096u64
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
                                                            let a163 = v1;
                                                            let a164 = 2048u32;
                                                            fixed_u32::bit_and(a163, a164)
                                                        };
                                                        let a166 = 2048u32;
                                                        if equal(a165, a166) {
                                                            2048u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a172 = {
                                                        let a169 = {
                                                            let a167 = v1;
                                                            let a168 = 1024u32;
                                                            fixed_u32::bit_and(a167, a168)
                                                        };
                                                        let a170 = 1024u32;
                                                        if equal(a169, a170) {
                                                            1024u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a171, a172)?
                                                };
                                                let a184 = {
                                                    let a181 = {
                                                        let a175 = {
                                                            let a173 = v1;
                                                            let a174 = 512u32;
                                                            fixed_u32::bit_and(a173, a174)
                                                        };
                                                        let a176 = 512u32;
                                                        if equal(a175, a176) {
                                                            512u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a182 = {
                                                        let a179 = {
                                                            let a177 = v1;
                                                            let a178 = 256u32;
                                                            fixed_u32::bit_and(a177, a178)
                                                        };
                                                        let a180 = 256u32;
                                                        if equal(a179, a180) {
                                                            256u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a181, a182)?
                                                };
                                                nat_add(a183, a184)?
                                            };
                                            nat_add(a185, a186)?
                                        };
                                        let a234 = {
                                            let a231 = {
                                                let a207 = {
                                                    let a195 = {
                                                        let a189 = {
                                                            let a187 = v1;
                                                            let a188 = 128u32;
                                                            fixed_u32::bit_and(a187, a188)
                                                        };
                                                        let a190 = 128u32;
                                                        if equal(a189, a190) {
                                                            128u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a196 = {
                                                        let a193 = {
                                                            let a191 = v1;
                                                            let a192 = 64u32;
                                                            fixed_u32::bit_and(a191, a192)
                                                        };
                                                        let a194 = 64u32;
                                                        if equal(a193, a194) {
                                                            64u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a195, a196)?
                                                };
                                                let a208 = {
                                                    let a205 = {
                                                        let a199 = {
                                                            let a197 = v1;
                                                            let a198 = 32u32;
                                                            fixed_u32::bit_and(a197, a198)
                                                        };
                                                        let a200 = 32u32;
                                                        if equal(a199, a200) {
                                                            32u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a206 = {
                                                        let a203 = {
                                                            let a201 = v1;
                                                            let a202 = 16u32;
                                                            fixed_u32::bit_and(a201, a202)
                                                        };
                                                        let a204 = 16u32;
                                                        if equal(a203, a204) {
                                                            16u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a205, a206)?
                                                };
                                                nat_add(a207, a208)?
                                            };
                                            let a232 = {
                                                let a229 = {
                                                    let a217 = {
                                                        let a211 = {
                                                            let a209 = v1;
                                                            let a210 = 8u32;
                                                            fixed_u32::bit_and(a209, a210)
                                                        };
                                                        let a212 = 8u32;
                                                        if equal(a211, a212) {
                                                            8u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a218 = {
                                                        let a215 = {
                                                            let a213 = v1;
                                                            let a214 = 4u32;
                                                            fixed_u32::bit_and(a213, a214)
                                                        };
                                                        let a216 = 4u32;
                                                        if equal(a215, a216) {
                                                            4u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a217, a218)?
                                                };
                                                let a230 = {
                                                    let a227 = {
                                                        let a221 = {
                                                            let a219 = v1;
                                                            let a220 = 2u32;
                                                            fixed_u32::bit_and(a219, a220)
                                                        };
                                                        let a222 = 2u32;
                                                        if equal(a221, a222) {
                                                            2u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    let a228 = {
                                                        let a225 = {
                                                            let a223 = v1;
                                                            let a224 = 1u32;
                                                            fixed_u32::bit_and(a223, a224)
                                                        };
                                                        let a226 = 1u32;
                                                        if equal(a225, a226) {
                                                            1u64
                                                        } else {
                                                            0u64
                                                        }
                                                    };
                                                    nat_add(a227, a228)?
                                                };
                                                nat_add(a229, a230)?
                                            };
                                            nat_add(a231, a232)?
                                        };
                                        nat_add(a233, a234)?
                                    };
                                    nat_add(a235, a236)?
                                });
                                append_bytes(a237, a238)
                            };
                            let a431 = f14(4u64, {
                                let v2: u32 = match v0.clone() {
                                    Adt0::C0(_, h239, _, _, _, _, _, _) => {
                                        h239
                                    }
                                };
                                let a428 = {
                                    let a332 = {
                                        let a284 = {
                                            let a260 = {
                                                let a248 = {
                                                    let a242 = {
                                                        let a240 = v2;
                                                        let a241 = 2147483648u32;
                                                        fixed_u32::bit_and(a240, a241)
                                                    };
                                                    let a243 = 2147483648u32;
                                                    if equal(a242, a243) {
                                                        2147483648u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a249 = {
                                                    let a246 = {
                                                        let a244 = v2;
                                                        let a245 = 1073741824u32;
                                                        fixed_u32::bit_and(a244, a245)
                                                    };
                                                    let a247 = 1073741824u32;
                                                    if equal(a246, a247) {
                                                        1073741824u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a248, a249)?
                                            };
                                            let a261 = {
                                                let a258 = {
                                                    let a252 = {
                                                        let a250 = v2;
                                                        let a251 = 536870912u32;
                                                        fixed_u32::bit_and(a250, a251)
                                                    };
                                                    let a253 = 536870912u32;
                                                    if equal(a252, a253) {
                                                        536870912u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a259 = {
                                                    let a256 = {
                                                        let a254 = v2;
                                                        let a255 = 268435456u32;
                                                        fixed_u32::bit_and(a254, a255)
                                                    };
                                                    let a257 = 268435456u32;
                                                    if equal(a256, a257) {
                                                        268435456u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a258, a259)?
                                            };
                                            nat_add(a260, a261)?
                                        };
                                        let a285 = {
                                            let a282 = {
                                                let a270 = {
                                                    let a264 = {
                                                        let a262 = v2;
                                                        let a263 = 134217728u32;
                                                        fixed_u32::bit_and(a262, a263)
                                                    };
                                                    let a265 = 134217728u32;
                                                    if equal(a264, a265) {
                                                        134217728u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a271 = {
                                                    let a268 = {
                                                        let a266 = v2;
                                                        let a267 = 67108864u32;
                                                        fixed_u32::bit_and(a266, a267)
                                                    };
                                                    let a269 = 67108864u32;
                                                    if equal(a268, a269) {
                                                        67108864u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a270, a271)?
                                            };
                                            let a283 = {
                                                let a280 = {
                                                    let a274 = {
                                                        let a272 = v2;
                                                        let a273 = 33554432u32;
                                                        fixed_u32::bit_and(a272, a273)
                                                    };
                                                    let a275 = 33554432u32;
                                                    if equal(a274, a275) {
                                                        33554432u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a281 = {
                                                    let a278 = {
                                                        let a276 = v2;
                                                        let a277 = 16777216u32;
                                                        fixed_u32::bit_and(a276, a277)
                                                    };
                                                    let a279 = 16777216u32;
                                                    if equal(a278, a279) {
                                                        16777216u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a280, a281)?
                                            };
                                            nat_add(a282, a283)?
                                        };
                                        nat_add(a284, a285)?
                                    };
                                    let a333 = {
                                        let a330 = {
                                            let a306 = {
                                                let a294 = {
                                                    let a288 = {
                                                        let a286 = v2;
                                                        let a287 = 8388608u32;
                                                        fixed_u32::bit_and(a286, a287)
                                                    };
                                                    let a289 = 8388608u32;
                                                    if equal(a288, a289) {
                                                        8388608u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a295 = {
                                                    let a292 = {
                                                        let a290 = v2;
                                                        let a291 = 4194304u32;
                                                        fixed_u32::bit_and(a290, a291)
                                                    };
                                                    let a293 = 4194304u32;
                                                    if equal(a292, a293) {
                                                        4194304u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a294, a295)?
                                            };
                                            let a307 = {
                                                let a304 = {
                                                    let a298 = {
                                                        let a296 = v2;
                                                        let a297 = 2097152u32;
                                                        fixed_u32::bit_and(a296, a297)
                                                    };
                                                    let a299 = 2097152u32;
                                                    if equal(a298, a299) {
                                                        2097152u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a305 = {
                                                    let a302 = {
                                                        let a300 = v2;
                                                        let a301 = 1048576u32;
                                                        fixed_u32::bit_and(a300, a301)
                                                    };
                                                    let a303 = 1048576u32;
                                                    if equal(a302, a303) {
                                                        1048576u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a304, a305)?
                                            };
                                            nat_add(a306, a307)?
                                        };
                                        let a331 = {
                                            let a328 = {
                                                let a316 = {
                                                    let a310 = {
                                                        let a308 = v2;
                                                        let a309 = 524288u32;
                                                        fixed_u32::bit_and(a308, a309)
                                                    };
                                                    let a311 = 524288u32;
                                                    if equal(a310, a311) {
                                                        524288u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a317 = {
                                                    let a314 = {
                                                        let a312 = v2;
                                                        let a313 = 262144u32;
                                                        fixed_u32::bit_and(a312, a313)
                                                    };
                                                    let a315 = 262144u32;
                                                    if equal(a314, a315) {
                                                        262144u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a316, a317)?
                                            };
                                            let a329 = {
                                                let a326 = {
                                                    let a320 = {
                                                        let a318 = v2;
                                                        let a319 = 131072u32;
                                                        fixed_u32::bit_and(a318, a319)
                                                    };
                                                    let a321 = 131072u32;
                                                    if equal(a320, a321) {
                                                        131072u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a327 = {
                                                    let a324 = {
                                                        let a322 = v2;
                                                        let a323 = 65536u32;
                                                        fixed_u32::bit_and(a322, a323)
                                                    };
                                                    let a325 = 65536u32;
                                                    if equal(a324, a325) {
                                                        65536u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a326, a327)?
                                            };
                                            nat_add(a328, a329)?
                                        };
                                        nat_add(a330, a331)?
                                    };
                                    nat_add(a332, a333)?
                                };
                                let a429 = {
                                    let a426 = {
                                        let a378 = {
                                            let a354 = {
                                                let a342 = {
                                                    let a336 = {
                                                        let a334 = v2;
                                                        let a335 = 32768u32;
                                                        fixed_u32::bit_and(a334, a335)
                                                    };
                                                    let a337 = 32768u32;
                                                    if equal(a336, a337) {
                                                        32768u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a343 = {
                                                    let a340 = {
                                                        let a338 = v2;
                                                        let a339 = 16384u32;
                                                        fixed_u32::bit_and(a338, a339)
                                                    };
                                                    let a341 = 16384u32;
                                                    if equal(a340, a341) {
                                                        16384u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a342, a343)?
                                            };
                                            let a355 = {
                                                let a352 = {
                                                    let a346 = {
                                                        let a344 = v2;
                                                        let a345 = 8192u32;
                                                        fixed_u32::bit_and(a344, a345)
                                                    };
                                                    let a347 = 8192u32;
                                                    if equal(a346, a347) {
                                                        8192u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a353 = {
                                                    let a350 = {
                                                        let a348 = v2;
                                                        let a349 = 4096u32;
                                                        fixed_u32::bit_and(a348, a349)
                                                    };
                                                    let a351 = 4096u32;
                                                    if equal(a350, a351) {
                                                        4096u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a352, a353)?
                                            };
                                            nat_add(a354, a355)?
                                        };
                                        let a379 = {
                                            let a376 = {
                                                let a364 = {
                                                    let a358 = {
                                                        let a356 = v2;
                                                        let a357 = 2048u32;
                                                        fixed_u32::bit_and(a356, a357)
                                                    };
                                                    let a359 = 2048u32;
                                                    if equal(a358, a359) {
                                                        2048u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a365 = {
                                                    let a362 = {
                                                        let a360 = v2;
                                                        let a361 = 1024u32;
                                                        fixed_u32::bit_and(a360, a361)
                                                    };
                                                    let a363 = 1024u32;
                                                    if equal(a362, a363) {
                                                        1024u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a364, a365)?
                                            };
                                            let a377 = {
                                                let a374 = {
                                                    let a368 = {
                                                        let a366 = v2;
                                                        let a367 = 512u32;
                                                        fixed_u32::bit_and(a366, a367)
                                                    };
                                                    let a369 = 512u32;
                                                    if equal(a368, a369) {
                                                        512u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a375 = {
                                                    let a372 = {
                                                        let a370 = v2;
                                                        let a371 = 256u32;
                                                        fixed_u32::bit_and(a370, a371)
                                                    };
                                                    let a373 = 256u32;
                                                    if equal(a372, a373) {
                                                        256u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a374, a375)?
                                            };
                                            nat_add(a376, a377)?
                                        };
                                        nat_add(a378, a379)?
                                    };
                                    let a427 = {
                                        let a424 = {
                                            let a400 = {
                                                let a388 = {
                                                    let a382 = {
                                                        let a380 = v2;
                                                        let a381 = 128u32;
                                                        fixed_u32::bit_and(a380, a381)
                                                    };
                                                    let a383 = 128u32;
                                                    if equal(a382, a383) {
                                                        128u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a389 = {
                                                    let a386 = {
                                                        let a384 = v2;
                                                        let a385 = 64u32;
                                                        fixed_u32::bit_and(a384, a385)
                                                    };
                                                    let a387 = 64u32;
                                                    if equal(a386, a387) {
                                                        64u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a388, a389)?
                                            };
                                            let a401 = {
                                                let a398 = {
                                                    let a392 = {
                                                        let a390 = v2;
                                                        let a391 = 32u32;
                                                        fixed_u32::bit_and(a390, a391)
                                                    };
                                                    let a393 = 32u32;
                                                    if equal(a392, a393) {
                                                        32u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a399 = {
                                                    let a396 = {
                                                        let a394 = v2;
                                                        let a395 = 16u32;
                                                        fixed_u32::bit_and(a394, a395)
                                                    };
                                                    let a397 = 16u32;
                                                    if equal(a396, a397) {
                                                        16u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a398, a399)?
                                            };
                                            nat_add(a400, a401)?
                                        };
                                        let a425 = {
                                            let a422 = {
                                                let a410 = {
                                                    let a404 = {
                                                        let a402 = v2;
                                                        let a403 = 8u32;
                                                        fixed_u32::bit_and(a402, a403)
                                                    };
                                                    let a405 = 8u32;
                                                    if equal(a404, a405) {
                                                        8u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a411 = {
                                                    let a408 = {
                                                        let a406 = v2;
                                                        let a407 = 4u32;
                                                        fixed_u32::bit_and(a406, a407)
                                                    };
                                                    let a409 = 4u32;
                                                    if equal(a408, a409) {
                                                        4u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a410, a411)?
                                            };
                                            let a423 = {
                                                let a420 = {
                                                    let a414 = {
                                                        let a412 = v2;
                                                        let a413 = 2u32;
                                                        fixed_u32::bit_and(a412, a413)
                                                    };
                                                    let a415 = 2u32;
                                                    if equal(a414, a415) {
                                                        2u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                let a421 = {
                                                    let a418 = {
                                                        let a416 = v2;
                                                        let a417 = 1u32;
                                                        fixed_u32::bit_and(a416, a417)
                                                    };
                                                    let a419 = 1u32;
                                                    if equal(a418, a419) {
                                                        1u64
                                                    } else {
                                                        0u64
                                                    }
                                                };
                                                nat_add(a420, a421)?
                                            };
                                            nat_add(a422, a423)?
                                        };
                                        nat_add(a424, a425)?
                                    };
                                    nat_add(a426, a427)?
                                };
                                nat_add(a428, a429)?
                            });
                            append_bytes(a430, a431)
                        };
                        let a624 = f14(4u64, {
                            let v3: u32 = match v0.clone() {
                                Adt0::C0(_, _, h432, _, _, _, _, _) => {
                                    h432
                                }
                            };
                            let a621 = {
                                let a525 = {
                                    let a477 = {
                                        let a453 = {
                                            let a441 = {
                                                let a435 = {
                                                    let a433 = v3;
                                                    let a434 = 2147483648u32;
                                                    fixed_u32::bit_and(a433, a434)
                                                };
                                                let a436 = 2147483648u32;
                                                if equal(a435, a436) {
                                                    2147483648u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a442 = {
                                                let a439 = {
                                                    let a437 = v3;
                                                    let a438 = 1073741824u32;
                                                    fixed_u32::bit_and(a437, a438)
                                                };
                                                let a440 = 1073741824u32;
                                                if equal(a439, a440) {
                                                    1073741824u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a441, a442)?
                                        };
                                        let a454 = {
                                            let a451 = {
                                                let a445 = {
                                                    let a443 = v3;
                                                    let a444 = 536870912u32;
                                                    fixed_u32::bit_and(a443, a444)
                                                };
                                                let a446 = 536870912u32;
                                                if equal(a445, a446) {
                                                    536870912u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a452 = {
                                                let a449 = {
                                                    let a447 = v3;
                                                    let a448 = 268435456u32;
                                                    fixed_u32::bit_and(a447, a448)
                                                };
                                                let a450 = 268435456u32;
                                                if equal(a449, a450) {
                                                    268435456u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a451, a452)?
                                        };
                                        nat_add(a453, a454)?
                                    };
                                    let a478 = {
                                        let a475 = {
                                            let a463 = {
                                                let a457 = {
                                                    let a455 = v3;
                                                    let a456 = 134217728u32;
                                                    fixed_u32::bit_and(a455, a456)
                                                };
                                                let a458 = 134217728u32;
                                                if equal(a457, a458) {
                                                    134217728u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a464 = {
                                                let a461 = {
                                                    let a459 = v3;
                                                    let a460 = 67108864u32;
                                                    fixed_u32::bit_and(a459, a460)
                                                };
                                                let a462 = 67108864u32;
                                                if equal(a461, a462) {
                                                    67108864u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a463, a464)?
                                        };
                                        let a476 = {
                                            let a473 = {
                                                let a467 = {
                                                    let a465 = v3;
                                                    let a466 = 33554432u32;
                                                    fixed_u32::bit_and(a465, a466)
                                                };
                                                let a468 = 33554432u32;
                                                if equal(a467, a468) {
                                                    33554432u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a474 = {
                                                let a471 = {
                                                    let a469 = v3;
                                                    let a470 = 16777216u32;
                                                    fixed_u32::bit_and(a469, a470)
                                                };
                                                let a472 = 16777216u32;
                                                if equal(a471, a472) {
                                                    16777216u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a473, a474)?
                                        };
                                        nat_add(a475, a476)?
                                    };
                                    nat_add(a477, a478)?
                                };
                                let a526 = {
                                    let a523 = {
                                        let a499 = {
                                            let a487 = {
                                                let a481 = {
                                                    let a479 = v3;
                                                    let a480 = 8388608u32;
                                                    fixed_u32::bit_and(a479, a480)
                                                };
                                                let a482 = 8388608u32;
                                                if equal(a481, a482) {
                                                    8388608u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a488 = {
                                                let a485 = {
                                                    let a483 = v3;
                                                    let a484 = 4194304u32;
                                                    fixed_u32::bit_and(a483, a484)
                                                };
                                                let a486 = 4194304u32;
                                                if equal(a485, a486) {
                                                    4194304u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a487, a488)?
                                        };
                                        let a500 = {
                                            let a497 = {
                                                let a491 = {
                                                    let a489 = v3;
                                                    let a490 = 2097152u32;
                                                    fixed_u32::bit_and(a489, a490)
                                                };
                                                let a492 = 2097152u32;
                                                if equal(a491, a492) {
                                                    2097152u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a498 = {
                                                let a495 = {
                                                    let a493 = v3;
                                                    let a494 = 1048576u32;
                                                    fixed_u32::bit_and(a493, a494)
                                                };
                                                let a496 = 1048576u32;
                                                if equal(a495, a496) {
                                                    1048576u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a497, a498)?
                                        };
                                        nat_add(a499, a500)?
                                    };
                                    let a524 = {
                                        let a521 = {
                                            let a509 = {
                                                let a503 = {
                                                    let a501 = v3;
                                                    let a502 = 524288u32;
                                                    fixed_u32::bit_and(a501, a502)
                                                };
                                                let a504 = 524288u32;
                                                if equal(a503, a504) {
                                                    524288u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a510 = {
                                                let a507 = {
                                                    let a505 = v3;
                                                    let a506 = 262144u32;
                                                    fixed_u32::bit_and(a505, a506)
                                                };
                                                let a508 = 262144u32;
                                                if equal(a507, a508) {
                                                    262144u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a509, a510)?
                                        };
                                        let a522 = {
                                            let a519 = {
                                                let a513 = {
                                                    let a511 = v3;
                                                    let a512 = 131072u32;
                                                    fixed_u32::bit_and(a511, a512)
                                                };
                                                let a514 = 131072u32;
                                                if equal(a513, a514) {
                                                    131072u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a520 = {
                                                let a517 = {
                                                    let a515 = v3;
                                                    let a516 = 65536u32;
                                                    fixed_u32::bit_and(a515, a516)
                                                };
                                                let a518 = 65536u32;
                                                if equal(a517, a518) {
                                                    65536u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a519, a520)?
                                        };
                                        nat_add(a521, a522)?
                                    };
                                    nat_add(a523, a524)?
                                };
                                nat_add(a525, a526)?
                            };
                            let a622 = {
                                let a619 = {
                                    let a571 = {
                                        let a547 = {
                                            let a535 = {
                                                let a529 = {
                                                    let a527 = v3;
                                                    let a528 = 32768u32;
                                                    fixed_u32::bit_and(a527, a528)
                                                };
                                                let a530 = 32768u32;
                                                if equal(a529, a530) {
                                                    32768u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a536 = {
                                                let a533 = {
                                                    let a531 = v3;
                                                    let a532 = 16384u32;
                                                    fixed_u32::bit_and(a531, a532)
                                                };
                                                let a534 = 16384u32;
                                                if equal(a533, a534) {
                                                    16384u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a535, a536)?
                                        };
                                        let a548 = {
                                            let a545 = {
                                                let a539 = {
                                                    let a537 = v3;
                                                    let a538 = 8192u32;
                                                    fixed_u32::bit_and(a537, a538)
                                                };
                                                let a540 = 8192u32;
                                                if equal(a539, a540) {
                                                    8192u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a546 = {
                                                let a543 = {
                                                    let a541 = v3;
                                                    let a542 = 4096u32;
                                                    fixed_u32::bit_and(a541, a542)
                                                };
                                                let a544 = 4096u32;
                                                if equal(a543, a544) {
                                                    4096u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a545, a546)?
                                        };
                                        nat_add(a547, a548)?
                                    };
                                    let a572 = {
                                        let a569 = {
                                            let a557 = {
                                                let a551 = {
                                                    let a549 = v3;
                                                    let a550 = 2048u32;
                                                    fixed_u32::bit_and(a549, a550)
                                                };
                                                let a552 = 2048u32;
                                                if equal(a551, a552) {
                                                    2048u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a558 = {
                                                let a555 = {
                                                    let a553 = v3;
                                                    let a554 = 1024u32;
                                                    fixed_u32::bit_and(a553, a554)
                                                };
                                                let a556 = 1024u32;
                                                if equal(a555, a556) {
                                                    1024u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a557, a558)?
                                        };
                                        let a570 = {
                                            let a567 = {
                                                let a561 = {
                                                    let a559 = v3;
                                                    let a560 = 512u32;
                                                    fixed_u32::bit_and(a559, a560)
                                                };
                                                let a562 = 512u32;
                                                if equal(a561, a562) {
                                                    512u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a568 = {
                                                let a565 = {
                                                    let a563 = v3;
                                                    let a564 = 256u32;
                                                    fixed_u32::bit_and(a563, a564)
                                                };
                                                let a566 = 256u32;
                                                if equal(a565, a566) {
                                                    256u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a567, a568)?
                                        };
                                        nat_add(a569, a570)?
                                    };
                                    nat_add(a571, a572)?
                                };
                                let a620 = {
                                    let a617 = {
                                        let a593 = {
                                            let a581 = {
                                                let a575 = {
                                                    let a573 = v3;
                                                    let a574 = 128u32;
                                                    fixed_u32::bit_and(a573, a574)
                                                };
                                                let a576 = 128u32;
                                                if equal(a575, a576) {
                                                    128u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a582 = {
                                                let a579 = {
                                                    let a577 = v3;
                                                    let a578 = 64u32;
                                                    fixed_u32::bit_and(a577, a578)
                                                };
                                                let a580 = 64u32;
                                                if equal(a579, a580) {
                                                    64u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a581, a582)?
                                        };
                                        let a594 = {
                                            let a591 = {
                                                let a585 = {
                                                    let a583 = v3;
                                                    let a584 = 32u32;
                                                    fixed_u32::bit_and(a583, a584)
                                                };
                                                let a586 = 32u32;
                                                if equal(a585, a586) {
                                                    32u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a592 = {
                                                let a589 = {
                                                    let a587 = v3;
                                                    let a588 = 16u32;
                                                    fixed_u32::bit_and(a587, a588)
                                                };
                                                let a590 = 16u32;
                                                if equal(a589, a590) {
                                                    16u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a591, a592)?
                                        };
                                        nat_add(a593, a594)?
                                    };
                                    let a618 = {
                                        let a615 = {
                                            let a603 = {
                                                let a597 = {
                                                    let a595 = v3;
                                                    let a596 = 8u32;
                                                    fixed_u32::bit_and(a595, a596)
                                                };
                                                let a598 = 8u32;
                                                if equal(a597, a598) {
                                                    8u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a604 = {
                                                let a601 = {
                                                    let a599 = v3;
                                                    let a600 = 4u32;
                                                    fixed_u32::bit_and(a599, a600)
                                                };
                                                let a602 = 4u32;
                                                if equal(a601, a602) {
                                                    4u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a603, a604)?
                                        };
                                        let a616 = {
                                            let a613 = {
                                                let a607 = {
                                                    let a605 = v3;
                                                    let a606 = 2u32;
                                                    fixed_u32::bit_and(a605, a606)
                                                };
                                                let a608 = 2u32;
                                                if equal(a607, a608) {
                                                    2u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            let a614 = {
                                                let a611 = {
                                                    let a609 = v3;
                                                    let a610 = 1u32;
                                                    fixed_u32::bit_and(a609, a610)
                                                };
                                                let a612 = 1u32;
                                                if equal(a611, a612) {
                                                    1u64
                                                } else {
                                                    0u64
                                                }
                                            };
                                            nat_add(a613, a614)?
                                        };
                                        nat_add(a615, a616)?
                                    };
                                    nat_add(a617, a618)?
                                };
                                nat_add(a619, a620)?
                            };
                            nat_add(a621, a622)?
                        });
                        append_bytes(a623, a624)
                    };
                    let a817 = f14(4u64, {
                        let v4: u32 = match v0.clone() {
                            Adt0::C0(_, _, _, h625, _, _, _, _) => {
                                h625
                            }
                        };
                        let a814 = {
                            let a718 = {
                                let a670 = {
                                    let a646 = {
                                        let a634 = {
                                            let a628 = {
                                                let a626 = v4;
                                                let a627 = 2147483648u32;
                                                fixed_u32::bit_and(a626, a627)
                                            };
                                            let a629 = 2147483648u32;
                                            if equal(a628, a629) {
                                                2147483648u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a635 = {
                                            let a632 = {
                                                let a630 = v4;
                                                let a631 = 1073741824u32;
                                                fixed_u32::bit_and(a630, a631)
                                            };
                                            let a633 = 1073741824u32;
                                            if equal(a632, a633) {
                                                1073741824u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a634, a635)?
                                    };
                                    let a647 = {
                                        let a644 = {
                                            let a638 = {
                                                let a636 = v4;
                                                let a637 = 536870912u32;
                                                fixed_u32::bit_and(a636, a637)
                                            };
                                            let a639 = 536870912u32;
                                            if equal(a638, a639) {
                                                536870912u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a645 = {
                                            let a642 = {
                                                let a640 = v4;
                                                let a641 = 268435456u32;
                                                fixed_u32::bit_and(a640, a641)
                                            };
                                            let a643 = 268435456u32;
                                            if equal(a642, a643) {
                                                268435456u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a644, a645)?
                                    };
                                    nat_add(a646, a647)?
                                };
                                let a671 = {
                                    let a668 = {
                                        let a656 = {
                                            let a650 = {
                                                let a648 = v4;
                                                let a649 = 134217728u32;
                                                fixed_u32::bit_and(a648, a649)
                                            };
                                            let a651 = 134217728u32;
                                            if equal(a650, a651) {
                                                134217728u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a657 = {
                                            let a654 = {
                                                let a652 = v4;
                                                let a653 = 67108864u32;
                                                fixed_u32::bit_and(a652, a653)
                                            };
                                            let a655 = 67108864u32;
                                            if equal(a654, a655) {
                                                67108864u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a656, a657)?
                                    };
                                    let a669 = {
                                        let a666 = {
                                            let a660 = {
                                                let a658 = v4;
                                                let a659 = 33554432u32;
                                                fixed_u32::bit_and(a658, a659)
                                            };
                                            let a661 = 33554432u32;
                                            if equal(a660, a661) {
                                                33554432u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a667 = {
                                            let a664 = {
                                                let a662 = v4;
                                                let a663 = 16777216u32;
                                                fixed_u32::bit_and(a662, a663)
                                            };
                                            let a665 = 16777216u32;
                                            if equal(a664, a665) {
                                                16777216u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a666, a667)?
                                    };
                                    nat_add(a668, a669)?
                                };
                                nat_add(a670, a671)?
                            };
                            let a719 = {
                                let a716 = {
                                    let a692 = {
                                        let a680 = {
                                            let a674 = {
                                                let a672 = v4;
                                                let a673 = 8388608u32;
                                                fixed_u32::bit_and(a672, a673)
                                            };
                                            let a675 = 8388608u32;
                                            if equal(a674, a675) {
                                                8388608u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a681 = {
                                            let a678 = {
                                                let a676 = v4;
                                                let a677 = 4194304u32;
                                                fixed_u32::bit_and(a676, a677)
                                            };
                                            let a679 = 4194304u32;
                                            if equal(a678, a679) {
                                                4194304u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a680, a681)?
                                    };
                                    let a693 = {
                                        let a690 = {
                                            let a684 = {
                                                let a682 = v4;
                                                let a683 = 2097152u32;
                                                fixed_u32::bit_and(a682, a683)
                                            };
                                            let a685 = 2097152u32;
                                            if equal(a684, a685) {
                                                2097152u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a691 = {
                                            let a688 = {
                                                let a686 = v4;
                                                let a687 = 1048576u32;
                                                fixed_u32::bit_and(a686, a687)
                                            };
                                            let a689 = 1048576u32;
                                            if equal(a688, a689) {
                                                1048576u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a690, a691)?
                                    };
                                    nat_add(a692, a693)?
                                };
                                let a717 = {
                                    let a714 = {
                                        let a702 = {
                                            let a696 = {
                                                let a694 = v4;
                                                let a695 = 524288u32;
                                                fixed_u32::bit_and(a694, a695)
                                            };
                                            let a697 = 524288u32;
                                            if equal(a696, a697) {
                                                524288u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a703 = {
                                            let a700 = {
                                                let a698 = v4;
                                                let a699 = 262144u32;
                                                fixed_u32::bit_and(a698, a699)
                                            };
                                            let a701 = 262144u32;
                                            if equal(a700, a701) {
                                                262144u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a702, a703)?
                                    };
                                    let a715 = {
                                        let a712 = {
                                            let a706 = {
                                                let a704 = v4;
                                                let a705 = 131072u32;
                                                fixed_u32::bit_and(a704, a705)
                                            };
                                            let a707 = 131072u32;
                                            if equal(a706, a707) {
                                                131072u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a713 = {
                                            let a710 = {
                                                let a708 = v4;
                                                let a709 = 65536u32;
                                                fixed_u32::bit_and(a708, a709)
                                            };
                                            let a711 = 65536u32;
                                            if equal(a710, a711) {
                                                65536u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a712, a713)?
                                    };
                                    nat_add(a714, a715)?
                                };
                                nat_add(a716, a717)?
                            };
                            nat_add(a718, a719)?
                        };
                        let a815 = {
                            let a812 = {
                                let a764 = {
                                    let a740 = {
                                        let a728 = {
                                            let a722 = {
                                                let a720 = v4;
                                                let a721 = 32768u32;
                                                fixed_u32::bit_and(a720, a721)
                                            };
                                            let a723 = 32768u32;
                                            if equal(a722, a723) {
                                                32768u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a729 = {
                                            let a726 = {
                                                let a724 = v4;
                                                let a725 = 16384u32;
                                                fixed_u32::bit_and(a724, a725)
                                            };
                                            let a727 = 16384u32;
                                            if equal(a726, a727) {
                                                16384u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a728, a729)?
                                    };
                                    let a741 = {
                                        let a738 = {
                                            let a732 = {
                                                let a730 = v4;
                                                let a731 = 8192u32;
                                                fixed_u32::bit_and(a730, a731)
                                            };
                                            let a733 = 8192u32;
                                            if equal(a732, a733) {
                                                8192u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a739 = {
                                            let a736 = {
                                                let a734 = v4;
                                                let a735 = 4096u32;
                                                fixed_u32::bit_and(a734, a735)
                                            };
                                            let a737 = 4096u32;
                                            if equal(a736, a737) {
                                                4096u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a738, a739)?
                                    };
                                    nat_add(a740, a741)?
                                };
                                let a765 = {
                                    let a762 = {
                                        let a750 = {
                                            let a744 = {
                                                let a742 = v4;
                                                let a743 = 2048u32;
                                                fixed_u32::bit_and(a742, a743)
                                            };
                                            let a745 = 2048u32;
                                            if equal(a744, a745) {
                                                2048u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a751 = {
                                            let a748 = {
                                                let a746 = v4;
                                                let a747 = 1024u32;
                                                fixed_u32::bit_and(a746, a747)
                                            };
                                            let a749 = 1024u32;
                                            if equal(a748, a749) {
                                                1024u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a750, a751)?
                                    };
                                    let a763 = {
                                        let a760 = {
                                            let a754 = {
                                                let a752 = v4;
                                                let a753 = 512u32;
                                                fixed_u32::bit_and(a752, a753)
                                            };
                                            let a755 = 512u32;
                                            if equal(a754, a755) {
                                                512u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a761 = {
                                            let a758 = {
                                                let a756 = v4;
                                                let a757 = 256u32;
                                                fixed_u32::bit_and(a756, a757)
                                            };
                                            let a759 = 256u32;
                                            if equal(a758, a759) {
                                                256u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a760, a761)?
                                    };
                                    nat_add(a762, a763)?
                                };
                                nat_add(a764, a765)?
                            };
                            let a813 = {
                                let a810 = {
                                    let a786 = {
                                        let a774 = {
                                            let a768 = {
                                                let a766 = v4;
                                                let a767 = 128u32;
                                                fixed_u32::bit_and(a766, a767)
                                            };
                                            let a769 = 128u32;
                                            if equal(a768, a769) {
                                                128u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a775 = {
                                            let a772 = {
                                                let a770 = v4;
                                                let a771 = 64u32;
                                                fixed_u32::bit_and(a770, a771)
                                            };
                                            let a773 = 64u32;
                                            if equal(a772, a773) {
                                                64u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a774, a775)?
                                    };
                                    let a787 = {
                                        let a784 = {
                                            let a778 = {
                                                let a776 = v4;
                                                let a777 = 32u32;
                                                fixed_u32::bit_and(a776, a777)
                                            };
                                            let a779 = 32u32;
                                            if equal(a778, a779) {
                                                32u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a785 = {
                                            let a782 = {
                                                let a780 = v4;
                                                let a781 = 16u32;
                                                fixed_u32::bit_and(a780, a781)
                                            };
                                            let a783 = 16u32;
                                            if equal(a782, a783) {
                                                16u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a784, a785)?
                                    };
                                    nat_add(a786, a787)?
                                };
                                let a811 = {
                                    let a808 = {
                                        let a796 = {
                                            let a790 = {
                                                let a788 = v4;
                                                let a789 = 8u32;
                                                fixed_u32::bit_and(a788, a789)
                                            };
                                            let a791 = 8u32;
                                            if equal(a790, a791) {
                                                8u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a797 = {
                                            let a794 = {
                                                let a792 = v4;
                                                let a793 = 4u32;
                                                fixed_u32::bit_and(a792, a793)
                                            };
                                            let a795 = 4u32;
                                            if equal(a794, a795) {
                                                4u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a796, a797)?
                                    };
                                    let a809 = {
                                        let a806 = {
                                            let a800 = {
                                                let a798 = v4;
                                                let a799 = 2u32;
                                                fixed_u32::bit_and(a798, a799)
                                            };
                                            let a801 = 2u32;
                                            if equal(a800, a801) {
                                                2u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        let a807 = {
                                            let a804 = {
                                                let a802 = v4;
                                                let a803 = 1u32;
                                                fixed_u32::bit_and(a802, a803)
                                            };
                                            let a805 = 1u32;
                                            if equal(a804, a805) {
                                                1u64
                                            } else {
                                                0u64
                                            }
                                        };
                                        nat_add(a806, a807)?
                                    };
                                    nat_add(a808, a809)?
                                };
                                nat_add(a810, a811)?
                            };
                            nat_add(a812, a813)?
                        };
                        nat_add(a814, a815)?
                    });
                    append_bytes(a816, a817)
                };
                let a1010 = f14(4u64, {
                    let v5: u32 = match v0.clone() {
                        Adt0::C0(_, _, _, _, h818, _, _, _) => {
                            h818
                        }
                    };
                    let a1007 = {
                        let a911 = {
                            let a863 = {
                                let a839 = {
                                    let a827 = {
                                        let a821 = {
                                            let a819 = v5;
                                            let a820 = 2147483648u32;
                                            fixed_u32::bit_and(a819, a820)
                                        };
                                        let a822 = 2147483648u32;
                                        if equal(a821, a822) {
                                            2147483648u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a828 = {
                                        let a825 = {
                                            let a823 = v5;
                                            let a824 = 1073741824u32;
                                            fixed_u32::bit_and(a823, a824)
                                        };
                                        let a826 = 1073741824u32;
                                        if equal(a825, a826) {
                                            1073741824u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a827, a828)?
                                };
                                let a840 = {
                                    let a837 = {
                                        let a831 = {
                                            let a829 = v5;
                                            let a830 = 536870912u32;
                                            fixed_u32::bit_and(a829, a830)
                                        };
                                        let a832 = 536870912u32;
                                        if equal(a831, a832) {
                                            536870912u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a838 = {
                                        let a835 = {
                                            let a833 = v5;
                                            let a834 = 268435456u32;
                                            fixed_u32::bit_and(a833, a834)
                                        };
                                        let a836 = 268435456u32;
                                        if equal(a835, a836) {
                                            268435456u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a837, a838)?
                                };
                                nat_add(a839, a840)?
                            };
                            let a864 = {
                                let a861 = {
                                    let a849 = {
                                        let a843 = {
                                            let a841 = v5;
                                            let a842 = 134217728u32;
                                            fixed_u32::bit_and(a841, a842)
                                        };
                                        let a844 = 134217728u32;
                                        if equal(a843, a844) {
                                            134217728u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a850 = {
                                        let a847 = {
                                            let a845 = v5;
                                            let a846 = 67108864u32;
                                            fixed_u32::bit_and(a845, a846)
                                        };
                                        let a848 = 67108864u32;
                                        if equal(a847, a848) {
                                            67108864u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a849, a850)?
                                };
                                let a862 = {
                                    let a859 = {
                                        let a853 = {
                                            let a851 = v5;
                                            let a852 = 33554432u32;
                                            fixed_u32::bit_and(a851, a852)
                                        };
                                        let a854 = 33554432u32;
                                        if equal(a853, a854) {
                                            33554432u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a860 = {
                                        let a857 = {
                                            let a855 = v5;
                                            let a856 = 16777216u32;
                                            fixed_u32::bit_and(a855, a856)
                                        };
                                        let a858 = 16777216u32;
                                        if equal(a857, a858) {
                                            16777216u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a859, a860)?
                                };
                                nat_add(a861, a862)?
                            };
                            nat_add(a863, a864)?
                        };
                        let a912 = {
                            let a909 = {
                                let a885 = {
                                    let a873 = {
                                        let a867 = {
                                            let a865 = v5;
                                            let a866 = 8388608u32;
                                            fixed_u32::bit_and(a865, a866)
                                        };
                                        let a868 = 8388608u32;
                                        if equal(a867, a868) {
                                            8388608u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a874 = {
                                        let a871 = {
                                            let a869 = v5;
                                            let a870 = 4194304u32;
                                            fixed_u32::bit_and(a869, a870)
                                        };
                                        let a872 = 4194304u32;
                                        if equal(a871, a872) {
                                            4194304u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a873, a874)?
                                };
                                let a886 = {
                                    let a883 = {
                                        let a877 = {
                                            let a875 = v5;
                                            let a876 = 2097152u32;
                                            fixed_u32::bit_and(a875, a876)
                                        };
                                        let a878 = 2097152u32;
                                        if equal(a877, a878) {
                                            2097152u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a884 = {
                                        let a881 = {
                                            let a879 = v5;
                                            let a880 = 1048576u32;
                                            fixed_u32::bit_and(a879, a880)
                                        };
                                        let a882 = 1048576u32;
                                        if equal(a881, a882) {
                                            1048576u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a883, a884)?
                                };
                                nat_add(a885, a886)?
                            };
                            let a910 = {
                                let a907 = {
                                    let a895 = {
                                        let a889 = {
                                            let a887 = v5;
                                            let a888 = 524288u32;
                                            fixed_u32::bit_and(a887, a888)
                                        };
                                        let a890 = 524288u32;
                                        if equal(a889, a890) {
                                            524288u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a896 = {
                                        let a893 = {
                                            let a891 = v5;
                                            let a892 = 262144u32;
                                            fixed_u32::bit_and(a891, a892)
                                        };
                                        let a894 = 262144u32;
                                        if equal(a893, a894) {
                                            262144u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a895, a896)?
                                };
                                let a908 = {
                                    let a905 = {
                                        let a899 = {
                                            let a897 = v5;
                                            let a898 = 131072u32;
                                            fixed_u32::bit_and(a897, a898)
                                        };
                                        let a900 = 131072u32;
                                        if equal(a899, a900) {
                                            131072u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a906 = {
                                        let a903 = {
                                            let a901 = v5;
                                            let a902 = 65536u32;
                                            fixed_u32::bit_and(a901, a902)
                                        };
                                        let a904 = 65536u32;
                                        if equal(a903, a904) {
                                            65536u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a905, a906)?
                                };
                                nat_add(a907, a908)?
                            };
                            nat_add(a909, a910)?
                        };
                        nat_add(a911, a912)?
                    };
                    let a1008 = {
                        let a1005 = {
                            let a957 = {
                                let a933 = {
                                    let a921 = {
                                        let a915 = {
                                            let a913 = v5;
                                            let a914 = 32768u32;
                                            fixed_u32::bit_and(a913, a914)
                                        };
                                        let a916 = 32768u32;
                                        if equal(a915, a916) {
                                            32768u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a922 = {
                                        let a919 = {
                                            let a917 = v5;
                                            let a918 = 16384u32;
                                            fixed_u32::bit_and(a917, a918)
                                        };
                                        let a920 = 16384u32;
                                        if equal(a919, a920) {
                                            16384u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a921, a922)?
                                };
                                let a934 = {
                                    let a931 = {
                                        let a925 = {
                                            let a923 = v5;
                                            let a924 = 8192u32;
                                            fixed_u32::bit_and(a923, a924)
                                        };
                                        let a926 = 8192u32;
                                        if equal(a925, a926) {
                                            8192u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a932 = {
                                        let a929 = {
                                            let a927 = v5;
                                            let a928 = 4096u32;
                                            fixed_u32::bit_and(a927, a928)
                                        };
                                        let a930 = 4096u32;
                                        if equal(a929, a930) {
                                            4096u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a931, a932)?
                                };
                                nat_add(a933, a934)?
                            };
                            let a958 = {
                                let a955 = {
                                    let a943 = {
                                        let a937 = {
                                            let a935 = v5;
                                            let a936 = 2048u32;
                                            fixed_u32::bit_and(a935, a936)
                                        };
                                        let a938 = 2048u32;
                                        if equal(a937, a938) {
                                            2048u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a944 = {
                                        let a941 = {
                                            let a939 = v5;
                                            let a940 = 1024u32;
                                            fixed_u32::bit_and(a939, a940)
                                        };
                                        let a942 = 1024u32;
                                        if equal(a941, a942) {
                                            1024u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a943, a944)?
                                };
                                let a956 = {
                                    let a953 = {
                                        let a947 = {
                                            let a945 = v5;
                                            let a946 = 512u32;
                                            fixed_u32::bit_and(a945, a946)
                                        };
                                        let a948 = 512u32;
                                        if equal(a947, a948) {
                                            512u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a954 = {
                                        let a951 = {
                                            let a949 = v5;
                                            let a950 = 256u32;
                                            fixed_u32::bit_and(a949, a950)
                                        };
                                        let a952 = 256u32;
                                        if equal(a951, a952) {
                                            256u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a953, a954)?
                                };
                                nat_add(a955, a956)?
                            };
                            nat_add(a957, a958)?
                        };
                        let a1006 = {
                            let a1003 = {
                                let a979 = {
                                    let a967 = {
                                        let a961 = {
                                            let a959 = v5;
                                            let a960 = 128u32;
                                            fixed_u32::bit_and(a959, a960)
                                        };
                                        let a962 = 128u32;
                                        if equal(a961, a962) {
                                            128u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a968 = {
                                        let a965 = {
                                            let a963 = v5;
                                            let a964 = 64u32;
                                            fixed_u32::bit_and(a963, a964)
                                        };
                                        let a966 = 64u32;
                                        if equal(a965, a966) {
                                            64u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a967, a968)?
                                };
                                let a980 = {
                                    let a977 = {
                                        let a971 = {
                                            let a969 = v5;
                                            let a970 = 32u32;
                                            fixed_u32::bit_and(a969, a970)
                                        };
                                        let a972 = 32u32;
                                        if equal(a971, a972) {
                                            32u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a978 = {
                                        let a975 = {
                                            let a973 = v5;
                                            let a974 = 16u32;
                                            fixed_u32::bit_and(a973, a974)
                                        };
                                        let a976 = 16u32;
                                        if equal(a975, a976) {
                                            16u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a977, a978)?
                                };
                                nat_add(a979, a980)?
                            };
                            let a1004 = {
                                let a1001 = {
                                    let a989 = {
                                        let a983 = {
                                            let a981 = v5;
                                            let a982 = 8u32;
                                            fixed_u32::bit_and(a981, a982)
                                        };
                                        let a984 = 8u32;
                                        if equal(a983, a984) {
                                            8u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a990 = {
                                        let a987 = {
                                            let a985 = v5;
                                            let a986 = 4u32;
                                            fixed_u32::bit_and(a985, a986)
                                        };
                                        let a988 = 4u32;
                                        if equal(a987, a988) {
                                            4u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a989, a990)?
                                };
                                let a1002 = {
                                    let a999 = {
                                        let a993 = {
                                            let a991 = v5;
                                            let a992 = 2u32;
                                            fixed_u32::bit_and(a991, a992)
                                        };
                                        let a994 = 2u32;
                                        if equal(a993, a994) {
                                            2u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    let a1000 = {
                                        let a997 = {
                                            let a995 = v5;
                                            let a996 = 1u32;
                                            fixed_u32::bit_and(a995, a996)
                                        };
                                        let a998 = 1u32;
                                        if equal(a997, a998) {
                                            1u64
                                        } else {
                                            0u64
                                        }
                                    };
                                    nat_add(a999, a1000)?
                                };
                                nat_add(a1001, a1002)?
                            };
                            nat_add(a1003, a1004)?
                        };
                        nat_add(a1005, a1006)?
                    };
                    nat_add(a1007, a1008)?
                });
                append_bytes(a1009, a1010)
            };
            let a1203 = f14(4u64, {
                let v6: u32 = match v0.clone() {
                    Adt0::C0(_, _, _, _, _, h1011, _, _) => {
                        h1011
                    }
                };
                let a1200 = {
                    let a1104 = {
                        let a1056 = {
                            let a1032 = {
                                let a1020 = {
                                    let a1014 = {
                                        let a1012 = v6;
                                        let a1013 = 2147483648u32;
                                        fixed_u32::bit_and(a1012, a1013)
                                    };
                                    let a1015 = 2147483648u32;
                                    if equal(a1014, a1015) {
                                        2147483648u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1021 = {
                                    let a1018 = {
                                        let a1016 = v6;
                                        let a1017 = 1073741824u32;
                                        fixed_u32::bit_and(a1016, a1017)
                                    };
                                    let a1019 = 1073741824u32;
                                    if equal(a1018, a1019) {
                                        1073741824u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1020, a1021)?
                            };
                            let a1033 = {
                                let a1030 = {
                                    let a1024 = {
                                        let a1022 = v6;
                                        let a1023 = 536870912u32;
                                        fixed_u32::bit_and(a1022, a1023)
                                    };
                                    let a1025 = 536870912u32;
                                    if equal(a1024, a1025) {
                                        536870912u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1031 = {
                                    let a1028 = {
                                        let a1026 = v6;
                                        let a1027 = 268435456u32;
                                        fixed_u32::bit_and(a1026, a1027)
                                    };
                                    let a1029 = 268435456u32;
                                    if equal(a1028, a1029) {
                                        268435456u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1030, a1031)?
                            };
                            nat_add(a1032, a1033)?
                        };
                        let a1057 = {
                            let a1054 = {
                                let a1042 = {
                                    let a1036 = {
                                        let a1034 = v6;
                                        let a1035 = 134217728u32;
                                        fixed_u32::bit_and(a1034, a1035)
                                    };
                                    let a1037 = 134217728u32;
                                    if equal(a1036, a1037) {
                                        134217728u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1043 = {
                                    let a1040 = {
                                        let a1038 = v6;
                                        let a1039 = 67108864u32;
                                        fixed_u32::bit_and(a1038, a1039)
                                    };
                                    let a1041 = 67108864u32;
                                    if equal(a1040, a1041) {
                                        67108864u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1042, a1043)?
                            };
                            let a1055 = {
                                let a1052 = {
                                    let a1046 = {
                                        let a1044 = v6;
                                        let a1045 = 33554432u32;
                                        fixed_u32::bit_and(a1044, a1045)
                                    };
                                    let a1047 = 33554432u32;
                                    if equal(a1046, a1047) {
                                        33554432u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1053 = {
                                    let a1050 = {
                                        let a1048 = v6;
                                        let a1049 = 16777216u32;
                                        fixed_u32::bit_and(a1048, a1049)
                                    };
                                    let a1051 = 16777216u32;
                                    if equal(a1050, a1051) {
                                        16777216u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1052, a1053)?
                            };
                            nat_add(a1054, a1055)?
                        };
                        nat_add(a1056, a1057)?
                    };
                    let a1105 = {
                        let a1102 = {
                            let a1078 = {
                                let a1066 = {
                                    let a1060 = {
                                        let a1058 = v6;
                                        let a1059 = 8388608u32;
                                        fixed_u32::bit_and(a1058, a1059)
                                    };
                                    let a1061 = 8388608u32;
                                    if equal(a1060, a1061) {
                                        8388608u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1067 = {
                                    let a1064 = {
                                        let a1062 = v6;
                                        let a1063 = 4194304u32;
                                        fixed_u32::bit_and(a1062, a1063)
                                    };
                                    let a1065 = 4194304u32;
                                    if equal(a1064, a1065) {
                                        4194304u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1066, a1067)?
                            };
                            let a1079 = {
                                let a1076 = {
                                    let a1070 = {
                                        let a1068 = v6;
                                        let a1069 = 2097152u32;
                                        fixed_u32::bit_and(a1068, a1069)
                                    };
                                    let a1071 = 2097152u32;
                                    if equal(a1070, a1071) {
                                        2097152u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1077 = {
                                    let a1074 = {
                                        let a1072 = v6;
                                        let a1073 = 1048576u32;
                                        fixed_u32::bit_and(a1072, a1073)
                                    };
                                    let a1075 = 1048576u32;
                                    if equal(a1074, a1075) {
                                        1048576u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1076, a1077)?
                            };
                            nat_add(a1078, a1079)?
                        };
                        let a1103 = {
                            let a1100 = {
                                let a1088 = {
                                    let a1082 = {
                                        let a1080 = v6;
                                        let a1081 = 524288u32;
                                        fixed_u32::bit_and(a1080, a1081)
                                    };
                                    let a1083 = 524288u32;
                                    if equal(a1082, a1083) {
                                        524288u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1089 = {
                                    let a1086 = {
                                        let a1084 = v6;
                                        let a1085 = 262144u32;
                                        fixed_u32::bit_and(a1084, a1085)
                                    };
                                    let a1087 = 262144u32;
                                    if equal(a1086, a1087) {
                                        262144u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1088, a1089)?
                            };
                            let a1101 = {
                                let a1098 = {
                                    let a1092 = {
                                        let a1090 = v6;
                                        let a1091 = 131072u32;
                                        fixed_u32::bit_and(a1090, a1091)
                                    };
                                    let a1093 = 131072u32;
                                    if equal(a1092, a1093) {
                                        131072u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1099 = {
                                    let a1096 = {
                                        let a1094 = v6;
                                        let a1095 = 65536u32;
                                        fixed_u32::bit_and(a1094, a1095)
                                    };
                                    let a1097 = 65536u32;
                                    if equal(a1096, a1097) {
                                        65536u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1098, a1099)?
                            };
                            nat_add(a1100, a1101)?
                        };
                        nat_add(a1102, a1103)?
                    };
                    nat_add(a1104, a1105)?
                };
                let a1201 = {
                    let a1198 = {
                        let a1150 = {
                            let a1126 = {
                                let a1114 = {
                                    let a1108 = {
                                        let a1106 = v6;
                                        let a1107 = 32768u32;
                                        fixed_u32::bit_and(a1106, a1107)
                                    };
                                    let a1109 = 32768u32;
                                    if equal(a1108, a1109) {
                                        32768u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1115 = {
                                    let a1112 = {
                                        let a1110 = v6;
                                        let a1111 = 16384u32;
                                        fixed_u32::bit_and(a1110, a1111)
                                    };
                                    let a1113 = 16384u32;
                                    if equal(a1112, a1113) {
                                        16384u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1114, a1115)?
                            };
                            let a1127 = {
                                let a1124 = {
                                    let a1118 = {
                                        let a1116 = v6;
                                        let a1117 = 8192u32;
                                        fixed_u32::bit_and(a1116, a1117)
                                    };
                                    let a1119 = 8192u32;
                                    if equal(a1118, a1119) {
                                        8192u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1125 = {
                                    let a1122 = {
                                        let a1120 = v6;
                                        let a1121 = 4096u32;
                                        fixed_u32::bit_and(a1120, a1121)
                                    };
                                    let a1123 = 4096u32;
                                    if equal(a1122, a1123) {
                                        4096u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1124, a1125)?
                            };
                            nat_add(a1126, a1127)?
                        };
                        let a1151 = {
                            let a1148 = {
                                let a1136 = {
                                    let a1130 = {
                                        let a1128 = v6;
                                        let a1129 = 2048u32;
                                        fixed_u32::bit_and(a1128, a1129)
                                    };
                                    let a1131 = 2048u32;
                                    if equal(a1130, a1131) {
                                        2048u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1137 = {
                                    let a1134 = {
                                        let a1132 = v6;
                                        let a1133 = 1024u32;
                                        fixed_u32::bit_and(a1132, a1133)
                                    };
                                    let a1135 = 1024u32;
                                    if equal(a1134, a1135) {
                                        1024u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1136, a1137)?
                            };
                            let a1149 = {
                                let a1146 = {
                                    let a1140 = {
                                        let a1138 = v6;
                                        let a1139 = 512u32;
                                        fixed_u32::bit_and(a1138, a1139)
                                    };
                                    let a1141 = 512u32;
                                    if equal(a1140, a1141) {
                                        512u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1147 = {
                                    let a1144 = {
                                        let a1142 = v6;
                                        let a1143 = 256u32;
                                        fixed_u32::bit_and(a1142, a1143)
                                    };
                                    let a1145 = 256u32;
                                    if equal(a1144, a1145) {
                                        256u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1146, a1147)?
                            };
                            nat_add(a1148, a1149)?
                        };
                        nat_add(a1150, a1151)?
                    };
                    let a1199 = {
                        let a1196 = {
                            let a1172 = {
                                let a1160 = {
                                    let a1154 = {
                                        let a1152 = v6;
                                        let a1153 = 128u32;
                                        fixed_u32::bit_and(a1152, a1153)
                                    };
                                    let a1155 = 128u32;
                                    if equal(a1154, a1155) {
                                        128u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1161 = {
                                    let a1158 = {
                                        let a1156 = v6;
                                        let a1157 = 64u32;
                                        fixed_u32::bit_and(a1156, a1157)
                                    };
                                    let a1159 = 64u32;
                                    if equal(a1158, a1159) {
                                        64u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1160, a1161)?
                            };
                            let a1173 = {
                                let a1170 = {
                                    let a1164 = {
                                        let a1162 = v6;
                                        let a1163 = 32u32;
                                        fixed_u32::bit_and(a1162, a1163)
                                    };
                                    let a1165 = 32u32;
                                    if equal(a1164, a1165) {
                                        32u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1171 = {
                                    let a1168 = {
                                        let a1166 = v6;
                                        let a1167 = 16u32;
                                        fixed_u32::bit_and(a1166, a1167)
                                    };
                                    let a1169 = 16u32;
                                    if equal(a1168, a1169) {
                                        16u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1170, a1171)?
                            };
                            nat_add(a1172, a1173)?
                        };
                        let a1197 = {
                            let a1194 = {
                                let a1182 = {
                                    let a1176 = {
                                        let a1174 = v6;
                                        let a1175 = 8u32;
                                        fixed_u32::bit_and(a1174, a1175)
                                    };
                                    let a1177 = 8u32;
                                    if equal(a1176, a1177) {
                                        8u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1183 = {
                                    let a1180 = {
                                        let a1178 = v6;
                                        let a1179 = 4u32;
                                        fixed_u32::bit_and(a1178, a1179)
                                    };
                                    let a1181 = 4u32;
                                    if equal(a1180, a1181) {
                                        4u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1182, a1183)?
                            };
                            let a1195 = {
                                let a1192 = {
                                    let a1186 = {
                                        let a1184 = v6;
                                        let a1185 = 2u32;
                                        fixed_u32::bit_and(a1184, a1185)
                                    };
                                    let a1187 = 2u32;
                                    if equal(a1186, a1187) {
                                        2u64
                                    } else {
                                        0u64
                                    }
                                };
                                let a1193 = {
                                    let a1190 = {
                                        let a1188 = v6;
                                        let a1189 = 1u32;
                                        fixed_u32::bit_and(a1188, a1189)
                                    };
                                    let a1191 = 1u32;
                                    if equal(a1190, a1191) {
                                        1u64
                                    } else {
                                        0u64
                                    }
                                };
                                nat_add(a1192, a1193)?
                            };
                            nat_add(a1194, a1195)?
                        };
                        nat_add(a1196, a1197)?
                    };
                    nat_add(a1198, a1199)?
                };
                nat_add(a1200, a1201)?
            });
            append_bytes(a1202, a1203)
        };
        let a1396 = f14(4u64, {
            let v7: u32 = match v0.clone() {
                Adt0::C0(_, _, _, _, _, _, h1204, _) => {
                    h1204
                }
            };
            let a1393 = {
                let a1297 = {
                    let a1249 = {
                        let a1225 = {
                            let a1213 = {
                                let a1207 = {
                                    let a1205 = v7;
                                    let a1206 = 2147483648u32;
                                    fixed_u32::bit_and(a1205, a1206)
                                };
                                let a1208 = 2147483648u32;
                                if equal(a1207, a1208) {
                                    2147483648u64
                                } else {
                                    0u64
                                }
                            };
                            let a1214 = {
                                let a1211 = {
                                    let a1209 = v7;
                                    let a1210 = 1073741824u32;
                                    fixed_u32::bit_and(a1209, a1210)
                                };
                                let a1212 = 1073741824u32;
                                if equal(a1211, a1212) {
                                    1073741824u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1213, a1214)?
                        };
                        let a1226 = {
                            let a1223 = {
                                let a1217 = {
                                    let a1215 = v7;
                                    let a1216 = 536870912u32;
                                    fixed_u32::bit_and(a1215, a1216)
                                };
                                let a1218 = 536870912u32;
                                if equal(a1217, a1218) {
                                    536870912u64
                                } else {
                                    0u64
                                }
                            };
                            let a1224 = {
                                let a1221 = {
                                    let a1219 = v7;
                                    let a1220 = 268435456u32;
                                    fixed_u32::bit_and(a1219, a1220)
                                };
                                let a1222 = 268435456u32;
                                if equal(a1221, a1222) {
                                    268435456u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1223, a1224)?
                        };
                        nat_add(a1225, a1226)?
                    };
                    let a1250 = {
                        let a1247 = {
                            let a1235 = {
                                let a1229 = {
                                    let a1227 = v7;
                                    let a1228 = 134217728u32;
                                    fixed_u32::bit_and(a1227, a1228)
                                };
                                let a1230 = 134217728u32;
                                if equal(a1229, a1230) {
                                    134217728u64
                                } else {
                                    0u64
                                }
                            };
                            let a1236 = {
                                let a1233 = {
                                    let a1231 = v7;
                                    let a1232 = 67108864u32;
                                    fixed_u32::bit_and(a1231, a1232)
                                };
                                let a1234 = 67108864u32;
                                if equal(a1233, a1234) {
                                    67108864u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1235, a1236)?
                        };
                        let a1248 = {
                            let a1245 = {
                                let a1239 = {
                                    let a1237 = v7;
                                    let a1238 = 33554432u32;
                                    fixed_u32::bit_and(a1237, a1238)
                                };
                                let a1240 = 33554432u32;
                                if equal(a1239, a1240) {
                                    33554432u64
                                } else {
                                    0u64
                                }
                            };
                            let a1246 = {
                                let a1243 = {
                                    let a1241 = v7;
                                    let a1242 = 16777216u32;
                                    fixed_u32::bit_and(a1241, a1242)
                                };
                                let a1244 = 16777216u32;
                                if equal(a1243, a1244) {
                                    16777216u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1245, a1246)?
                        };
                        nat_add(a1247, a1248)?
                    };
                    nat_add(a1249, a1250)?
                };
                let a1298 = {
                    let a1295 = {
                        let a1271 = {
                            let a1259 = {
                                let a1253 = {
                                    let a1251 = v7;
                                    let a1252 = 8388608u32;
                                    fixed_u32::bit_and(a1251, a1252)
                                };
                                let a1254 = 8388608u32;
                                if equal(a1253, a1254) {
                                    8388608u64
                                } else {
                                    0u64
                                }
                            };
                            let a1260 = {
                                let a1257 = {
                                    let a1255 = v7;
                                    let a1256 = 4194304u32;
                                    fixed_u32::bit_and(a1255, a1256)
                                };
                                let a1258 = 4194304u32;
                                if equal(a1257, a1258) {
                                    4194304u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1259, a1260)?
                        };
                        let a1272 = {
                            let a1269 = {
                                let a1263 = {
                                    let a1261 = v7;
                                    let a1262 = 2097152u32;
                                    fixed_u32::bit_and(a1261, a1262)
                                };
                                let a1264 = 2097152u32;
                                if equal(a1263, a1264) {
                                    2097152u64
                                } else {
                                    0u64
                                }
                            };
                            let a1270 = {
                                let a1267 = {
                                    let a1265 = v7;
                                    let a1266 = 1048576u32;
                                    fixed_u32::bit_and(a1265, a1266)
                                };
                                let a1268 = 1048576u32;
                                if equal(a1267, a1268) {
                                    1048576u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1269, a1270)?
                        };
                        nat_add(a1271, a1272)?
                    };
                    let a1296 = {
                        let a1293 = {
                            let a1281 = {
                                let a1275 = {
                                    let a1273 = v7;
                                    let a1274 = 524288u32;
                                    fixed_u32::bit_and(a1273, a1274)
                                };
                                let a1276 = 524288u32;
                                if equal(a1275, a1276) {
                                    524288u64
                                } else {
                                    0u64
                                }
                            };
                            let a1282 = {
                                let a1279 = {
                                    let a1277 = v7;
                                    let a1278 = 262144u32;
                                    fixed_u32::bit_and(a1277, a1278)
                                };
                                let a1280 = 262144u32;
                                if equal(a1279, a1280) {
                                    262144u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1281, a1282)?
                        };
                        let a1294 = {
                            let a1291 = {
                                let a1285 = {
                                    let a1283 = v7;
                                    let a1284 = 131072u32;
                                    fixed_u32::bit_and(a1283, a1284)
                                };
                                let a1286 = 131072u32;
                                if equal(a1285, a1286) {
                                    131072u64
                                } else {
                                    0u64
                                }
                            };
                            let a1292 = {
                                let a1289 = {
                                    let a1287 = v7;
                                    let a1288 = 65536u32;
                                    fixed_u32::bit_and(a1287, a1288)
                                };
                                let a1290 = 65536u32;
                                if equal(a1289, a1290) {
                                    65536u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1291, a1292)?
                        };
                        nat_add(a1293, a1294)?
                    };
                    nat_add(a1295, a1296)?
                };
                nat_add(a1297, a1298)?
            };
            let a1394 = {
                let a1391 = {
                    let a1343 = {
                        let a1319 = {
                            let a1307 = {
                                let a1301 = {
                                    let a1299 = v7;
                                    let a1300 = 32768u32;
                                    fixed_u32::bit_and(a1299, a1300)
                                };
                                let a1302 = 32768u32;
                                if equal(a1301, a1302) {
                                    32768u64
                                } else {
                                    0u64
                                }
                            };
                            let a1308 = {
                                let a1305 = {
                                    let a1303 = v7;
                                    let a1304 = 16384u32;
                                    fixed_u32::bit_and(a1303, a1304)
                                };
                                let a1306 = 16384u32;
                                if equal(a1305, a1306) {
                                    16384u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1307, a1308)?
                        };
                        let a1320 = {
                            let a1317 = {
                                let a1311 = {
                                    let a1309 = v7;
                                    let a1310 = 8192u32;
                                    fixed_u32::bit_and(a1309, a1310)
                                };
                                let a1312 = 8192u32;
                                if equal(a1311, a1312) {
                                    8192u64
                                } else {
                                    0u64
                                }
                            };
                            let a1318 = {
                                let a1315 = {
                                    let a1313 = v7;
                                    let a1314 = 4096u32;
                                    fixed_u32::bit_and(a1313, a1314)
                                };
                                let a1316 = 4096u32;
                                if equal(a1315, a1316) {
                                    4096u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1317, a1318)?
                        };
                        nat_add(a1319, a1320)?
                    };
                    let a1344 = {
                        let a1341 = {
                            let a1329 = {
                                let a1323 = {
                                    let a1321 = v7;
                                    let a1322 = 2048u32;
                                    fixed_u32::bit_and(a1321, a1322)
                                };
                                let a1324 = 2048u32;
                                if equal(a1323, a1324) {
                                    2048u64
                                } else {
                                    0u64
                                }
                            };
                            let a1330 = {
                                let a1327 = {
                                    let a1325 = v7;
                                    let a1326 = 1024u32;
                                    fixed_u32::bit_and(a1325, a1326)
                                };
                                let a1328 = 1024u32;
                                if equal(a1327, a1328) {
                                    1024u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1329, a1330)?
                        };
                        let a1342 = {
                            let a1339 = {
                                let a1333 = {
                                    let a1331 = v7;
                                    let a1332 = 512u32;
                                    fixed_u32::bit_and(a1331, a1332)
                                };
                                let a1334 = 512u32;
                                if equal(a1333, a1334) {
                                    512u64
                                } else {
                                    0u64
                                }
                            };
                            let a1340 = {
                                let a1337 = {
                                    let a1335 = v7;
                                    let a1336 = 256u32;
                                    fixed_u32::bit_and(a1335, a1336)
                                };
                                let a1338 = 256u32;
                                if equal(a1337, a1338) {
                                    256u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1339, a1340)?
                        };
                        nat_add(a1341, a1342)?
                    };
                    nat_add(a1343, a1344)?
                };
                let a1392 = {
                    let a1389 = {
                        let a1365 = {
                            let a1353 = {
                                let a1347 = {
                                    let a1345 = v7;
                                    let a1346 = 128u32;
                                    fixed_u32::bit_and(a1345, a1346)
                                };
                                let a1348 = 128u32;
                                if equal(a1347, a1348) {
                                    128u64
                                } else {
                                    0u64
                                }
                            };
                            let a1354 = {
                                let a1351 = {
                                    let a1349 = v7;
                                    let a1350 = 64u32;
                                    fixed_u32::bit_and(a1349, a1350)
                                };
                                let a1352 = 64u32;
                                if equal(a1351, a1352) {
                                    64u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1353, a1354)?
                        };
                        let a1366 = {
                            let a1363 = {
                                let a1357 = {
                                    let a1355 = v7;
                                    let a1356 = 32u32;
                                    fixed_u32::bit_and(a1355, a1356)
                                };
                                let a1358 = 32u32;
                                if equal(a1357, a1358) {
                                    32u64
                                } else {
                                    0u64
                                }
                            };
                            let a1364 = {
                                let a1361 = {
                                    let a1359 = v7;
                                    let a1360 = 16u32;
                                    fixed_u32::bit_and(a1359, a1360)
                                };
                                let a1362 = 16u32;
                                if equal(a1361, a1362) {
                                    16u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1363, a1364)?
                        };
                        nat_add(a1365, a1366)?
                    };
                    let a1390 = {
                        let a1387 = {
                            let a1375 = {
                                let a1369 = {
                                    let a1367 = v7;
                                    let a1368 = 8u32;
                                    fixed_u32::bit_and(a1367, a1368)
                                };
                                let a1370 = 8u32;
                                if equal(a1369, a1370) {
                                    8u64
                                } else {
                                    0u64
                                }
                            };
                            let a1376 = {
                                let a1373 = {
                                    let a1371 = v7;
                                    let a1372 = 4u32;
                                    fixed_u32::bit_and(a1371, a1372)
                                };
                                let a1374 = 4u32;
                                if equal(a1373, a1374) {
                                    4u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1375, a1376)?
                        };
                        let a1388 = {
                            let a1385 = {
                                let a1379 = {
                                    let a1377 = v7;
                                    let a1378 = 2u32;
                                    fixed_u32::bit_and(a1377, a1378)
                                };
                                let a1380 = 2u32;
                                if equal(a1379, a1380) {
                                    2u64
                                } else {
                                    0u64
                                }
                            };
                            let a1386 = {
                                let a1383 = {
                                    let a1381 = v7;
                                    let a1382 = 1u32;
                                    fixed_u32::bit_and(a1381, a1382)
                                };
                                let a1384 = 1u32;
                                if equal(a1383, a1384) {
                                    1u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1385, a1386)?
                        };
                        nat_add(a1387, a1388)?
                    };
                    nat_add(a1389, a1390)?
                };
                nat_add(a1391, a1392)?
            };
            nat_add(a1393, a1394)?
        });
        append_bytes(a1395, a1396)
    };
    let a1589 = f14(4u64, {
        let v8: u32 = match v0 {
            Adt0::C0(_, _, _, _, _, _, _, h1397) => {
                h1397
            }
        };
        let a1586 = {
            let a1490 = {
                let a1442 = {
                    let a1418 = {
                        let a1406 = {
                            let a1400 = {
                                let a1398 = v8;
                                let a1399 = 2147483648u32;
                                fixed_u32::bit_and(a1398, a1399)
                            };
                            let a1401 = 2147483648u32;
                            if equal(a1400, a1401) {
                                2147483648u64
                            } else {
                                0u64
                            }
                        };
                        let a1407 = {
                            let a1404 = {
                                let a1402 = v8;
                                let a1403 = 1073741824u32;
                                fixed_u32::bit_and(a1402, a1403)
                            };
                            let a1405 = 1073741824u32;
                            if equal(a1404, a1405) {
                                1073741824u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1406, a1407)?
                    };
                    let a1419 = {
                        let a1416 = {
                            let a1410 = {
                                let a1408 = v8;
                                let a1409 = 536870912u32;
                                fixed_u32::bit_and(a1408, a1409)
                            };
                            let a1411 = 536870912u32;
                            if equal(a1410, a1411) {
                                536870912u64
                            } else {
                                0u64
                            }
                        };
                        let a1417 = {
                            let a1414 = {
                                let a1412 = v8;
                                let a1413 = 268435456u32;
                                fixed_u32::bit_and(a1412, a1413)
                            };
                            let a1415 = 268435456u32;
                            if equal(a1414, a1415) {
                                268435456u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1416, a1417)?
                    };
                    nat_add(a1418, a1419)?
                };
                let a1443 = {
                    let a1440 = {
                        let a1428 = {
                            let a1422 = {
                                let a1420 = v8;
                                let a1421 = 134217728u32;
                                fixed_u32::bit_and(a1420, a1421)
                            };
                            let a1423 = 134217728u32;
                            if equal(a1422, a1423) {
                                134217728u64
                            } else {
                                0u64
                            }
                        };
                        let a1429 = {
                            let a1426 = {
                                let a1424 = v8;
                                let a1425 = 67108864u32;
                                fixed_u32::bit_and(a1424, a1425)
                            };
                            let a1427 = 67108864u32;
                            if equal(a1426, a1427) {
                                67108864u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1428, a1429)?
                    };
                    let a1441 = {
                        let a1438 = {
                            let a1432 = {
                                let a1430 = v8;
                                let a1431 = 33554432u32;
                                fixed_u32::bit_and(a1430, a1431)
                            };
                            let a1433 = 33554432u32;
                            if equal(a1432, a1433) {
                                33554432u64
                            } else {
                                0u64
                            }
                        };
                        let a1439 = {
                            let a1436 = {
                                let a1434 = v8;
                                let a1435 = 16777216u32;
                                fixed_u32::bit_and(a1434, a1435)
                            };
                            let a1437 = 16777216u32;
                            if equal(a1436, a1437) {
                                16777216u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1438, a1439)?
                    };
                    nat_add(a1440, a1441)?
                };
                nat_add(a1442, a1443)?
            };
            let a1491 = {
                let a1488 = {
                    let a1464 = {
                        let a1452 = {
                            let a1446 = {
                                let a1444 = v8;
                                let a1445 = 8388608u32;
                                fixed_u32::bit_and(a1444, a1445)
                            };
                            let a1447 = 8388608u32;
                            if equal(a1446, a1447) {
                                8388608u64
                            } else {
                                0u64
                            }
                        };
                        let a1453 = {
                            let a1450 = {
                                let a1448 = v8;
                                let a1449 = 4194304u32;
                                fixed_u32::bit_and(a1448, a1449)
                            };
                            let a1451 = 4194304u32;
                            if equal(a1450, a1451) {
                                4194304u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1452, a1453)?
                    };
                    let a1465 = {
                        let a1462 = {
                            let a1456 = {
                                let a1454 = v8;
                                let a1455 = 2097152u32;
                                fixed_u32::bit_and(a1454, a1455)
                            };
                            let a1457 = 2097152u32;
                            if equal(a1456, a1457) {
                                2097152u64
                            } else {
                                0u64
                            }
                        };
                        let a1463 = {
                            let a1460 = {
                                let a1458 = v8;
                                let a1459 = 1048576u32;
                                fixed_u32::bit_and(a1458, a1459)
                            };
                            let a1461 = 1048576u32;
                            if equal(a1460, a1461) {
                                1048576u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1462, a1463)?
                    };
                    nat_add(a1464, a1465)?
                };
                let a1489 = {
                    let a1486 = {
                        let a1474 = {
                            let a1468 = {
                                let a1466 = v8;
                                let a1467 = 524288u32;
                                fixed_u32::bit_and(a1466, a1467)
                            };
                            let a1469 = 524288u32;
                            if equal(a1468, a1469) {
                                524288u64
                            } else {
                                0u64
                            }
                        };
                        let a1475 = {
                            let a1472 = {
                                let a1470 = v8;
                                let a1471 = 262144u32;
                                fixed_u32::bit_and(a1470, a1471)
                            };
                            let a1473 = 262144u32;
                            if equal(a1472, a1473) {
                                262144u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1474, a1475)?
                    };
                    let a1487 = {
                        let a1484 = {
                            let a1478 = {
                                let a1476 = v8;
                                let a1477 = 131072u32;
                                fixed_u32::bit_and(a1476, a1477)
                            };
                            let a1479 = 131072u32;
                            if equal(a1478, a1479) {
                                131072u64
                            } else {
                                0u64
                            }
                        };
                        let a1485 = {
                            let a1482 = {
                                let a1480 = v8;
                                let a1481 = 65536u32;
                                fixed_u32::bit_and(a1480, a1481)
                            };
                            let a1483 = 65536u32;
                            if equal(a1482, a1483) {
                                65536u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1484, a1485)?
                    };
                    nat_add(a1486, a1487)?
                };
                nat_add(a1488, a1489)?
            };
            nat_add(a1490, a1491)?
        };
        let a1587 = {
            let a1584 = {
                let a1536 = {
                    let a1512 = {
                        let a1500 = {
                            let a1494 = {
                                let a1492 = v8;
                                let a1493 = 32768u32;
                                fixed_u32::bit_and(a1492, a1493)
                            };
                            let a1495 = 32768u32;
                            if equal(a1494, a1495) {
                                32768u64
                            } else {
                                0u64
                            }
                        };
                        let a1501 = {
                            let a1498 = {
                                let a1496 = v8;
                                let a1497 = 16384u32;
                                fixed_u32::bit_and(a1496, a1497)
                            };
                            let a1499 = 16384u32;
                            if equal(a1498, a1499) {
                                16384u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1500, a1501)?
                    };
                    let a1513 = {
                        let a1510 = {
                            let a1504 = {
                                let a1502 = v8;
                                let a1503 = 8192u32;
                                fixed_u32::bit_and(a1502, a1503)
                            };
                            let a1505 = 8192u32;
                            if equal(a1504, a1505) {
                                8192u64
                            } else {
                                0u64
                            }
                        };
                        let a1511 = {
                            let a1508 = {
                                let a1506 = v8;
                                let a1507 = 4096u32;
                                fixed_u32::bit_and(a1506, a1507)
                            };
                            let a1509 = 4096u32;
                            if equal(a1508, a1509) {
                                4096u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1510, a1511)?
                    };
                    nat_add(a1512, a1513)?
                };
                let a1537 = {
                    let a1534 = {
                        let a1522 = {
                            let a1516 = {
                                let a1514 = v8;
                                let a1515 = 2048u32;
                                fixed_u32::bit_and(a1514, a1515)
                            };
                            let a1517 = 2048u32;
                            if equal(a1516, a1517) {
                                2048u64
                            } else {
                                0u64
                            }
                        };
                        let a1523 = {
                            let a1520 = {
                                let a1518 = v8;
                                let a1519 = 1024u32;
                                fixed_u32::bit_and(a1518, a1519)
                            };
                            let a1521 = 1024u32;
                            if equal(a1520, a1521) {
                                1024u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1522, a1523)?
                    };
                    let a1535 = {
                        let a1532 = {
                            let a1526 = {
                                let a1524 = v8;
                                let a1525 = 512u32;
                                fixed_u32::bit_and(a1524, a1525)
                            };
                            let a1527 = 512u32;
                            if equal(a1526, a1527) {
                                512u64
                            } else {
                                0u64
                            }
                        };
                        let a1533 = {
                            let a1530 = {
                                let a1528 = v8;
                                let a1529 = 256u32;
                                fixed_u32::bit_and(a1528, a1529)
                            };
                            let a1531 = 256u32;
                            if equal(a1530, a1531) {
                                256u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1532, a1533)?
                    };
                    nat_add(a1534, a1535)?
                };
                nat_add(a1536, a1537)?
            };
            let a1585 = {
                let a1582 = {
                    let a1558 = {
                        let a1546 = {
                            let a1540 = {
                                let a1538 = v8;
                                let a1539 = 128u32;
                                fixed_u32::bit_and(a1538, a1539)
                            };
                            let a1541 = 128u32;
                            if equal(a1540, a1541) {
                                128u64
                            } else {
                                0u64
                            }
                        };
                        let a1547 = {
                            let a1544 = {
                                let a1542 = v8;
                                let a1543 = 64u32;
                                fixed_u32::bit_and(a1542, a1543)
                            };
                            let a1545 = 64u32;
                            if equal(a1544, a1545) {
                                64u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1546, a1547)?
                    };
                    let a1559 = {
                        let a1556 = {
                            let a1550 = {
                                let a1548 = v8;
                                let a1549 = 32u32;
                                fixed_u32::bit_and(a1548, a1549)
                            };
                            let a1551 = 32u32;
                            if equal(a1550, a1551) {
                                32u64
                            } else {
                                0u64
                            }
                        };
                        let a1557 = {
                            let a1554 = {
                                let a1552 = v8;
                                let a1553 = 16u32;
                                fixed_u32::bit_and(a1552, a1553)
                            };
                            let a1555 = 16u32;
                            if equal(a1554, a1555) {
                                16u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1556, a1557)?
                    };
                    nat_add(a1558, a1559)?
                };
                let a1583 = {
                    let a1580 = {
                        let a1568 = {
                            let a1562 = {
                                let a1560 = v8;
                                let a1561 = 8u32;
                                fixed_u32::bit_and(a1560, a1561)
                            };
                            let a1563 = 8u32;
                            if equal(a1562, a1563) {
                                8u64
                            } else {
                                0u64
                            }
                        };
                        let a1569 = {
                            let a1566 = {
                                let a1564 = v8;
                                let a1565 = 4u32;
                                fixed_u32::bit_and(a1564, a1565)
                            };
                            let a1567 = 4u32;
                            if equal(a1566, a1567) {
                                4u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1568, a1569)?
                    };
                    let a1581 = {
                        let a1578 = {
                            let a1572 = {
                                let a1570 = v8;
                                let a1571 = 2u32;
                                fixed_u32::bit_and(a1570, a1571)
                            };
                            let a1573 = 2u32;
                            if equal(a1572, a1573) {
                                2u64
                            } else {
                                0u64
                            }
                        };
                        let a1579 = {
                            let a1576 = {
                                let a1574 = v8;
                                let a1575 = 1u32;
                                fixed_u32::bit_and(a1574, a1575)
                            };
                            let a1577 = 1u32;
                            if equal(a1576, a1577) {
                                1u64
                            } else {
                                0u64
                            }
                        };
                        nat_add(a1578, a1579)?
                    };
                    nat_add(a1580, a1581)?
                };
                nat_add(a1582, a1583)?
            };
            nat_add(a1584, a1585)?
        };
        nat_add(a1586, a1587)?
    });
    Ok(append_bytes(a1588, a1589))
}

pub fn f7(p1594: u64, p1595: Adt0, v2: Bytes, p1596: u64) -> R<Adt0> {
    let m1593 = (p1594, (p1595, p1596));
    {
        let mut m1593 = m1593;
        loop {
            let step = {
                let (v0, (v1, v3)) = m1593;
                let m1590 = v0;
                if m1590 == 0 {
                    Err::<(u64, (Adt0, u64)), R<Adt0>>(Ok(v1))
                } else {
                    let v4 = m1590 - 1;
                    Ok::<(u64, (Adt0, u64)), R<Adt0>>((v4, (f16(v1, v2.clone(), v3)?, {
                        let a1591 = v3;
                        let a1592 = f8();
                        nat_add(a1591, a1592)?
                    })))
                }
            };
            match step {
                Ok(next) => { m1593 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f8() -> u64 {
    64u64
}

pub fn f9() -> Adt0 {
    Adt0::C0(1779033703u32, 3144134277u32, 1013904242u32, 2773480762u32, 1359893119u32, 2600822924u32, 528734635u32, 1541459225u32)
}

pub fn f10(v0: u64, v1: Bytes, v2: u64) -> R<List<u64>> {
    let m1597 = v0;
    if m1597 == 0 {
        Ok(List::<u64>::nil())
    } else {
        let v3 = m1597 - 1;
        let m1600 = {
            let a1598 = v1.clone();
            let a1599 = v2;
            index_bytes(a1598, a1599)
        };
        match m1600 {
            None => {
                Ok(List::<u64>::nil())
            }
            Some(v4) => {
                Ok(List::cons({
                    let v5: u8 = v4;
                    let a1645 = {
                        let a1621 = {
                            let a1609 = {
                                let a1603 = {
                                    let a1601 = v5;
                                    let a1602 = 128u8;
                                    fixed_u8::bit_and(a1601, a1602)
                                };
                                let a1604 = 128u8;
                                if equal(a1603, a1604) {
                                    128u64
                                } else {
                                    0u64
                                }
                            };
                            let a1610 = {
                                let a1607 = {
                                    let a1605 = v5;
                                    let a1606 = 64u8;
                                    fixed_u8::bit_and(a1605, a1606)
                                };
                                let a1608 = 64u8;
                                if equal(a1607, a1608) {
                                    64u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1609, a1610)?
                        };
                        let a1622 = {
                            let a1619 = {
                                let a1613 = {
                                    let a1611 = v5;
                                    let a1612 = 32u8;
                                    fixed_u8::bit_and(a1611, a1612)
                                };
                                let a1614 = 32u8;
                                if equal(a1613, a1614) {
                                    32u64
                                } else {
                                    0u64
                                }
                            };
                            let a1620 = {
                                let a1617 = {
                                    let a1615 = v5;
                                    let a1616 = 16u8;
                                    fixed_u8::bit_and(a1615, a1616)
                                };
                                let a1618 = 16u8;
                                if equal(a1617, a1618) {
                                    16u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1619, a1620)?
                        };
                        nat_add(a1621, a1622)?
                    };
                    let a1646 = {
                        let a1643 = {
                            let a1631 = {
                                let a1625 = {
                                    let a1623 = v5;
                                    let a1624 = 8u8;
                                    fixed_u8::bit_and(a1623, a1624)
                                };
                                let a1626 = 8u8;
                                if equal(a1625, a1626) {
                                    8u64
                                } else {
                                    0u64
                                }
                            };
                            let a1632 = {
                                let a1629 = {
                                    let a1627 = v5;
                                    let a1628 = 4u8;
                                    fixed_u8::bit_and(a1627, a1628)
                                };
                                let a1630 = 4u8;
                                if equal(a1629, a1630) {
                                    4u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1631, a1632)?
                        };
                        let a1644 = {
                            let a1641 = {
                                let a1635 = {
                                    let a1633 = v5;
                                    let a1634 = 2u8;
                                    fixed_u8::bit_and(a1633, a1634)
                                };
                                let a1636 = 2u8;
                                if equal(a1635, a1636) {
                                    2u64
                                } else {
                                    0u64
                                }
                            };
                            let a1642 = {
                                let a1639 = {
                                    let a1637 = v5;
                                    let a1638 = 1u8;
                                    fixed_u8::bit_and(a1637, a1638)
                                };
                                let a1640 = 1u8;
                                if equal(a1639, a1640) {
                                    1u64
                                } else {
                                    0u64
                                }
                            };
                            nat_add(a1641, a1642)?
                        };
                        nat_add(a1643, a1644)?
                    };
                    nat_add(a1645, a1646)?
                }, f10(v3, v1, {
                    let a1647 = v2;
                    let a1648 = 1u64;
                    nat_add(a1647, a1648)?
                })?))
            }
        }
    }
}

pub fn f11(v0: u64) -> Bytes {
    let m1652 = {
        let a1649 = Bytes::lit(&[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66]);
        let a1650 = v0;
        let a1651 = 1u64;
        slice_bytes(a1649, a1650, a1651)
    };
    match m1652 {
        None => {
            Bytes::lit(&[])
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f12() -> u64 {
    8u64
}

pub fn f13(v0: u64) -> Bytes {
    let m1653 = v0;
    if m1653 == 0 {
        Bytes::lit(&[])
    } else {
        let v1 = m1653 - 1;
        let a1654 = Bytes::lit(&[0x00]);
        let a1655 = f13(v1);
        append_bytes(a1654, a1655)
    }
}

pub fn f14(v0: u64, v1: u64) -> Bytes {
    let m1656 = v0;
    if m1656 == 0 {
        Bytes::lit(&[])
    } else {
        let v2 = m1656 - 1;
        let a1663 = f14(v2, {
            let a1657 = v1;
            let a1658 = 256u64;
            let a1659 = 0u64;
            nat_quot(a1657, a1658, a1659)
        });
        let a1664 = f15({
            let a1660 = v1;
            let a1661 = 256u64;
            let a1662 = 0u64;
            nat_rem(a1660, a1661, a1662)
        });
        append_bytes(a1663, a1664)
    }
}

pub fn f15(v0: u64) -> Bytes {
    let m1668 = {
        let a1665 = Bytes::lit(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf, 0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff]);
        let a1666 = v0;
        let a1667 = 1u64;
        slice_bytes(a1665, a1666, a1667)
    };
    match m1668 {
        None => {
            Bytes::lit(&[])
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f16(v0: Adt0, v1: Bytes, v2: u64) -> R<Adt0> {
    let v3: Adt0 = {
        let m1702 = f17(f18(), Adt1::C0(v0.clone(), List::cons(f19(v1.clone(), {
            let a1670 = v2;
            let a1671 = 0u64;
            nat_add(a1670, a1671)?
        })?, List::cons(f19(v1.clone(), {
            let a1672 = v2;
            let a1673 = 4u64;
            nat_add(a1672, a1673)?
        })?, List::cons(f19(v1.clone(), {
            let a1674 = v2;
            let a1675 = 8u64;
            nat_add(a1674, a1675)?
        })?, List::cons(f19(v1.clone(), {
            let a1676 = v2;
            let a1677 = 12u64;
            nat_add(a1676, a1677)?
        })?, List::cons(f19(v1.clone(), {
            let a1678 = v2;
            let a1679 = 16u64;
            nat_add(a1678, a1679)?
        })?, List::cons(f19(v1.clone(), {
            let a1680 = v2;
            let a1681 = 20u64;
            nat_add(a1680, a1681)?
        })?, List::cons(f19(v1.clone(), {
            let a1682 = v2;
            let a1683 = 24u64;
            nat_add(a1682, a1683)?
        })?, List::cons(f19(v1.clone(), {
            let a1684 = v2;
            let a1685 = 28u64;
            nat_add(a1684, a1685)?
        })?, List::cons(f19(v1.clone(), {
            let a1686 = v2;
            let a1687 = 32u64;
            nat_add(a1686, a1687)?
        })?, List::cons(f19(v1.clone(), {
            let a1688 = v2;
            let a1689 = 36u64;
            nat_add(a1688, a1689)?
        })?, List::cons(f19(v1.clone(), {
            let a1690 = v2;
            let a1691 = 40u64;
            nat_add(a1690, a1691)?
        })?, List::cons(f19(v1.clone(), {
            let a1692 = v2;
            let a1693 = 44u64;
            nat_add(a1692, a1693)?
        })?, List::cons(f19(v1.clone(), {
            let a1694 = v2;
            let a1695 = 48u64;
            nat_add(a1694, a1695)?
        })?, List::cons(f19(v1.clone(), {
            let a1696 = v2;
            let a1697 = 52u64;
            nat_add(a1696, a1697)?
        })?, List::cons(f19(v1.clone(), {
            let a1698 = v2;
            let a1699 = 56u64;
            nat_add(a1698, a1699)?
        })?, List::cons(f19(v1, {
            let a1700 = v2;
            let a1701 = 60u64;
            nat_add(a1700, a1701)?
        })?, List::<u32>::nil())))))))))))))))), 0u64))?;
        match m1702 {
            Adt1::C0(h1669, _, _) => {
                h1669
            }
        }
    };
    Ok(Adt0::C0(f20(match v0.clone() {
        Adt0::C0(h1703, _, _, _, _, _, _, _) => {
            h1703
        }
    }, match v3.clone() {
        Adt0::C0(h1704, _, _, _, _, _, _, _) => {
            h1704
        }
    }), f20(match v0.clone() {
        Adt0::C0(_, h1705, _, _, _, _, _, _) => {
            h1705
        }
    }, match v3.clone() {
        Adt0::C0(_, h1706, _, _, _, _, _, _) => {
            h1706
        }
    }), f20(match v0.clone() {
        Adt0::C0(_, _, h1707, _, _, _, _, _) => {
            h1707
        }
    }, match v3.clone() {
        Adt0::C0(_, _, h1708, _, _, _, _, _) => {
            h1708
        }
    }), f20(match v0.clone() {
        Adt0::C0(_, _, _, h1709, _, _, _, _) => {
            h1709
        }
    }, match v3.clone() {
        Adt0::C0(_, _, _, h1710, _, _, _, _) => {
            h1710
        }
    }), f20(match v0.clone() {
        Adt0::C0(_, _, _, _, h1711, _, _, _) => {
            h1711
        }
    }, match v3.clone() {
        Adt0::C0(_, _, _, _, h1712, _, _, _) => {
            h1712
        }
    }), f20(match v0.clone() {
        Adt0::C0(_, _, _, _, _, h1713, _, _) => {
            h1713
        }
    }, match v3.clone() {
        Adt0::C0(_, _, _, _, _, h1714, _, _) => {
            h1714
        }
    }), f20(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, h1715, _) => {
            h1715
        }
    }, match v3.clone() {
        Adt0::C0(_, _, _, _, _, _, h1716, _) => {
            h1716
        }
    }), f20(match v0 {
        Adt0::C0(_, _, _, _, _, _, _, h1717) => {
            h1717
        }
    }, match v3 {
        Adt0::C0(_, _, _, _, _, _, _, h1718) => {
            h1718
        }
    })))
}

pub fn f17(p1721: List<u32>, p1722: Adt1) -> R<Adt1> {
    let m1720 = (p1721, p1722);
    {
        let mut m1720 = m1720;
        loop {
            let step = {
                let (v0, v1) = m1720;
                let m1719 = v0;
                match m1719.uncons() {
                    None => {
                        Err::<(List<u32>, Adt1), R<Adt1>>(Ok(v1))
                    }
                    Some((v2, v3)) => {
                        Ok::<(List<u32>, Adt1), R<Adt1>>((v3, f21(v1, v2)?))
                    }
                }
            };
            match step {
                Ok(next) => { m1720 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f18() -> List<u32> {
    List::cons(1116352408u32, List::cons(1899447441u32, List::cons(3049323471u32, List::cons(3921009573u32, List::cons(961987163u32, List::cons(1508970993u32, List::cons(2453635748u32, List::cons(2870763221u32, List::cons(3624381080u32, List::cons(310598401u32, List::cons(607225278u32, List::cons(1426881987u32, List::cons(1925078388u32, List::cons(2162078206u32, List::cons(2614888103u32, List::cons(3248222580u32, List::cons(3835390401u32, List::cons(4022224774u32, List::cons(264347078u32, List::cons(604807628u32, List::cons(770255983u32, List::cons(1249150122u32, List::cons(1555081692u32, List::cons(1996064986u32, List::cons(2554220882u32, List::cons(2821834349u32, List::cons(2952996808u32, List::cons(3210313671u32, List::cons(3336571891u32, List::cons(3584528711u32, List::cons(113926993u32, List::cons(338241895u32, List::cons(666307205u32, List::cons(773529912u32, List::cons(1294757372u32, List::cons(1396182291u32, List::cons(1695183700u32, List::cons(1986661051u32, List::cons(2177026350u32, List::cons(2456956037u32, List::cons(2730485921u32, List::cons(2820302411u32, List::cons(3259730800u32, List::cons(3345764771u32, List::cons(3516065817u32, List::cons(3600352804u32, List::cons(4094571909u32, List::cons(275423344u32, List::cons(430227734u32, List::cons(506948616u32, List::cons(659060556u32, List::cons(883997877u32, List::cons(958139571u32, List::cons(1322822218u32, List::cons(1537002063u32, List::cons(1747873779u32, List::cons(1955562222u32, List::cons(2024104815u32, List::cons(2227730452u32, List::cons(2361852424u32, List::cons(2428436474u32, List::cons(2756734187u32, List::cons(3204031479u32, List::cons(3329325298u32, List::<u32>::nil()))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))
}

pub fn f19(v0: Bytes, v1: u64) -> R<u32> {
    let a1754 = {
        let a1737 = {
            let m1728 = {
                let a1726 = f22({
                    let m1725 = {
                        let a1723 = v0.clone();
                        let a1724 = v1;
                        index_bytes(a1723, a1724)
                    };
                    match m1725 {
                        None => {
                            0u8
                        }
                        Some(v2) => {
                            v2
                        }
                    }
                });
                let a1727 = 24u32;
                fixed_u32::shift_left(a1726, a1727)
            };
            match m1728 {
                None => {
                    0u32
                }
                Some(v3) => {
                    v3
                }
            }
        };
        let a1738 = {
            let m1736 = {
                let a1734 = f22({
                    let m1733 = {
                        let a1731 = v0.clone();
                        let a1732 = {
                            let a1729 = v1;
                            let a1730 = 1u64;
                            nat_add(a1729, a1730)?
                        };
                        index_bytes(a1731, a1732)
                    };
                    match m1733 {
                        None => {
                            0u8
                        }
                        Some(v4) => {
                            v4
                        }
                    }
                });
                let a1735 = 16u32;
                fixed_u32::shift_left(a1734, a1735)
            };
            match m1736 {
                None => {
                    0u32
                }
                Some(v5) => {
                    v5
                }
            }
        };
        fixed_u32::bit_or(a1737, a1738)
    };
    let a1755 = {
        let a1752 = {
            let m1746 = {
                let a1744 = f22({
                    let m1743 = {
                        let a1741 = v0.clone();
                        let a1742 = {
                            let a1739 = v1;
                            let a1740 = 2u64;
                            nat_add(a1739, a1740)?
                        };
                        index_bytes(a1741, a1742)
                    };
                    match m1743 {
                        None => {
                            0u8
                        }
                        Some(v6) => {
                            v6
                        }
                    }
                });
                let a1745 = 8u32;
                fixed_u32::shift_left(a1744, a1745)
            };
            match m1746 {
                None => {
                    0u32
                }
                Some(v7) => {
                    v7
                }
            }
        };
        let a1753 = f22({
            let m1751 = {
                let a1749 = v0;
                let a1750 = {
                    let a1747 = v1;
                    let a1748 = 3u64;
                    nat_add(a1747, a1748)?
                };
                index_bytes(a1749, a1750)
            };
            match m1751 {
                None => {
                    0u8
                }
                Some(v8) => {
                    v8
                }
            }
        });
        fixed_u32::bit_or(a1752, a1753)
    };
    Ok(fixed_u32::bit_or(a1754, a1755))
}

pub fn f20(v0: u32, v1: u32) -> u32 {
    let m1758 = {
        let a1756 = v0;
        let a1757 = v1;
        fixed_u32::checked_add(a1756, a1757)
    };
    match m1758 {
        None => {
            f23(f23(v0, {
                let a1759 = v1;
                fixed_u32::bit_not(a1759)
            }), 1u32)
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f21(v0: Adt1, v1: u32) -> R<Adt1> {
    Ok(Adt1::C0(f24(match v0.clone() {
        Adt1::C0(h1760, _, _) => {
            h1760
        }
    }, f25(v0.clone())?, v1), f26(match v0.clone() {
        Adt1::C0(_, h1761, _) => {
            h1761
        }
    }, {
        let a1763 = match v0.clone() {
            Adt1::C0(_, _, h1762) => {
                h1762
            }
        };
        let a1764 = 16u64;
        let a1765 = 0u64;
        nat_rem(a1763, a1764, a1765)
    }, f25(v0.clone())?), {
        let a1767 = match v0 {
            Adt1::C0(_, _, h1766) => {
                h1766
            }
        };
        let a1768 = 1u64;
        nat_add(a1767, a1768)?
    }))
}

pub fn f22(v0: u8) -> u32 {
    let m1770 = {
        let a1769 = v0;
        fixed_u32::convert(i128::from(a1769))
    };
    match m1770 {
        None => {
            0u32
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f23(v0: u32, v1: u32) -> u32 {
    let m1773 = {
        let a1771 = v0;
        let a1772 = v1;
        fixed_u32::checked_sub(a1771, a1772)
    };
    match m1773 {
        None => {
            0u32
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f24(v0: Adt0, v1: u32, v2: u32) -> Adt0 {
    Adt0::C0(f20(f27(v0.clone(), v1, v2), f28(v0.clone())), match v0.clone() {
        Adt0::C0(h1774, _, _, _, _, _, _, _) => {
            h1774
        }
    }, match v0.clone() {
        Adt0::C0(_, h1775, _, _, _, _, _, _) => {
            h1775
        }
    }, match v0.clone() {
        Adt0::C0(_, _, h1776, _, _, _, _, _) => {
            h1776
        }
    }, f20(match v0.clone() {
        Adt0::C0(_, _, _, h1777, _, _, _, _) => {
            h1777
        }
    }, f27(v0.clone(), v1, v2)), match v0.clone() {
        Adt0::C0(_, _, _, _, h1778, _, _, _) => {
            h1778
        }
    }, match v0.clone() {
        Adt0::C0(_, _, _, _, _, h1779, _, _) => {
            h1779
        }
    }, match v0 {
        Adt0::C0(_, _, _, _, _, _, h1780, _) => {
            h1780
        }
    })
}

pub fn f25(v0: Adt1) -> R<u32> {
    let a1782 = match v0.clone() {
        Adt1::C0(_, _, h1781) => {
            h1781
        }
    };
    let a1783 = 16u64;
    if nat_lt(a1782, a1783) {
        let m1788 = {
            let a1786 = match v0.clone() {
                Adt1::C0(_, h1784, _) => {
                    h1784
                }
            };
            let a1787 = match v0 {
                Adt1::C0(_, _, h1785) => {
                    h1785
                }
            };
            index_list(a1786, a1787)
        };
        match m1788 {
            None => {
                Ok(0u32)
            }
            Some(v1) => {
                Ok(v1)
            }
        }
    } else {
        Ok(f20(f20(f20({
            let m1796 = {
                let a1794 = match v0.clone() {
                    Adt1::C0(_, h1789, _) => {
                        h1789
                    }
                };
                let a1795 = {
                    let a1791 = match v0.clone() {
                        Adt1::C0(_, _, h1790) => {
                            h1790
                        }
                    };
                    let a1792 = 16u64;
                    let a1793 = 0u64;
                    nat_rem(a1791, a1792, a1793)
                };
                index_list(a1794, a1795)
            };
            match m1796 {
                None => {
                    0u32
                }
                Some(v2) => {
                    v2
                }
            }
        }, f29({
            let m1806 = {
                let a1804 = match v0.clone() {
                    Adt1::C0(_, h1797, _) => {
                        h1797
                    }
                };
                let a1805 = {
                    let a1801 = {
                        let a1799 = match v0.clone() {
                            Adt1::C0(_, _, h1798) => {
                                h1798
                            }
                        };
                        let a1800 = 1u64;
                        nat_add(a1799, a1800)?
                    };
                    let a1802 = 16u64;
                    let a1803 = 0u64;
                    nat_rem(a1801, a1802, a1803)
                };
                index_list(a1804, a1805)
            };
            match m1806 {
                None => {
                    0u32
                }
                Some(v3) => {
                    v3
                }
            }
        })), {
            let m1816 = {
                let a1814 = match v0.clone() {
                    Adt1::C0(_, h1807, _) => {
                        h1807
                    }
                };
                let a1815 = {
                    let a1811 = {
                        let a1809 = match v0.clone() {
                            Adt1::C0(_, _, h1808) => {
                                h1808
                            }
                        };
                        let a1810 = 9u64;
                        nat_add(a1809, a1810)?
                    };
                    let a1812 = 16u64;
                    let a1813 = 0u64;
                    nat_rem(a1811, a1812, a1813)
                };
                index_list(a1814, a1815)
            };
            match m1816 {
                None => {
                    0u32
                }
                Some(v4) => {
                    v4
                }
            }
        }), f30({
            let m1826 = {
                let a1824 = match v0.clone() {
                    Adt1::C0(_, h1817, _) => {
                        h1817
                    }
                };
                let a1825 = {
                    let a1821 = {
                        let a1819 = match v0 {
                            Adt1::C0(_, _, h1818) => {
                                h1818
                            }
                        };
                        let a1820 = 14u64;
                        nat_add(a1819, a1820)?
                    };
                    let a1822 = 16u64;
                    let a1823 = 0u64;
                    nat_rem(a1821, a1822, a1823)
                };
                index_list(a1824, a1825)
            };
            match m1826 {
                None => {
                    0u32
                }
                Some(v5) => {
                    v5
                }
            }
        })))
    }
}

pub fn f26(v0: List<u32>, v1: u64, v2: u32) -> List<u32> {
    let m1827 = v0;
    match m1827.uncons() {
        None => {
            List::<u32>::nil()
        }
        Some((v3, v4)) => {
            List::cons({
                let a1828 = v1;
                let a1829 = 0u64;
                if nat_eq(a1828, a1829) {
                    v2
                } else {
                    v3
                }
            }, {
                let a1830 = v1;
                let a1831 = 0u64;
                if nat_eq(a1830, a1831) {
                    v4
                } else {
                    f26(v4, {
                        let a1832 = v1;
                        let a1833 = 1u64;
                        nat_sub(a1832, a1833)
                    }, v2)
                }
            })
        }
    }
}

pub fn f27(v0: Adt0, v1: u32, v2: u32) -> u32 {
    f20(f20(f20(f20(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, h1834) => {
            h1834
        }
    }, f31(match v0.clone() {
        Adt0::C0(_, _, _, _, h1835, _, _, _) => {
            h1835
        }
    })), f32(match v0.clone() {
        Adt0::C0(_, _, _, _, h1836, _, _, _) => {
            h1836
        }
    }, match v0.clone() {
        Adt0::C0(_, _, _, _, _, h1837, _, _) => {
            h1837
        }
    }, match v0 {
        Adt0::C0(_, _, _, _, _, _, h1838, _) => {
            h1838
        }
    })), v2), v1)
}

pub fn f28(v0: Adt0) -> u32 {
    f20(f33(match v0.clone() {
        Adt0::C0(h1839, _, _, _, _, _, _, _) => {
            h1839
        }
    }), f34(match v0.clone() {
        Adt0::C0(h1840, _, _, _, _, _, _, _) => {
            h1840
        }
    }, match v0.clone() {
        Adt0::C0(_, h1841, _, _, _, _, _, _) => {
            h1841
        }
    }, match v0 {
        Adt0::C0(_, _, h1842, _, _, _, _, _) => {
            h1842
        }
    }))
}

pub fn f29(v0: u32) -> u32 {
    let a1864 = {
        let a1859 = {
            let a1849 = {
                let m1845 = {
                    let a1843 = v0;
                    let a1844 = 7u32;
                    fixed_u32::shift_right(a1843, a1844)
                };
                match m1845 {
                    None => {
                        0u32
                    }
                    Some(v1) => {
                        v1
                    }
                }
            };
            let a1850 = {
                let m1848 = {
                    let a1846 = v0;
                    let a1847 = 25u32;
                    fixed_u32::shift_left(a1846, a1847)
                };
                match m1848 {
                    None => {
                        0u32
                    }
                    Some(v2) => {
                        v2
                    }
                }
            };
            fixed_u32::bit_or(a1849, a1850)
        };
        let a1860 = {
            let a1857 = {
                let m1853 = {
                    let a1851 = v0;
                    let a1852 = 18u32;
                    fixed_u32::shift_right(a1851, a1852)
                };
                match m1853 {
                    None => {
                        0u32
                    }
                    Some(v3) => {
                        v3
                    }
                }
            };
            let a1858 = {
                let m1856 = {
                    let a1854 = v0;
                    let a1855 = 14u32;
                    fixed_u32::shift_left(a1854, a1855)
                };
                match m1856 {
                    None => {
                        0u32
                    }
                    Some(v4) => {
                        v4
                    }
                }
            };
            fixed_u32::bit_or(a1857, a1858)
        };
        fixed_u32::bit_xor(a1859, a1860)
    };
    let a1865 = {
        let m1863 = {
            let a1861 = v0;
            let a1862 = 3u32;
            fixed_u32::shift_right(a1861, a1862)
        };
        match m1863 {
            None => {
                0u32
            }
            Some(v5) => {
                v5
            }
        }
    };
    fixed_u32::bit_xor(a1864, a1865)
}

pub fn f30(v0: u32) -> u32 {
    let a1887 = {
        let a1882 = {
            let a1872 = {
                let m1868 = {
                    let a1866 = v0;
                    let a1867 = 17u32;
                    fixed_u32::shift_right(a1866, a1867)
                };
                match m1868 {
                    None => {
                        0u32
                    }
                    Some(v1) => {
                        v1
                    }
                }
            };
            let a1873 = {
                let m1871 = {
                    let a1869 = v0;
                    let a1870 = 15u32;
                    fixed_u32::shift_left(a1869, a1870)
                };
                match m1871 {
                    None => {
                        0u32
                    }
                    Some(v2) => {
                        v2
                    }
                }
            };
            fixed_u32::bit_or(a1872, a1873)
        };
        let a1883 = {
            let a1880 = {
                let m1876 = {
                    let a1874 = v0;
                    let a1875 = 19u32;
                    fixed_u32::shift_right(a1874, a1875)
                };
                match m1876 {
                    None => {
                        0u32
                    }
                    Some(v3) => {
                        v3
                    }
                }
            };
            let a1881 = {
                let m1879 = {
                    let a1877 = v0;
                    let a1878 = 13u32;
                    fixed_u32::shift_left(a1877, a1878)
                };
                match m1879 {
                    None => {
                        0u32
                    }
                    Some(v4) => {
                        v4
                    }
                }
            };
            fixed_u32::bit_or(a1880, a1881)
        };
        fixed_u32::bit_xor(a1882, a1883)
    };
    let a1888 = {
        let m1886 = {
            let a1884 = v0;
            let a1885 = 10u32;
            fixed_u32::shift_right(a1884, a1885)
        };
        match m1886 {
            None => {
                0u32
            }
            Some(v5) => {
                v5
            }
        }
    };
    fixed_u32::bit_xor(a1887, a1888)
}

pub fn f31(v0: u32) -> u32 {
    let a1915 = {
        let a1905 = {
            let a1895 = {
                let m1891 = {
                    let a1889 = v0;
                    let a1890 = 6u32;
                    fixed_u32::shift_right(a1889, a1890)
                };
                match m1891 {
                    None => {
                        0u32
                    }
                    Some(v1) => {
                        v1
                    }
                }
            };
            let a1896 = {
                let m1894 = {
                    let a1892 = v0;
                    let a1893 = 26u32;
                    fixed_u32::shift_left(a1892, a1893)
                };
                match m1894 {
                    None => {
                        0u32
                    }
                    Some(v2) => {
                        v2
                    }
                }
            };
            fixed_u32::bit_or(a1895, a1896)
        };
        let a1906 = {
            let a1903 = {
                let m1899 = {
                    let a1897 = v0;
                    let a1898 = 11u32;
                    fixed_u32::shift_right(a1897, a1898)
                };
                match m1899 {
                    None => {
                        0u32
                    }
                    Some(v3) => {
                        v3
                    }
                }
            };
            let a1904 = {
                let m1902 = {
                    let a1900 = v0;
                    let a1901 = 21u32;
                    fixed_u32::shift_left(a1900, a1901)
                };
                match m1902 {
                    None => {
                        0u32
                    }
                    Some(v4) => {
                        v4
                    }
                }
            };
            fixed_u32::bit_or(a1903, a1904)
        };
        fixed_u32::bit_xor(a1905, a1906)
    };
    let a1916 = {
        let a1913 = {
            let m1909 = {
                let a1907 = v0;
                let a1908 = 25u32;
                fixed_u32::shift_right(a1907, a1908)
            };
            match m1909 {
                None => {
                    0u32
                }
                Some(v5) => {
                    v5
                }
            }
        };
        let a1914 = {
            let m1912 = {
                let a1910 = v0;
                let a1911 = 7u32;
                fixed_u32::shift_left(a1910, a1911)
            };
            match m1912 {
                None => {
                    0u32
                }
                Some(v6) => {
                    v6
                }
            }
        };
        fixed_u32::bit_or(a1913, a1914)
    };
    fixed_u32::bit_xor(a1915, a1916)
}

pub fn f32(v0: u32, v1: u32, v2: u32) -> u32 {
    let a1922 = {
        let a1917 = v0;
        let a1918 = v1;
        fixed_u32::bit_and(a1917, a1918)
    };
    let a1923 = {
        let a1920 = {
            let a1919 = v0;
            fixed_u32::bit_not(a1919)
        };
        let a1921 = v2;
        fixed_u32::bit_and(a1920, a1921)
    };
    fixed_u32::bit_xor(a1922, a1923)
}

pub fn f33(v0: u32) -> u32 {
    let a1950 = {
        let a1940 = {
            let a1930 = {
                let m1926 = {
                    let a1924 = v0;
                    let a1925 = 2u32;
                    fixed_u32::shift_right(a1924, a1925)
                };
                match m1926 {
                    None => {
                        0u32
                    }
                    Some(v1) => {
                        v1
                    }
                }
            };
            let a1931 = {
                let m1929 = {
                    let a1927 = v0;
                    let a1928 = 30u32;
                    fixed_u32::shift_left(a1927, a1928)
                };
                match m1929 {
                    None => {
                        0u32
                    }
                    Some(v2) => {
                        v2
                    }
                }
            };
            fixed_u32::bit_or(a1930, a1931)
        };
        let a1941 = {
            let a1938 = {
                let m1934 = {
                    let a1932 = v0;
                    let a1933 = 13u32;
                    fixed_u32::shift_right(a1932, a1933)
                };
                match m1934 {
                    None => {
                        0u32
                    }
                    Some(v3) => {
                        v3
                    }
                }
            };
            let a1939 = {
                let m1937 = {
                    let a1935 = v0;
                    let a1936 = 19u32;
                    fixed_u32::shift_left(a1935, a1936)
                };
                match m1937 {
                    None => {
                        0u32
                    }
                    Some(v4) => {
                        v4
                    }
                }
            };
            fixed_u32::bit_or(a1938, a1939)
        };
        fixed_u32::bit_xor(a1940, a1941)
    };
    let a1951 = {
        let a1948 = {
            let m1944 = {
                let a1942 = v0;
                let a1943 = 22u32;
                fixed_u32::shift_right(a1942, a1943)
            };
            match m1944 {
                None => {
                    0u32
                }
                Some(v5) => {
                    v5
                }
            }
        };
        let a1949 = {
            let m1947 = {
                let a1945 = v0;
                let a1946 = 10u32;
                fixed_u32::shift_left(a1945, a1946)
            };
            match m1947 {
                None => {
                    0u32
                }
                Some(v6) => {
                    v6
                }
            }
        };
        fixed_u32::bit_or(a1948, a1949)
    };
    fixed_u32::bit_xor(a1950, a1951)
}

pub fn f34(v0: u32, v1: u32, v2: u32) -> u32 {
    let a1960 = {
        let a1956 = {
            let a1952 = v0;
            let a1953 = v1;
            fixed_u32::bit_and(a1952, a1953)
        };
        let a1957 = {
            let a1954 = v0;
            let a1955 = v2;
            fixed_u32::bit_and(a1954, a1955)
        };
        fixed_u32::bit_xor(a1956, a1957)
    };
    let a1961 = {
        let a1958 = v1;
        let a1959 = v2;
        fixed_u32::bit_and(a1958, a1959)
    };
    fixed_u32::bit_xor(a1960, a1961)
}

pub fn run(p0: List<u64>) -> R<Str> {
    f0(p0)
}
