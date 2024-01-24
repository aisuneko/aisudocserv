use regex::Regex;
use std::{collections::HashMap, env, fs, path::PathBuf};
use tantivy::{collector::TopDocs, doc, query::QueryParser, schema::*, Index};
use tera::Tera;
use walkdir::{DirEntry, WalkDir};
use warp::Filter;
#[derive(Clone)]
struct Context {
    index: Index,
    schema: Schema,
}
fn cwd() -> PathBuf {
    env::current_dir().expect("Error opening current dir")
}
fn is_html(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".html"))
        .unwrap_or(false)
}
fn build_index(ctx: &Context) {
    let title_field = ctx.schema.get_field("title").unwrap();
    let url_field = ctx.schema.get_field("url").unwrap();
    let mut index_writer = ctx.index.writer(20_000_000).unwrap();
    let walker = WalkDir::new(cwd()).into_iter();
    for entry in walker {
        let entry = entry.unwrap();
        if is_html(&entry) {
            let path = entry.path();
            let re = Regex::new(r"<title>(.*)<\/title>").unwrap();
            let title_text = match re.captures(&fs::read_to_string(path).unwrap()) {
                Some(e) => e[1].to_owned(),
                None => entry.file_name().to_str().unwrap().to_owned(),
            };
            index_writer
                .add_document(doc!(
                    title_field => title_text,
                    url_field => path.strip_prefix(cwd()).unwrap().to_str().unwrap(),
                ))
                .unwrap();
        }
    }
    index_writer.commit().unwrap();
}
fn build_search_engine() -> Context {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("url", TEXT | STORED);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema.clone());
    let main_ctx = Context { index, schema };
    build_index(&main_ctx);
    main_ctx
}
fn search(input: &str, ctx: &Context) -> Vec<(String, String)> {
    if input.is_empty() {
        return vec![];
    };
    let query_parser =
        QueryParser::for_index(&ctx.index, ctx.schema.fields().map(|e| e.0).collect());
    let searcher = ctx.index.reader().unwrap().searcher();
    let query = query_parser.parse_query(input).unwrap();
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    let mut vec: Vec<(String, String)> = Vec::new();
    for (_score, doc_address) in top_docs {
        let doc_address = searcher.doc(doc_address).unwrap();
        let values = doc_address.get_sorted_field_values().to_owned();
        vec.push((
            values[0].1[0].as_text().unwrap().to_owned(),
            values[1].1[0].as_text().unwrap().to_owned(),
        ))
    }
    vec
}
fn build_search_result(p: HashMap<String, String>, ctx: &Context) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("search_page", include_str!("search.html"))
        .unwrap();
    let mut tera_ctx = tera::Context::new();
    tera_ctx.insert(
        "list",
        &search(p.get("q").unwrap_or(&String::from("")), ctx),
    );
    tera.render("search_page", &tera_ctx).unwrap()
}

#[tokio::main]
async fn serve(ctx: Context) {
    let route = warp::get().and(warp::fs::dir(cwd()));
    let search_route = warp::path("search")
        .and(warp::query::<HashMap<String, String>>())
        .map(move |p: HashMap<String, String>| warp::reply::html(build_search_result(p, &ctx)));
    // let css_route = warp::path("chota.min.css")
    //     .map(|| include_str!("chota.min.css"))
    //     .map(|reply| warp::reply::with_header(reply, "content-type", "text/css"));
    println!("Server running at 127.0.0.1:3030");
    warp::serve(route.or(search_route))
        .run(([127, 0, 0, 1], 3030))
        .await;
}
fn main() {
    println!("Building index...");
    let ctx = build_search_engine();
    serve(ctx);
}
