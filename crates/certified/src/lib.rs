//! The certified translator: the Rust `lexlean production` exported from the
//! lifted translator's definitions, one module per production root, each file
//! the certified package's `src/lib.rs` byte for byte. Written by `uorust
//! certify sync`; `crates/certified/manifest.toml` keys each by the κ of the
//! Rust function it certifies.

#![forbid(unsafe_code)]

/// `uorust::lift::Acc::all_axioms`, certified as `Behavior::accAllAxioms`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_all_axioms.rs"]
pub mod acc_all_axioms;

/// `uorust::lift::Acc::apart`, certified as `Behavior::accApart`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_apart.rs"]
pub mod acc_apart;

/// `uorust::lift::Acc::axiom`, certified as `Behavior::accAxiom`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_axiom.rs"]
pub mod acc_axiom;

/// `uorust::lift::Acc::axioms_all`, certified as `Behavior::accAxiomsAll`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_axioms_all.rs"]
pub mod acc_axioms_all;

/// `uorust::lift::Acc::called`, certified as `Behavior::accCalled`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_called.rs"]
pub mod acc_called;

/// `uorust::lift::Acc::called_all`, certified as `Behavior::accCalledAll`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_called_all.rs"]
pub mod acc_called_all;

/// `uorust::lift::Acc::descended`, certified as `Behavior::accDescended`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_descended.rs"]
pub mod acc_descended;

/// `uorust::lift::Acc::helper`, certified as `Behavior::accHelper`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_helper.rs"]
pub mod acc_helper;

/// `uorust::lift::Acc::rejoin`, certified as `Behavior::accRejoin`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_acc_rejoin.rs"]
pub mod acc_rejoin;

/// `uorust::mir::lower::address`, certified as `Behavior::address`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_address.rs"]
pub mod address;

/// `uorust::mir::coverage::aggregate`, certified as `Behavior::aggregate`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_aggregate.rs"]
pub mod aggregate;

/// `uorust::lift::apart`, certified as `Behavior::apart`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_apart.rs"]
pub mod apart;

/// `uorust::mir::coverage::assert`, certified as `Behavior::assert`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_assert.rs"]
pub mod assert;

/// `uorust::mir::parse::assert_kind`, certified as `Behavior::assertKind`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_assert_kind.rs"]
pub mod assert_kind;

/// `uorust::lift::barred`, certified as `Behavior::barred`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_barred.rs"]
pub mod barred;

/// `uorust::mir::coverage::binop`, certified as `Behavior::binop`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_binop.rs"]
pub mod binop;

/// `uorust::mir::coverage::borrow`, certified as `Behavior::borrow`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_borrow.rs"]
pub mod borrow;

/// `uorust::mir::lower::Called::value`, certified as `Behavior::calledValue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_called_value.rs"]
pub mod called_value;

/// `uorust::lex::camel`, certified as `Behavior::camel`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_camel.rs"]
pub mod camel;

/// `uorust::lift::captures`, certified as `Behavior::captures`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_captures.rs"]
pub mod captures;

/// `uorust::lift::comb_collect`, certified as `Behavior::combCollect`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_comb_collect.rs"]
pub mod comb_collect;

/// `uorust::mir::core::const_marker`, certified as `Behavior::constMarker`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_const_marker.rs"]
pub mod const_marker;

/// `uorust::mir::core::const_of_marker`, certified as `Behavior::constOfMarker`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_const_of_marker.rs"]
pub mod const_of_marker;

/// `uorust::mir::core::CoreShard::translated_count`, certified as `Behavior::coreShardTranslatedCount`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_core_shard_translated_count.rs"]
pub mod core_shard_translated_count;

/// `uorust::mir::core::CoreStruct::name`, certified as `Behavior::coreStructName`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_core_struct_name.rs"]
pub mod core_struct_name;

/// `uorust::lift::ctor`, certified as `Behavior::ctorValue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_ctor_value.rs"]
pub mod ctor_value;

/// `uorust::lift::Ctx::below`, certified as `Behavior::ctxBelow`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_ctx_below.rs"]
pub mod ctx_below;

/// `uorust::lift::Ctx::fresh`, certified as `Behavior::ctxFresh`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_ctx_fresh.rs"]
pub mod ctx_fresh;

