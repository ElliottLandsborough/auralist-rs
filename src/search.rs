use std::path::Path;

/*
use tantivy::collector::{Count, TopDocs};
use tantivy::query::FuzzyTermQuery;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
*/

/*
pub fn write_index() -> tantivy::Result<()> {
    let mut schema_builder = Schema::builder();
    let file_name = schema_builder.add_text_field("file_name", TEXT | STORED);
    let artist = schema_builder.add_text_field("artist", TEXT | STORED);
    let album = schema_builder.add_text_field("album", TEXT | STORED);
    let title = schema_builder.add_text_field("title", TEXT | STORED);
    let schema = schema_builder.build();
    let index = Index::create_in_dir(Path::new("./search"), schema.clone())?;
    let mut index_writer: IndexWriter = index.writer(100_000_000)?;

    /*
    index_writer.add_document(doc!(
        file_name => f.file_name,
        artist => f.artist,
        album => "The Name of the Wind",
        title => "The Name of the Wind",
    ))?;
    */

    index_writer.commit()?;

    Ok(())
}
*/

/*
fn anything() {
    // # Searching
    //
    // ### Searcher
    //
    // A reader is required first in order to search an index.
    // It acts as a `Searcher` pool that reloads itself,
    // depending on a `ReloadPolicy`.
    //
    // For a search server you will typically create one reader for the entire lifetime of your
    // program, and acquire a new searcher for every single request.
    //
    // In the code below, we rely on the 'ON_COMMIT' policy: the reader
    // will reload the index automatically after each commit.
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    // We now need to acquire a searcher.
    //
    // A searcher points to a snapshotted, immutable version of the index.
    //
    // Some search experience might require more than
    // one query. Using the same searcher ensures that all of these queries will run on the
    // same version of the index.
    //
    // Acquiring a `searcher` is very cheap.
    //
    // You should acquire a searcher every time you start processing a request and
    // and release it right after your query is finished.
    let searcher = reader.searcher();

    // ### FuzzyTermQuery
    {
        let term = Term::from_field_text(title, "Diary");
        let query = FuzzyTermQuery::new(term, 2, true);

        let (top_docs, count) = searcher
            .search(&query, &(TopDocs::with_limit(5), Count))
            .unwrap();
        assert_eq!(count, 3);
        assert_eq!(top_docs.len(), 3);
        for (score, doc_address) in top_docs {
            // Note that the score is not lower for the fuzzy hit.
            // There's an issue open for that: https://github.com/quickwit-oss/tantivy/issues/563
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("score {score:?} doc {}", retrieved_doc.to_json(&schema));
            // score 1.0 doc {"title":["The Diary of Muadib"]}
            //
            // score 1.0 doc {"title":["The Diary of a Young Girl"]}
            //
            // score 1.0 doc {"title":["A Dairy Cow"]}
        }
    }

    Ok(())
}
*/
