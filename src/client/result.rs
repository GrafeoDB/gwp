//! Result cursor for iterating over streaming query results.

use crate::error::GqlError;
use crate::proto;
use crate::status;
use crate::types::Value;

/// A cursor over the streaming results from a GQL statement.
///
/// Provides access to column metadata, rows, and the final summary.
pub struct ResultCursor {
    stream: tonic::Streaming<proto::ExecuteResponse>,
    header: Option<proto::ResultHeader>,
    summary: Option<proto::ResultSummary>,
    buffered_rows: Vec<Vec<Value>>,
    done: bool,
}

impl ResultCursor {
    pub(crate) fn new(stream: tonic::Streaming<proto::ExecuteResponse>) -> Self {
        Self {
            stream,
            header: None,
            summary: None,
            buffered_rows: Vec::new(),
            done: false,
        }
    }

    /// Get the result header (column metadata).
    ///
    /// Consumes frames until the header is found. Returns `None` if
    /// the stream ends without a header.
    pub async fn header(&mut self) -> Result<Option<&proto::ResultHeader>, GqlError> {
        if self.header.is_some() {
            return Ok(self.header.as_ref());
        }

        self.advance_to_header().await?;
        Ok(self.header.as_ref())
    }

    /// Get the column names from the result header.
    pub async fn column_names(&mut self) -> Result<Vec<String>, GqlError> {
        self.header().await?;
        Ok(self
            .header
            .as_ref()
            .map(|h| h.columns.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default())
    }

    /// Get the next row of results.
    ///
    /// Returns `None` when all rows have been consumed.
    pub async fn next_row(&mut self) -> Result<Option<Vec<Value>>, GqlError> {
        // Drain buffered rows first
        if !self.buffered_rows.is_empty() {
            return Ok(Some(self.buffered_rows.remove(0)));
        }

        if self.done {
            return Ok(None);
        }

        // Fetch more frames
        loop {
            match self.stream.message().await? {
                Some(response) => match response.frame {
                    Some(proto::execute_response::Frame::Header(h)) => {
                        self.header = Some(h);
                        continue;
                    }
                    Some(proto::execute_response::Frame::RowBatch(batch)) => {
                        let mut rows: Vec<Vec<Value>> = batch
                            .rows
                            .into_iter()
                            .map(|r| r.values.into_iter().map(Value::from).collect())
                            .collect();

                        if rows.is_empty() {
                            continue;
                        }

                        let first = rows.remove(0);
                        self.buffered_rows = rows;
                        return Ok(Some(first));
                    }
                    Some(proto::execute_response::Frame::Summary(s)) => {
                        self.summary = Some(s);
                        self.done = true;
                        return Ok(None);
                    }
                    None => continue,
                },
                None => {
                    self.done = true;
                    return Ok(None);
                }
            }
        }
    }

    /// Collect all remaining rows into a vector.
    pub async fn collect_rows(&mut self) -> Result<Vec<Vec<Value>>, GqlError> {
        let mut all_rows = Vec::new();
        while let Some(row) = self.next_row().await? {
            all_rows.push(row);
        }
        Ok(all_rows)
    }

    /// Get the result summary (available after all rows consumed).
    ///
    /// Consumes remaining frames if needed.
    pub async fn summary(&mut self) -> Result<Option<&proto::ResultSummary>, GqlError> {
        if self.summary.is_some() {
            return Ok(self.summary.as_ref());
        }

        // Consume remaining frames
        while !self.done {
            self.next_row().await?;
        }

        Ok(self.summary.as_ref())
    }

    /// Check if the result completed successfully.
    ///
    /// Consumes remaining frames if needed.
    pub async fn is_success(&mut self) -> Result<bool, GqlError> {
        let summary = self.summary().await?;
        Ok(summary
            .and_then(|s| s.status.as_ref())
            .map_or(false, |s| status::is_success(&s.code)))
    }

    /// Get the number of rows affected (for DML operations).
    ///
    /// Consumes remaining frames if needed.
    pub async fn rows_affected(&mut self) -> Result<i64, GqlError> {
        let summary = self.summary().await?;
        Ok(summary.map_or(0, |s| s.rows_affected))
    }

    /// Advance the stream until we find the header.
    async fn advance_to_header(&mut self) -> Result<(), GqlError> {
        while !self.done {
            match self.stream.message().await? {
                Some(response) => match response.frame {
                    Some(proto::execute_response::Frame::Header(h)) => {
                        self.header = Some(h);
                        return Ok(());
                    }
                    Some(proto::execute_response::Frame::RowBatch(batch)) => {
                        let rows: Vec<Vec<Value>> = batch
                            .rows
                            .into_iter()
                            .map(|r| r.values.into_iter().map(Value::from).collect())
                            .collect();
                        self.buffered_rows.extend(rows);
                    }
                    Some(proto::execute_response::Frame::Summary(s)) => {
                        self.summary = Some(s);
                        self.done = true;
                        return Ok(());
                    }
                    None => continue,
                },
                None => {
                    self.done = true;
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}