/// `uorust::lift::Ctx::helper_name`, certified as `Behavior::ctxHelperName`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_ctx_helper_name.rs"]
pub mod ctx_helper_name;

/// `uorust::lift::Ctx::names_from`, certified as `Behavior::ctxNamesFrom`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_ctx_names_from.rs"]
pub mod ctx_names_from;

/// `uorust::lift::Ctx::taken`, certified as `Behavior::ctxTaken`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_ctx_taken.rs"]
pub mod ctx_taken;

/// `uorust::lift::data_variant`, certified as `Behavior::dataVariant`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_data_variant.rs"]
pub mod data_variant;

/// `uorust::lex::Decl::name`, certified as `Behavior::declName`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_decl_name.rs"]
pub mod decl_name;

/// `uorust::mir::lower::decoded`, certified as `Behavior::decoded`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_decoded.rs"]
pub mod decoded;

/// `uorust::mir::memory::definition`, certified as `Behavior::definition`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_definition.rs"]
pub mod definition;

/// `uorust::lift::equality`, certified as `Behavior::equality`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_equality.rs"]
pub mod equality;

/// `uorust::lift::equality_name`, certified as `Behavior::equalityName`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_equality_name.rs"]
pub mod equality_name;

/// `uorust::lift::evidence_axioms`, certified as `Behavior::evidenceAxioms`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_evidence_axioms.rs"]
pub mod evidence_axioms;

/// `uorust::lift::evidence_proof`, certified as `Behavior::evidenceProof`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_evidence_proof.rs"]
pub mod evidence_proof;

/// `uorust::lex::Expr::is_flat`, certified as `Behavior::exprIsFlat`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_expr_is_flat.rs"]
pub mod expr_is_flat;

/// `uorust::lex::Expr::pieces`, certified as `Behavior::exprPieces`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_expr_pieces.rs"]
pub mod expr_pieces;

/// `uorust::lift::field_name`, certified as `Behavior::fieldName`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_field_name.rs"]
pub mod field_name;

/// `uorust::lift::first_free`, certified as `Behavior::firstFree`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_first_free.rs"]
pub mod first_free;

/// `uorust::mir::lower::fixed`, certified as `Behavior::fixed`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_fixed.rs"]
pub mod fixed;

/// `uorust::lift::flat_lift`, certified as `Behavior::flatLift`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_flat_lift.rs"]
pub mod flat_lift;

/// `uorust::lift::flat_some`, certified as `Behavior::flatSome`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_flat_some.rs"]
pub mod flat_some;

/// `uorust::mir::lower::holds_no_pointer`, certified as `Behavior::holdsNoPointer`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_holds_no_pointer.rs"]
pub mod holds_no_pointer;

/// `uorust::mir::lower::hypotheses`, certified as `Behavior::hypotheses`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_hypotheses.rs"]
pub mod hypotheses;

/// `uorust::mir::lower::implication`, certified as `Behavior::implication`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_implication.rs"]
pub mod implication;

/// `uorust::lift::insert_all`, certified as `Behavior::insertAll`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_insert_all.rs"]
pub mod insert_all;

/// `uorust::lift::int_literal`, certified as `Behavior::intLiteral`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_int_literal.rs"]
pub mod int_literal;

/// `uorust::mir::lower::is_const_argument`, certified as `Behavior::isConstArgument`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_is_const_argument.rs"]
pub mod is_const_argument;

/// `uorust::lift::is_enum_or_bool`, certified as `Behavior::isEnumOrBool`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_is_enum_or_bool.rs"]
pub mod is_enum_or_bool;

/// `uorust::lift::is_lifetime`, certified as `Behavior::isLifetime`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_is_lifetime.rs"]
pub mod is_lifetime;

/// `uorust::mir::lower::is_needs_address`, certified as `Behavior::isNeedsAddress`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_is_needs_address.rs"]
pub mod is_needs_address;

/// `uorust::mir::lower::is_needs_memory`, certified as `Behavior::isNeedsMemory`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_is_needs_memory.rs"]
pub mod is_needs_memory;

/// `uorust::mir::lower::is_pointer`, certified as `Behavior::isPointer`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_is_pointer.rs"]
pub mod is_pointer;

