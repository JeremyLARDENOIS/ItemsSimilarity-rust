# ItemsSimilarity

In order to run the project, you need to have tinkerpop installed and maybe java 11.

## Data migration

In order to do the data migration, you need to have access to data. Migrate data can help you to get these data from polyflix to your local database.

### How to use it

You need to have a database tinkerpop running.

In order to populate the database, you need to run the following command:

```bash
cd migrate_data
cargo run dump
cargo run store
cd ..
```

After that, you can run the web application.

```bash
cd recommend
cargo run
```

### How to verify

From json files, you can check the number of items and the number of users.

```bash
jq length ../user_ids.json # 276
jq length ../videos.json # 85
jq length ../likes.json # 47
jq '[ .[] | select(.is_watched == true or .watch_percentage >= 0.7) ] | length' ../history.json # 90
```

We consider that a user has watched a video if this one is finished or if the user has watched more than 70% of the video.

You can also connect to the gremlin console and check the number of vertices and edges.

```bash
/opt/apache-tinkerpop-gremlin/apache-tinkerpop-gremlin-console-3.6.1/bin/gremlin.sh
:remote connect tinkerpop.server conf/remote.yaml session
:remote console
```
