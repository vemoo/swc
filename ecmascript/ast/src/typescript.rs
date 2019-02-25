#![allow(missing_copy_implementations)]
use crate::{
    class::Decorator,
    expr::Expr,
    ident::Ident,
    lit::{Bool, Number, Str},
    module::ModuleItem,
    pat::{AssignPat, ObjectPat, RestPat},
};
use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::fmt;
#[cfg(feature = "fold")]
use swc_common::Fold;
use swc_common::{ast_node, Span};

#[ast_node("TsTypeAnnotation")]
pub struct TsTypeAnn {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: Box<TsType>,
}

#[ast_node("TsTypeParameterDeclaration")]
pub struct TsTypeParamDecl {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "parameters")]
    pub params: Vec<TsTypeParam>,
}

#[ast_node("TsTypeParameter")]
pub struct TsTypeParam {
    #[serde(default)]
    pub span: Span,
    pub name: Ident,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraint: Option<Box<TsType>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Box<TsType>>,
}

#[ast_node("TsTypeParameterInstantiation")]
pub struct TsTypeParamInstantiation {
    #[serde(default)]
    pub span: Span,
    pub params: Vec<Box<TsType>>,
}

#[ast_node("TsTypeCastExpression")]
pub struct TsTypeCastExpr {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "expression")]
    pub expr: Box<Expr>,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: TsTypeAnn,
}

#[ast_node("TsParameterProperty")]
pub struct TsParamProp {
    #[serde(default)]
    pub span: Span,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    pub decorators: Vec<Decorator>,
    /// At least one of `accessibility` or `readonly` must be set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessibility: Option<Accessibility>,
    pub readonly: bool,
    pub param: TsParamPropParam,
}

#[ast_node]
pub enum TsParamPropParam {
    Ident(Ident),
    Assign(AssignPat),
}

#[ast_node("TsQualifiedName")]
pub struct TsQualifiedName {
    #[span(lo)]
    pub left: TsEntityName,
    #[span(hi)]
    pub right: Ident,
}

#[ast_node]
#[allow(variant_size_differences)]
pub enum TsEntityName {
    TsQualifiedName(Box<TsQualifiedName>),
    Ident(Ident),
}

#[ast_node]
pub enum TsSignatureDecl {
    TsCallSignatureDecl(TsCallSignatureDecl),
    TsConstructSignatureDecl(TsConstructSignatureDecl),
    TsMethodSignature(TsMethodSignature),
    TsFnType(TsFnType),
    TsConstructorType(TsConstructorType),
}

// ================
// TypeScript type members (for type literal / interface / class)
// ================

#[ast_node]
pub enum TsTypeElement {
    TsCallSignatureDecl(TsCallSignatureDecl),
    TsConstructSignatureDecl(TsConstructSignatureDecl),
    TsPropertySignature(TsPropertySignature),
    TsMethodSignature(TsMethodSignature),
    TsIndexSignature(TsIndexSignature),
}

#[ast_node("TsCallSignatureDeclaration")]
pub struct TsCallSignatureDecl {
    #[serde(default)]
    pub span: Span,
    pub params: Vec<TsFnParam>,
    #[serde(
        default,
        rename = "typeAnnotation",
        skip_serializing_if = "Option::is_none"
    )]
    pub type_ann: Option<TsTypeAnn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamDecl>,
}

#[ast_node("TsConstructSignatureDeclaration")]
pub struct TsConstructSignatureDecl {
    #[serde(default)]
    pub span: Span,
    pub params: Vec<TsFnParam>,
    #[serde(
        default,
        rename = "typeAnnotation",
        skip_serializing_if = "Option::is_none"
    )]
    pub type_ann: Option<TsTypeAnn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamDecl>,
}

#[ast_node("TsPropertySignature")]
pub struct TsPropertySignature {
    #[serde(default)]
    pub span: Span,
    pub readonly: bool,
    pub key: Box<Expr>,
    pub computed: bool,
    pub optional: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init: Option<Box<Expr>>,
    pub params: Vec<TsFnParam>,
    #[serde(
        default,
        rename = "typeAnnotation",
        skip_serializing_if = "Option::is_none"
    )]
    pub type_ann: Option<TsTypeAnn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamDecl>,
}

#[ast_node("TsMethodSignature")]
pub struct TsMethodSignature {
    #[serde(default)]
    pub span: Span,
    pub readonly: bool,
    pub key: Box<Expr>,
    pub computed: bool,
    pub optional: bool,
    pub params: Vec<TsFnParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_ann: Option<TsTypeAnn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamDecl>,
}