/// `uorust::mir::lower::kind`, certified as `Behavior::kind`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_kind.rs"]
pub mod kind;

/// `uorust::lift::Lambda::wrap`, certified as `Behavior::lambdaWrap`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_lambda_wrap.rs"]
pub mod lambda_wrap;

/// `uorust::lift::Lambda::wrap_expr`, certified as `Behavior::lambdaWrapExpr`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_lambda_wrap_expr.rs"]
pub mod lambda_wrap_expr;

/// `uorust::lift::leaves`, certified as `Behavior::leaves`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_leaves.rs"]
pub mod leaves;

/// `uorust::lift::leaving_match`, certified as `Behavior::leavingMatch`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_leaving_match.rs"]
pub mod leaving_match;

/// `uorust::lift::leaving_value`, certified as `Behavior::leavingValue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_leaving_value.rs"]
pub mod leaving_value;

/// `uorust::lift::max_of`, certified as `Behavior::liftMaxOf`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_lift_max_of.rs"]
pub mod lift_max_of;

/// `uorust::lift::pow2`, certified as `Behavior::liftPow2`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_lift_pow2.rs"]
pub mod lift_pow2;

/// `uorust::lift::local_name`, certified as `Behavior::localName`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_local_name.rs"]
pub mod local_name;

/// `uorust::lex::members`, certified as `Behavior::members`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_members.rs"]
pub mod members;

/// `uorust::mir::memory::memory_fields`, certified as `Behavior::memoryFields`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_memory_fields.rs"]
pub mod memory_fields;

/// `uorust::mir::memory::memory_ty`, certified as `Behavior::memoryTy`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_memory_ty.rs"]
pub mod memory_ty;

/// `uorust::lift::mentions_word`, certified as `Behavior::mentionsWord`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mentions_word.rs"]
pub mod mentions_word;

/// `uorust::lift::method_char_any`, certified as `Behavior::methodCharAny`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_char_any.rs"]
pub mod method_char_any;

/// `uorust::lift::method_int_checked_neg`, certified as `Behavior::methodIntCheckedNeg`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_int_checked_neg.rs"]
pub mod method_int_checked_neg;

/// `uorust::lift::method_list_is_empty`, certified as `Behavior::methodListIsEmpty`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_list_is_empty.rs"]
pub mod method_list_is_empty;

/// `uorust::lift::method_list_len`, certified as `Behavior::methodListLen`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_list_len.rs"]
pub mod method_list_len;

/// `uorust::lift::method_option_is_some`, certified as `Behavior::methodOptionIsSome`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_option_is_some.rs"]
pub mod method_option_is_some;

/// `uorust::lift::method_outcome_is_ok`, certified as `Behavior::methodOutcomeIsOk`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_outcome_is_ok.rs"]
pub mod method_outcome_is_ok;

/// `uorust::lift::method_outcome_ok`, certified as `Behavior::methodOutcomeOk`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_outcome_ok.rs"]
pub mod method_outcome_ok;

/// `uorust::lift::method_str_is_empty`, certified as `Behavior::methodStrIsEmpty`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_str_is_empty.rs"]
pub mod method_str_is_empty;

/// `uorust::lift::method_str_to_string`, certified as `Behavior::methodStrToString`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_method_str_to_string.rs"]
pub mod method_str_to_string;

/// `uorust::mir::spec::MirAggregateKind::path`, certified as `Behavior::mirAggregateKindPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_aggregate_kind_path.rs"]
pub mod mir_aggregate_kind_path;

/// `uorust::mir::spec::MirAssertKind::path`, certified as `Behavior::mirAssertKindPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_assert_kind_path.rs"]
pub mod mir_assert_kind_path;

/// `uorust::mir::spec::MirBinOp::path`, certified as `Behavior::mirBinOpPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_bin_op_path.rs"]
pub mod mir_bin_op_path;

/// `uorust::mir::spec::MirBorrowKind::path`, certified as `Behavior::mirBorrowKindPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_borrow_kind_path.rs"]
pub mod mir_borrow_kind_path;

/// `uorust::mir::spec::MirCastKind::path`, certified as `Behavior::mirCastKindPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_cast_kind_path.rs"]
pub mod mir_cast_kind_path;

