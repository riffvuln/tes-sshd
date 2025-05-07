use search_with_google::search;

#[tokio::main]
async fn main() {
    let query = "rust programming language";
    let results = search(query, 10, None).await.unwrap();

    for result in results {
        println!("Title: {}", result.title);
        println!("Link: {}", result.link);
        println!("Snippet: {}", result.description);
        println!();
    }
}
