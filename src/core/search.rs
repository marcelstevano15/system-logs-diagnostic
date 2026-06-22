use crate::errors::AppResult;
use crate::models::log_entry::LogEntry;
use anyhow::Context;
use arc_swap::ArcSwap;
use std::collections::HashSet;
use std::ops::Bound;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, FuzzyTermQuery, Occur, Query, RangeQuery, TermQuery};
use tantivy::schema::{
    Field, IndexRecordOption, NumericOptions, Schema, TextFieldIndexing, TextOptions, FAST,
    INDEXED, STORED,
};
use tantivy::tokenizer::{LowerCaser, RawTokenizer, RemoveLongFilter, TextAnalyzer};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};
use tantivy::schema::OwnedValue;
use parking_lot::Mutex;

const WRITER_HEAP_BYTES: usize = 64_000_000;
const KEYWORD_TOKENIZER_NAME: &str = "keyword_lower";
const MAX_KEYWORD_TOKEN_BYTES: usize = 256;
const EXACT_MATCH_BOOST: f32 = 4.0;
const FUZZY_MATCH_BOOST: f32 = 1.0;
const FUZZY_MIN_QUERY_LEN: usize = 4;
const FUZZY_MAX_DISTANCE: u8 = 1;

#[derive(Clone)]
pub struct SearchFields {
    pub seq_id: Field,
    pub timestamp: Field,
    pub priority: Field,
    pub process: Field,
    pub message: Field,
    pub severity: Field,
    pub unit: Field,
    pub hostname: Field,
    pub executable: Field,
}

#[derive(Clone)]
pub struct LogSearchEngine {
    pub schema: Schema,
    pub fields: SearchFields,
    pub index: Index,
    pub writer: Arc<Mutex<IndexWriter>>,
    pub reader: Arc<ArcSwap<IndexReader>>,
}

impl LogSearchEngine {
    pub fn new() -> AppResult<Self> {
        let mut schema_builder = Schema::builder();

        let num_options = NumericOptions::default()
            .set_indexed()
            .set_fast()
            .set_stored();

        let keyword_indexing = TextFieldIndexing::default()
            .set_tokenizer(KEYWORD_TOKENIZER_NAME)
            .set_index_option(IndexRecordOption::WithFreqs);

        let keyword_options = TextOptions::default()
            .set_indexing_options(keyword_indexing)
            .set_stored();

        let text_indexing = TextFieldIndexing::default()
            .set_tokenizer("default")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);

        let text_options = TextOptions::default()
            .set_indexing_options(text_indexing)
            .set_stored();

