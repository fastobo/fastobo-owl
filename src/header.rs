use std::collections::BTreeSet;
use std::iter::FromIterator;

use fastobo::ast as obo;
use horned_owl::model as owl;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::datatype;
use crate::constants::property;

impl IntoOwlCtx for obo::HeaderClause {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            // `oboInOwl:hasOBOFormatVersion` annotation
            obo::HeaderClause::FormatVersion(v) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(owl::Annotation {
                    annotation_property: ctx
                        .build
                        .annotation_property(property::obo_in_owl::HAS_OBO_FORMAT_VERSION),
                    annotation_value: v.into_owl(ctx).into(),
                }),
            )),

            // no equivalent
            // --> should be added as the Ontology IRI
            obo::HeaderClause::DataVersion(_) => None,

            // `oboInOwl:hasDate` annotation
            // --> QUESTION: should the datatype_iri be `dateTime` or `string` ?
            obo::HeaderClause::Date(dt) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(owl::Annotation {
                    annotation_property: ctx
                        .build
                        .annotation_property(property::obo_in_owl::HAS_DATE),
                    annotation_value: dt.into_owl(ctx).into(),
                }),
            )),

            // `oboInOwl:savedBy` annotation
            obo::HeaderClause::SavedBy(n) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(owl::Annotation {
                    annotation_property: ctx
                        .build
                        .annotation_property(property::obo_in_owl::SAVED_BY),
                    annotation_value: n.into_owl(ctx).into(),
                }),
            )),

            // `oboInOwl:autoGeneratedBy` annotation
            // --> FIXME: not actually declared in `oboInOwl`!
            obo::HeaderClause::AutoGeneratedBy(n) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(owl::Annotation {
                    annotation_property: ctx
                        .build
                        .annotation_property(property::obo_in_owl::AUTO_GENERATED_BY),
                    annotation_value: n.into_owl(ctx).into(),
                }),
            )),

            // `owl::imports`:
            // --> if in abbreviated form, use default http://purl.obolibrary.org/obo/ prefix
            // --> if URL, simply use that
            obo::HeaderClause::Import(import) => Some(owl::AnnotatedAxiom::from(owl::Axiom::from(
                horned_owl::model::Import(import.into_owl(ctx)),
            ))),

            // `owl:AnnotationProperty`
            //     <owl:AnnotationProperty rdf:about=T(subset)>
            //         <rdfs:comment rdf:datatype="xsd:string">T(description)</rdfs:comment>
            //         <rdfs:subPropertyOf rdf:resource="http://www.geneontology.org/formats/oboInOwl#SubsetProperty"/>
            //     </owl:AnnotationProperty>
            obo::HeaderClause::Subsetdef(subset, desc) => Some(owl::AnnotatedAxiom::new(
                owl::AnnotationAssertion {
                    annotation_subject: obo::Ident::from(subset).into_owl(ctx),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::rdfs::SUB_PROPERTY_OF),
                        annotation_value: owl::AnnotationValue::IRI(
                            ctx.build.iri(property::obo_in_owl::SUBSET_PROPERTY),
                        ),
                    },
                },
                BTreeSet::from_iter(Some(owl::Annotation {
                    annotation_property: ctx.build.annotation_property(property::rdfs::COMMENT),
                    annotation_value: desc.into_owl(ctx).into(),
                })),
            )),

            // `owl:AnnotationProperty`
            //      <owl:AnnotationProperty rdf:about="http://purl.obolibrary.org/obo/go#systematic_synonym">
            //          <oboInOwl:hasScope rdf:resource="http://www.geneontology.org/formats/oboInOwl#hasExactSynonym"/>
            //          <rdfs:label rdf:datatype="http://www.w3.org/2001/XMLSchema#string">Systematic synonym</rdfs:label>
            //          <rdfs:subPropertyOf rdf:resource="http://www.geneontology.org/formats/oboInOwl#SynonymTypeProperty"/>
            //      </owl:AnnotationProperty>
            // FIXME: Add description and scope
            obo::HeaderClause::SynonymTypedef(ty, desc, scope) => {
                Some(owl::AnnotatedAxiom::from(owl::AnnotationAssertion {
                    annotation_subject: obo::Ident::from(ty).into_owl(ctx),
                    annotation: owl::Annotation {
                        annotation_property: ctx
                            .build
                            .annotation_property(property::rdfs::SUB_PROPERTY_OF),
                        annotation_value: owl::AnnotationValue::IRI(
                            ctx.build.iri(property::obo_in_owl::SYNONYM_TYPE_PROPERTY),
                        ),
                    },
                }))
            }

            // `oboInOwl:hasDefaultNamespace` annotation
            obo::HeaderClause::DefaultNamespace(ns) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(owl::Annotation {
                    annotation_property: ctx
                        .build
                        .annotation_property(property::obo_in_owl::HAS_DEFAULT_NAMESPACE),
                    annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                        datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                        literal: Some(ns.to_string()),
                        lang: None,
                    }),
                }),
            )),

            // `oboInOwl:namespaceIdRule` annotation
            obo::HeaderClause::NamespaceIdRule(r) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(owl::Annotation {
                    annotation_property: ctx
                        .build
                        .annotation_property(property::obo_in_owl::NAMESPACE_ID_RULE),
                    annotation_value: r.into_owl(ctx).into(),
                }),
            )),

            // no actual OWL equivalent, but we expose the IDspace as an OWL
            // prefix to retain the same CURIES in the OWL ontology.
            // earlier when creating the conversion context.
            obo::HeaderClause::Idspace(_, _, _) => None,

            // no equivalent, macros should be resolved before conversion.
            obo::HeaderClause::TreatXrefsAsEquivalent(_) => None,
            obo::HeaderClause::TreatXrefsAsGenusDifferentia(_, _, _) => None,
            obo::HeaderClause::TreatXrefsAsReverseGenusDifferentia(_, _, _) => None,
            obo::HeaderClause::TreatXrefsAsRelationship(_, _) => None,
            obo::HeaderClause::TreatXrefsAsIsA(_) => None,
            obo::HeaderClause::TreatXrefsAsHasSubclass(_) => None,

            // `rdfs:comment` annotation
            obo::HeaderClause::Remark(v) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(owl::Annotation {
                    annotation_property: ctx.build.annotation_property(property::rdfs::COMMENT),
                    annotation_value: v.into_owl(ctx).into(),
                }),
            )),

            // translate as an annotation
            obo::HeaderClause::PropertyValue(pv) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(pv.into_owl(ctx)),
            )),

            // no actual OWL equivalent, but exposed as the Ontology IRI
            // when creating the conversion context.
            obo::HeaderClause::Ontology(_) => None,

            // handled in the header frame translation.
            obo::HeaderClause::OwlAxioms(_) => None,

            // no equivalent for undefined header tag/values
            obo::HeaderClause::Unreserved(_, _) => None,
        }
    }
}

