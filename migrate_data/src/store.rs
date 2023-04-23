use std::{fs::File, io::Read};

use gremlin_client::{
    process::traversal::{traversal, GraphTraversalSource, SyncTerminator},
    GremlinClient,
};

use crate::{models::{HistoryJson, LikesJson, UserJson, VideosJson}, tokenize::recommendations};

fn users(ref g: &GraphTraversalSource<SyncTerminator>) {
    // Get data from json file
    let mut file = File::open("../user_ids.json").expect("Failed to open file");

    // Read the JSON contents of the file as an instance of `User`.
    let mut buff = String::new();
    file.read_to_string(&mut buff).expect("Failed to read file");
    let users: Vec<UserJson> = serde_json::from_str(&buff).expect("Failed to deserialize json");

    // Store data in database
    for user in users {
        g.add_v("user")
            .property("user_id", user.user_id)
            .next()
            .expect("Failed to add user to database");
    }
}

fn videos(ref g: &GraphTraversalSource<SyncTerminator>) {
    // Get data from json file
    let mut file = File::open("../videos.json").expect("Failed to open file");

    // Read the JSON contents of the file as an instance of `Video`.
    let mut buff = String::new();
    file.read_to_string(&mut buff).expect("Failed to read file");
    let videos: Vec<VideosJson> = serde_json::from_str(&buff).expect("Failed to deserialize json");

    // Store data in database
    for video in videos {
        g.add_v("video")
            .property("video_id", video.video_id)
            .property("title", video.title)
            .property("description", video.description)
            .property("publisher_id", video.publisher_id)
            .next()
            .expect("Failed to add video to database");
    }
}

fn add_like_edge(ref g: &GraphTraversalSource<SyncTerminator>, user_id: i32, video_id: i32) {
    let user = g
        .v(())
        .has_label("user")
        .has(("user_id", user_id))
        .next()
        // .expect("Failed to get user")
        // .expect("No user found");
        ;
    let video = g
        .v(())
        .has_label("video")
        .has(("video_id", video_id))
        .next()
        // .expect("Failed to get video")
        // .expect("No video found");
        ;
    if let (Ok(Some(user)), Ok(Some(video))) = (user, video) {
        g.add_e("likes")
            .from(&user)
            .to(&video)
            .next()
            .expect("Failed to add like to database");
    }

    // g.add_e("likes")
    //     .from(&user)
    //     .to(&video)
    //     .next()
    //     .expect("Failed to add like to database");
}

fn likes(ref g: &GraphTraversalSource<SyncTerminator>) {
    // Get data from json file
    let mut file = File::open("../likes.json").expect("Failed to open file");

    // Read the JSON contents of the file as an instance of `Like`.
    let mut buff = String::new();
    file.read_to_string(&mut buff).expect("Failed to read file");
    let likes: Vec<LikesJson> = serde_json::from_str(&buff).expect("Failed to deserialize json");

    // Store data in database
    for like in likes {
        let user = g
            .v(())
            .has_label("user")
            .has(("user_id", like.user_id.clone()))
            .next();
        let video = g
            .v(())
            .has_label("video")
            .has(("video_id", like.video_id.clone()))
            .next();
        if let (Ok(Some(user)), Ok(Some(video))) = (user, video) {
            g.add_e("likes")
                .from(&user)
                .to(&video)
                .next()
                .expect("Failed to add like to database");
        } else {
            println!("User with id {} or video with id {} not found", like.user_id, like.video_id);
        }
    }
}

fn history(ref g: &GraphTraversalSource<SyncTerminator>) {
    // Get data from json file
    let mut file = File::open("../history.json").expect("Failed to open file");

    // Read the JSON contents of the file as an instance of `History`.
    let mut buff = String::new();
    file.read_to_string(&mut buff).expect("Failed to read file");
    let history: Vec<HistoryJson> =
        serde_json::from_str(&buff).expect("Failed to deserialize json");

    // Store data in database
    let mut count = 0;
    for view in history {
        // We consider that a video is viewed if it is in the history and isWatched is true or if watchedPercentage is greater than 0.7
        if view.is_watched || view.watch_percentage >= 0.7 {
            count += 1;
            // Get user
            let user = g
                .v(())
                .has_label("user")
                .has(("user_id", view.user_id.clone()))
                .next()
                .expect("Failed to get user");
            // Get video
            let video = g
                .v(())
                .has_label("video")
                .has(("video_id", view.video_id.clone()))
                .next()
                .expect("Failed to get video");
            // Add view if user and video exist
            if user.is_some() && video.is_some() {
                g.add_e("watched")
                    .from(&user.unwrap())
                    .to(&video.unwrap())
                    .property("watchedPercentage", view.watch_percentage)
                    .next()
                    .expect("Failed to add view to database");
            } else {
                println!("User or video not found for view: {:?}", view);
            }
        }
    }
    println!("Count: {}", count);
}

fn verify_db_is_running() -> bool {
    // Verify that database is running
    let client_result = GremlinClient::connect("localhost");
    if client_result.is_err() {
        println!("Database is not running");
        return false;
    }
    println!("Database is running");
    true
}

fn launch_db() {
    // If database is not running, throw an error
    if !verify_db_is_running() {
        panic!("Database is not running");
        // println!("Launching database");
        // std::process::Command::new("/opt/apache-tinkerpop-gremlin/apache-tinkerpop-gremlin-server-3.6.1/bin/gremlin-server.sh")
        //     .arg("start")
        //     .spawn()
        //     .expect("Failed to launch database");
    }
}

fn get_transversal() -> GraphTraversalSource<SyncTerminator> {
    // Get transversal
    let client = GremlinClient::connect("localhost").expect("Failed to connect to database");
    let g = traversal().with_remote(client);
    g
}

pub fn store() {
    launch_db();
    let g = get_transversal();

    println!("Dropping vertices");
    g.v(()).drop().next().expect("Failed to drop vertices");

    println!("Adding users");
    users(&g);

    println!("Adding videos");
    videos(&g);

    println!("Adding likes");
    likes(&g);

    println!("Adding history");
    history(&g);

    println!("Adding recommendations");
    recommendations(&g);

    println!("Show results");
    println!(
        "Users: {}",
        g.v(())
            .has_label("user")
            .count()
            .next()
            .expect("Failed to count users")
            .expect("No users found")
    );
    println!(
        "Videos: {}",
        g.v(())
            .has_label("video")
            .count()
            .next()
            .expect("Failed to count videos")
            .expect("No videos found")
    );
    println!(
        "Likes: {}",
        g.e(())
            .has_label("likes")
            .count()
            .next()
            .expect("Failed to count likes")
            .expect("No likes found")
    );
    println!(
        "History: {}",
        g.e(())
            .has_label("watched")
            .count()
            .next()
            .expect("Failed to count history")
            .expect("No history found")
    );
    println!(
        "Recommendations: {}",
        g.e(())
            .has_label("similar_to")
            .count()
            .next()
            .expect("Failed to count recommendations")
            .expect("No recommendations found")
    );
}
