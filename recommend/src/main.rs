use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use gremlin_client::{
    process::traversal::{traversal, GraphTraversalSource, SyncTerminator, __},
    GremlinClient, Vertex,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Params {
    limit: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VideoRecommendation {
    id: String,
    title: String,
    score: f32,
}

fn get_transversal() -> GraphTraversalSource<SyncTerminator> {
    // Get transversal
    let client = GremlinClient::connect("localhost").expect("Failed to connect to database");
    let g = traversal().with_remote(client);
    g
}

fn get_videos_watched_by_user_id(user_id: Uuid) -> Vec<gremlin_client::Vertex> {
    // Get videos seen and liked by user
    let g = get_transversal();

    // For the user with user_id, get the videos they have seen and liked, and dedup
    g.v(())
        .has(("user", "user_id", user_id.to_string()))
        .out(()) // caution: .out(("watched", "likes")) does not work
        .dedup(())
        .to_list()
        .expect("Failed to get videos")
        .to_owned()
}

fn get_videos_not_seen_by_user_id(user_id: Uuid) -> Vec<gremlin_client::Vertex> {
    // Get videos not seen or not liked by user
    let g = get_transversal();

    // Get all videos, then filter out the videos the user has seen or liked
    g.v(())
        .has_label("video")
        .not(
            __.in_(())
                .has(("user", "user_id", user_id.to_string()))
                .dedup(()),
        )
        .to_list()
        .expect("Failed to get videos")
        .to_owned()
}

fn calculate_recommendation_score(
    watched_videos: &[Vertex],
    not_watched_videos: &[Vertex],
) -> Vec<VideoRecommendation> {
    let g = get_transversal();
    // Initialize recommendations vector
    let mut recommendations = Vec::new();

    // For each video not watched by the user, calculate the recommendation score
    for not_watched_video_vertex in not_watched_videos.iter() {
        let property = g
            .v(not_watched_video_vertex.id())
            .properties("video_id")
            .next();
        if let Ok(Some(property)) = property {
            let not_watched_video_id = property.value().get::<String>().unwrap();

            let mut total_similarity = 0.0;
            let mut total_weight = 0.0;
            // For each video watched by the user of each not watched video, get the similarity score
            // Add the similarity score to the total similarity and increment the total
            // weight of the recommendation score
            for watched_video_vertex in watched_videos {
                let edge = g
                    .v(watched_video_vertex.id())
                    .out("similar_to")
                    .has(("video_id", not_watched_video_id.to_owned()))
                    .out_e("similar_to")
                    .next();

                if let Ok(Some(edge)) = edge {
                    let property = g.e(edge.id()).properties("similarity").next();
                    if let Ok(Some(property)) = property {
                        let similarity = property.value().get::<f64>().unwrap();
                        total_similarity += similarity;
                        total_weight += 1.0;
                    } else {
                        println!("No property found");
                    }
                } else {
                    println!("No edge found");
                }
            }

            // Calculate the average similarity score
            let recommendation_score = total_similarity / total_weight;

            let property = g
                .v(not_watched_video_vertex.id())
                .properties("title")
                .next();

            if let Ok(Some(property)) = property {
                let video_title = property.value().get::<String>().unwrap();

                recommendations.push(VideoRecommendation {
                    id: not_watched_video_id.to_owned(),
                    title: video_title.to_string(),
                    score: recommendation_score as f32,
                });
            }
        }
    }
    // Sort the recommendations by score
    recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    recommendations
}

async fn get_user_recommends(
    Path(user_id): Path<Uuid>,
    Query(query_params): Query<Params>,
) -> axum::Json<Vec<VideoRecommendation>> {
    /*
    get videos seen and liked by user
    get videos unseen by user
    sort unseen videos by score
    return the top 10 videos
     */
    let watched_videos = get_videos_watched_by_user_id(user_id);
    println!("watched_videos {}", watched_videos.len(),);

    let not_watched_videos = get_videos_not_seen_by_user_id(user_id);
    println!("not_watched_videos {}", not_watched_videos.len(),);

    println!("Calculating recommendation score...");
    let recommendations = calculate_recommendation_score(&watched_videos, &not_watched_videos);

    // Limit the number of recommendations
    let limit = query_params.limit.unwrap_or(10);
    let recommendations = recommendations.into_iter().take(limit as usize).collect();

    // Return an empty list for now
    println!("Returning recommendations");
    axum::Json(recommendations)
}

#[tokio::main]
async fn main() {
    // // build our application with a single route
    let app = Router::new().route("/recommendations/:id", get(get_user_recommends));

    // run it with hyper on localhost:3000
    println!("Listening on http://localhost:3000");
    println!("Try: curl http://localhost:3000/recommendations/b8d26a9a-af81-4cec-abf9-1bac3101c8d0");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    // Test with curl http://localhost:3000/recommendations/b8d26a9a-af81-4cec-abf9-1bac3101c8d0
    // println!(
    //     "{:?}",
    //     get_user_recommends(
    //         Path(uuid::uuid!("b8d26a9a-af81-4cec-abf9-1bac3101c8d0")),
    //         Query(Params { limit: Some(10) }),
    //     )
    //     .await
    // );

    /*
    In console:
    /opt/apache-tinkerpop-gremlin/apache-tinkerpop-gremlin-console-3.6.1/bin/gremlin.sh
    :remote connect tinkerpop.server conf/remote.yaml session
    :remote console
    g.V().properties("user_id").value().toList()
    g.V().hasLabel("video").count() // 85
    g.V().has("user", "user_id", "b8d26a9a-af81-4cec-abf9-1bac3101c8d0")
    g.V().hasLabel("user").has("user_id", "b8d26a9a-af81-4cec-abf9-1bac3101c8d0")
    g.V().hasLabel("user").has("user_id", "b8d26a9a-af81-4cec-abf9-1bac3101c8d0").out().hasLabel("video").dedup().count() // 22
    g.V().hasLabel("video").not(__.in().has("user_id", "b8d26a9a-af81-4cec-abf9-1bac3101c8d0")).count() // 63
    // Unseen videos is 85 - 22 = 63, so it works
     */
}