/// `uorust::mir::coverage::cast`, certified as `Behavior::mirCoverageCast`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_coverage_cast.rs"]
pub mod mir_coverage_cast;

/// `uorust::mir::coverage::intrinsic`, certified as `Behavior::mirCoverageIntrinsic`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_coverage_intrinsic.rs"]
pub mod mir_coverage_intrinsic;

/// `uorust::mir::coverage::operand`, certified as `Behavior::mirCoverageOperand`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_coverage_operand.rs"]
pub mod mir_coverage_operand;

/// `uorust::mir::coverage::rvalue`, certified as `Behavior::mirCoverageRvalue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_coverage_rvalue.rs"]
pub mod mir_coverage_rvalue;

/// `uorust::mir::coverage::statement`, certified as `Behavior::mirCoverageStatement`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_coverage_statement.rs"]
pub mod mir_coverage_statement;

/// `uorust::mir::coverage::terminator`, certified as `Behavior::mirCoverageTerminator`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_coverage_terminator.rs"]
pub mod mir_coverage_terminator;

/// `uorust::mir::memory::axioms`, certified as `Behavior::mirMemoryAxioms`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_memory_axioms.rs"]
pub mod mir_memory_axioms;

/// `uorust::mir::spec::MirNonDivergingIntrinsic::path`, certified as `Behavior::mirNonDivergingIntrinsicPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_non_diverging_intrinsic_path.rs"]
pub mod mir_non_diverging_intrinsic_path;

/// `uorust::mir::spec::MirOperand::path`, certified as `Behavior::mirOperandPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_operand_path.rs"]
pub mod mir_operand_path;

/// `uorust::mir::spec::MirProjectionElem::path`, certified as `Behavior::mirProjectionElemPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_projection_elem_path.rs"]
pub mod mir_projection_elem_path;

/// `uorust::mir::spec::MirRuntimeChecks::path`, certified as `Behavior::mirRuntimeChecksPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_runtime_checks_path.rs"]
pub mod mir_runtime_checks_path;

/// `uorust::mir::spec::MirRvalue::path`, certified as `Behavior::mirRvaluePath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_rvalue_path.rs"]
pub mod mir_rvalue_path;

/// `uorust::mir::spec::MirStatementKind::path`, certified as `Behavior::mirStatementKindPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_statement_kind_path.rs"]
pub mod mir_statement_kind_path;

/// `uorust::mir::spec::MirTerminatorKind::path`, certified as `Behavior::mirTerminatorKindPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_terminator_kind_path.rs"]
pub mod mir_terminator_kind_path;

/// `uorust::mir::spec::MirUnOp::path`, certified as `Behavior::mirUnOpPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mir_un_op_path.rs"]
pub mod mir_un_op_path;

/// `uorust::mir::lower::mut_target`, certified as `Behavior::mutTarget`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_mut_target.rs"]
pub mod mut_target;

/// `uorust::lift::name_taken`, certified as `Behavior::nameTaken`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_name_taken.rs"]
pub mod name_taken;

/// `uorust::lift::named_field`, certified as `Behavior::namedField`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_named_field.rs"]
pub mod named_field;

/// `uorust::lift::named_in`, certified as `Behavior::namedIn`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_named_in.rs"]
pub mod named_in;

/// `uorust::mir::lower::needs_address`, certified as `Behavior::needsAddress`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_needs_address.rs"]
pub mod needs_address;

/// `uorust::mir::lower::needs_memory`, certified as `Behavior::needsMemory`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_needs_memory.rs"]
pub mod needs_memory;

/// `uorust::lift::none_of`, certified as `Behavior::noneOf`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_none_of.rs"]
pub mod none_of;

/// `uorust::mir::memory::of`, certified as `Behavior::of`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_of.rs"]
pub mod of;

/// `uorust::lift::opt_of`, certified as `Behavior::optOf`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_opt_of.rs"]
pub mod opt_of;

/// `uorust::lex::params`, certified as `Behavior::params`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_params.rs"]
pub mod params;

/// `uorust::lex::pascal`, certified as `Behavior::pascal`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_pascal.rs"]
pub mod pascal;

