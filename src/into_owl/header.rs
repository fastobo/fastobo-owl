use fastobo::ast as obo;
use horned_owl::model as owl;
use horned_owl::model::AnnotatedComponent;
use horned_owl::model::ForIRI;
use horned_owl::model::HigherKinded;
use horned_owl::model::OntologyID;

use super::Context;
use super::IntoOwlCtx;
use crate::constants::property;
use crate::constants::uri;

impl<A: ForIRI> IntoOwlCtx<A> for obo::HeaderClause {
    type Owl = Vec<owl::AnnotatedComponent<A>>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        match self {
            // `oboInOwl:hasOBOFormatVersion` annotation
            obo::HeaderClause::FormatVersion(v) => vec![owl::AnnotatedComponent::from(
                owl::OntologyAnnotation(owl::Annotation {
                    ap: ctx
                        .build
                        .annotation_property(property::obo_in_owl::HAS_OBO_FORMAT_VERSION),
                    av: v.into_owl(ctx).into(),
                }),
            )],

            // no equivalent
            // --> should be added as the Ontology IRI
            obo::HeaderClause::DataVersion(_) => Vec::new(),

            // `oboInOwl:hasDate` annotation
            // --> QUESTION: should the datatype_iri be `dateTime` or `string` ?
            obo::HeaderClause::Date(dt) => vec![owl::AnnotatedComponent::from(
                owl::OntologyAnnotation(owl::Annotation {
                    ap: ctx
                        .build
                        .annotation_property(property::obo_in_owl::HAS_DATE),
                    av: dt.into_owl(ctx).into(),
                }),
            )],

            // `oboInOwl:savedBy` annotation
            obo::HeaderClause::SavedBy(n) => vec![owl::AnnotatedComponent::from(
                owl::OntologyAnnotation(owl::Annotation {
                    ap: ctx
                        .build
                        .annotation_property(property::obo_in_owl::SAVED_BY),
                    av: n.into_owl(ctx).into(),
                }),
            )],

            // `oboInOwl:autoGeneratedBy` annotation
            // --> FIXME: not actually declared in `oboInOwl`!
            obo::HeaderClause::AutoGeneratedBy(n) => vec![owl::AnnotatedComponent::from(
                owl::OntologyAnnotation(owl::Annotation {
                    ap: ctx
                        .build
                        .annotation_property(property::obo_in_owl::AUTO_GENERATED_BY),
                    av: n.into_owl(ctx).into(),
                }),
            )],

            // `owl::imports`:
            // --> if in abbreviated form, use default http://purl.obolibrary.org/obo/ prefix
            // --> if URL, simply use that
            obo::HeaderClause::Import(import) => vec![owl::AnnotatedComponent::from(
                owl::Component::from(horned_owl::model::Import(import.into_owl(ctx))),
            )],

            // `owl:AnnotationProperty`
            //     <owl:AnnotationProperty rdf:about=T(subset)>
            //         <rdfs:comment rdf:datatype="xsd:string">T(description)</rdfs:comment>
            //         <rdfs:subPropertyOf rdf:resource="http://www.geneontology.org/formats/oboInOwl#SubsetProperty"/>
            //     </owl:AnnotationProperty>
            obo::HeaderClause::Subsetdef(subset, desc) => vec![
                owl::AnnotatedComponent::from(owl::DeclareAnnotationProperty(
                    owl::AnnotationProperty::from(subset.into_owl(ctx)),
                )),
                owl::AnnotatedComponent::from(owl::SubAnnotationPropertyOf {
                    sub: owl::AnnotationProperty::from(subset.into_owl(ctx)),
                    sup: owl::AnnotationProperty::from(
                        ctx.build.iri(property::obo_in_owl::SUBSET_PROPERTY),
                    ),
                }),
                owl::AnnotatedComponent::from(owl::AnnotationAssertion {
                    subject: owl::AnnotationSubject::from(subset.into_owl(ctx)),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::rdfs::LABEL),
                        av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                            literal: subset.to_string(),
                        }),
                    },
                }),
                owl::AnnotatedComponent::from(owl::AnnotationAssertion {
                    subject: owl::AnnotationSubject::from(subset.into_owl(ctx)),
                    ann: owl::Annotation {
                        ap: ctx.build.annotation_property(property::rdfs::COMMENT),
                        av: owl::AnnotationValue::Literal(desc.into_owl(ctx)),
                    },
                }),
            ],

            // `owl:AnnotationProperty`
            //      <owl:AnnotationProperty rdf:about="http://purl.obolibrary.org/obo/go#systematic_synonym">
            //          <oboInOwl:hasScope rdf:resource="http://www.geneontology.org/formats/oboInOwl#hasExactSynonym"/>
            //          <rdfs:label rdf:datatype="http://www.w3.org/2001/XMLSchema#string">Systematic synonym</rdfs:label>
            //          <rdfs:subPropertyOf rdf:resource="http://www.geneontology.org/formats/oboInOwl#SynonymTypeProperty"/>
            //      </owl:AnnotationProperty>
            obo::HeaderClause::SynonymTypedef(ty, desc, optscope) => {
                let mut axioms = vec![
                    owl::AnnotatedComponent::from(owl::DeclareAnnotationProperty(
                        owl::AnnotationProperty::from(ty.into_owl(ctx)),
                    )),
                    owl::AnnotatedComponent::from(owl::SubAnnotationPropertyOf {
                        sub: owl::AnnotationProperty::from(ty.into_owl(ctx)),
                        sup: owl::AnnotationProperty::from(
                            ctx.build.iri(property::obo_in_owl::SYNONYM_TYPE_PROPERTY),
                        ),
                    }),
                    owl::AnnotatedComponent::from(owl::AnnotationAssertion {
                        subject: owl::AnnotationSubject::from(ty.into_owl(ctx)),
                        ann: owl::Annotation {
                            ap: ctx.build.annotation_property(property::rdfs::LABEL),
                            av: owl::AnnotationValue::Literal(desc.into_owl(ctx)),
                        },
                    }),
                ];
                if let Some(scope) = optscope {
                    axioms.push(owl::AnnotatedComponent::from(owl::AnnotationAssertion {
                        subject: owl::AnnotationSubject::from(ty.into_owl(ctx)),
                        ann: owl::Annotation {
                            ap: ctx
                                .build
                                .annotation_property(property::obo_in_owl::HAS_SCOPE),
                            av: owl::AnnotationValue::IRI(scope.into_owl(ctx)),
                        },
                    }));
                }
                axioms
            }

            // `oboInOwl:hasDefaultNamespace` annotation
            obo::HeaderClause::DefaultNamespace(ns) => vec![owl::AnnotatedComponent::from(
                owl::OntologyAnnotation(owl::Annotation {
                    ap: ctx
                        .build
                        .annotation_property(property::obo_in_owl::HAS_DEFAULT_NAMESPACE),
                    av: owl::AnnotationValue::Literal(owl::Literal::Simple {
                        literal: ns.to_string(),
                    }),
                }),
            )],

            // `oboInOwl:namespaceIdRule` annotation
            obo::HeaderClause::NamespaceIdRule(r) => vec![owl::AnnotatedComponent::from(
                owl::OntologyAnnotation(owl::Annotation {
                    ap: ctx
                        .build
                        .annotation_property(property::obo_in_owl::NAMESPACE_ID_RULE),
                    av: r.into_owl(ctx).into(),
                }),
            )],

            // no actual OWL equivalent, but we expose the IDspace as an OWL
            // prefix to retain the same CURIES in the OWL ontology.
            // earlier when creating the conversion context.
            obo::HeaderClause::Idspace(_, _, _) => Vec::new(),

            // no equivalent, macros should be resolved before conversion.
            obo::HeaderClause::TreatXrefsAsEquivalent(_) => Vec::new(),
            obo::HeaderClause::TreatXrefsAsGenusDifferentia(_, _, _) => Vec::new(),
            obo::HeaderClause::TreatXrefsAsReverseGenusDifferentia(_, _, _) => Vec::new(),
            obo::HeaderClause::TreatXrefsAsRelationship(_, _) => Vec::new(),
            obo::HeaderClause::TreatXrefsAsIsA(_) => Vec::new(),
            obo::HeaderClause::TreatXrefsAsHasSubclass(_) => Vec::new(),

            // `rdfs:comment` annotation
            obo::HeaderClause::Remark(v) => vec![owl::AnnotatedComponent::from(
                owl::OntologyAnnotation(owl::Annotation {
                    ap: ctx.build.annotation_property(property::rdfs::COMMENT),
                    av: v.into_owl(ctx).into(),
                }),
            )],

            // translate as an annotation
            obo::HeaderClause::PropertyValue(pv) => vec![owl::AnnotatedComponent::from(
                owl::OntologyAnnotation(pv.into_owl(ctx)),
            )],

            // no actual OWL equivalent, but exposed as the Ontology IRI
            // when creating the conversion context.
            obo::HeaderClause::Ontology(_) => Vec::new(),

            // handled in the header frame translation.
            obo::HeaderClause::OwlAxioms(_) => Vec::new(),

            // no equivalent for undefined header tag/values
            obo::HeaderClause::Unreserved(_, _) => Vec::new(),
        }
    }
}