impl IntoOwlCtx for obo::HeaderFrame {
    type Owl = Vec<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        let mut owl_axioms: Vec<String> = Vec::new();
        let mut axioms: Vec<owl::AnnotatedAxiom> = Vec::with_capacity(self.len());

        // Process the header frame clauses
        for clause in self.into_iter() {
            if let obo::HeaderClause::OwlAxioms(s) = clause {
                owl_axioms.push(s.into_string());
            } else if let Some(axiom) = clause.into_owl(ctx) {
                axioms.push(axiom);
            }
        }

        // FIXME: https://github.com/owlcollab/oboformat/issues/116
        // Parse the remaining axioms in `owl-axioms` clauses.
        if !owl_axioms.is_empty() {
            let (ont, _) = horned_functional::parse(&owl_axioms.join("\n"))
                .expect("invalid functional ontology");
            axioms.extend(ont);
        }

        axioms
    }
}

impl IntoOwlCtx for obo::Import {
    type Owl = owl::IRI;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        // `owl::imports`:
        // --> if in abbreviated form, use default http://purl.obolibrary.org/obo/ prefix
        // --> if URL, simply use that
        match self {
            obo::Import::Url(url) => ctx.build.iri(url.as_str()),
            obo::Import::Abbreviated(id) => ctx
                .build
                .iri(format!("http://purl.obolibrary.org/obo/{}.owl", id)),
        }
    }
}