/// `uorust::mir::lower::plain_cell`, certified as `Behavior::plainCell`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_plain_cell.rs"]
pub mod plain_cell;

/// `uorust::mir::lower::pointer_fields`, certified as `Behavior::pointerFields`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_pointer_fields.rs"]
pub mod pointer_fields;

/// `uorust::mir::lower::pointer_value`, certified as `Behavior::pointerValue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_pointer_value.rs"]
pub mod pointer_value;

/// `uorust::lift::pointer_width`, certified as `Behavior::pointerWidth`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_pointer_width.rs"]
pub mod pointer_width;

/// `uorust::lift::pointer_widths`, certified as `Behavior::pointerWidths`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_pointer_widths.rs"]
pub mod pointer_widths;

/// `uorust::mir::coverage::projection`, certified as `Behavior::projection`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_projection.rs"]
pub mod projection;

/// `uorust::mir::layout::reason`, certified as `Behavior::reason`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_reason.rs"]
pub mod reason;

/// `uorust::mir::lower::referent`, certified as `Behavior::referent`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_referent.rs"]
pub mod referent;

/// `uorust::lift::refused`, certified as `Behavior::refused`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_refused.rs"]
pub mod refused;

/// `uorust::lift::Residue::code`, certified as `Behavior::residueCode`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_residue_code.rs"]
pub mod residue_code;

/// `uorust::lift::Residue::detail`, certified as `Behavior::residueDetail`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_residue_detail.rs"]
pub mod residue_detail;

/// `uorust::lift::returned`, certified as `Behavior::returned`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_returned.rs"]
pub mod returned;

/// `uorust::mir::coverage::runtime_checks`, certified as `Behavior::runtimeChecks`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_runtime_checks.rs"]
pub mod runtime_checks;

/// `uorust::lift::same_constructor`, certified as `Behavior::sameConstructor`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_same_constructor.rs"]
pub mod same_constructor;

/// `uorust::mir::lower::scalar_value`, certified as `Behavior::scalarValue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_scalar_value.rs"]
pub mod scalar_value;

/// `uorust::lift::settled`, certified as `Behavior::settled`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_settled.rs"]
pub mod settled;

/// `uorust::mir::lower::single_field`, certified as `Behavior::singleField`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_single_field.rs"]
pub mod single_field;

/// `uorust::lift::size_suffix`, certified as `Behavior::sizeSuffix`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_size_suffix.rs"]
pub mod size_suffix;

/// `uorust::lift::some_of`, certified as `Behavior::someOf`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_some_of.rs"]
pub mod some_of;

/// `uorust::lift::some`, certified as `Behavior::someValue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_some_value.rs"]
pub mod some_value;

/// `uorust::lift::state_expr`, certified as `Behavior::stateExpr`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_state_expr.rs"]
pub mod state_expr;

/// `uorust::lift::std_path`, certified as `Behavior::stdPath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_std_path.rs"]
pub mod std_path;

/// `uorust::lift::struct_cells`, certified as `Behavior::structCells`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_struct_cells.rs"]
pub mod struct_cells;

/// `uorust::lift::struct_value`, certified as `Behavior::structValue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_struct_value.rs"]
pub mod struct_value;

/// `uorust::lift::then_rest`, certified as `Behavior::thenRest`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_then_rest.rs"]
pub mod then_rest;

/// `uorust::mir::lower::through_pointer`, certified as `Behavior::throughPointer`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_through_pointer.rs"]
pub mod through_pointer;

/// `uorust::mir::Trial::admits`, certified as `Behavior::trialAdmits`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_trial_admits.rs"]
pub mod trial_admits;

/// `uorust::lift::tuple_item`, certified as `Behavior::tupleItem`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_tuple_item.rs"]
pub mod tuple_item;

/// `uorust::lift::tuple_struct`, certified as `Behavior::tupleStruct`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_tuple_struct.rs"]
pub mod tuple_struct;

/// `uorust::lift::tuple_value`, certified as `Behavior::tupleValue`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_tuple_value.rs"]
pub mod tuple_value;

/// `uorust::lift::Types::accessor`, certified as `Behavior::typesAccessor`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_types_accessor.rs"]
pub mod types_accessor;

