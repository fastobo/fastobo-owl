

use fastobo::ast as obo;
use horned_owl::model as owl;

use crate::constants::datatype;
use crate::constants::property;
use super::Context;
use super::IntoOwlCtx;

impl IntoOwlCtx for obo::HeaderClause {
    type Owl = Option<owl::AnnotatedAxiom>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        match self {
            // `oboInOwl:hasOBOFormatVersion` annotation
            obo::HeaderClause::FormatVersion(v) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(
                    owl::Annotation {
                        annotation_property: ctx.build.annotation_property(
                            property::obo_in_owl::HAS_OBO_FORMAT_VERSION
                        ),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                            literal: Some(v.into_string()),
                            lang: None,
                        })
                    }
                )
            )),

            // no equivalent
            // --> should be added as the Ontology IRI
            obo::HeaderClause::DataVersion(_) => None,

            // `oboInOwl:hasDate` annotation
            // --> QUESTION: should the datatype_iri be `dateTime` or `string` ?
            obo::HeaderClause::Date(dt) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(
                    owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::obo_in_owl::HAS_DATE),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::DATETIME)),
                            lang: None,
                            literal: Some(obo::DateTime::to_xsd_datetime(&dt)),
                        })
                    }
                )
            )),

            // `oboInOwl:savedBy` annotation
            obo::HeaderClause::SavedBy(n) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(
                    owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::obo_in_owl::SAVED_BY),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                            literal: Some(n.into_string()),
                            lang: None,
                        })
                    }
                )
            )),

            // `oboInOwl:autoGeneratedBy` annotation
            // --> FIXME: not actually declared in `oboInOwl`!
            obo::HeaderClause::AutoGeneratedBy(n) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(
                    owl::Annotation {
                        annotation_property: ctx.build.annotation_property(
                            property::obo_in_owl::AUTO_GENERATED_BY
                        ),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                            literal: Some(n.into_string()),
                            lang: None,
                        })
                    }
                )
            )),

            // `owl::imports`:
            // --> if in abbreviated form, use default http://purl.obolibrary.org/obo/ prefix
            // --> if URL, simply use that
            obo::HeaderClause::Import(import) => Some(
                owl::AnnotatedAxiom::from(
                    owl::Axiom::from(
                        horned_owl::model::Import(import.into_owl(ctx)
                    )
                )
            )),

            // `owl:AnnotationProperty`
            //     <owl:AnnotationProperty rdf:about=T(subset)>
            //         <rdfs:comment rdf:datatype="xsd:string">T(description)</rdfs:comment>
            //         <rdfs:subPropertyOf rdf:resource="http://www.geneontology.org/formats/oboInOwl#SubsetProperty"/>
            //     </owl:AnnotationProperty>
            // FIXME: Add description
            obo::HeaderClause::Subsetdef(subset, description) => Some(
                owl::AnnotatedAxiom::from(
                    owl::AnnotationAssertion {
                        annotation_subject: obo::Ident::from(subset).into_owl(ctx),
                        annotation: owl::Annotation {
                            annotation_property: ctx.build.annotation_property(
                                property::rdfs::SUB_PROPERTY_OF
                            ),
                            annotation_value: owl::AnnotationValue::IRI(
                                ctx.build.iri(property::obo_in_owl::SUBSET_PROPERTY)
                            )
                        }
                    }
                )
            ),

            // `owl:AnnotationProperty`
            //      <owl:AnnotationProperty rdf:about="http://purl.obolibrary.org/obo/go#systematic_synonym">
            //          <oboInOwl:hasScope rdf:resource="http://www.geneontology.org/formats/oboInOwl#hasExactSynonym"/>
            //          <rdfs:label rdf:datatype="http://www.w3.org/2001/XMLSchema#string">Systematic synonym</rdfs:label>
            //          <rdfs:subPropertyOf rdf:resource="http://www.geneontology.org/formats/oboInOwl#SynonymTypeProperty"/>
            //      </owl:AnnotationProperty>
            // FIXME: Add description and scope
            obo::HeaderClause::SynonymTypedef(ty, desc, scope) => Some(
                owl::AnnotatedAxiom::from(
                    owl::AnnotationAssertion {
                        annotation_subject: obo::Ident::from(ty).into_owl(ctx),
                        annotation: owl::Annotation {
                            annotation_property: ctx.build.annotation_property(
                                property::rdfs::SUB_PROPERTY_OF
                            ),
                            annotation_value: owl::AnnotationValue::IRI(
                                ctx.build.iri(property::obo_in_owl::SYNONYM_TYPE_PROPERTY),
                            )
                        }
                    }
                )
            ),

            // `oboInOwl:hasDefaultNamespace` annotation
            obo::HeaderClause::DefaultNamespace(ns) => Some(
                owl::AnnotatedAxiom::from(
                    owl::OntologyAnnotation(
                        owl::Annotation {
                            annotation_property: ctx.build.annotation_property(
                                property::obo_in_owl::HAS_DEFAULT_NAMESPACE
                            ),
                            annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                                datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                                literal: Some(ns.to_string()),
                                lang: None,
                            })
                        }
                    )
                )
            ),

            obo::HeaderClause::NamespaceIdRule(r) => Some(
                owl::AnnotatedAxiom::from(
                    owl::OntologyAnnotation(
                        owl::Annotation {
                            annotation_property: ctx.build.annotation_property(
                                property::obo_in_owl::NAMESPACE_ID_RULE
                            ),
                            annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                                datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                                literal: Some(r.into_string()),
                                lang: None,
                            })
                        }
                    )
                )
            ),

            // no actual OWL equivalent, but we expose the IDspace as an OWL
            // prefix to retain the same CURIES in the OWL ontology.
            // earlier when creating the conversion context.
            obo::HeaderClause::Idspace(prefix, url, _) => None,

            // no equivalent, macros should be resolved before conversion.
            obo::HeaderClause::TreatXrefsAsEquivalent(_) => None,
            obo::HeaderClause::TreatXrefsAsGenusDifferentia(_, _, _) => None,
            obo::HeaderClause::TreatXrefsAsReverseGenusDifferentia(_, _, _) => None,
            obo::HeaderClause::TreatXrefsAsRelationship(_, _) => None,
            obo::HeaderClause::TreatXrefsAsIsA(_) => None,
            obo::HeaderClause::TreatXrefsAsHasSubclass(_) => None,

            // `rdfs:comment` annotation
            obo::HeaderClause::Remark(v) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(
                    owl::Annotation {
                        annotation_property: ctx.build.annotation_property(property::rdfs::COMMENT),
                        annotation_value: owl::AnnotationValue::Literal(owl::Literal {
                            datatype_iri: Some(ctx.build.iri(datatype::xsd::STRING)),
                            literal: Some(v.into_string()),
                            lang: None,
                        })
                    }
                )
            )),

            // translate as an annotation
            obo::HeaderClause::PropertyValue(pv) => Some(owl::AnnotatedAxiom::from(
                owl::OntologyAnnotation(pv.into_owl(ctx))
            )),

            // no equivalent:
            // --> should be added as the Ontology IRI
            obo::HeaderClause::Ontology(_) => None,

            // should be added as-is but needs a Manchester-syntax parser
            // obo::HeaderClause::OwlAxioms(_) => unimplemented!("cannot translate `owl-axioms` currently"),
            obo::HeaderClause::OwlAxioms(_) => None,

            // no equivalent
            // --> FIXME: namespace-id-rule ?
            obo::HeaderClause::Unreserved(_, _) => None, // FIXME ?
        }
    }
}

impl IntoOwlCtx for obo::HeaderFrame {
    type Owl = Vec<Option<owl::AnnotatedAxiom>>;
    fn into_owl(self, ctx: &mut Context) -> Self::Owl {
        self.into_iter().map(|clause| clause.into_owl(ctx)).collect()
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
            obo::Import::Abbreviated(id) => ctx.build.iri(format!(
                "http://purl.obolibrary.org/obo/{}.owl",
                id
            )),
        }
    }
}
