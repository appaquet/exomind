use exocore_protos::store::{
    boolean_predicate, entity_query::Predicate, ordering, trait_field_predicate, trait_query,
    EntityQuery, MatchPredicate, Ordering, Paging, TraitFieldPredicate,
    TraitFieldReferencePredicate,
};
use tantivy::{
    query::{AllQuery, BooleanQuery, FuzzyTermQuery, Occur, PhraseQuery, Query, TermQuery},
    schema::{Field, IndexRecordOption},
    Index, Term,
};

use super::{schema::MutationIndexSchema, MutationIndexConfig};
use crate::error::Error;

pub(crate) struct ParsedQuery {
    pub tantivy: Box<dyn Query>,
    pub paging: Paging,
    pub ordering: Ordering,
    pub trait_name: Option<String>,
}

pub(crate) struct QueryParser<'s> {
    index: &'s Index,
    fields: &'s MutationIndexSchema,
    config: &'s MutationIndexConfig,
    proto: &'s EntityQuery,
    tantivy: Option<Box<dyn Query>>,
    paging: Paging,
    ordering: Ordering,
    trait_name: Option<String>,
}

impl<'s> QueryParser<'s> {
    pub fn parse(
        index: &'s Index,
        fields: &'s MutationIndexSchema,
        config: &'s MutationIndexConfig,
        proto: &'s EntityQuery,
    ) -> Result<ParsedQuery, Error> {
        let mut parser = QueryParser {
            index,
            fields,
            config,
            proto,
            tantivy: None,
            paging: Default::default(),
            ordering: Default::default(),
            trait_name: None,
        };

        parser.inner_parse()?;

        let tantivy = parser
            .tantivy
            .as_ref()
            .map(|q| q.box_clone())
            .ok_or_else(|| Error::QueryParsing(anyhow!("query didn't didn't get parsed")))?;

        Ok(ParsedQuery {
            tantivy,
            paging: parser.paging,
            ordering: parser.ordering,
            trait_name: parser.trait_name,
        })
    }

    fn inner_parse(&mut self) -> Result<(), Error> {
        let predicate = self
            .proto
            .predicate
            .as_ref()
            .ok_or(Error::ProtoFieldExpected("predicate"))?;

        self.paging = self.proto.paging.clone().unwrap_or(Paging {
            after_ordering_value: None,
            before_ordering_value: None,
            count: self.config.iterator_page_size,
            offset: 0,
        });
        self.ordering = self.proto.ordering.clone().unwrap_or_default();
        self.tantivy = Some(self.parse_predicate(predicate)?);

        Ok(())
    }

    fn parse_predicate(&mut self, predicate: &Predicate) -> Result<Box<dyn Query>, Error> {
        match predicate {
            Predicate::Match(match_pred) => self.parse_match_pred(match_pred),
            Predicate::Trait(trait_pred) => self.parse_trait_pred(trait_pred),
            Predicate::Ids(ids_pred) => self.parse_ids_pred(ids_pred),
            Predicate::Reference(ref_pred) => self.parse_ref_pred(self.fields.all_refs, ref_pred),
            Predicate::Operations(op_pred) => self.parse_operation_pred(op_pred),
            Predicate::All(_all_pred) => self.parse_all_pred(),
            Predicate::Boolean(bool_pred) => self.parse_bool_pred(bool_pred),
            Predicate::QueryString(query_pred) => self.query_string_pred(query_pred),
            Predicate::Test(_) => Err(anyhow!("Query failed for tests").into()),
        }
    }

    fn parse_match_pred(&mut self, match_pred: &MatchPredicate) -> Result<Box<dyn Query>, Error> {
        if self.ordering.value.is_none() {
            self.ordering.value = Some(ordering::Value::Score(true));
        }

        let field = self.fields.all_text;
        let text = match_pred.query.as_str();
        let no_fuzzy = match_pred.no_fuzzy;
        Ok(Box::new(self.new_fuzzy_match_query(field, text, no_fuzzy)?))
    }