/// `uorust::lift::Types::boxed`, certified as `Behavior::typesBoxed`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_types_boxed.rs"]
pub mod types_boxed;

/// `uorust::lift::Types::declares`, certified as `Behavior::typesDeclares`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_types_declares.rs"]
pub mod types_declares;

/// `uorust::lift::Types::family_of`, certified as `Behavior::typesFamilyOf`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_types_family_of.rs"]
pub mod types_family_of;

/// `uorust::lift::types_has_data`, certified as `Behavior::typesHasData`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_types_has_data.rs"]
pub mod types_has_data;

/// `uorust::lift::Types::resolve`, certified as `Behavior::typesResolve`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_types_resolve.rs"]
pub mod types_resolve;

/// `uorust::lift::Types::size_name`, certified as `Behavior::typesSizeName`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_types_size_name.rs"]
pub mod types_size_name;

/// `uorust::lift::Types::unnamed`, certified as `Behavior::typesUnnamed`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_types_unnamed.rs"]
pub mod types_unnamed;

/// `uorust::lift::unicode_property`, certified as `Behavior::unicodeProperty`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_unicode_property.rs"]
pub mod unicode_property;

/// `uorust::mir::lower::uninhabited`, certified as `Behavior::uninhabited`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uninhabited.rs"]
pub mod uninhabited;

/// `uorust::mir::coverage::unop`, certified as `Behavior::unop`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_unop.rs"]
pub mod unop;

/// `uorust::lift::unread_shape`, certified as `Behavior::unreadShape`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_unread_shape.rs"]
pub mod unread_shape;

/// `uorust::lift::untyped`, certified as `Behavior::untyped`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_untyped.rs"]
pub mod untyped;

/// `uorust::mir::int::helper`, certified as `Behavior::uorustHelper`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_helper.rs"]
pub mod uorust_helper;

/// `uorust::lex::Module::path`, certified as `Behavior::uorustLexModulePath`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_lex_module_path.rs"]
pub mod uorust_lex_module_path;

/// `uorust::lift::Ty::entry`, certified as `Behavior::uorustLiftTyEntry`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_lift_ty_entry.rs"]
pub mod uorust_lift_ty_entry;

/// `uorust::lift::Ty::render`, certified as `Behavior::uorustLiftTyRender`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_lift_ty_render.rs"]
pub mod uorust_lift_ty_render;

/// `uorust::lift::Ty::same`, certified as `Behavior::uorustLiftTySame`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_lift_ty_same.rs"]
pub mod uorust_lift_ty_same;

/// `uorust::lift::Ty::tag`, certified as `Behavior::uorustLiftTyTag`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_lift_ty_tag.rs"]
pub mod uorust_lift_ty_tag;

/// `uorust::mir::core::Scope::core`, certified as `Behavior::uorustMirCoreScopeCore`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_mir_core_scope_core.rs"]
pub mod uorust_mir_core_scope_core;

/// `uorust::mir::core::Scope::library`, certified as `Behavior::uorustMirCoreScopeLibrary`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_mir_core_scope_library.rs"]
pub mod uorust_mir_core_scope_library;

/// `uorust::mir::core::Scope::registry`, certified as `Behavior::uorustMirCoreScopeRegistry`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_mir_core_scope_registry.rs"]
pub mod uorust_mir_core_scope_registry;

/// `uorust::mir::layout::Refusal::text`, certified as `Behavior::uorustMirLayoutRefusalText`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_uorust_mir_layout_refusal_text.rs"]
pub mod uorust_mir_layout_refusal_text;

/// `uorust::lift::variant_field`, certified as `Behavior::variantField`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_variant_field.rs"]
pub mod variant_field;

/// `uorust::lift::variant_owner`, certified as `Behavior::variantOwner`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_variant_owner.rs"]
pub mod variant_owner;

/// `uorust::lift::word_char`, certified as `Behavior::wordChar`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_word_char.rs"]
pub mod word_char;

/// `uorust::lift::wrap_lets`, certified as `Behavior::wrapLets`.
#[rustfmt::skip]
#[allow(missing_docs, dead_code, clippy::all)]
#[path = "../roots/uorust_certified_wrap_lets.rs"]
pub mod wrap_lets;