        let f_seq_id = schema_builder.add_u64_field("seq_id", num_options.clone());
        let f_timestamp = schema_builder.add_date_field("timestamp", INDEXED | STORED | FAST);
        let f_priority = schema_builder.add_u64_field("priority", num_options.clone());
        let f_process = schema_builder.add_text_field("process", keyword_options.clone());
        let f_message = schema_builder.add_text_field("message", text_options);
        let f_severity = schema_builder.add_text_field("severity", keyword_options.clone());
        let f_unit = schema_builder.add_text_field("unit", keyword_options.clone());
        let f_hostname = schema_builder.add_text_field("hostname", keyword_options.clone());
        let f_executable = schema_builder.add_text_field("executable", keyword_options);

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());

        let keyword_tokenizer = TextAnalyzer::builder(RawTokenizer::default())
            .filter(RemoveLongFilter::limit(MAX_KEYWORD_TOKEN_BYTES))
            .filter(LowerCaser)
            .build();
        index
            .tokenizers()
            .register(KEYWORD_TOKENIZER_NAME, keyword_tokenizer);

        let index_writer = index
            .writer(WRITER_HEAP_BYTES)
            .context("Failed to create index writer")?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create index reader")?;

        let fields = SearchFields {
            seq_id: f_seq_id,
            timestamp: f_timestamp,
            priority: f_priority,
            process: f_process,
            message: f_message,
            severity: f_severity,
            unit: f_unit,
            hostname: f_hostname,
            executable: f_executable,
        };

        Ok(Self {
            schema,
            fields,
            index,
            writer: Arc::new(Mutex::new(index_writer)),
            reader: Arc::new(ArcSwap::from_pointee(reader)),
        })
    }

    pub fn index_logs(&self, logs: &[LogEntry]) -> AppResult<()> {
        let docs: Vec<TantivyDocument> = logs
            .iter()
            .map(|log| log.to_tantivy_doc_with_fields(&self.fields))
            .collect();

        let mut writer = self.writer.lock();
        writer.delete_all_documents()?;
        for doc in docs {
            writer.add_document(doc)?;
        }
        writer.commit()?;
        drop(writer);

        self.reader.load().reload()?;
        Ok(())
    }

    pub fn index_batch(&self, logs: &[LogEntry]) -> AppResult<()> {
        if logs.is_empty() {
            return Ok(());
        }

        let docs: Vec<TantivyDocument> = logs
            .iter()
            .map(|log| log.to_tantivy_doc_with_fields(&self.fields))
            .collect();

        let mut writer = self.writer.lock();
        for doc in docs {
            writer.add_document(doc)?;
        }
        writer.commit()?;
        drop(writer);

        self.reader.load().reload()?;
        Ok(())
    }

    pub fn index_batch_no_commit(&self, logs: &[LogEntry]) -> AppResult<()> {
        if logs.is_empty() {
            return Ok(());
        }

        let docs: Vec<TantivyDocument> = logs
            .iter()
            .map(|log| log.to_tantivy_doc_with_fields(&self.fields))
            .collect();

        let mut writer = self.writer.lock();
        for doc in docs {
            writer.add_document(doc)?;
        }

        Ok(())
    }

    pub fn commit_and_reload(&self) -> AppResult<()> {
        let mut writer = self.writer.lock();
        writer.commit()?;
        drop(writer);
        self.reader.load().reload()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> AppResult<HashSet<u64>> {
        let reader_arc = self.reader.load();
        let searcher = reader_arc.searcher();

        let cleaned = query_str.trim().to_lowercase();
        if cleaned.is_empty() {
            return Ok(HashSet::new());
        }

        let keyword_clause = self.build_keyword_clause(&cleaned);
        let text_clause = self.build_text_clause(&cleaned);

        let mut top_level: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        if let Some(q) = keyword_clause {
            top_level.push((Occur::Should, q));
        }
        if let Some(q) = text_clause {
            top_level.push((Occur::Should, q));
        }

        if top_level.is_empty() {
            return Ok(HashSet::new());
        }

        let final_query = BooleanQuery::new(top_level);
        let top_docs = searcher.search(&final_query, &TopDocs::with_limit(limit))?;

        let mut matched_ids = HashSet::with_capacity(top_docs.len());
        for (_score, doc_address) in top_docs {
            let retrieved: TantivyDocument = searcher.doc(doc_address)?;
            if let Some(OwnedValue::U64(id)) = retrieved.get_first(self.fields.seq_id) {
                matched_ids.insert(*id);
            }
        }

        Ok(matched_ids)
    }

    fn build_keyword_clause(&self, cleaned: &str) -> Option<Box<dyn Query>> {
        let keyword_fields = [
            self.fields.process,
            self.fields.unit,
            self.fields.hostname,
            self.fields.executable,
            self.fields.severity,
        ];

        let mut field_clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        for &field in &keyword_fields {
            let term = Term::from_field_text(field, cleaned);

            let exact_q = TermQuery::new(term.clone(), IndexRecordOption::WithFreqs);
            field_clauses.push((
                Occur::Should,
                Box::new(BoostQuery::new(Box::new(exact_q), EXACT_MATCH_BOOST)),
            ));

            if let Some(prefix_end) = next_str_prefix(cleaned) {
                let start_term = Term::from_field_text(field, cleaned);
                let end_term = Term::from_field_text(field, &prefix_end);
                let field_name = self.schema.get_field_name(field).to_string();
                let range_q = RangeQuery::new_term_bounds(
                    field_name,
                    tantivy::schema::Type::Str,
                    &Bound::Included(start_term),
                    &Bound::Excluded(end_term),
                );
                field_clauses.push((Occur::Should, Box::new(range_q)));
            }

            if cleaned.chars().count() >= FUZZY_MIN_QUERY_LEN {
                let fuzzy_q = FuzzyTermQuery::new(term, FUZZY_MAX_DISTANCE, true);
                field_clauses.push((
                    Occur::Should,
                    Box::new(BoostQuery::new(Box::new(fuzzy_q), FUZZY_MATCH_BOOST)),
                ));
            }
        }

        if field_clauses.is_empty() {
            None
        } else {
            Some(Box::new(BooleanQuery::new(field_clauses)))
        }
    }

    fn build_text_clause(&self, cleaned: &str) -> Option<Box<dyn Query>> {
        let f_message = self.fields.message;
        let words: Vec<&str> = cleaned.split_whitespace().collect();

        if words.is_empty() {
            return None;
        }

        let mut word_clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        for word in &words {
            let term = Term::from_field_text(f_message, word);
            let mut per_word: Vec<(Occur, Box<dyn Query>)> = Vec::new();

            let exact_q = TermQuery::new(term.clone(), IndexRecordOption::WithFreqsAndPositions);
            per_word.push((
                Occur::Should,
                Box::new(BoostQuery::new(Box::new(exact_q), EXACT_MATCH_BOOST)),
            ));

            if word.chars().count() >= FUZZY_MIN_QUERY_LEN {
                let fuzzy_q = FuzzyTermQuery::new(term, FUZZY_MAX_DISTANCE, true);
                per_word.push((
                    Occur::Should,
                    Box::new(BoostQuery::new(Box::new(fuzzy_q), FUZZY_MATCH_BOOST)),
                ));
            }

            word_clauses.push((Occur::Must, Box::new(BooleanQuery::new(per_word))));
        }

        Some(Box::new(BooleanQuery::new(word_clauses)))
    }
}

fn next_str_prefix(s: &str) -> Option<String> {
    let mut chars: Vec<char> = s.chars().collect();
    loop {
        match chars.last_mut() {
            None => return None,
            Some(c) => {
                let next = char::from_u32(*c as u32 + 1);
                match next {
                    Some(nc) => {
                        *c = nc;
                        return Some(chars.into_iter().collect());
                    }
                    None => {
                        chars.pop();
                    }
                }
            }
        }
    }
}