    fn parse_trait_pred(
        &mut self,
        trait_pred: &exocore_protos::store::TraitPredicate,
    ) -> Result<Box<dyn Query>, Error> {
        if self.ordering.value.is_none() {
            self.ordering.value = Some(ordering::Value::OperationId(true));
        }

        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        let trait_type_query = self.trait_type_query(&trait_pred.trait_name);
        queries.push((Occur::Must, Box::new(trait_type_query)));

        if let Some(cur_trait_name) = self.trait_name.as_ref() {
            if cur_trait_name != &trait_pred.trait_name {
                return Err(Error::QueryParsing(anyhow!(
                    "can't query multiple traits: current={} new={}",
                    cur_trait_name,
                    trait_pred.trait_name
                )));
            }
        } else {
            self.trait_name = Some(trait_pred.trait_name.clone());
        }

        if let Some(trait_query) = &trait_pred.query {
            match &trait_query.predicate {
                Some(trait_query::Predicate::Match(trait_pred)) => {
                    queries.push((Occur::Must, self.parse_match_pred(trait_pred)?));
                }
                Some(trait_query::Predicate::Field(field_trait_pred)) => {
                    queries.push((Occur::Must, self.parse_field_predicate(field_trait_pred)?));
                }
                Some(trait_query::Predicate::Reference(field_ref_pred)) => {
                    queries.push((
                        Occur::Must,
                        self.parse_trait_field_reference_predicate(field_ref_pred)?,
                    ));
                }
                Some(trait_query::Predicate::QueryString(query_pred)) => {
                    queries.push((Occur::Must, self.query_string_pred(query_pred)?));
                }
                None => {}
            }
        }

        Ok(Box::new(BooleanQuery::from(queries)))
    }

    fn trait_type_query(&mut self, trait_name: &str) -> TermQuery {
        let trait_type = Term::from_field_text(self.fields.trait_type, trait_name);
        TermQuery::new(trait_type, IndexRecordOption::Basic)
    }

    fn parse_field_predicate(
        &self,
        predicate: &TraitFieldPredicate,
    ) -> Result<Box<dyn Query>, Error> {
        use exocore_protos::reflect::FieldType as FT;
        use trait_field_predicate::Value as PV;

        let trait_name = self
            .trait_name
            .as_ref()
            .ok_or_else(|| Error::QueryParsing(anyhow!("expected trait name")))?;

        let fields = self
            .fields
            .get_dynamic_trait_field_prefix(trait_name, &predicate.field)?;

        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for field in fields {
            match (&field.field_type, &predicate.value) {
                (FT::String, Some(PV::String(value))) => {
                    let term = Term::from_field_text(field.field, value);

                    queries.push((Occur::Should, Box::new(TermQuery::new(term, IndexRecordOption::Basic))));
                }
                (ft, pv) => {
                    return Err(
                        Error::QueryParsing(
                            anyhow!(
                                "Incompatible field type vs field value in predicate: trait_name={} field={}, field_type={:?}, value={:?}",
                                trait_name,
                                predicate.field,
                                ft,
                                pv,
                            ))
                    )
                }
            }
        }

        Ok(Box::new(BooleanQuery::from(queries)))
    }

    fn parse_trait_field_reference_predicate(
        &mut self,
        predicate: &TraitFieldReferencePredicate,
    ) -> Result<Box<dyn Query>, Error> {
        let trait_name = self
            .trait_name
            .as_ref()
            .ok_or_else(|| Error::QueryParsing(anyhow!("expected trait name")))?;

        let field = self
            .fields
            .get_dynamic_trait_field(trait_name, &predicate.field)?;

        let reference = predicate
            .reference
            .as_ref()
            .ok_or(Error::ProtoFieldExpected("reference"))?;

        self.parse_ref_pred(field.field, reference)
    }

    fn parse_ids_pred(
        &mut self,
        ids_pred: &exocore_protos::store::IdsPredicate,
    ) -> Result<Box<dyn Query>, Error> {
        if self.ordering.value.is_none() {
            self.ordering.value = Some(ordering::Value::OperationId(true));
            self.ordering.ascending = true;
        }

        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for entity_id in &ids_pred.ids {
            let term = Term::from_field_text(self.fields.entity_id, entity_id);
            let query = TermQuery::new(term, IndexRecordOption::Basic);
            queries.push((Occur::Should, Box::new(query)));
        }
        Ok(Box::new(BooleanQuery::from(queries)))
    }