#[ast_node("TsIndexSignature")]
pub struct TsIndexSignature {
    pub params: Vec<TsFnParam>,
    #[serde(
        default,
        rename = "typeAnnotation",
        skip_serializing_if = "Option::is_none"
    )]
    pub type_ann: Option<TsTypeAnn>,

    pub readonly: bool,
    #[serde(default)]
    pub span: Span,
}

// ================
// TypeScript types
// ================

#[ast_node]
pub enum TsType {
    TsKeywordType(TsKeywordType),
    TsThisType(TsThisType),
    TsFnOrConstructorType(TsFnOrConstructorType),
    TsTypeRef(TsTypeRef),
    TsTypeQuery(TsTypeQuery),
    TsTypeLit(TsTypeLit),
    TsArrayType(TsArrayType),
    TsTupleType(TsTupleType),
    TsOptionalType(TsOptionalType),
    TsRestType(TsRestType),
    TsUnionOrIntersectionType(TsUnionOrIntersectionType),
    TsConditionalType(TsConditionalType),
    TsInferType(TsInferType),
    TsParenthesizedType(TsParenthesizedType),
    TsTypeOperator(TsTypeOperator),
    TsIndexedAccessType(TsIndexedAccessType),
    TsMappedType(TsMappedType),
    TsLitType(TsLitType),
    TsTypePredicate(TsTypePredicate),
}

#[ast_node]
pub enum TsFnOrConstructorType {
    TsFnType(TsFnType),
    TsConstructorType(TsConstructorType),
}

impl From<TsFnType> for TsType {
    fn from(t: TsFnType) -> Self {
        TsFnOrConstructorType::TsFnType(t).into()
    }
}

impl From<TsConstructorType> for TsType {
    fn from(t: TsConstructorType) -> Self {
        TsFnOrConstructorType::TsConstructorType(t).into()
    }
}

impl From<TsUnionType> for TsType {
    fn from(t: TsUnionType) -> Self {
        TsUnionOrIntersectionType::TsUnionType(t).into()
    }
}

impl From<TsIntersectionType> for TsType {
    fn from(t: TsIntersectionType) -> Self {
        TsUnionOrIntersectionType::TsIntersectionType(t).into()
    }
}

#[ast_node("TsKeywordType")]
pub struct TsKeywordType {
    #[serde(default)]
    pub span: Span,
    pub kind: TsKeywordTypeKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "fold", derive(Fold))]
pub enum TsKeywordTypeKind {
    #[serde(rename = "any")]
    TsAnyKeyword,

    #[serde(rename = "unknown")]
    TsUnknownKeyword,

    #[serde(rename = "number")]
    TsNumberKeyword,

    #[serde(rename = "object")]
    TsObjectKeyword,

    #[serde(rename = "boolean")]
    TsBooleanKeyword,

    #[serde(rename = "bigint")]
    TsBigIntKeyword,

    #[serde(rename = "string")]
    TsStringKeyword,

    #[serde(rename = "symbol")]
    TsSymbolKeyword,

    #[serde(rename = "void")]
    TsVoidKeyword,

    #[serde(rename = "undefined")]
    TsUndefinedKeyword,

    #[serde(rename = "null")]
    TsNullKeyword,

    #[serde(rename = "never")]
    TsNeverKeyword,
}

#[ast_node("TsThisType")]
pub struct TsThisType {
    #[serde(default)]
    pub span: Span,
}

#[ast_node]
pub enum TsFnParam {
    Ident(Ident),
    Rest(RestPat),
    Object(ObjectPat),
}

#[ast_node("TsFunctionType")]
pub struct TsFnType {
    #[serde(default)]
    pub span: Span,
    pub params: Vec<TsFnParam>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamDecl>,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: TsTypeAnn,
}

#[ast_node("TsConstructorType")]
pub struct TsConstructorType {
    #[serde(default)]
    pub span: Span,
    pub params: Vec<TsFnParam>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamDecl>,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: TsTypeAnn,
}

#[ast_node("TsTypeReference")]
pub struct TsTypeRef {
    #[serde(default)]
    pub span: Span,
    pub type_name: TsEntityName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamInstantiation>,
}

#[ast_node("TsTypePredicate")]
pub struct TsTypePredicate {
    #[serde(default)]
    pub span: Span,
    pub param_name: TsThisTypeOrIdent,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: TsTypeAnn,
}

#[ast_node]
#[allow(variant_size_differences)]
pub enum TsThisTypeOrIdent {
    TsThisType(TsThisType),
    Ident(Ident),
}