impl<A: ForIRI> IntoOwlCtx<A> for obo::HeaderFrame {
    type Owl = Vec<owl::AnnotatedComponent<A>>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
        let mut owl_axioms: Vec<String> = Vec::new();
        let mut axioms: Vec<owl::AnnotatedComponent<A>> = Vec::with_capacity(self.len());

        // declare the IRI and Version IRI for the ontology.
        if let Ok(name) = self.ontology() {
            let oid = OntologyID {
                iri: Some(ctx.build.iri(format!("{}{}.owl", uri::OBO, name))),
                viri: self
                    .data_version()
                    .map(|dv| {
                        ctx.build
                            .iri(format!("{}{}/{}/{}.owl", uri::OBO, name, dv, name))
                    })
                    .ok(),
            };
            axioms.push(AnnotatedComponent::from(oid));
        } else {
            axioms.push(AnnotatedComponent::from(OntologyID::default()));
        }

        // Process the header frame clauses
        for clause in self.into_iter() {
            if let obo::HeaderClause::OwlAxioms(s) = clause {
                owl_axioms.push(s.into_string());
            } else {
                axioms.append(&mut clause.into_owl(ctx))
            }
        }

        // FIXME: https://github.com/owlcollab/oboformat/issues/116
        // Parse the remaining axioms in `owl-axioms` clauses.
        if !owl_axioms.is_empty() {
            let text = owl_axioms.join("\n");
            let reader = std::io::BufReader::new(std::io::Cursor::new(&text));
            let (ont, _) = horned_owl::io::ofn::reader::read_with_build(reader, &ctx.build)
                .expect("invalid functional ontology");
            axioms.extend(ont.into_iter().filter(|c| !c.is_meta()));
        }

        axioms
    }
}

impl<A: ForIRI> IntoOwlCtx<A> for obo::Import {
    type Owl = owl::IRI<A>;
    fn into_owl(self, ctx: &mut Context<A>) -> Self::Owl {
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