    fn parse_ref_pred(
        &mut self,
        field: Field,
        ref_pred: &exocore_protos::store::ReferencePredicate,
    ) -> Result<Box<dyn Query>, Error> {
        if self.ordering.value.is_none() {
            self.ordering.value = Some(ordering::Value::OperationId(true));
            self.ordering.ascending = true;
        }

        let query: Box<dyn tantivy::query::Query> = if !ref_pred.trait_id.is_empty() {
            let terms = vec![
                Term::from_field_text(field, &format!("entity{}", ref_pred.entity_id)),
                Term::from_field_text(field, &format!("trait{}", ref_pred.trait_id)),
            ];
            Box::new(PhraseQuery::new(terms))
        } else {
            Box::new(TermQuery::new(
                Term::from_field_text(field, &format!("entity{}", ref_pred.entity_id)),
                IndexRecordOption::Basic,
            ))
        };

        Ok(query)
    }

    fn parse_operation_pred(
        &mut self,
        op_pred: &exocore_protos::store::OperationsPredicate,
    ) -> Result<Box<dyn Query>, Error> {
        if self.ordering.value.is_none() {
            self.ordering.value = Some(ordering::Value::OperationId(true));
            self.ordering.ascending = true;
        }

        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for operation_id in &op_pred.operation_ids {
            let op_term = Term::from_field_u64(self.fields.operation_id, *operation_id);
            let op_query = TermQuery::new(op_term, IndexRecordOption::Basic);
            queries.push((Occur::Should, Box::new(op_query)));
        }
        Ok(Box::new(BooleanQuery::from(queries)))
    }

    fn parse_all_pred(&mut self) -> Result<Box<dyn Query>, Error> {
        if self.ordering.value.is_none() {
            self.ordering.value = Some(ordering::Value::OperationId(true));
            self.ordering.ascending = false;
        }

        Ok(Box::new(AllQuery))
    }

    fn parse_bool_pred(
        &mut self,
        bool_pred: &exocore_protos::store::BooleanPredicate,
    ) -> Result<Box<dyn Query>, Error> {
        use boolean_predicate::{sub_query::Predicate as SubPredicate, Occur as ProtoOccur};
        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        for sub_query in &bool_pred.queries {
            let predicate = sub_query
                .predicate
                .as_ref()
                .ok_or(Error::ProtoFieldExpected("predicate"))?;

            let tantivy_query = match predicate {
                SubPredicate::Match(match_pred) => self.parse_match_pred(match_pred)?,
                SubPredicate::Trait(trait_pred) => self.parse_trait_pred(trait_pred)?,
                SubPredicate::Ids(ids_pred) => self.parse_ids_pred(ids_pred)?,
                SubPredicate::Reference(ref_pred) => {
                    self.parse_ref_pred(self.fields.all_refs, ref_pred)?
                }
                SubPredicate::Operations(op_pred) => self.parse_operation_pred(op_pred)?,
                SubPredicate::All(_all_pred) => self.parse_all_pred()?,
                SubPredicate::Boolean(bool_pred) => self.parse_bool_pred(bool_pred)?,
            };

            let tantivy_occur = match ProtoOccur::try_from(sub_query.occur) {
                Ok(ProtoOccur::Should) => Occur::Should,
                Ok(ProtoOccur::Must) => Occur::Must,
                Ok(ProtoOccur::MustNot) => Occur::MustNot,
                Err(err) => {
                    return Err(Error::QueryParsing(anyhow!(
                        "Invalid occur value: {}. err: {err}",
                        sub_query.occur
                    )));
                }
            };

            queries.push((tantivy_occur, tantivy_query));
        }

        Ok(Box::new(BooleanQuery::from(queries)))
    }