/// `typeof` operator
#[ast_node("TsTypeQuery")]
pub struct TsTypeQuery {
    #[serde(default)]
    pub span: Span,
    pub expr_name: TsEntityName,
}

#[ast_node("TsTypeLiteral")]
pub struct TsTypeLit {
    #[serde(default)]
    pub span: Span,
    pub members: Vec<TsTypeElement>,
}

#[ast_node("TsArrayType")]
pub struct TsArrayType {
    #[serde(default)]
    pub span: Span,
    pub elem_type: Box<TsType>,
}

#[ast_node("TsTupleType")]
pub struct TsTupleType {
    #[serde(default)]
    pub span: Span,
    pub elem_types: Vec<Box<TsType>>,
}

#[ast_node("TsOptionalType")]
pub struct TsOptionalType {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: Box<TsType>,
}

#[ast_node("TsRestType")]
pub struct TsRestType {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: Box<TsType>,
}

#[ast_node]
pub enum TsUnionOrIntersectionType {
    TsUnionType(TsUnionType),
    TsIntersectionType(TsIntersectionType),
}

#[ast_node("TsUnionType")]
pub struct TsUnionType {
    #[serde(default)]
    pub span: Span,
    pub types: Vec<Box<TsType>>,
}

#[ast_node("TsIntersectionType")]
pub struct TsIntersectionType {
    #[serde(default)]
    pub span: Span,
    pub types: Vec<Box<TsType>>,
}

#[ast_node("TsConditionalType")]
pub struct TsConditionalType {
    #[serde(default)]
    pub span: Span,
    pub check_type: Box<TsType>,
    pub extends_type: Box<TsType>,
    pub true_type: Box<TsType>,
    pub false_type: Box<TsType>,
}

#[ast_node("TsInferType")]
pub struct TsInferType {
    #[serde(default)]
    pub span: Span,
    pub type_param: TsTypeParam,
}

#[ast_node("TsParenthesizedType")]
pub struct TsParenthesizedType {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: Box<TsType>,
}

#[ast_node("TsTypeOperator")]
pub struct TsTypeOperator {
    #[serde(default)]
    pub span: Span,
    pub op: TsTypeOperatorOp,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: Box<TsType>,
}

#[derive(StringEnum, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "fold", derive(Fold))]
pub enum TsTypeOperatorOp {
    /// `keyof`
    KeyOf,
    /// `unique`
    Unique,
}

#[ast_node("TsIndexedAccessType")]
pub struct TsIndexedAccessType {
    #[serde(default)]
    pub span: Span,
    $[serde(rename="objectType")]
    pub obj_type: Box<TsType>,
    pub index_type: Box<TsType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "fold", derive(Fold))]
pub enum TruePlusMinus {
    True,
    Plus,
    Minus,
}

impl Serialize for TruePlusMinus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        match *self {
            TruePlusMinus::True => serializer.serialize_bool(true),
            TruePlusMinus::Plus => serializer.serialize_str("+"),
            TruePlusMinus::Minus => serializer.serialize_str("-"),
        }
    }
}

impl<'de> Deserialize<'de> for TruePlusMinus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TruePlusMinusVisitor;

        impl<'de> Visitor<'de> for TruePlusMinusVisitor {
            type Value = TruePlusMinus;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("one of '+', '-', true")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "+" => Ok(TruePlusMinus::Plus),
                    "-" => Ok(TruePlusMinus::Minus),
                    "true" => Ok(TruePlusMinus::True),
                    _ => Err(de::Error::invalid_value(Unexpected::Str(value), &self)),
                }
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value {
                    Ok(TruePlusMinus::True)
                } else {
                    Err(de::Error::invalid_value(Unexpected::Bool(value), &self))
                }
            }
        }

        deserializer.deserialize_any(TruePlusMinusVisitor)
    }
}

#[ast_node("TsMappedType")]
pub struct TsMappedType {
    #[serde(default)]
    pub span: Span,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readonly: Option<TruePlusMinus>,
    pub type_param: TsTypeParam,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<TruePlusMinus>,
    #[serde(default,rename="typeAnnotation", skip_serializing_if = "Option::is_none")]
    pub type_ann: Option<Box<TsType>>,
}

#[ast_node("TsLiteralType")]
pub struct TsLitType {
    #[serde(default)]
    pub span: Span,
    #[serde(rename="literal")]
    pub lit: TsLit,
}

#[ast_node]
pub enum TsLit {
    Number(Number),
    Str(Str),
    Bool(Bool),
}

// // ================
// // TypeScript declarations
// // ================

