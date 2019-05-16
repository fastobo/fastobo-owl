use std::collections::BTreeSet;
use std::iter::FromIterator;

use fastobo::ast as obo;
use horned_owl::model as owl;

use crate::constants::datatype;
use crate::constants::property;
use super::Context;
use super::IntoOwlCtx;
use super::OwlEntity;


impl IntoOwlCtx for obo::TermFrame {
    type Owl = BTreeSet<owl::AnnotatedAxiom>;
    fn into_owl(mut self, ctx: &mut Context) -> Self::Owl {

        // The ID of this frame translated to an IRI.
        let id = obo::Ident::from(self.id().clone().into_inner()).into_owl(ctx);

        // The translated axioms.
        let mut axioms: Self::Owl = BTreeSet::new();

        // Build the annotated class declaration.
        axioms.insert(owl::AnnotatedAxiom {
            annotation: BTreeSet::new(),
            axiom: owl::Axiom::from(owl::DeclareClass(owl::Class(id.clone()))),
        });

        // Add the original OBO ID as an annotation.
        // FIXME: maybe only do that if the term is not declared as being
        //        anonymous ?
        axioms.insert(
            owl::AnnotatedAxiom::from(
                owl::AnnotationAssertion {
                    annotation_subject: id.clone(),
                    annotation: owl::Annotation {
                        annotation_property: ctx.build.annotation_property(
                            property::obo_in_owl::ID
                        ),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                            literal: Some(self.id_mut().as_ref().to_string()),
                            lang: None,
                        })
                    }
                }
            )
        );

        // Group assertion clauses together:
        // - IntersectionOf(Option<RelationIdent>, ClassIdent),
        // - UnionOf(ClassIdent),

        // Convert remaining clauses to axioms.
        for line in self.into_iter() {
            match line.into_inner().into_owl(ctx) {
                OwlEntity::Annotation(annot) => axioms.insert(
                    owl::AnnotatedAxiom::from(owl::AnnotationAssertion::new(id.clone(), annot))
                ),
                OwlEntity::Axiom(axiom) => axioms.insert(owl::AnnotatedAxiom::from(axiom)),
                OwlEntity::None => true,
            };
        }

        // Return the axioms
        axioms
    }
}

impl IntoOwlCtx for obo::TermClause {
    type Owl = OwlEntity;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {

            // IsAnonymous(bool),

            obo::TermClause::Name(name) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::rdfs::LABEL,
                    ),
                    annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                        datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                        literal: Some(name.into_string()),
                        lang: None,
                    })
                }
            ),
            obo::TermClause::Namespace(ns) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::obo_in_owl::HAS_OBO_NAMESPACE,
                    ),
                    annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                        datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                        literal: Some(ns.to_string()),
                        lang: None,
                    })
                }
            ),
            obo::TermClause::AltId(id) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::obo_in_owl::HAS_ALTERNATIVE_ID
                    ),
                    annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                        datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                        literal: Some(id.to_string()),
                        lang: None,
                    })
                }
            ),

            obo::TermClause::Def(desc, xrefs) => OwlEntity::from(
                owl::AnnotatedAxiom::new(
                    owl::AnnotationAssertion::new(
                        ctx.current_frame.clone(),
                        owl::Annotation {
                            annotation_property: ctx.build.annotation_property(
                                property::iao::DEFINITION
                            ),
                            annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                                datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                                literal: Some(desc.into_string()),
                                lang: None,
                            })
                        },
                    ),
                    xrefs.into_owl(ctx),
                )
            ),

            obo::TermClause::Comment(comment) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::rdfs::COMMENT
                    ),
                    annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                        datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                        literal: Some(comment.into_string()),
                        lang: None,
                    })
                }
            ),
            obo::TermClause::Subset(subset) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::obo_in_owl::IN_SUBSET
                    ),
                    annotation_value: owl::AnnotationValue::IRI(
                        obo::Ident::from(subset).into_owl(ctx),
                    )
                }
            ),

            obo::TermClause::Xref(xref) => OwlEntity::from(
                owl::AnnotatedAxiom::new(
                    owl::Axiom::from(
                        owl::AnnotationAssertion{
                            annotation_subject: ctx.current_frame.clone(),
                            annotation: xref.clone().into_owl(ctx),
                        }
                    ),
                    BTreeSet::from_iter(
                        xref.description().map(|desc| owl::Annotation {
                            annotation_property: ctx.build.annotation_property(property::rdfs::LABEL),
                            annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                                datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                                literal: Some(desc.clone().into_string()),
                                lang: None,
                            })
                        })
                    ),
                )
            ),

            obo::TermClause::Builtin(_) => OwlEntity::None,

            obo::TermClause::PropertyValue(pv) => OwlEntity::from(
                owl::Axiom::from(
                    owl::AnnotationAssertion{
                        annotation_subject: ctx.current_frame.clone(),
                        annotation: pv.into_owl(ctx),
                    }
                )
            ),

            obo::TermClause::IsA(supercls) => OwlEntity::from(
                owl::Axiom::from(
                    owl::SubClassOf {
                        super_class: owl::ClassExpression::Class(
                            owl::Class(supercls.into_owl(ctx))
                        ),
                        sub_class: owl::ClassExpression::Class(
                            owl::Class(ctx.current_frame.clone())
                        )
                    }
                )
            ),

            // IntersectionOf(Option<RelationIdent>, ClassIdent),
            // UnionOf(ClassIdent),

            // FIXME: should be all grouped into a single axiom ?
            obo::TermClause::EquivalentTo(cid) => OwlEntity::from(
                owl::Axiom::from(
                    owl::EquivalentClasses(
                        vec![
                            owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                            owl::ClassExpression::from(owl::Class(cid.into_owl(ctx))),
                        ],
                    )
                )
            ),

            obo::TermClause::DisjointFrom(cid) => OwlEntity::from(
                owl::Axiom::from(
                    owl::DisjointClasses(
                        vec![
                            owl::ClassExpression::from(owl::Class(ctx.current_frame.clone())),
                            owl::ClassExpression::from(owl::Class(cid.into_owl(ctx))),
                        ],
                    )
                )
            ),

            // Relationship(RelationIdent, ClassIdent),
            obo::TermClause::IsObsolete(b) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::owl::DEPRECATED
                    ),
                    annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                        datatype_iri: Some(ctx.build.iri(datatype::xsd::BOOLEAN)),
                        literal: Some(b.to_string()),
                        lang: None,
                    })
                }
            ),

            obo::TermClause::ReplacedBy(id) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::iao::REPLACED_BY
                    ),
                    annotation_value: owl::AnnotationValue::IRI(
                        id.into_owl(ctx),
                    )
                }
            ),

            obo::TermClause::Consider(id) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::obo_in_owl::CONSIDER
                    ),
                    annotation_value: owl::AnnotationValue::IRI(
                        obo::Ident::from(id).into_owl(ctx),
                    )
                }
            ),

            obo::TermClause::CreatedBy(c) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::dc::CREATOR
                    ),
                    annotation_value: owl::AnnotationValue::Literal(
                        owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                            literal: Some(c.into_string()),
                            lang: None,
                        }
                    )
                }
            ),

            obo::TermClause::CreationDate(dt) => OwlEntity::from(
                owl::Annotation {
                    annotation_property: ctx.build.annotation_property(
                        property::dc::DATE
                    ),
                    annotation_value: owl::AnnotationValue::Literal(
                        owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::DATETIME)),
                            lang: None,
                            literal: Some(obo::DateTime::to_xsd_datetime(&dt)),
                        }
                    )
                }
            ),

            _ => OwlEntity::None,
        }


    }
}