    fn query_string_pred(
        &mut self,
        query_pred: &exocore_protos::store::QueryStringPredicate,
    ) -> Result<Box<dyn Query>, Error> {
        if self.ordering.value.is_none() {
            self.ordering.value = Some(ordering::Value::Score(true));
        }

        let parsed = QueryString::parse(&query_pred.query)?;
        if parsed.parts.is_empty() {
            self.parse_all_pred()
        } else {
            let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
            let mut one_positive = false;
            for part in parsed.parts {
                let mut occur = part.occur.to_tantivy();
                if part.field != "sort" && (occur == Occur::Should || occur == Occur::Must) {
                    one_positive = true;
                }

                if part.field == "type" {
                    if occur == Occur::Should {
                        occur = Occur::Must;
                    }

                    let trait_name = self
                        .fields
                        .get_message_name_from_short(&part.text)
                        .unwrap_or(&part.text);
                    queries.push((occur, Box::new(self.trait_type_query(trait_name))));

                    if occur == Occur::Must || (occur == Occur::Should && self.trait_name.is_none())
                    {
                        self.trait_name = Some(trait_name.to_string());
                    }
                } else if part.field == "sort" {
                    if part.text.starts_with("update") {
                        self.ordering.value = Some(ordering::Value::UpdatedAt(true));
                        self.ordering.ascending = false;
                    } else if part.text == "date" || part.text.starts_with("create") {
                        self.ordering.value = Some(ordering::Value::CreatedAt(true));
                        self.ordering.ascending = false;
                    } else if part.text == "score" {
                        self.ordering.value = Some(ordering::Value::Score(true));
                        self.ordering.ascending = false;
                    }

                    if occur == Occur::MustNot {
                        self.ordering.ascending = !self.ordering.ascending;
                    }
                } else if part.phrase {
                    if occur == Occur::Should {
                        occur = Occur::Must;
                    }

                    let field = if part.field.is_empty() {
                        self.fields.all_text
                    } else {
                        self.field_or_text(&part.field)?
                    };

                    queries.push((occur, Box::new(self.new_phrase_query(field, &part.text)?)));
                } else {
                    let field = if part.field.is_empty() {
                        self.fields.all_text
                    } else {
                        self.field_or_text(&part.field)?
                    };

                    queries.push((
                        part.occur.to_tantivy(),
                        Box::new(self.new_fuzzy_match_query(field, &part.text, false)?),
                    ));
                }
            }

            if !one_positive {
                queries.push((Occur::Should, Box::new(AllQuery)));
            }

            Ok(Box::new(BooleanQuery::from(queries)))
        }
    }

    fn field_or_text(&self, field: &str) -> Result<Field, Error> {
        match &self.trait_name {
            Some(trait_name) => Ok(self
                .fields
                .get_dynamic_trait_field(trait_name, field)?
                .field),
            None => Ok(self.fields.all_text),
        }
    }

    fn new_fuzzy_match_query(
        &self,
        field: Field,
        text: &str,
        no_fuzzy: bool,
    ) -> Result<BooleanQuery, Error> {
        let tok = self.index.tokenizer_for_field(field)?;
        let mut queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        let mut stream = tok.token_stream(text);

        while stream.advance() {
            let token = stream.token().text.as_str();
            let term = Term::from_field_text(field, token);

            if !no_fuzzy && token.len() > 3 {
                let max_distance = if token.len() > 6 { 2 } else { 1 };
                let query = Box::new(FuzzyTermQuery::new(term.clone(), max_distance, true));
                queries.push((Occur::Should, query));
            }

            // even if fuzzy is enabled, we add the term again so that an exact match scores
            // more
            let query = Box::new(TermQuery::new(
                term,
                IndexRecordOption::WithFreqsAndPositions,
            ));
            queries.push((Occur::Should, query));
        }

        Ok(BooleanQuery::from(queries))
    }

    fn new_phrase_query(&self, field: Field, text: &str) -> Result<Box<dyn Query>, Error> {
        let tok = self.index.tokenizer_for_field(field)?;
        let mut terms = Vec::new();
        let mut stream = tok.token_stream(text);

        while stream.advance() {
            let token = stream.token().text.as_str();
            let term = Term::from_field_text(field, token);
            terms.push(term);
        }

        if terms.is_empty() {
            Err(Error::QueryParsing(anyhow!("empty phrase query")))
        } else if terms.len() == 1 {
            Ok(Box::new(TermQuery::new(
                terms[0].clone(),
                IndexRecordOption::WithFreqsAndPositions,
            )))
        } else {
            Ok(Box::new(PhraseQuery::new(terms)))
        }
    }
}