#[ast_node("TsInterfaceDeclaration")]
pub struct TsInterfaceDecl {
    #[serde(default)]
    pub span: Span,
    pub id: Ident,
    pub declare: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamDecl>,
    pub extends: Vec<TsExprWithTypeArgs>,
    pub body: TsInterfaceBody,
}

#[ast_node("TsInterfaceBody")]
pub struct TsInterfaceBody {
    #[serde(default)]
    pub span: Span,
    pub body: Vec<TsTypeElement>,
}

#[ast_node("TsExpressionWithTypeArguments")]
pub struct TsExprWithTypeArgs {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "expression")]
    pub expr: TsEntityName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamInstantiation>,
}

#[ast_node("TsTypeAliasDeclaration")]
pub struct TsTypeAliasDecl {
    #[serde(default)]
    pub span: Span,
    pub declare: bool,
    pub id: Ident,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_params: Option<TsTypeParamDecl>,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: Box<TsType>,
}

#[ast_node("TsEnumDeclaration")]
pub struct TsEnumDecl {
    #[serde(default)]
    pub span: Span,
    pub declare: bool,
    pub is_const: bool,
    pub id: Ident,
    pub members: Vec<TsEnumMember>,
}

#[ast_node("TsEnumMember")]
pub struct TsEnumMember {
    #[serde(default)]
    pub span: Span,
    pub id: TsEnumMemberId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init: Option<Box<Expr>>,
}

#[ast_node]
pub enum TsEnumMemberId {
    Ident(Ident),
    Str(Str),
}

#[ast_node("TsModuleDeclaration")]
pub struct TsModuleDecl {
    #[serde(default)]
    pub span: Span,
    pub declare: bool,
    /// In TypeScript, this is only available through`node.flags`.
    pub global: bool,
    pub id: TsModuleName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<TsNamespaceBody>,
}

/// `namespace A.B { }` is a namespace named `A` with another TsNamespaceDecl as
/// its body.
#[ast_node]
pub enum TsNamespaceBody {
    TsModuleBlock(TsModuleBlock),
    TsNamespaceDecl(TsNamespaceDecl),
}

#[ast_node("TsModuleBlock")]
pub struct TsModuleBlock {
    #[serde(default)]
    pub span: Span,
    pub body: Vec<ModuleItem>,
}

#[ast_node("TsNamespaceDeclaration")]
pub struct TsNamespaceDecl {
    #[serde(default)]
    pub span: Span,
    pub declare: bool,
    /// In TypeScript, this is only available through`node.flags`.
    pub global: bool,
    pub id: Ident,
    pub body: Box<TsNamespaceBody>,
}

#[ast_node]
pub enum TsModuleName {
    Ident(Ident),
    Str(Str),
}

#[ast_node("TsImportEqualsDeclaration")]
pub struct TsImportEqualsDecl {
    #[serde(default)]
    pub span: Span,
    pub declare: bool,
    pub is_export: bool,
    pub id: Ident,
    pub module_ref: TsModuleRef,
}

#[ast_node]
pub enum TsModuleRef {
    TsEntityName(TsEntityName),
    TsExternalModuleRef(TsExternalModuleRef),
}

#[ast_node("TsExternalModuleReference")]
pub struct TsExternalModuleRef {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "expression")]
    pub expr: Str,
}

/// TypeScript's own parser uses ExportAssignment for both `export default` and
/// `export =`. But for @babel/parser, `export default` is an ExportDefaultDecl,
/// so a TsExportAssignment is always `export =`.
#[ast_node("TsExportAssignment")]
pub struct TsExportAssignment {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "expression")]
    pub expr: Box<Expr>,
}

#[ast_node("TsNamespaceExportDeclaration")]
pub struct TsNamespaceExportDecl {
    #[serde(default)]
    pub span: Span,
    pub id: Ident,
}

// // ================
// // TypeScript exprs
// // ================

#[ast_node("TsAsExpression")]
pub struct TsAsExpr {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "expression")]
    pub expr: Box<Expr>,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: Box<TsType>,
}

#[ast_node("TsTypeAssertion")]
pub struct TsTypeAssertion {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "expression")]
    pub expr: Box<Expr>,
    #[serde(rename = "typeAnnotation")]
    pub type_ann: Box<TsType>,
}

#[ast_node("TsNonNullExpression")]
pub struct TsNonNullExpr {
    #[serde(default)]
    pub span: Span,
    #[serde(rename = "expression")]
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "fold", derive(Fold))]
pub enum Accessibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "protected")]
    Protected,
    #[serde(rename = "private")]
    Private,
}
