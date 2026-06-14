use crate::models::log_entry::{LogEntry, Severity};
use anyhow::Result;
use std::sync::Arc;
use tantivy::schema::{Schema, TEXT, STORED, INDEXED, NumericOptions, FAST, OwnedValue};
use tantivy::{Index, IndexWriter, IndexReader, ReloadPolicy, TantivyDocument};
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, BooleanQuery, Occur, TermQuery};
use tantivy::schema::Term;
use chrono::{Utc, TimeZone};

#[derive(Clone)]
pub struct LogSearchEngine {
    pub schema: Schema,
    pub index: Index,
    pub writer: Arc<parking_lot::Mutex<IndexWriter>>,
    pub reader: IndexReader,
}

impl LogSearchEngine {
    pub fn new() -> Result<Self> {
        let mut schema_builder = Schema::builder();
        let num_options = NumericOptions::default()
            .set_indexed()
            .set_fast()
            .set_stored();
        schema_builder.add_date_field("timestamp", INDEXED | STORED | FAST);
        schema_builder.add_u64_field("priority", num_options);
        schema_builder.add_text_field("process", TEXT | STORED);
        schema_builder.add_text_field("message", TEXT | STORED);
        schema_builder.add_text_field("severity", TEXT | STORED);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());
        
        let index_writer = index.writer(50_000_000)?;
        let writer = Arc::new(parking_lot::Mutex::new(index_writer));
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;

        Ok(Self {
            schema,
            index,
            writer,
            reader,
        })
    }

    pub fn index_logs(&self, logs: &[LogEntry]) -> Result<()> {
        let mut writer_guard = self.writer.lock();
        writer_guard.delete_all_documents()?;
        
        for log in logs {
            let doc = log.to_tantivy_doc(&self.schema);
            writer_guard.add_document(doc)?;
        }
        
        writer_guard.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn index_single_log(&self, log: &LogEntry) -> Result<()> {
        let mut writer_guard = self.writer.lock();
        let doc = log.to_tantivy_doc(&self.schema);
        writer_guard.add_document(doc)?;
        writer_guard.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn search(&self, search_query: &str, limit: usize) -> Result<Vec<LogEntry>> {
        let searcher = self.reader.searcher();
        let f_process = self.schema.get_field("process").unwrap();
        let f_message = self.schema.get_field("message").unwrap();
        
        let cleaned_query = search_query.trim().to_lowercase();
        if cleaned_query.is_empty() {
            let query_parser = QueryParser::for_index(&self.index, vec![f_process, f_message]);
            let parsed_query = query_parser.parse_query("*")?;
            let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(limit))?;
            return self.resolve_documents(&searcher, top_docs);
        }

        let words: Vec<&str> = cleaned_query.split_whitespace().collect();
        let mut subqueries = Vec::new();
        for word in words {
            let mut word_queries = Vec::new();
            let term_process = Term::from_field_text(f_process, word);
            let query_process = TermQuery::new(term_process, tantivy::schema::IndexRecordOption::WithFreqs);
            word_queries.push((Occur::Should, Box::new(query_process) as Box<dyn tantivy::query::Query>));

            let term_message = Term::from_field_text(f_message, word);
            let query_message = TermQuery::new(term_message, tantivy::schema::IndexRecordOption::WithFreqs);
            word_queries.push((Occur::Should, Box::new(query_message) as Box<dyn tantivy::query::Query>));

            let wildcard_word = format!("{}*", word);
            let query_parser = QueryParser::for_index(&self.index, vec![f_process, f_message]);
            if let Ok(parsed_wildcard) = query_parser.parse_query(&wildcard_word) {
                word_queries.push((Occur::Should, parsed_wildcard));
            }

            let combined_word_query = BooleanQuery::new(word_queries);
            subqueries.push((Occur::Must, Box::new(combined_word_query) as Box<dyn tantivy::query::Query>));
        }

        let final_query = BooleanQuery::new(subqueries);
        let top_docs = searcher.search(&final_query, &TopDocs::with_limit(limit))?;
        
        self.resolve_documents(&searcher, top_docs)
    }

    fn resolve_documents(&self, searcher: &tantivy::Searcher, top_docs: Vec<(f32, tantivy::DocAddress)>) -> Result<Vec<LogEntry>> {
        let mut matched_entries = Vec::new();
        let f_timestamp = self.schema.get_field("timestamp").unwrap();
        let f_priority = self.schema.get_field("priority").unwrap();
        let f_process = self.schema.get_field("process").unwrap();
        let f_message = self.schema.get_field("message").unwrap();
        let f_severity = self.schema.get_field("severity").unwrap();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            
            let mut chrono_date = Utc::now();
if let Some(OwnedValue::Date(t_date)) = retrieved_doc.get_first(f_timestamp) {
    if let Some(dt) = Utc.timestamp_opt(t_date.into_utc().unix_timestamp(), 0).single() {
        chrono_date = dt;
    }
}


            let mut prio = 6u8;
            if let Some(OwnedValue::U64(prio_val)) = retrieved_doc.get_first(f_priority) {
                prio = *prio_val as u8;
            }

            let mut proc = String::new();
            if let Some(OwnedValue::Str(proc_val)) = retrieved_doc.get_first(f_process) {
                proc = proc_val.to_string();
            }

            let mut msg = String::new();
            if let Some(OwnedValue::Str(msg_val)) = retrieved_doc.get_first(f_message) {
                msg = msg_val.to_string();
            }

            let mut severity = Severity::Debug;
            if let Some(OwnedValue::Str(sev_str)) = retrieved_doc.get_first(f_severity) {
                severity = match sev_str.as_str() {
                    "critical" => Severity::Critical,
                    "error" => Severity::Error,
                    "warning" => Severity::Warning,
                    "info" => Severity::Info,
                    _ => Severity::Debug,
                };
            }

            matched_entries.push(LogEntry {
                timestamp: chrono_date,
                priority: prio,
                process: proc,
                pid: None,
                systemd_unit: None,
                transport: None,
                hostname: None,
                executable: None,
                message: msg,
                severity,
                raw: serde_json::Value::Null,
            });
        }

        Ok(matched_entries)
    }
}