#[derive(Default)]
struct QueryString {
    pub parts: Vec<QSPart>,
    plain_part: QSPart,
}

impl QueryString {
    fn parse(query: &str) -> Result<QueryString, Error> {
        let mut qs = QueryString::default();
        let mut part = QSPart::default();

        for chr in query.to_lowercase().chars() {
            if part.phrase {
                if chr == '"' {
                    qs.push(part);
                    part = QSPart::default();
                } else {
                    part.text.push(chr);
                }
            } else if part.in_parenthesis {
                if chr == ')' {
                    qs.push(part);
                    part = QSPart::default();
                } else {
                    part.text.push(chr);
                }
            } else if chr.is_whitespace() {
                qs.push(part);
                part = QSPart::default();
            } else if chr == '+' {
                part.occur = QSOccur::Must;
            } else if chr == '-' {
                part.occur = QSOccur::MustNot;
            } else if chr == ':' {
                part.field.clone_from(&part.text);
                part.text.clear();
            } else if chr == '"' {
                part.phrase = true;
            } else if chr == '(' {
                part.in_parenthesis = true;
            } else {
                part.text.push(chr);
            }
        }

        qs.push(part);

        if !qs.plain_part.is_empty() {
            let part = std::mem::take(&mut qs.plain_part);
            qs.parts.push(part);
        }

        Ok(qs)
    }

    fn push(&mut self, part: QSPart) {
        if part.is_empty() {
            return;
        }

        if part.is_plain() {
            self.plain_part.combine_from(&part);
        } else {
            self.parts.push(part);
        }
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.parts.is_empty() && self.plain_part.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QSOccur {
    Must,
    MustNot,
    Should,
}

impl QSOccur {
    fn to_tantivy(self) -> Occur {
        match self {
            QSOccur::Must => Occur::Must,
            QSOccur::MustNot => Occur::MustNot,
            QSOccur::Should => Occur::Should,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QSPart {
    occur: QSOccur,
    text: String,
    field: String,
    phrase: bool,
    in_parenthesis: bool,
}

impl Default for QSPart {
    fn default() -> Self {
        Self {
            occur: QSOccur::Should,
            text: String::new(),
            field: String::new(),
            phrase: false,
            in_parenthesis: false,
        }
    }
}

impl QSPart {
    fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn is_plain(&self) -> bool {
        self.occur == QSOccur::Should
            && !self.phrase
            && !self.in_parenthesis
            && self.field.is_empty()
    }

    fn combine_from(&mut self, other: &QSPart) {
        if !self.text.is_empty() {
            self.text += " ";
        }
        self.text += &other.text;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let qs = QueryString::parse("").unwrap();
        assert!(qs.is_empty());

        let qs = QueryString::parse("hello world").unwrap();
        assert_eq!(qs.parts.len(), 1);
        assert_eq!(qs.parts[0].occur, QSOccur::Should);
        assert_eq!(qs.parts[0].text, "hello world");

        let qs = QueryString::parse("+hello -world").unwrap();
        assert_eq!(qs.parts.len(), 2);
        assert_eq!(qs.parts[0].occur, QSOccur::Must);
        assert_eq!(qs.parts[0].text, "hello");
        assert_eq!(qs.parts[1].occur, QSOccur::MustNot);
        assert_eq!(qs.parts[1].text, "world");

        let qs = QueryString::parse("\"hello world\"").unwrap();
        assert_eq!(qs.parts.len(), 1);
        assert_eq!(qs.parts[0].occur, QSOccur::Should);
        assert_eq!(qs.parts[0].text, "hello world");
        assert!(qs.parts[0].phrase);

        let qs = QueryString::parse("field:token").unwrap();
        assert_eq!(qs.parts.len(), 1);
        assert_eq!(qs.parts[0].occur, QSOccur::Should);
        assert_eq!(qs.parts[0].field, "field");
        assert_eq!(qs.parts[0].text, "token");

        let qs = QueryString::parse("+field:(hello world)").unwrap();
        assert_eq!(qs.parts.len(), 1);
        assert_eq!(qs.parts[0].occur, QSOccur::Must);
        assert_eq!(qs.parts[0].field, "field");
        assert_eq!(qs.parts[0].text, "hello world");
    }
}
